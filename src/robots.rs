use bevy::prelude::*;
use rand::prelude::*;
use noise::{NoiseFn, Perlin};
use bevy::ecs::system::ParamSet;

// Composants
#[derive(Component)]
struct Explorer;

#[derive(Component)]
struct Miner;

#[derive(Component)]
struct Base;

#[derive(Component)]
struct Crystal;

#[derive(Component)]
struct DebugGrid;

// Nouveaux composants pour les ressources
#[derive(Component, Clone, Debug)]
enum Resource {
    Energy,
    Mineral,
    ScientificSite,
}

#[derive(Resource)]
struct GameMap {
    size: Vec2,
    cell_size: f32,
    obstacles: Vec<Vec<bool>>,
    seed: u32,
}

#[derive(Resource)]
struct MapResources {
    energy_positions: Vec<Vec2>,
    mineral_positions: Vec<Vec2>,
    scientific_sites: Vec<Vec2>,
}

// Ressources
#[derive(Resource)]
struct DiscoveredResource {
    position: Option<Vec2>,
}

impl Default for DiscoveredResource {
    fn default() -> Self {
        Self { position: None }
    }
}

#[derive(Resource)]
struct ExploredZones {
    grid: Vec<Vec<bool>>,
    cell_size: f32,
}

impl Default for ExploredZones {
    fn default() -> Self {
        Self {
            grid: vec![vec![false; 80]; 60],  // 800/10 x 600/10
            cell_size: 10.0,
        }
    }
}

#[derive(Resource)]
struct ExplorerState {
    current_direction: Vec2,
    time_until_change: f32,
}

// Composant pour les tuiles de brouillard
#[derive(Component)]
struct FogTile;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        let seed = rand::random::<u32>();
        let game_map = generate_map(800.0, 600.0, 10.0, seed);
        
        let explored_zones = ExploredZones {
            grid: vec![vec![false; 80]; 60],
            cell_size: 10.0,
        };
        
        let map_resources = generate_resources(&game_map);

        app
            .insert_resource(game_map)
            .insert_resource(map_resources)
            .insert_resource(explored_zones)
            .insert_resource(DiscoveredResource::default())
            .insert_resource(ExplorerState {
                current_direction: Vec2::new(1.0, 0.0),
                time_until_change: 2.0,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, (
                move_explorer,
                check_resource_discovery,
                move_miners,
                update_explored_map,
                update_fog_of_war,
                debug_draw_map,
            ).chain());
    }
}

fn setup(
    mut commands: Commands,
) {
    // Caméra
    commands.spawn(Camera2dBundle::default());

    // Base (Bleu)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Base,
    ));

    // Explorateur (Vert)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(20.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 50., 0.),
            ..default()
        },
        Explorer,
    ));

    // Mineurs (Orange)
    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.0),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(30. * (i as f32 - 1.), -30., 0.),
                ..default()
            },
            Miner,
        ));
    }

    // Créer le brouillard de guerre - tuiles plus petites pour une meilleure résolution
    let fog_size = 10.0; // Taille réduite des tuiles
    let width = 800.0;
    let height = 600.0;
    
    let cols = (width / fog_size) as i32;
    let rows = (height / fog_size) as i32;
    
    for y in 0..rows {
        for x in 0..cols {
            let pos_x = (x as f32 * fog_size) - (width / 2.0) + (fog_size / 2.0);
            let pos_y = (y as f32 * fog_size) - (height / 2.0) + (fog_size / 2.0);
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.1, 0.1, 0.1, 1.0), // Opacité complète
                        custom_size: Some(Vec2::new(fog_size, fog_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(pos_x, pos_y, 2.0), // Z-index plus élevé
                    ..default()
                },
                FogTile,
            ));
        }
    }
}

fn move_explorer(
    mut query_set: ParamSet<(
        Query<&mut Transform, With<Explorer>>,
        Query<(&mut Sprite, &Transform), With<FogTile>>,
    )>,
    time: Res<Time>,
    mut explorer_state: ResMut<ExplorerState>,
    discovered_resource: Res<DiscoveredResource>,
    game_map: Res<GameMap>,
) {
    if discovered_resource.position.is_some() {
        return;
    }

    let mut explorer_pos = Vec3::ZERO;

    // Mise à jour de l'explorateur
    {
        let mut explorer_query = query_set.p0();
        for mut transform in explorer_query.iter_mut() {
            explorer_state.time_until_change -= time.delta_seconds();
            let speed = 150.0;
            let mut should_change_direction = false;

            // Vérifier si la direction actuelle est bloquée
            let test_pos = transform.translation + Vec3::new(
                explorer_state.current_direction.x * speed * time.delta_seconds() * 2.0,
                explorer_state.current_direction.y * speed * time.delta_seconds() * 2.0,
                0.0
            );

            if is_position_blocked(test_pos, &game_map) || test_pos.x.abs() > 350.0 || test_pos.y.abs() > 250.0 {
                should_change_direction = true;
            }

            // Changer de direction si nécessaire
            if should_change_direction || explorer_state.time_until_change <= 0.0 {
                // Essayer plusieurs directions jusqu'à en trouver une valide
                for _ in 0..8 {
                    let new_direction = Vec2::new(
                        rand::thread_rng().gen_range(-1.0..=1.0),
                        rand::thread_rng().gen_range(-1.0..=1.0),
                    ).normalize();

                    let test_pos = transform.translation + Vec3::new(
                        new_direction.x * speed * time.delta_seconds() * 2.0,
                        new_direction.y * speed * time.delta_seconds() * 2.0,
                        0.0
                    );

                    if !is_position_blocked(test_pos, &game_map) 
                       && test_pos.x.abs() <= 350.0 
                       && test_pos.y.abs() <= 250.0 {
                        explorer_state.current_direction = new_direction;
                        break;
                    }
                }
                explorer_state.time_until_change = 2.0;
            }

            // Déplacement
            let mut new_pos = transform.translation;
            new_pos.x += explorer_state.current_direction.x * speed * time.delta_seconds();
            new_pos.y += explorer_state.current_direction.y * speed * time.delta_seconds();
            new_pos.x = new_pos.x.clamp(-350.0, 350.0);
            new_pos.y = new_pos.y.clamp(-250.0, 250.0);

            // Vérifier une dernière fois si la nouvelle position est valide
            if !is_position_blocked(new_pos, &game_map) {
                transform.translation = new_pos;
                transform.rotation = Quat::from_rotation_z(
                    -explorer_state.current_direction.y.atan2(explorer_state.current_direction.x)
                );
            }

            explorer_pos = transform.translation;
        }
    }

    // Mise à jour du brouillard
    {
        let mut fog_query = query_set.p1();
        let vision_radius = 60.0;
        let fade_distance = 20.0;

        for (mut fog_sprite, fog_transform) in fog_query.iter_mut() {
            let distance = explorer_pos.distance(fog_transform.translation);
            
            if distance < vision_radius {
                fog_sprite.color.set_a(0.0);
            } else if distance < vision_radius + fade_distance {
                let fade = (distance - vision_radius) / fade_distance;
                let current_alpha = fog_sprite.color.a();
                let new_alpha = fade.min(current_alpha);
                fog_sprite.color.set_a(new_alpha);
            }
        }
    }
}

fn find_nearest_unexplored(pos: &Vec3, explored_zones: &ExploredZones, game_map: &GameMap) -> Vec2 {
    let current_x = ((pos.x + 400.0) / explored_zones.cell_size) as i32;
    let current_y = ((pos.y + 300.0) / explored_zones.cell_size) as i32;
    let mut nearest_unexplored = Vec2::new(1.0, 0.0);
    let mut min_distance = f32::MAX;

    // Chercher dans toute la grille
    for y in 0..60 {
        for x in 0..80 {
            if !explored_zones.grid[y][x] {
                let dx = x as f32 - current_x as f32;
                let dy = y as f32 - current_y as f32;
                let distance = dx * dx + dy * dy;
                
                if distance < min_distance {
                    let direction = Vec2::new(dx, dy).normalize();
                    let test_pos = Vec3::new(
                        pos.x + direction.x * 50.0, 
                        pos.y + direction.y * 50.0, 
                        0.0
                    );
                    
                    if !is_position_blocked(test_pos, game_map) {
                        min_distance = distance;
                        nearest_unexplored = direction;
                    }
                }
            }
        }
    }

    nearest_unexplored
}

fn update_explored_map(
    explorer_query: Query<&Transform, With<Explorer>>,
    mut explored_zones: ResMut<ExploredZones>,
    game_map: Res<GameMap>,
) {
    if let Ok(explorer_transform) = explorer_query.get_single() {
        let pos = explorer_transform.translation;
        
        let grid_x = ((pos.x + game_map.size.x/2.0) / game_map.cell_size) as usize;
        let grid_y = ((pos.y + game_map.size.y/2.0) / game_map.cell_size) as usize;

        let radius = 2;
        for dy in -(radius as i32)..=radius as i32 {
            for dx in -(radius as i32)..=radius as i32 {
                let x = grid_x as i32 + dx;
                let y = grid_y as i32 + dy;

                if x >= 0 && x < explored_zones.grid[0].len() as i32 
                   && y >= 0 && y < explored_zones.grid.len() as i32 {
                    explored_zones.grid[y as usize][x as usize] = true;
                }
            }
        }
    }
}

fn check_resource_discovery(
    explorer_query: Query<&Transform, With<Explorer>>,
    resources_query: Query<(&Transform, &Resource)>,
    mut discovered_resource: ResMut<DiscoveredResource>,
) {
    // Si une ressource est déjà découverte, ne rien faire
    if discovered_resource.position.is_some() {
        return;
    }

    if let Ok(explorer_transform) = explorer_query.get_single() {
        let explorer_pos = explorer_transform.translation;
        
        // Vérifier chaque ressource
        for (resource_transform, resource_type) in resources_query.iter() {
            let resource_pos = resource_transform.translation;
            let distance = explorer_pos.distance(resource_pos);

            // Vérifier uniquement les minéraux et l'énergie
            match resource_type {
                Resource::Mineral | Resource::Energy => {
                    if distance < 50.0 {  // Augmentation de la distance de détection
                        println!("Ressource trouvée! Type: {:?}, Distance: {}", resource_type, distance);
                        discovered_resource.position = Some(Vec2::new(
                            resource_pos.x,
                            resource_pos.y
                        ));
                        return;  // Sortir immédiatement après avoir trouvé une ressource
                    }
                },
                _ => continue,
            }
        }
    }
}

fn move_miners(
    mut query_set: ParamSet<(
        Query<&mut Transform, (With<Miner>, Without<Base>)>,
        Query<(Entity, &Transform, &Resource)>,
        Query<(&Sprite, &Transform), With<FogTile>>,
    )>,
    mut discovered_resource: ResMut<DiscoveredResource>,
    mut commands: Commands,
    time: Res<Time>,
    game_map: Res<GameMap>,
) {
    if let Some(resource_pos) = discovered_resource.position {
        let resource_pos_3d = Vec3::new(resource_pos.x, resource_pos.y, 0.0);
        let mut resource_to_remove = None;
        let mut resource_exists = false;

        // Vérifier si la ressource existe toujours
        {
            let resources = query_set.p1();
            for (entity, transform, _) in resources.iter() {
                if transform.translation.distance(resource_pos_3d) < 15.0 {
                    resource_exists = true;
                    resource_to_remove = Some(entity);
                    break;
                }
            }
        }

        if !resource_exists {
            discovered_resource.position = None;
            return;
        }

        // Collecter d'abord les informations sur les zones révélées
        let revealed_zones: Vec<(Vec3, bool)> = {
            let fog_query = query_set.p2();
            fog_query.iter()
                .map(|(sprite, transform)| (transform.translation, sprite.color.a() < 0.5))
                .collect()
        };

        // Ensuite, déplacer les mineurs
        let mut miners = query_set.p0();
        for mut miner_transform in miners.iter_mut() {
            let current_pos = miner_transform.translation;
            let direction = (resource_pos_3d - current_pos).normalize();
            let speed = 200.0;
            let step_size = speed * time.delta_seconds();

            // Vérifier si le mineur a atteint la ressource
            if current_pos.distance(resource_pos_3d) < 15.0 {
                if let Some(entity) = resource_to_remove {
                    commands.entity(entity).despawn();
                    discovered_resource.position = None;
                    println!("Ressource collectée!");
                    return;
                }
            }

            // Trouver un chemin dans la zone révélée
            let mut valid_move = false;
            let test_angles: [f32; 8] = [0.0, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0];
            
            for angle in test_angles.iter() {
                let rotated_direction = rotate_vector(direction, angle.to_radians());
                let test_pos = current_pos + rotated_direction * step_size;
                
                // Vérifier si la position est dans une zone révélée
                let is_revealed = revealed_zones.iter().any(|(pos, is_revealed)| {
                    pos.distance(test_pos) < 30.0 && *is_revealed
                });

                if is_revealed && !is_position_blocked(test_pos, &game_map) {
                    miner_transform.translation = test_pos;
                    miner_transform.rotation = Quat::from_rotation_z(
                        -rotated_direction.y.atan2(rotated_direction.x)
                    );
                    valid_move = true;
                    break;
                }
            }

            if !valid_move {
                continue;
            }
        }
    }
}

fn rotate_vector(v: Vec3, angle: f32) -> Vec3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Vec3::new(
        v.x * cos_a - v.y * sin_a,
        v.x * sin_a + v.y * cos_a,
        0.0
    )
}

// Système pour révéler la carte autour de l'explorateur
fn update_fog_of_war(
    explorer_query: Query<&Transform, With<Explorer>>,
    mut fog_tiles: Query<(&mut Sprite, &Transform), With<FogTile>>,
) {
    if let Ok(explorer_transform) = explorer_query.get_single() {
        let explorer_pos = explorer_transform.translation;
        let vision_radius = 60.0; // Rayon de vision augmenté
        let fade_distance = 20.0;

        for (mut fog_sprite, fog_transform) in fog_tiles.iter_mut() {
            let distance = explorer_pos.distance(fog_transform.translation);
            
            if distance < vision_radius {
                // Zone complètement révélée
                fog_sprite.color.set_a(0.0);
            } else if distance < vision_radius + fade_distance {
                // Zone de transition
                let fade = (distance - vision_radius) / fade_distance;
                fog_sprite.color.set_a(fade);
            } else {
                // Zone non explorée
                fog_sprite.color.set_a(1.0);
            }
        }
    }
}

fn debug_draw_map(
    mut commands: Commands,
    query: Query<Entity, With<DebugGrid>>,
    map: Res<GameMap>,
    map_resources: Res<MapResources>,
) {
    // Nettoyer les anciennes entités
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Dessiner les obstacles
    for (y, row) in map.obstacles.iter().enumerate() {
        for (x, &is_obstacle) in row.iter().enumerate() {
            if is_obstacle {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.3, 0.3, 0.3),
                            custom_size: Some(Vec2::new(map.cell_size, map.cell_size)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 * map.cell_size - map.size.x/2.0,
                            y as f32 * map.cell_size - map.size.y/2.0,
                            0.0, // Sous le brouillard
                        ),
                        ..default()
                    },
                    DebugGrid,
                ));
            }
        }
    }

    // Dessiner les ressources
    for pos in &map_resources.energy_positions {
        spawn_resource(&mut commands, pos, Color::YELLOW);
    }
    for pos in &map_resources.mineral_positions {
        spawn_resource(&mut commands, pos, Color::BLUE);
    }
}

// Fonction helper pour spawn les ressources
fn spawn_resource(commands: &mut Commands, pos: &Vec2, color: Color) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0), // Sous le brouillard
            ..default()
        },
        Resource::Energy,
        DebugGrid,
    ));
}

fn generate_map(width: f32, height: f32, cell_size: f32, seed: u32) -> GameMap {
    let cols = (width / cell_size) as usize;
    let rows = (height / cell_size) as usize;
    let perlin = Perlin::new(seed);
    
    let mut obstacles = vec![vec![false; cols]; rows];
    
    for y in 0..rows {
        for x in 0..cols {
            let nx = x as f64 * 0.2;
            let ny = y as f64 * 0.2;
            let value = perlin.get([nx, ny]);
            obstacles[y][x] = value > 0.6;
        }
    }

    GameMap {
        size: Vec2::new(width, height),
        cell_size,
        obstacles,
        seed,
    }
}

fn generate_resources(map: &GameMap) -> MapResources {
    let mut rng = rand::thread_rng();
    let mut energy_positions = Vec::new();
    let mut mineral_positions = Vec::new();
    let mut scientific_sites = Vec::new();

    for _ in 0..15 {
        let pos = get_valid_position(map, &mut rng);
        energy_positions.push(pos);
    }

    for _ in 0..10 {
        let pos = get_valid_position(map, &mut rng);
        mineral_positions.push(pos);
    }

    for _ in 0..5 {
        let pos = get_valid_position(map, &mut rng);
        scientific_sites.push(pos);
    }

    MapResources {
        energy_positions,
        mineral_positions,
        scientific_sites,
    }
}

fn get_valid_position(map: &GameMap, rng: &mut impl Rng) -> Vec2 {
    loop {
        let x = rng.gen_range(-map.size.x/2.0..map.size.x/2.0);
        let y = rng.gen_range(-map.size.y/2.0..map.size.y/2.0);
        
        let grid_x = ((x + map.size.x/2.0) / map.cell_size) as usize;
        let grid_y = ((y + map.size.y/2.0) / map.cell_size) as usize;
        
        if grid_x < map.obstacles[0].len() && grid_y < map.obstacles.len() 
            && !map.obstacles[grid_y][grid_x] {
            return Vec2::new(x, y);
        }
    }
}

fn is_position_blocked(pos: Vec3, game_map: &GameMap) -> bool {
    let grid_x = ((pos.x + game_map.size.x/2.0) / game_map.cell_size) as i32;
    let grid_y = ((pos.y + game_map.size.y/2.0) / game_map.cell_size) as i32;
    
    // Vérifier si la position est hors des limites
    if grid_x < 0 || grid_x >= game_map.obstacles[0].len() as i32 
        || grid_y < 0 || grid_y >= game_map.obstacles.len() as i32 {
        return true;
    }
    
    // Vérifier les cellules environnantes pour une meilleure détection
    for dy in -1..=1 {
        for dx in -1..=1 {
            let check_x = grid_x + dx;
            let check_y = grid_y + dy;
            
            if check_x >= 0 && check_x < game_map.obstacles[0].len() as i32 
                && check_y >= 0 && check_y < game_map.obstacles.len() as i32 
                && game_map.obstacles[check_y as usize][check_x as usize] {
                return true;
            }
        }
    }
    
    false
}