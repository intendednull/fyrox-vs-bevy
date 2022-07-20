use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::{
    controllers::orbit::{
        ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
    },
    LookTransform, LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin {
            override_input_system: true,
        })
        .add_startup_system(setup)
        .add_system(setup_scene_once_loaded)
        .add_system(camera_input_map)
        .add_system(look_at_character)
        .add_system(move_character)
        .add_system(face_character)
        .run();
}

#[derive(Component)]
struct Monster;

#[derive(Component)]
struct Character;

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
        asset_server.load("monster-idleGLTF.glb#Animation0"), // asset_server.load("monster-idleGLTF.glb#Animation1")
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
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        // .insert(Restitution::coefficient(0.7))
        .insert(Character);
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<MonsterAnimations>,
    mut player: Query<&mut AnimationPlayer, With<Monster>>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok(mut player) = player.get_single_mut() {
            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
        }
    }
}

fn face_character(
    mut transforms: Query<&mut Transform>,
    character: Query<Entity, With<Character>>,
    camera: Query<Entity, With<OrbitCameraController>>,
) {
    let character = match character.get_single() {
        Ok(x) => x,
        _ => return,
    };
    let camera = match camera.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let rot = transforms.get(camera).expect("transform").rotation;
    let mut character = transforms.get_mut(character).expect("transform");

    // character.rotation = rot;
    // character.rotate(Quat::from_xyzw(0., rot.y, 0., 0.));
    character.rotation = Quat::from_xyzw(
        rot.x, // character.rotation.x,
        rot.y, // character.rotation.y,
        rot.z, // character.rotation.z,
        rot.w,
        // character.rotation.w,
    );
}

fn move_character(
    mut character: Query<&mut Transform, With<Character>>,
    camera: Query<&LookTransform, With<OrbitCameraController>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut character = match character.get_single_mut() {
        Ok(x) => x,
        _ => return,
    };
    let camera = match camera.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let line = (character.translation - camera.eye).normalize() * Vec3::new(1., 0., 1.);
    let line_perp = {
        let line = Vec2::new(line.x, line.z);
        let perp = line.perp();

        Vec3::new(perp.x, 0., perp.y)
    };

    if keyboard.pressed(KeyCode::W) {
        character.translation += line;
    }

    if keyboard.pressed(KeyCode::S) {
        character.translation -= line;
    }

    if keyboard.pressed(KeyCode::D) {
        character.translation += line_perp;
    }

    if keyboard.pressed(KeyCode::A) {
        character.translation -= line_perp;
    }
}

fn look_at_character(
    mut cameras: Query<&mut LookTransform, With<OrbitCameraController>>,
    character: Query<&Transform, (Changed<Transform>, With<Character>)>,
) {
    for mut look in cameras.iter_mut() {
        if let Ok(target) = character.get_single() {
            look.target = target.translation;

            let line = (look.eye - target.translation).normalize();
            look.eye = target.translation + line * 50.;
        }
    }
}

fn camera_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if cursor_delta != Vec2::ZERO {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    // let mut scalar = 1.0;
    // for event in mouse_wheel_reader.iter() {
    // // scale the event magnitude per pixel or per line
    // let scroll_amount = match event.unit {
    // MouseScrollUnit::Line => event.y,
    // MouseScrollUnit::Pixel => event.y / pixels_per_line,
    // };
    // scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    // }

    // events.send(ControlEvent::Zoom(scalar));
}
