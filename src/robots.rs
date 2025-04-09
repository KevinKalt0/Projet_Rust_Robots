use bevy::prelude::*;
use rand::prelude::*;
use noise::{NoiseFn, Perlin};
use bevy::ecs::system::ParamSet;
use std::collections::{BinaryHeap, HashMap};
use ordered_float::OrderedFloat;
use bevy::time::TimerMode;

#[derive(Component)]
struct Explorer;

#[derive(Component)]
struct Miner;

#[derive(Component)]
struct Base;

#[derive(Component)]
struct IdleMiner;

#[derive(Component, Debug, Clone)]
pub enum Resource {
    Energy,
    Mineral,
}

#[derive(Component)]
struct DebugGrid;

#[derive(Resource)]
pub struct GameMap {
    pub size: Vec2,
    pub cell_size: f32,
    pub obstacles: Vec<Vec<bool>>,
    pub seed: u32,
}

#[derive(Resource, Default)]
pub struct MapResources {
    pub energy_positions: Vec<Vec2>,
    pub mineral_positions: Vec<Vec2>,
    pub scientific_sites: Vec<Vec2>,
}

#[derive(Resource, Default)]
pub struct DiscoveredResource {
    pub position: Option<Vec2>,
}

#[derive(Resource)]
struct ExploredZones {
    grid: Vec<Vec<bool>>,
    cell_size: f32,
}

impl Default for ExploredZones {
    fn default() -> Self {
        Self {
            grid: vec![vec![false; 80]; 60],
            cell_size: 10.0,
        }
    }
}

#[derive(Resource)]
pub struct ExplorerState {
    pub current_direction: Vec2,
    pub time_until_change: f32,
}

#[derive(Component)]
struct ReturningMiner;

#[derive(Resource)]
struct CollectionState {
    timer: Timer,
    resource_entity: Option<Entity>,
    collecting: bool,
    position: Option<Vec2>,
}

impl Default for CollectionState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            resource_entity: None,
            collecting: false,
            position: None,
        }
    }
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        let seed = rand::random::<u32>();
        let cell_size_astar = 20.0;
        let game_map = generate_map(800.0, 600.0, cell_size_astar, seed);

        let map_resources = MapResources {
            energy_positions: vec![
                Vec2::new(200.0, 150.0),
                Vec2::new(-200.0, 150.0),
                Vec2::new(0.0, -150.0),
                Vec2::new(150.0, 0.0),
                Vec2::new(-150.0, 0.0),
                Vec2::new(250.0, -100.0),
                Vec2::new(-250.0, 100.0),
            ],
            mineral_positions: vec![
                Vec2::new(-200.0, -150.0),
                Vec2::new(200.0, -150.0),
                Vec2::new(0.0, 150.0),
                Vec2::new(100.0, -50.0),
                Vec2::new(-100.0, 50.0),
                Vec2::new(150.0, 200.0),
                Vec2::new(-150.0, -200.0),
            ],
            scientific_sites: vec![],
        };

        app.insert_resource(game_map)
            .insert_resource(map_resources)
            .insert_resource(DiscoveredResource::default())
            .insert_resource(ExploredZones::default())
            .insert_resource(CollectionState::default())
            .insert_resource(ExplorerState {
                current_direction: Vec2::new(1.0, 0.0),
                time_until_change: 2.0,
            })
            .add_systems(Startup, (setup, debug_draw_map))
            .add_systems(
                Update,
                (
                    check_resource_discovery,
                    move_explorer,
                    move_miners,
                    update_explored_map,
                )
            );
    }
}

fn setup(
    mut commands: Commands,
    map_resources: Res<MapResources>,
    mut game_map: ResMut<GameMap>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Base,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(15.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 50., 0.),
            ..default()
        },
        Explorer,
    ));

    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.0),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                transform: Transform::from_xyz(30. * (i as f32 - 1.), -30., 0.),
                ..default()
            },
            Miner,
            IdleMiner,
        ));
    }

    clear_obstacles_around_resources(&mut game_map, &map_resources);

    for pos in &map_resources.energy_positions {
        spawn_persistent_resource(&mut commands, pos, Color::YELLOW, Resource::Energy);
    }
    
    for pos in &map_resources.mineral_positions {
        spawn_persistent_resource(&mut commands, pos, Color::BLUE, Resource::Mineral);
    }
}

pub fn clear_obstacles_around_resources(game_map: &mut GameMap, map_resources: &MapResources) {
    let clear_radius = 2;
    
    let clear_around_position = |map: &mut GameMap, pos: &Vec2| {
        let grid_x = ((pos.x + map.size.x/2.0) / map.cell_size) as i32;
        let grid_y = ((pos.y + map.size.y/2.0) / map.cell_size) as i32;
        
        for dy in -clear_radius..=clear_radius {
            for dx in -clear_radius..=clear_radius {
                let x = grid_x + dx;
                let y = grid_y + dy;
                
                if x >= 0 && x < map.obstacles[0].len() as i32 
                   && y >= 0 && y < map.obstacles.len() as i32 {
                    map.obstacles[y as usize][x as usize] = false;
                }
            }
        }
    };
    
    for pos in &map_resources.energy_positions {
        clear_around_position(game_map, pos);
    }
    
    for pos in &map_resources.mineral_positions {
        clear_around_position(game_map, pos);
    }
    
    for pos in &map_resources.scientific_sites {
        clear_around_position(game_map, pos);
    }
}

fn move_explorer(
    mut explorer_query: Query<&mut Transform, With<Explorer>>,
    time: Res<Time>,
    mut explorer_state: ResMut<ExplorerState>,
    discovered_resource: Res<DiscoveredResource>,
    game_map: Res<GameMap>,
    miners_query: Query<(), (With<Miner>, Without<IdleMiner>)>,
) {
    if discovered_resource.position.is_some() || !miners_query.is_empty() {
        return;
    }

    for mut transform in explorer_query.iter_mut() {
        explorer_state.time_until_change -= time.delta_seconds();

        if explorer_state.time_until_change <= 0.0 {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            )
            .normalize_or_zero();
            explorer_state.time_until_change = 2.0;
        }

        let speed = 100.0;
        let target_pos = transform.translation + Vec3::new(
            explorer_state.current_direction.x, 
            explorer_state.current_direction.y, 
            0.0
        ) * 50.0;
        
        let (new_pos, rotation) = move_entity_avoiding_obstacles(
            transform.translation, 
            target_pos, 
            speed, 
            time.delta_seconds(), 
            &game_map
        );
        
        if new_pos == transform.translation {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            )
            .normalize_or_zero();
            explorer_state.time_until_change = 1.0;
        } else {
            transform.translation = new_pos;
            transform.rotation = rotation;
            
            let world_bounds_x = game_map.size.x / 2.0 - 10.0;
            let world_bounds_y = game_map.size.y / 2.0 - 10.0;
            
            if transform.translation.x.abs() > world_bounds_x || transform.translation.y.abs() > world_bounds_y {
                explorer_state.current_direction = -explorer_state.current_direction;
                explorer_state.time_until_change = 1.0;
            }
        }
    }
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
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<&Transform, With<Explorer>>,
        Query<(Entity, &Transform, &Resource)>
    )>,
    mut discovered_resource: ResMut<DiscoveredResource>,
    idle_miners_query: Query<Entity, With<IdleMiner>>,
    active_miners: Query<(), (With<Miner>, Without<IdleMiner>)>,
) {
    if discovered_resource.position.is_some() || !active_miners.is_empty() {
        return;
    }

    if idle_miners_query.is_empty() {
        return;
    }

    if let Ok(explorer_transform) = param_set.p0().get_single() {
        let explorer_pos = explorer_transform.translation;
        
        let mut closest_resource: Option<(Vec2, f32)> = None;
        
        for (_, res_transform, _) in param_set.p1().iter() {
            let distance = explorer_pos.distance(res_transform.translation);
            if distance < 35.0 {
                if closest_resource.is_none() || distance < closest_resource.unwrap().1 {
                    closest_resource = Some((
                        Vec2::new(res_transform.translation.x, res_transform.translation.y), 
                        distance
                    ));
                }
            }
        }
        
        if let Some((pos, dist)) = closest_resource {
            println!("🎯 Ressource détectée à {:?} (distance: {:.1})!", pos, dist);
            discovered_resource.position = Some(pos);
            
            let mut miners_activated = 0;
            for miner_entity in idle_miners_query.iter() {
                println!("🚀 Activation du mineur {:?}", miner_entity);
                commands.entity(miner_entity).remove::<IdleMiner>();
                miners_activated += 1;
            }
            println!("✅ Activé {} mineurs pour collecter la ressource", miners_activated);
        }
    }
}

fn move_miners(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &mut Transform), (With<Miner>, Without<IdleMiner>, Without<ReturningMiner>)>,
        Query<(Entity, &Transform, &Resource)>,
        Query<&Transform, With<Base>>,
        Query<(Entity, &mut Transform), With<ReturningMiner>>
    )>,
    mut discovered_resource: ResMut<DiscoveredResource>,
    mut collection_state: ResMut<CollectionState>,
    time: Res<Time>,
    game_map: Res<GameMap>,
) {
    let base_pos = if let Ok(base_transform) = param_set.p2().get_single() {
        base_transform.translation
    } else {
        Vec3::ZERO
    };
    
    let mut miners_reached_base = Vec::new();
    
    for (entity, mut transform) in param_set.p3().iter_mut() {
        let current_pos = transform.translation;
        
        if current_pos.distance_squared(base_pos) < 15.0 * 15.0 {
            println!("🏠 Mineur {:?} est revenu à la base", entity);
            miners_reached_base.push(entity);
            continue;
        }
        
        let (new_pos, rotation) = move_entity_avoiding_obstacles(
            current_pos, 
            base_pos, 
            120.0, 
            time.delta_seconds(), 
            &game_map
        );
        
        transform.translation = new_pos;
        transform.rotation = rotation;
    }
    
    for entity in miners_reached_base {
        println!("🔄 Mineur {:?} est maintenant inactif", entity);
        commands.entity(entity).remove::<ReturningMiner>().insert(IdleMiner);
    }
    
    if collection_state.collecting {
        collection_state.timer.tick(time.delta());
        
        if collection_state.timer.finished() {
            println!("⏱️ Temps de collecte terminé!");
            
            if let Some(entity) = collection_state.resource_entity {
                println!("🗑️ Suppression de la ressource {:?}", entity);
                commands.entity(entity).despawn();
            }
            
            collection_state.collecting = false;
            collection_state.resource_entity = None;
            collection_state.position = None;
            discovered_resource.position = None;
            
            for (entity, _) in param_set.p0().iter() {
                println!("🏠 Mineur {:?} retourne à la base", entity);
                commands.entity(entity).insert(ReturningMiner);
            }
            
            return;
        }
    }
    
    if discovered_resource.position.is_none() {
        return;
    }

    let target_pos = discovered_resource.position.unwrap();
    let target_vec3 = Vec3::new(target_pos.x, target_pos.y, 0.0);
    
    if collection_state.position != Some(target_pos) {
        collection_state.position = Some(target_pos);
    }
    
    if !collection_state.collecting && collection_state.resource_entity.is_none() {
        for (entity, transform, _) in param_set.p1().iter() {
            if transform.translation.distance_squared(target_vec3) < 20.0 * 20.0 {
                collection_state.resource_entity = Some(entity);
                break;
            }
        }
    }
    
    if collection_state.resource_entity.is_none() {
        println!("🔍 Aucune ressource trouvée à la position cible, réinitialisation");
        discovered_resource.position = None;
        collection_state.position = None;
        
        for (entity, _) in param_set.p0().iter() {
            commands.entity(entity).insert(ReturningMiner);
        }
        return;
    }
    
    let mut miners_at_resource = 0;
    let mut total_miners = 0;
    
    for (_, mut transform) in param_set.p0().iter_mut() {
        total_miners += 1;
        let current_pos = transform.translation;
        
        if current_pos.distance_squared(target_vec3) < 15.0 * 15.0 {
            miners_at_resource += 1;
            continue;
        }
        
        let (new_pos, rotation) = move_entity_avoiding_obstacles(
            current_pos, 
            target_vec3, 
            120.0, 
            time.delta_seconds(), 
            &game_map
        );
        
        transform.translation = new_pos;
        transform.rotation = rotation;
    }
    
    if miners_at_resource > 0 && !collection_state.collecting {
        println!("⏱️ Début de la collecte! {}/{} mineurs sont arrivés", miners_at_resource, total_miners);
        collection_state.collecting = true;
        collection_state.timer = Timer::from_seconds(2.0, TimerMode::Once);
    }
}

pub fn move_entity_avoiding_obstacles(
    current_pos: Vec3,
    target_pos: Vec3,
    speed: f32,
    delta_time: f32,
    game_map: &GameMap,
) -> (Vec3, Quat) {
    let direction = (target_pos - current_pos).normalize_or_zero();
    
    if direction == Vec3::ZERO {
        return (current_pos, Quat::IDENTITY);
    }
    
    let straight_move = direction * speed * delta_time;
    let next_pos = current_pos + straight_move;
    
    if !is_position_blocked(next_pos, game_map) {
        let rotation = Quat::from_rotation_z(-direction.y.atan2(direction.x));
        return (next_pos, rotation);
    }
    
    let angles = [
        0.3, -0.3,
        0.6, -0.6,
        1.0, -1.0,
        1.5, -1.5,
        2.0, -2.0,
        2.5, -2.5
    ];
    
    for angle in angles {
        let test_direction = rotate_vec2(Vec2::new(direction.x, direction.y), angle).extend(0.0);
        let test_pos = current_pos + test_direction * speed * delta_time;
        
        if !is_position_blocked(test_pos, game_map) {
            let rotation = Quat::from_rotation_z(-test_direction.y.atan2(test_direction.x));
            return (test_pos, rotation);
        }
    }
    
    (current_pos, Quat::from_rotation_z(-direction.y.atan2(direction.x)))
}

fn debug_draw_map(
    mut commands: Commands,
    query: Query<Entity, With<DebugGrid>>,
    map: Res<GameMap>,
    map_resources: Res<MapResources>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

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

    for pos in &map_resources.energy_positions {
        spawn_resource(&mut commands, pos, Color::YELLOW, Resource::Energy);
    }

    for pos in &map_resources.mineral_positions {
        spawn_resource(&mut commands, pos, Color::BLUE, Resource::Mineral);
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

fn spawn_persistent_resource(commands: &mut Commands, pos: &Vec2, color: Color, resource_type: Resource) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0),
            ..default()
        },
        resource_type,
    ));
}

pub fn generate_map(width: f32, height: f32, cell_size: f32, seed: u32) -> GameMap {
    let cols = (width / cell_size) as usize;
    let rows = (height / cell_size) as usize;
    let perlin = Perlin::new(seed);
    
    let mut obstacles = vec![vec![false; cols]; rows];
    
    for y in 0..rows {
        for x in 0..cols {
            let nx = x as f64 * 0.07;
            let ny = y as f64 * 0.07;
            let value = perlin.get([nx, ny]);
            
            obstacles[y][x] = value > 0.55;
            
            if !obstacles[y][x] && rand::random::<f32>() < 0.05 {
                obstacles[y][x] = true;
            }
        }
    }
    
    if cols >= 15 && rows >= 15 {
        for _ in 0..5 {
            let start_x = rand::thread_rng().gen_range(5..cols-5);
            let start_y = rand::thread_rng().gen_range(5..rows-5);
            let length = rand::thread_rng().gen_range(3..10);
            let horizontal = rand::random::<bool>();
            
            for i in 0..length {
                if horizontal {
                    if start_x + i < cols {
                        obstacles[start_y][start_x + i] = true;
                    }
                } else {
                    if start_y + i < rows {
                        obstacles[start_y + i][start_x] = true;
                    }
                }
            }
        }
    }
    
    let center_x = cols / 2;
    let center_y = rows / 2;
    let safe_radius = 5;
    
    for y in center_y.saturating_sub(safe_radius)..std::cmp::min(center_y + safe_radius, rows) {
        for x in center_x.saturating_sub(safe_radius)..std::cmp::min(center_x + safe_radius, cols) {
            obstacles[y][x] = false;
        }
    }
    
    GameMap {
        size: Vec2::new(width, height),
        cell_size,
        obstacles,
        seed,
    }
}

pub fn is_position_blocked(pos: Vec3, game_map: &GameMap) -> bool {
    let grid_x = ((pos.x + game_map.size.x/2.0) / game_map.cell_size) as i32;
    let grid_y = ((pos.y + game_map.size.y/2.0) / game_map.cell_size) as i32;
    
    if grid_x < 0 || grid_x >= game_map.obstacles[0].len() as i32 
        || grid_y < 0 || grid_y >= game_map.obstacles.len() as i32 {
        return true;
    }
    
    if game_map.obstacles[grid_y as usize][grid_x as usize] {
        return true;
    }
    
    let radius = 0.7;
    for dy in [-radius, 0.0, radius] {
        for dx in [-radius, 0.0, radius] {
            if dx == 0.0 && dy == 0.0 {
                continue;
            }
            
            let check_x = ((pos.x + dx * game_map.cell_size + game_map.size.x/2.0) / game_map.cell_size) as i32;
            let check_y = ((pos.y + dy * game_map.cell_size + game_map.size.y/2.0) / game_map.cell_size) as i32;
            
            if check_x >= 0 && check_x < game_map.obstacles[0].len() as i32 
                && check_y >= 0 && check_y < game_map.obstacles.len() as i32 
                && game_map.obstacles[check_y as usize][check_x as usize] {
                return true;
            }
        }
    }
    
    false
}

pub fn rotate_vec2(v: Vec2, angle_rad: f32) -> Vec2 {
    let (s, c) = angle_rad.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}
