mod game_config;

use bevy::prelude::*;
use bevy::utils::default;
use game_config::GameConfig;

#[derive(Component)]
struct Velocity(Vec2);

fn main() {
    App::new()
    .insert_resource(GameConfig {target_number_of_boids: 2})
    .insert_resource(AmbientLight{ color: Color::ALICE_BLUE, brightness: 0.1 })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup)
    .add_system(apply_velocity_system)
    .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    println!("hello world!");
    commands.spawn_bundle(PerspectiveCameraBundle{
        transform: Transform::from_xyz(0.0,10.0,1.0*10.0).looking_at((0.0,0.0,0.0).into(), (0.0,1.0,0.0).into()),
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
        ..default()
    });

    // some cube
    commands.spawn_bundle(PbrBundle{
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
    }).insert(Velocity(Vec2::new(0.0,0.0)));

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
            rotation: Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4, 0.0),
            ..default()
        },
        ..default()
    });
}

fn apply_velocity_system(mut boids_query: Query<(&Velocity, &mut Transform)>, time: Res<Time>){
    let delta_time = time.delta_seconds();
    for (velocity, mut boid_transform) in boids_query.iter_mut() {
        boid_transform.translation.x += 1.5 *delta_time;
        boid_transform.translation.x = boid_transform.translation.x % 10.0;
    }
}