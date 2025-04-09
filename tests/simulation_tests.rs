use bevy::prelude::*;
use simulation_robots::robots::{
    GameMap, MapResources, 
    is_position_blocked, rotate_vec2, 
    move_entity_avoiding_obstacles, 
    generate_map, clear_obstacles_around_resources
};

// Test de la fonction is_position_blocked
#[test]
fn test_is_position_blocked() {
    let mut obstacles = vec![vec![false; 10]; 10];
    obstacles[5][5] = true;
    obstacles[5][6] = true;
    obstacles[6][5] = true;
    
    let game_map = GameMap {
        size: Vec2::new(100.0, 100.0),
        cell_size: 10.0,
        obstacles,
        seed: 42,
    };
    
    assert_eq!(is_position_blocked(Vec3::new(-45.0, -45.0, 0.0), &game_map), false);
    assert_eq!(is_position_blocked(Vec3::new(5.0, 5.0, 0.0), &game_map), true);
}

// Test de la fonction rotate_vec2
#[test]
fn test_rotate_vec2() {
    let v = Vec2::new(1.0, 0.0);
    
    let rotated = rotate_vec2(v, std::f32::consts::PI / 2.0);
    assert!((rotated.x - 0.0).abs() < 0.001);
    assert!((rotated.y - 1.0).abs() < 0.001);
    
    let rotated = rotate_vec2(v, std::f32::consts::PI);
    assert!((rotated.x + 1.0).abs() < 0.001);
    assert!((rotated.y - 0.0).abs() < 0.001);
}

// Test de la fonction move_entity_avoiding_obstacles
#[test]
fn test_movement_with_obstacles() {
    let mut obstacles = vec![vec![false; 10]; 10];
    obstacles[5][5] = true;
    
    let game_map = GameMap {
        size: Vec2::new(100.0, 100.0),
        cell_size: 10.0,
        obstacles,
        seed: 42,
    };
    
    let current_pos = Vec3::new(0.0, 0.0, 0.0);
    let target_pos = Vec3::new(10.0, 0.0, 0.0);
    let (new_pos, _) = move_entity_avoiding_obstacles(current_pos, target_pos, 10.0, 1.0, &game_map);
    
    assert_ne!(new_pos, current_pos);
    
    assert!(!is_position_blocked(new_pos, &game_map));
}

// Test de la fonction generate_map
#[test]
fn test_generate_map() {
    let map = generate_map(800.0, 600.0, 20.0, 42);
    
    assert_eq!(map.size.x, 800.0);
    assert_eq!(map.size.y, 600.0);
    assert_eq!(map.cell_size, 20.0);
    
    let center_x = map.obstacles[0].len() / 2;
    let center_y = map.obstacles.len() / 2;
    assert_eq!(map.obstacles[center_y][center_x], false);
    
    assert_eq!(map.obstacles[0].len(), (800.0 / 20.0) as usize);
    assert_eq!(map.obstacles.len(), (600.0 / 20.0) as usize);
}

// Test de la fonction clear_obstacles_around_resources
#[test]
fn test_clear_obstacles() {
    let mut obstacles = vec![vec![true; 10]; 10];
    let mut game_map = GameMap {
        size: Vec2::new(100.0, 100.0),
        cell_size: 10.0,
        obstacles,
        seed: 42,
    };
    
    let map_resources = MapResources {
        energy_positions: vec![Vec2::new(0.0, 0.0)],
        mineral_positions: vec![Vec2::new(20.0, 20.0)],
        scientific_sites: vec![],
    };
    
    clear_obstacles_around_resources(&mut game_map, &map_resources);
    
    let grid_x1 = ((0.0 + game_map.size.x/2.0) / game_map.cell_size) as usize;
    let grid_y1 = ((0.0 + game_map.size.y/2.0) / game_map.cell_size) as usize;
    
    assert!(grid_x1 < game_map.obstacles[0].len(), "grid_x1 out of bounds: {} >= {}", grid_x1, game_map.obstacles[0].len());
    assert!(grid_y1 < game_map.obstacles.len(), "grid_y1 out of bounds: {} >= {}", grid_y1, game_map.obstacles.len());
    
    assert_eq!(game_map.obstacles[grid_y1][grid_x1], false);
    
    let grid_x2 = ((20.0 + game_map.size.x/2.0) / game_map.cell_size) as usize;
    let grid_y2 = ((20.0 + game_map.size.y/2.0) / game_map.cell_size) as usize;
    
    assert!(grid_x2 < game_map.obstacles[0].len(), "grid_x2 out of bounds: {} >= {}", grid_x2, game_map.obstacles[0].len());
    assert!(grid_y2 < game_map.obstacles.len(), "grid_y2 out of bounds: {} >= {}", grid_y2, game_map.obstacles.len());
    
    assert_eq!(game_map.obstacles[grid_y2][grid_x2], false);
}