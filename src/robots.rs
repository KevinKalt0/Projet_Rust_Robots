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

#[derive(Component, Debug)]
pub enum Resource {
    Energy,
    Mineral,
}

#[derive(Component)]
struct DebugGrid;

#[derive(Resource)]
struct GameMap {
    size: Vec2,
    cell_size: f32,
    obstacles: Vec<Vec<bool>>,
    seed: u32,
}

#[derive(Resource)]
pub struct MapResources {
    pub energy_positions: Vec<Vec2>,
    pub mineral_positions: Vec<Vec2>,
    pub scientific_sites: Vec<Vec2>,
}

impl Default for MapResources {
    fn default() -> Self {
        Self {
            energy_positions: Vec::new(),
            mineral_positions: Vec::new(),
            scientific_sites: Vec::new(),
        }
    }
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

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        let seed = rand::random::<u32>();
        let game_map = generate_map(800.0, 600.0, 10.0, seed);
        
        let map_resources = MapResources {
            energy_positions: vec![
                Vec2::new(200.0, 150.0),
                Vec2::new(-200.0, 150.0),
                Vec2::new(0.0, -150.0),
                Vec2::new(150.0, 0.0),     // Nouveaux points jaunes
                Vec2::new(-150.0, 0.0),
                Vec2::new(250.0, -100.0),
                Vec2::new(-250.0, 100.0),
            ],
            mineral_positions: vec![
                Vec2::new(-200.0, -150.0),
                Vec2::new(200.0, -150.0),
                Vec2::new(0.0, 150.0),
                Vec2::new(100.0, -50.0),   // Nouveaux points bleus
                Vec2::new(-100.0, 50.0),
                Vec2::new(150.0, 200.0),
                Vec2::new(-150.0, -200.0),
            ],
            scientific_sites: vec![
                Vec2::new(100.0, 100.0),
                Vec2::new(-100.0, -100.0),
            ],
        };

        app.insert_resource(game_map)
            .insert_resource(map_resources)
            .insert_resource(DiscoveredResource::default())
            .insert_resource(ExplorerState {
                current_direction: Vec2::new(1.0, 0.0),
                time_until_change: 2.0,
            })
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    move_explorer,
                    check_resource_discovery,
                    move_miners,
                    debug_draw_map,
                ).chain()
            );
    }
}

fn setup(
    mut commands: Commands,
) {
    // Caméra
    commands.spawn(Camera2dBundle::default());

    // Base (Bleu) - Taille réduite
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(30.0, 30.0)), // Réduit de 40 à 30
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Base,
    ));

    // Explorateur (Vert) - Taille réduite
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(15.0, 20.0)), // Réduit de 20x30 à 15x20
                ..default()
            },
            transform: Transform::from_xyz(0., 50., 0.),
            ..default()
        },
        Explorer,
    ));

    // Mineurs (Orange) - Taille réduite
    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.0),
                    custom_size: Some(Vec2::new(10.0, 10.0)), // Réduit de 15 à 10
                    ..default()
                },
                transform: Transform::from_xyz(30. * (i as f32 - 1.), -30., 0.),
                ..default()
            },
            Miner,
        ));
    }
}

fn move_explorer(
    mut explorer_query: Query<&mut Transform, With<Explorer>>,
    time: Res<Time>,
    mut explorer_state: ResMut<ExplorerState>,
    discovered_resource: Res<DiscoveredResource>,
    game_map: Res<GameMap>,
) {
    if discovered_resource.position.is_some() {
        return;
    }

    for mut transform in explorer_query.iter_mut() {
        explorer_state.time_until_change -= time.delta_seconds();

        if explorer_state.time_until_change <= 0.0 {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            ).normalize();
            explorer_state.time_until_change = 2.0;
        }

        let speed = 100.0; // Réduit de 150 à 100

        let mut new_pos = transform.translation;
        new_pos.x += explorer_state.current_direction.x * speed * time.delta_seconds();
        new_pos.y += explorer_state.current_direction.y * speed * time.delta_seconds();

        if is_position_blocked(new_pos, &game_map) || new_pos.x.abs() > 350.0 || new_pos.y.abs() > 250.0 {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            ).normalize();
            
            new_pos = transform.translation;
            new_pos.x += explorer_state.current_direction.x * speed * time.delta_seconds();
            new_pos.y += explorer_state.current_direction.y * speed * time.delta_seconds();
        }

        new_pos.x = new_pos.x.clamp(-350.0, 350.0);
        new_pos.y = new_pos.y.clamp(-250.0, 250.0);
        transform.translation = new_pos;
        transform.rotation = Quat::from_rotation_z(
            -explorer_state.current_direction.y.atan2(explorer_state.current_direction.x)
        );
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
    if discovered_resource.position.is_some() {
        return;
    }

    for explorer_transform in explorer_query.iter() {
        let explorer_pos = explorer_transform.translation;
        
        for (resource_transform, _resource_type) in resources_query.iter() {
            let distance = explorer_pos.distance(resource_transform.translation);
            
            if distance < 50.0 {
                discovered_resource.position = Some(Vec2::new(
                    resource_transform.translation.x,
                    resource_transform.translation.y,
                ));
                break;
            }
        }
    }
}

fn move_miners(
    mut query_set: ParamSet<(
        Query<&mut Transform, (With<Miner>, Without<Base>)>,
        Query<(Entity, &Transform, &Resource)>,
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

        let mut miners = query_set.p0();
        for mut miner_transform in miners.iter_mut() {
            let current_pos = miner_transform.translation;
            let direction = (resource_pos_3d - current_pos).normalize();
            let speed = 120.0;
            let step_size = speed * time.delta_seconds();

            if current_pos.distance(resource_pos_3d) < 15.0 {
                if let Some(entity) = resource_to_remove {
                    commands.entity(entity).despawn();
                    discovered_resource.position = None;
                    println!("Ressource collectée!");
                    return;
                }
            }

            // Amélioration du système de contournement d'obstacles
            let mut best_direction = direction;
            let mut min_obstacle_count = i32::MAX;
            let test_angles: [f32; 13] = [
                0.0, 15.0, -15.0, 30.0, -30.0, 45.0, -45.0, 
                60.0, -60.0, 90.0, -90.0, 120.0, -120.0
            ];

            for angle in test_angles {
                let test_direction = rotate_vector(direction, angle.to_radians());
                let mut obstacle_count = 0;
                let mut can_move = true;

                // Vérifier plusieurs points le long de la trajectoire
                for i in 1..=5 {
                    let test_pos = current_pos + test_direction * (step_size * 0.5 * i as f32);
                    if is_position_blocked(test_pos, &game_map) {
                        obstacle_count += 1;
                        if i <= 2 { // Bloquer les mouvements qui mènent directement à un obstacle
                            can_move = false;
                            break;
                        }
                    }
                }

                if can_move && obstacle_count < min_obstacle_count {
                    min_obstacle_count = obstacle_count;
                    best_direction = test_direction;
                }
            }

            let new_pos = current_pos + best_direction * step_size;
            if !is_position_blocked(new_pos, &game_map) {
                miner_transform.translation = new_pos;
                miner_transform.rotation = Quat::from_rotation_z(
                    -best_direction.y.atan2(best_direction.x)
                );
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
                            0.0,
                        ),
                        ..default()
                    },
                    DebugGrid,
                ));
            }
        }
    }

    // Dessiner les ressources d'énergie (jaunes)
    for pos in &map_resources.energy_positions {
        spawn_resource(&mut commands, pos, Color::YELLOW, Resource::Energy);
    }

    // Dessiner les ressources minérales (bleues)
    for pos in &map_resources.mineral_positions {
        spawn_resource(&mut commands, pos, Color::BLUE, Resource::Mineral);
    }

    // Dessiner les sites scientifiques (verts)
    for pos in &map_resources.scientific_sites {
        spawn_resource(&mut commands, pos, Color::GREEN, Resource::Energy);
    }
}

fn spawn_resource(commands: &mut Commands, pos: &Vec2, color: Color, resource_type: Resource) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0),
            ..default()
        },
        resource_type,
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

struct Map {
    width: u32,
    height: u32,
    fog: Vec<bool>, // true means fog, false means clear
}

impl Map {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fog: vec![true; (width * height) as usize],
        }
    }

    fn clear_fog(&mut self) {
        for cell in self.fog.iter_mut() {
            *cell = false;
        }
    }
}

fn main() {
    let mut map = Map::new(10, 10);
    map.clear_fog();
    // Now the map has no fog
}