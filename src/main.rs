mod game_config;

use std::f32::consts::PI;

use approx::abs_diff_eq;
use bevy::render::camera::Projection;
use bevy::utils::default;
use bevy::{math::Vec3Swizzles, prelude::*};
// use bevy_inspector_egui::WorldInspectorPlugin;
use game_config::GameConfig;
use rand::distributions::{Distribution, Uniform};

#[cfg(target_family = "wasm")]
use bevy_web_fullscreen::FullViewportPlugin;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct PreviousVelocity(Vec2);

fn main() {
    let mut app = App::new();
    app.insert_resource(GameConfig {
        target_number_of_boids: 200,
        view_range: 1.0,
    })
    .insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.2,
    })
    .add_plugins(DefaultPlugins)
    // .add_plugin(WorldInspectorPlugin::new())
    .add_startup_system(setup)
    .add_system(boids_rules_system.before(apply_velocity_system))
    .add_system(apply_velocity_system)
    .add_system(camera_fit_system.after(apply_velocity_system));

    #[cfg(target_family = "wasm")]
    app.add_plugin(FullViewportPlugin);

    app.run();
}

fn setup(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 1.0 * 10.0)
            .looking_at((0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into()),
        ..default()
    });

    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::ALICE_BLUE,
            perceptual_roughness: 0.0,
            ..default()
        }),
        transform: Transform::default().with_scale((40.0, 1.0, 40.0).into()),
        ..default()
    });

    // spawn boids
    let velocity_between = Uniform::from(-0.1..0.1f32); //Uniform::from(-1.0..1.0f32);
    let position_between = Uniform::from(-5.0..5.0f32);
    let mut rng = rand::thread_rng();

    for _ in 0..game_config.target_number_of_boids {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::YELLOW_GREEN,
                    ..default()
                }),
                transform: Transform::from_xyz(
                    position_between.sample(&mut rng),
                    0.4,
                    position_between.sample(&mut rng),
                )
                .with_scale((0.15, 0.8, 0.15).into()),
                ..default()
            })
            .insert(Velocity(Vec2::new(
                velocity_between.sample(&mut rng),
                velocity_between.sample(&mut rng),
            )))
            .insert(PreviousVelocity(Vec2::default()));
    }

    // directional light
    const HALF_SIZE: f32 = 30.0;
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

fn boids_rules_system(
    mut boids_query: Query<(&mut Velocity, &Transform, &mut PreviousVelocity)>,
    game_config: Res<GameConfig>,
) {
    let view_range_sq = game_config.view_range * game_config.view_range;
    let separation_distance_sq = 0.4f32.powf(2.0);

    // Vec<(Velocity,translation)>
    let all_boids: Vec<(Vec2, Vec3)> = boids_query
        .iter()
        .map(|b| (b.0 .0, b.1.translation))
        .collect();

    // let max_z = all_boids.iter().map(|b| b.1.z).reduce(f32::max).unwrap();

    for (mut velocity, boid_transform, mut previous_velocity) in boids_query.iter_mut() {
        previous_velocity.0 = velocity.0;
        let this_boid_pos = boid_transform.translation;
        let mut sum_of_other_boids_positions = Vec2::default();
        let mut boids_in_view_range = 0;
        let mut separation_vector = Vec2::default();
        let mut sum_of_other_boids_velocities = Vec2::default();
        for other_boid in all_boids.iter() {
            let other_boid_pos = (*other_boid).1;
            if this_boid_pos == other_boid_pos {
                // naive, and compares floats, but seems to work
                continue;
            }

            let distance_sq = this_boid_pos.distance_squared(other_boid_pos);
            // dbg!(boid_transform.forward().angle_between(other_boid_pos - this_boid_pos));

            if distance_sq > view_range_sq
                || boid_transform
                    .forward()
                    .angle_between(other_boid_pos - this_boid_pos)
                    > (2.0 * PI) / 3.0
            {
                //PI/2.0{
                // print!("Behind me");
                continue;
            }
            sum_of_other_boids_positions += other_boid_pos.xz();
            sum_of_other_boids_velocities += (*other_boid).0;
            boids_in_view_range += 1;

            if distance_sq > separation_distance_sq {
                continue;
            }

            separation_vector -= other_boid_pos.xz() - this_boid_pos.xz();
        }

        // bounds
        velocity.0 += get_velocity_away_from_walls(this_boid_pos, 0.1, -10.0, 10.0, -10.0, 10.0);

        if boids_in_view_range != 0 {
            // cohesion
            velocity.0 += (sum_of_other_boids_positions / (boids_in_view_range as f32)
                - this_boid_pos.xz())
                / 100.0;

            // separation
            velocity.0 += separation_vector;

            // alignment
            velocity.0 += (sum_of_other_boids_velocities / (boids_in_view_range as f32)) / 8.0;
        }

        const MAX_SPEED: f32 = 3.0; //1.0;
        let speed = velocity.0.length();
        if speed > MAX_SPEED {
            let velocity_normalized = velocity.0.normalize();
            velocity.0 = velocity_normalized * MAX_SPEED;
        }
    }
}

fn apply_velocity_system(
    mut boids_query: Query<(&Velocity, &mut Transform, &PreviousVelocity)>,
    time: Res<Time>,
) {
    // const BOID_HEIGHT: f32 = 0.8;
    for (velocity, mut boid_transform, previous_velocity) in boids_query.iter_mut() {
        let Velocity(velocity_value) = velocity;
        if velocity_value.is_nan() {
            panic!("Velocity is NaN");
        }
        let velocity_this_frame = *velocity_value * time.delta_seconds();
        let velocity_this_frame_3d = Vec3::new(velocity_this_frame.x, 0.0, velocity_this_frame.y);
        boid_transform.translation += velocity_this_frame_3d;
        let new_translation = boid_transform.translation;

        boid_transform.rotation = Quat::from_rotation_arc(
            -Vec3::Z,
            Vec3::new(velocity_this_frame.x, 0.0, velocity_this_frame.y).normalize(),
        );

        let acceleration = (*velocity_value - previous_velocity.0) * time.delta_seconds();
        if abs_diff_eq!(acceleration.length(), 0.0) {
            continue;
        }
        let rotation_axis = Vec3::new(acceleration.x, 0.0, acceleration.y)
            .normalize()
            .cross(Vec3::Y);

        let lean_angle = -(acceleration.length() * 20.0).clamp(-PI/4.0, PI/4.0);
        boid_transform.rotate_around(
            Vec3::new(new_translation.x, 0.0, new_translation.z),
            Quat::from_axis_angle(rotation_axis, lean_angle),
        );
    }
}

fn camera_fit_system(
    boids_query: Query<&Transform, With<Velocity>>,
    mut camera_query: Query<(&Projection, &mut Transform), Without<Velocity>>,
) {
    let boids_positions: Vec<Vec3> = boids_query.iter().map(|t| t.translation).collect();
    if boids_positions.is_empty() {
        return;
    }
    let mut aabb_min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut aabb_max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for boid in boids_positions.iter() {
        aabb_min = Vec3::new(
            aabb_min.x.min(boid.x),
            aabb_min.y.min(boid.y),
            aabb_min.z.min(boid.z),
        );

        aabb_max = Vec3::new(
            aabb_max.x.max(boid.x),
            aabb_max.y.max(boid.y),
            aabb_max.z.max(boid.z),
        )
    }

    let aabb_center = aabb_min.lerp(aabb_max, 0.5);
    let mut boid_farthest_from_aabb_center = boids_positions[0];
    let mut farthest_dist_sq = 0f32;

    for boid in boids_positions {
        let distance_from_center_sq = aabb_center.distance_squared(boid);
        if distance_from_center_sq > farthest_dist_sq {
            farthest_dist_sq = distance_from_center_sq;
            boid_farthest_from_aabb_center = boid;
        }
    }

    let bounding_sphere_radius = (boid_farthest_from_aabb_center - aabb_center).length();
    let (projection, mut camera_transform) = camera_query.single_mut();
    let perspective_projection = match projection {
        Projection::Perspective(pp) => pp,
        _ => unreachable!(),
    };
    let mut fov = perspective_projection.fov; // this is vertical
    let h_fov = ((fov / 2.0).tan() * perspective_projection.aspect_ratio).atan() * 2.0;
    fov = fov.min(h_fov);
    let required_camera_distance = bounding_sphere_radius / (fov / 2.0).sin();
    if required_camera_distance.is_nan() {
        panic!("required_camera_distance is NaN!")
    }

    camera_transform.translation = Vec3::new(aabb_center.x, 0.0, aabb_center.z) + 
        (Vec3::new(0.0,1.0,1.0) * required_camera_distance);
    
}

fn get_velocity_away_from_walls(
    position: Vec3,
    repel_value: f32,
    x_min: f32,
    x_max: f32,
    z_min: f32,
    z_max: f32,
) -> Vec2 {
    let mut result = Vec2::default();
    if position.x < x_min {
        result.x = repel_value;
    } else if position.x > x_max {
        result.x = -repel_value;
    }

    if position.z < z_min {
        result.y = repel_value;
    } else if position.z > z_max {
        result.y = -repel_value;
    }
    if result.x != 0.0 || result.y != 0.0 {}
    result
}
