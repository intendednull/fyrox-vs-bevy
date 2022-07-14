use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_startup_system(setup)
        .add_system(move_enemies)
        .add_system(setup_scene_once_loaded)
        .run();
}

#[derive(Component)]
struct Monster;

struct MonsterAnimations(Vec<Handle<AnimationClip>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            PerspectiveCameraBundle::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
        ));

    // Ground
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // commands
    // .spawn()
    // .insert(RigidBody::Dynamic)
    // .insert(Collider::ball(0.5))
    // .insert(Restitution::coefficient(0.7))
    // .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)));

    // Monster

    // Insert a resource with the current scene information
    commands.insert_resource(MonsterAnimations(vec![
        asset_server.load("monster-idleGLTF.glb#Animation0")
    ]));

    let my_gltf = asset_server.load("monster-idleGLTF.glb#Scene0");
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_xyz(2.0, 0.0, -5.0),
            global: GlobalTransform::identity(),
        })
        .with_children(|parent| {
            parent.spawn_scene(my_gltf);
        })
        .insert(Monster);

    // Player
    let my_gltf = asset_server.load("m_player.glb#Scene0");
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_xyz(10.0, 0.0, -5.0),
            global: GlobalTransform::identity(),
        })
        .with_children(|parent| {
            parent.spawn_scene(my_gltf);
        });
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<MonsterAnimations>,
    mut player: Query<&mut AnimationPlayer>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok(mut player) = player.get_single_mut() {
            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
        }
    }
}

fn move_enemies(mut query: Query<&mut Transform, With<Monster>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 2.0 * time.delta_seconds();
    }
}
