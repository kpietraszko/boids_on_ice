mod game_config;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::utils::default;
use game_config::GameConfig;
use rand::distributions::{Distribution, Uniform};

#[derive(Component)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .insert_resource(GameConfig {
            target_number_of_boids: 200,
        })
        .insert_resource(AmbientLight {
            color: Color::ALICE_BLUE,
            brightness: 0.1,
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(apply_velocity_system)
        .run();
}

fn setup(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("hello world!");
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 10.0, 1.0 * 10.0)
            .looking_at((0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into()),
        ..default()
    });

    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.1,
            ..default()
        }),
        transform: Transform::default().with_scale((1.5,1.0,1.5).into()),
        ..default()
    });

    // spawn boids
    let velocity_between = Uniform::from(-1.0..1.0f32);
    let mut rng = rand::thread_rng();

    for boid in 0..game_config.target_number_of_boids {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.5, 0.0).with_scale((0.15,1.0,0.15).into()),
                ..default()
            })
            .insert(Velocity(Vec2::new(
                velocity_between.sample(&mut rng),
                velocity_between.sample(&mut rng),
            )));
    }

    // directional light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            illuminance: 100000.0 / 7.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                -std::f32::consts::FRAC_PI_4,
                0.0,
            ),
            ..default()
        },
        ..default()
    });
}

fn apply_velocity_system(mut boids_query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut boid_transform) in boids_query.iter_mut() {
        let Velocity(velocity_value) = velocity;
        let velocity_this_frame = *velocity_value * time.delta_seconds();
        let new_translation = Vec3::new(velocity_this_frame.x, 0.0, velocity_this_frame.y);
        boid_transform.translation += new_translation;
        boid_transform.rotation = Quat::from_rotation_arc(-Vec3::Z, Vec3::new(velocity_this_frame.x, 0.0, velocity_this_frame.y).normalize());
    }
}
