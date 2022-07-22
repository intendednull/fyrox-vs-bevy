use bevy::{input::mouse::MouseMotion, prelude::*, winit::WinitSettings};
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::CollisionEventFlags};
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
        .insert_resource(WinitSettings::desktop_app())
        .add_startup_system(setup)
        .add_system(setup_scene_once_loaded)
        .add_system(camera_input_map)
        .add_system(look_at_character)
        .add_system(move_character)
        .add_system(setup_helpers)
        .add_system(hacky_height_fix)
        .add_system(launch_projectile)
        .add_system(detect_projectile_collision)
        .run();
}

#[derive(Component)]
struct HitDetection;

#[derive(Component)]
struct Monster;

#[derive(Component)]
struct Character;

struct MonsterAnimations {
    idle: Handle<AnimationClip>,
}

struct CharacterAnimations {
    idle: Handle<AnimationClip>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            PerspectiveCameraBundle::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
        ));

    // UI Camera
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(ImageBundle {
        style: Style {
            align_self: AlignSelf::Center,
            position: Rect {
                left: Val::Percent(47.5),
                ..default()
            },
            ..default()
        },
        image_mode: bevy::ui::widget::ImageMode::KeepAspect,
        image: UiImage(asset_server.load("crosshair.png")),
        ..default()
    });

    // Ground
    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1000. })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
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

    // Monster
    spawn_monster(&mut commands, &asset_server);

    commands.insert_resource(MonsterAnimations {
        idle: asset_server.load("monster-idleGLTF.glb#Animation0"),
    });

    // Player
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_xyz(10.0, 0.0, 0.0).with_scale(Vec3::ONE * 0.3),
            global: GlobalTransform::identity(),
        })
        .with_children(|parent| {
            let my_gltf = asset_server.load("m_player.glb#Scene0");
            parent.spawn_scene(my_gltf);

            // parent
            // .spawn_bundle(TransformBundle {
            // local: Transform::from_xyz(0., 5.0, 0.),
            // global: GlobalTransform::identity(),
            // })
            // .insert(RigidBody::Dynamic)
            // .insert(Collider::cuboid(0.5, 5.5, 0.5));
        })
        .insert(AnimationHelperSetup)
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(2., 9., 2.))
        .insert(Character);

    commands.insert_resource(CharacterAnimations {
        idle: asset_server.load("m_player.glb#Animation0"),
    });
}

fn spawn_monster(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_xyz(0., 1.0, 10.).with_scale(Vec3::ONE * 1.4),
            global: GlobalTransform::identity(),
        })
        .with_children(|parent| {
            let my_gltf = asset_server.load("monster-idleGLTF.glb#Scene0");
            parent.spawn_scene(my_gltf);
        })
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(10.))
        .insert(Collider::cuboid(1., 3., 1.))
        .insert(AnimationHelperSetup)
        .insert(Monster)
        .insert(HitDetection);
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    monster_animations: Res<MonsterAnimations>,
    character_animations: Res<CharacterAnimations>,
    monsters: Query<&Monster>,
    characters: Query<&Character>,
    animation_helpers: Query<(Entity, &AnimationHelper)>,
    mut players: Query<&mut AnimationPlayer>,
    mut done: Local<bool>,
) {
    if !*done {
        for (id, &AnimationHelper(player_id)) in animation_helpers.iter() {
            if monsters.get(id).is_ok() {
                let mut player = players.get_mut(player_id).unwrap();
                player.play(monster_animations.idle.clone_weak()).repeat();
            } else if characters.get(id).is_ok() {
                let mut player = players.get_mut(player_id).unwrap();
                player.play(character_animations.idle.clone_weak()).repeat();
            }

            *done = true;
        }
    }
}

fn hacky_height_fix(
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
    characters: Query<Entity, With<Character>>,
    monsters: Query<Entity, With<Monster>>,
    mut fixed: Local<bool>,
) {
    for character in characters.iter() {
        let children = children.get(character).unwrap();
        for child in children.iter() {
            transforms.get_mut(*child).unwrap().translation.y = -9.;
            *fixed = true;
        }
    }

    for monster in monsters.iter() {
        let children = children.get(monster).unwrap();
        for child in children.iter() {
            transforms.get_mut(*child).unwrap().translation.y = -3.;
            *fixed = true;
        }
    }
}

fn move_character(
    mut character: Query<&mut Transform, With<Character>>,
    camera: Query<&LookTransform, With<OrbitCameraController>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut character = match character.get_single_mut() {
        Ok(x) => x,
        _ => return,
    };
    let camera = match camera.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let direction = (character.translation - camera.eye).normalize() * Vec3::new(1., 0., 1.);
    let direction_perp = {
        let line = Vec2::new(direction.x, direction.z);
        let perp = line.perp();

        Vec3::new(perp.x, 0., perp.y)
    };

    let t = character.translation - direction;
    let dest_rot = character.looking_at(t, Vec3::Y).rotation;
    character.rotation = character.rotation.lerp(dest_rot, 5. * time.delta_seconds());

    if keyboard.pressed(KeyCode::W) {
        character.translation += direction;
    }

    if keyboard.pressed(KeyCode::S) {
        character.translation -= direction;
    }

    if keyboard.pressed(KeyCode::D) {
        character.translation += direction_perp;
    }

    if keyboard.pressed(KeyCode::A) {
        character.translation -= direction_perp;
    }
}

fn look_at_character(
    mut cameras: Query<&mut LookTransform, With<OrbitCameraController>>,
    character: Query<&Transform, (Changed<Transform>, With<Character>)>,
) {
    for mut look in cameras.iter_mut() {
        if let Ok(target) = character.get_single() {
            look.target = target.translation + Vec3::Y * 4.;

            let line = (look.eye - target.translation).normalize();
            look.eye = target.translation + line * 20.;
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
}

#[derive(Debug, Component)]
pub struct AnimationHelperSetup;
#[derive(Debug, Component)]
pub struct AnimationHelper(Entity);

pub fn setup_helpers(
    mut commands: Commands,
    to_setup: Query<Entity, With<AnimationHelperSetup>>,
    children: Query<&Children>,
    players: Query<&AnimationPlayer>,
) {
    for host_entity in to_setup.iter() {
        if let Some(animation_player) =
            find_animation_player_entity(host_entity, &children, &players)
        {
            commands
                .entity(host_entity)
                .remove::<AnimationHelperSetup>()
                .insert(AnimationHelper(animation_player)); // This is how I find it later and  what I query for
        }
    }
}

fn find_animation_player_entity(
    parent: Entity,
    children: &Query<&Children>,
    players: &Query<&AnimationPlayer>,
) -> Option<Entity> {
    if let Ok(candidates) = children.get(parent) {
        let mut next_candidates: Vec<Entity> = candidates.iter().map(|e| e.to_owned()).collect();
        while !next_candidates.is_empty() {
            for candidate in next_candidates.drain(..).collect::<Vec<Entity>>() {
                if players.get(candidate).is_ok() {
                    return Some(candidate);
                } else if let Ok(new) = children.get(candidate) {
                    next_candidates.extend(new.iter());
                }
            }
        }
    }
    None
}

#[derive(Component)]
struct Projectile;

fn detect_projectile_collision(
    detection: Query<Entity, With<HitDetection>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(a, b, _flags) = collision_event {
            let detection = detection.get(*a).or_else(|_| detection.get(*b));
            let projectile = projectiles.get(*a).or_else(|_| projectiles.get(*b));

            if let (Ok(detection), Ok(_)) = (detection, projectile) {
                // commands.entity(detection).remove::<Projectile>();
                spawn_monster(&mut commands, &asset_server);
            }
        }
    }
}

fn launch_projectile(
    character: Query<&mut Transform, With<Character>>,
    camera: Query<&LookTransform, With<OrbitCameraController>>,
    mouse_button: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let character = match character.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let camera = match camera.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let direction = -(camera.eye - camera.target).normalize();
    let pos = character.translation + direction * 2.;
    let radius = 0.5;

    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: direction * 100.,
            angvel: Vec3::ZERO,
            // angvel: Vec3::new(0.2, 0.4, 0.8),
        })
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius,
                ..default()
            })),
            material: materials.add(Color::DARK_GREEN.into()),
            ..Default::default()
        })
        .insert(Collider::ball(radius))
        .insert(Projectile)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert_bundle(TransformBundle::from(Transform::from_translation(pos)));
}
