use avian3d::prelude::*;
use bevy::{
    camera::Exposure,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    core_pipeline::tonemapping::Tonemapping,
    light::{AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, light_consts::lux},
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};

#[cfg(debug_assertions)]
#[allow(unused)]
use avian3d::prelude::PhysicsDebugPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::LinearRgba(LinearRgba {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        })))
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            PhysicsPlugins::default(),
            // #[cfg(debug_assertions)]
            PhysicsDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update)
        .run();
}

#[derive(Component)]
struct SpringObject;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: 5000.0,
        ..Default::default()
    }
    .build();

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        cascade,
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereEnvironmentMapLight::default(),
        AtmosphereSettings::default(),
        Exposure::SUNLIGHT,
        Tonemapping::AgX,
        Bloom::NATURAL,
        FreeCamera {
            run_speed: 100.0,
            walk_speed: 10.0,
            ..default()
        },
        Hdr,
        Transform::from_xyz(0.0, 2.0, 10.0),
    ));

    commands.spawn((
        Collider::half_space(Vec3::Y),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        RigidBody::Static,
    ));

    commands.spawn((
        Collider::sphere(0.5),
        RigidBody::Dynamic,
        Mesh3d(meshes.add(Sphere::new(0.5))),
        Transform::from_xyz(0.0, 3.2, 0.0),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        SpringObject,
    ));
}

fn update(
    mut query: Single<(Forces, &GlobalTransform), With<SpringObject>>,
    spatial_query: SpatialQuery,
) {
    let transform = *query.1;
    let force = &mut query.0;

    let vel = force.linear_velocity();
    let origin = transform.translation() + Vec3::new(2.0, -0.0, 0.0);
    let filter = SpatialQueryFilter::default();

    if let Some(hit) = spatial_query.cast_ray(origin, Dir3::NEG_Y, 2000.0, false, &filter) {
        info!(hit.distance);

        let rest = 4.0;
        let strength = 20.0;
        let damping_f = 1.1;

        let end_force = spring(hit.distance, rest, strength, damping_f, vel.y) * Dir3::Y;
        info!("{}", &end_force);
        force.apply_force(end_force);
    }
}

fn spring(
    distance: f32,
    rest_length: f32,
    strength: f32,
    damping_factor: f32,
    velocity: f32,
) -> f32 {
    let offset = rest_length - distance;
    let spring = offset * strength;

    let damping = velocity * damping_factor;

    spring - damping
}
