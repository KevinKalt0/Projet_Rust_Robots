use simulation_robots::robots::*;
use bevy::prelude::*;

#[test]
fn test_is_position_blocked_should_return_true_for_obstacle() {
    let mut map = generate_map(100.0, 100.0, 10.0, 42);
    let x = (100.0 / 2.0 / 10.0) as usize;
    let y = (100.0 / 2.0 / 10.0) as usize;
    map.obstacles[y][x] = true;

    let pos = Vec3::new(0.0, 0.0, 0.0);
    assert!(is_position_blocked(pos, &map));
}

#[test]
fn test_generate_map_should_have_expected_size() {
    let map = generate_map(100.0, 50.0, 10.0, 123);
    assert_eq!(map.obstacles.len(), 5);
    assert_eq!(map.obstacles[0].len(), 10);
}

#[test]
fn test_find_path_a_star_returns_some_path() {
    let map = generate_map(100.0, 100.0, 10.0, 1);
    let start = Vec2::new(-40.0, -40.0);
    let end = Vec2::new(40.0, 40.0);
    let path = find_path_a_star(start, end, &map);
    assert!(path.is_some());
}
