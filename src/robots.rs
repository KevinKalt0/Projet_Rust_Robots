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
struct IdleMiner;


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

#[derive(Resource)]
struct DiscoveredResource {
    position: Option<Vec2>,
    is_being_collected: bool,
}

impl Default for DiscoveredResource {
    fn default() -> Self {
        Self {
            position: None,
            is_being_collected: false,
        }
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
    if discovered_resource.is_being_collected {
        return; // Explorer s’arrête SEULEMENT si les mineurs sont en route
    }

    for mut transform in explorer_query.iter_mut() {
        explorer_state.time_until_change -= time.delta_seconds();

        if explorer_state.time_until_change <= 0.0 {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            ).normalize_or_zero();
            explorer_state.time_until_change = 2.0;
        }

        let speed = 100.0;
        let mut new_pos = transform.translation;
        new_pos.x += explorer_state.current_direction.x * speed * time.delta_seconds();
        new_pos.y += explorer_state.current_direction.y * speed * time.delta_seconds();

        if is_position_blocked(new_pos, &game_map) || new_pos.x.abs() > 350.0 || new_pos.y.abs() > 250.0 {
            explorer_state.current_direction = Vec2::new(
                rand::thread_rng().gen_range(-1.0..=1.0),
                rand::thread_rng().gen_range(-1.0..=1.0),
            ).normalize_or_zero();
        } else {
            transform.translation = new_pos;
            transform.rotation = Quat::from_rotation_z(-explorer_state.current_direction.y.atan2(explorer_state.current_direction.x));
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
    if discovered_resource.position.is_some() {
        return;
    }

    for explorer_transform in explorer_query.iter() {
        let explorer_pos = explorer_transform.translation;

        for (res_transform, _res_type) in resources_query.iter() {
            if explorer_pos.distance(res_transform.translation) <= 10.0 {
                discovered_resource.position = Some(Vec2::new(
                    res_transform.translation.x,
                    res_transform.translation.y,
                ));
                discovered_resource.is_being_collected = true;
                println!("🔍 Ressource détectée, appel des mineurs !");
                return;
            }
        }
    }
}


fn move_miners(
    mut commands: Commands,
    mut q: ParamSet<(
        Query<(Entity, &mut Transform, Option<&mut MinerPath>), (With<Miner>, Without<IdleMiner>)>,
        Query<(Entity, &Transform), With<Resource>>,
    )>,
    mut discovered_resource: ResMut<DiscoveredResource>,
    time: Res<Time>,
    game_map: Res<GameMap>,
) {
    if !(discovered_resource.is_being_collected && discovered_resource.position.is_some()) {
        return;
    }

    let target_pos = discovered_resource.position.unwrap();
    let target_vec3 = Vec3::new(target_pos.x, target_pos.y, 0.0);

    let resources: Vec<(Entity, Vec3)> = q
        .p1()
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();

    for (entity, mut transform, maybe_path) in q.p0().iter_mut() {
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);

        let mut path = if let Some(p) = maybe_path {
            p.path.clone()
        } else {
            match find_path_a_star(current_pos, target_pos, &game_map) {
                Some(p) => {
                    commands.entity(entity).insert(MinerPath { path: p.clone() });
                    p
                }
                None => continue,
            }
        };

        if let Some(next_point) = path.first() {
            let direction = (*next_point - current_pos).normalize_or_zero();
            let speed = 120.0;
            let step = direction * time.delta_seconds() * speed;

            if current_pos.distance(*next_point) <= step.length() {
                path.remove(0);
                commands.entity(entity).insert(MinerPath { path });
            } else {
                transform.translation += Vec3::new(step.x, step.y, 0.0);
                transform.rotation = Quat::from_rotation_z(-direction.y.atan2(direction.x));
            }
        }

        if transform.translation.distance(target_vec3) <= 10.0 {
            for (res_entity, res_pos) in &resources {
                if res_pos.distance(target_vec3) <= 10.0 {
                    // 💣 Supprimer ressource
                    commands.entity(*res_entity).despawn();
                    // ⛏️ Stopper le mineur
                    commands.entity(entity).remove::<MinerPath>();
                    commands.entity(entity).insert(IdleMiner);
                    println!("💰 Ressource collectée !");
                    break;
                }
            }
        }
    }

    let still_exists = q
        .p1()
        .iter()
        .any(|(_, t)| t.translation.distance(target_vec3) <= 10.0);

    if !still_exists {
        discovered_resource.position = None;
        discovered_resource.is_being_collected = false;
        println!("✅ Ressource nettoyée → Explorateur peut repartir !");
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

    // =obstacles
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
use std::collections::{BinaryHeap, HashMap};
use ordered_float::OrderedFloat;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: (usize, usize),
    cost: OrderedFloat<f32>,
    priority: OrderedFloat<f32>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn find_path_a_star(start: Vec2, goal: Vec2, map: &GameMap) -> Option<Vec<Vec2>> {
    let to_grid = |pos: Vec2| (
        ((pos.x + map.size.x / 2.0) / map.cell_size) as usize,
        ((pos.y + map.size.y / 2.0) / map.cell_size) as usize
    );

    let (start_x, start_y) = to_grid(start);
    let (goal_x, goal_y) = to_grid(goal);

    let mut open = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();

    open.push(Node {
        pos: (start_x, start_y),
        cost: OrderedFloat(0.0),
        priority: OrderedFloat(0.0),
    });

    cost_so_far.insert((start_x, start_y), OrderedFloat(0.0));

    let directions = [
        (0, -1), (-1, 0), (1, 0), (0, 1),
        (-1, -1), (-1, 1), (1, -1), (1, 1),
    ];

    while let Some(current) = open.pop() {
        if current.pos == (goal_x, goal_y) {
            let mut path = Vec::new();
            let mut curr = current.pos;

            while curr != (start_x, start_y) {
                let world = Vec2::new(
                    curr.0 as f32 * map.cell_size - map.size.x / 2.0 + map.cell_size / 2.0,
                    curr.1 as f32 * map.cell_size - map.size.y / 2.0 + map.cell_size / 2.0,
                );
                path.push(world);
                curr = came_from[&curr];
            }

            path.reverse();
            return Some(path);
        }

        for (dx, dy) in directions {
            let new_x = current.pos.0 as i32 + dx;
            let new_y = current.pos.1 as i32 + dy;

            if new_x < 0 || new_y < 0 {
                continue;
            }

            let new_pos = (new_x as usize, new_y as usize);
            if new_pos.0 >= map.obstacles[0].len() || new_pos.1 >= map.obstacles.len() {
                continue;
            }

            if map.obstacles[new_pos.1][new_pos.0] {
                continue;
            }

            let new_cost = OrderedFloat(cost_so_far[&current.pos].0 + 1.0);
            if !cost_so_far.contains_key(&new_pos) || new_cost < cost_so_far[&new_pos] {
                cost_so_far.insert(new_pos, new_cost);
                let heuristic = ((goal_x as i32 - new_pos.0 as i32).abs()
                    + (goal_y as i32 - new_pos.1 as i32).abs()) as f32;
                let priority = OrderedFloat(new_cost.0 + heuristic);
                open.push(Node {
                    pos: new_pos,
                    cost: new_cost,
                    priority,
                });
                came_from.insert(new_pos, current.pos);
            }
        }
    }

    None
}

struct Map {
    width: u32,
    height: u32,
    fog: Vec<bool>,
}

#[derive(Component)]
struct MinerPath {
    path: Vec<Vec2>,
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