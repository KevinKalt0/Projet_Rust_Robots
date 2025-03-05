use bevy::prelude::*;
use rand::prelude::*;

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

// Ressources
#[derive(Resource)]
struct DiscoveredCrystal(Option<Vec2>);

#[derive(Resource)]
struct ExploredMap {
    grid: Vec<Vec<bool>>,
    cell_size: f32,
}

#[derive(Resource)]
struct ExplorerState {
    current_direction: Vec2,
    time_until_change: f32,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        let explored_map = ExploredMap {
            grid: vec![vec![false; 40]; 30],
            cell_size: 20.0,
        };

        app.insert_resource(DiscoveredCrystal(None))
            .insert_resource(explored_map)
            .insert_resource(ExplorerState {
                current_direction: Vec2::new(1.0, 0.0),
                time_until_change: 2.0,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, (
                move_explorer,
                check_crystal_discovery,
                move_miners,
                update_explored_map,
            ).chain());
    }
}

fn setup(mut commands: Commands) {
    // Caméra
    commands.spawn(Camera2dBundle::default());

    // Base (Triangle bleu)
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

    // Explorateur (Triangle vert)
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

    // Mineurs (Carrés jaunes)
    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(30. * (i as f32 - 1.), -30., 0.),
                ..default()
            },
            Miner,
        ));
    }

    // Cristaux (Diamants violets)
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let x = rng.gen_range(-350.0..350.0);
        let y = rng.gen_range(-250.0..250.0);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::PURPLE,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, y, 0.),
                    rotation: Quat::from_rotation_z(45.0_f32.to_radians()),
                    ..default()
                },
                ..default()
            },
            Crystal,
        ));
    }
}

fn move_explorer(
    mut explorer_query: Query<&mut Transform, With<Explorer>>,
    time: Res<Time>,
    mut explorer_state: ResMut<ExplorerState>,
) {
    explorer_state.time_until_change -= time.delta_seconds();

    if explorer_state.time_until_change <= 0.0 {
        explorer_state.current_direction = Vec2::new(
            rand::thread_rng().gen_range(-1.0..=1.0),
            rand::thread_rng().gen_range(-1.0..=1.0),
        ).normalize();
        explorer_state.time_until_change = 2.0;
    }

    for mut transform in explorer_query.iter_mut() {
        let speed = 150.0;
        
        transform.translation.x += explorer_state.current_direction.x * speed * time.delta_seconds();
        transform.translation.y += explorer_state.current_direction.y * speed * time.delta_seconds();

        // Rotation du sprite dans la direction du mouvement
        transform.rotation = Quat::from_rotation_z(
            -explorer_state.current_direction.y.atan2(explorer_state.current_direction.x)
        );

        if transform.translation.x.abs() > 350.0 {
            explorer_state.current_direction.x *= -1.0;
            transform.translation.x = transform.translation.x.signum() * 350.0;
        }
        if transform.translation.y.abs() > 250.0 {
            explorer_state.current_direction.y *= -1.0;
            transform.translation.y = transform.translation.y.signum() * 250.0;
        }
    }
}

fn update_explored_map(
    explorer_query: Query<&Transform, With<Explorer>>,
    mut explored_map: ResMut<ExploredMap>,
) {
    for transform in explorer_query.iter() {
        let pos = transform.translation;
        let grid_x = ((pos.x + 400.0) / explored_map.cell_size) as usize;
        let grid_y = ((pos.y + 300.0) / explored_map.cell_size) as usize;
        
        if grid_x < explored_map.grid[0].len() && grid_y < explored_map.grid.len() {
            explored_map.grid[grid_y][grid_x] = true;
        }
    }
}

fn check_crystal_discovery(
    explorer_query: Query<&Transform, With<Explorer>>,
    crystal_query: Query<(Entity, &Transform), With<Crystal>>,
    mut commands: Commands,
    mut discovered_crystal: ResMut<DiscoveredCrystal>,
) {
    let explorer_transform = explorer_query.single();
    
    for (crystal_entity, crystal_transform) in crystal_query.iter() {
        let distance = explorer_transform.translation.distance(crystal_transform.translation);
        
        if distance < 30.0 {
            discovered_crystal.0 = Some(Vec2::new(
                crystal_transform.translation.x,
                crystal_transform.translation.y
            ));
            commands.entity(crystal_entity).despawn();
            println!("Crystal découvert! Les mineurs sont en route.");
            break;
        }
    }
}

fn move_miners(
    mut miner_query: Query<&mut Transform, (With<Miner>, Without<Base>)>,
    base_query: Query<&Transform, (With<Base>, Without<Miner>)>,
    discovered_crystal: Res<DiscoveredCrystal>,
    time: Res<Time>,
) {
    if let Some(crystal_pos) = discovered_crystal.0 {
        if let Ok(base_transform) = base_query.get_single() {
            let base_pos = base_transform.translation;
            
            for mut miner_transform in miner_query.iter_mut() {
                let current_pos = miner_transform.translation;
                let crystal_pos_3d = Vec3::new(crystal_pos.x, crystal_pos.y, 0.0);
                
                let target = if current_pos.distance(crystal_pos_3d) < 5.0 {
                    base_pos
                } else {
                    crystal_pos_3d
                };
                
                let direction = (target - current_pos).normalize();
                miner_transform.translation += direction * 400. * time.delta_seconds();

                // Rotation des mineurs dans la direction du mouvement
                miner_transform.rotation = Quat::from_rotation_z(
                    -direction.y.atan2(direction.x)
                );
            }
        }
    }
}

// Système de debug pour visualiser la grille d'exploration
fn debug_draw_grid(
    mut commands: Commands,
    query: Query<Entity, With<DebugGrid>>,
    explored_map: Res<ExploredMap>,
) {
    // Supprimer l'ancienne grille
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Dessiner la nouvelle grille
    for (y, row) in explored_map.grid.iter().enumerate() {
        for (x, &explored) in row.iter().enumerate() {
            if explored {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0.5, 0.5, 0.5, 0.2),
                            custom_size: Some(Vec2::new(
                                explored_map.cell_size,
                                explored_map.cell_size
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 * explored_map.cell_size - 400.0,
                            y as f32 * explored_map.cell_size - 300.0,
                            0.0
                        ),
                        ..default()
                    },
                    DebugGrid,
                ));
            }
        }
    }
}