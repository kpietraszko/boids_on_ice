mod game_config;

use bevy::prelude::*;
use bevy::utils::default;
use game_config::GameConfig;

fn main() {
    App::new()
    .insert_resource(GameConfig {target_number_of_boids: 2})
    .insert_resource(AmbientLight{ color: Color::WHITE, brightness: 0.5 })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup)
    .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    println!("hello world!");
    commands.spawn_bundle(PerspectiveCameraBundle{
        transform: Transform::from_xyz(0.7*10.0,0.7,1.0*10.0).looking_at((0.0,0.0,0.0).into(), (0.0,1.0,0.0).into()),
        ..default()
    });

    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        }),
        ..default()
    });

    // some cube
    commands.spawn_bundle(PbrBundle{
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::PINK,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
    });
}