use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, scene::{SceneInstance, SceneInstanceReady}, window::{CursorGrabMode, CursorOptions}
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, reply};
use clap::Parser;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// Skein test component
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct Speak {
    phrase: String
}

#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
            EnhancedInputPlugin,
            AhoyPlugins::default(),
            ConsolePlugin,
            bevy_skein::SkeinPlugin::default(),
            WorldInspectorPlugin::new(),
        ))
        .add_input_context::<PlayerInput>()
        .add_systems(
            Startup,
            (
                setup,
            )
        )
        .add_systems(
            Update,
            (
                test,
                capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
            ),
        )
        .add_observer(on_new_scene)
        .insert_resource(ConsoleConfiguration {
            keys: vec![
                KeyCode::F7,
            ],
            ..Default::default()
        })
        .add_console_command::<EchoCommand, _>(echo_command)
        .add_console_command::<SpeakCommand, _>(speak_command)
        .run()
}

// Dummy console command
#[derive(Parser, ConsoleCommand)]
#[command(name = "echo")]
struct EchoCommand {
    msg: String,
}

fn echo_command(mut log: ConsoleCommand<EchoCommand>) {
    if let Some(Ok(EchoCommand { msg })) = log.take() {
        log.reply(msg);
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "speak")]
struct SpeakCommand {}

fn speak_command(mut log: ConsoleCommand<SpeakCommand>, q: Query<(&Speak, &Name)>) {
    if let Some(Ok(_)) = log.take() {
        for (speak, name) in q {
            reply!(log, "{}: {}", name, speak.phrase);
        }
    }
}

// Generic test system on Update
fn test(mut commands: Commands, q: Query<&ColliderConstructorHierarchy>) {
    for n in q {
        println!("{:?}", n);
    }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // Spawn the player
    let player = commands
        .spawn((
            // Add the character controller configuration. We'll use the default settings for now.
            CharacterController::default(),
            // The KCC currently behaves best when using a cylinder
            Collider::cylinder(0.7, 1.8),
            Transform::from_xyz(0.0, 20.0, 0.0),
            // Configure inputs. The actions `Movement`, `Jump`, etc. are provided by Ahoy, you just need to bind them.
            PlayerInput,
            actions!(PlayerInput[
                (
                    Action::<Movement>::new(),
                    // Normalize the input vector
                    DeadZone::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    ))
                ),
                (
                    Action::<Jump>::new(),
                    bindings![KeyCode::Space,  GamepadButton::South],
                ),
                (
                    Action::<Crouch>::new(),
                    bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<RotateCamera>::new(),
                    Bindings::spawn((
                        // tweak mouse and right stick sensitivity
                        // in Scale::splat values
                        Spawn((Binding::mouse_motion(), Scale::splat(0.07))),
                        Axial::right_stick().with((Scale::splat(4.0), DeadZone::default())),
                    ))
                ),
            ]),
        ))
        .id();

    // Spawn the player camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection{
            fov: 90.0_f32.to_radians(),
            ..default()
        }),
        // Enable the optional builtin camera controller
        CharacterControllerCameraOf::new(player),
    ));

    // Spawn a directional light
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(vec3(1.0, -2.0, -2.0), Vec3::Y),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
    ));

    // Spawn the level. This can be done in whatever way you prefer: spawn individual colliders, load a scene, use Skein, use bevy_trenchbroom, etc.
    // Ahoy will deal with it all.
    // Here we load a glTF file and create a convex hull collider for each mesh.
    commands.spawn((
        SceneRoot(assets.load("main.glb#Scene0")),
        //RigidBody::Static,
        //ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
    ));
}

#[derive(Component, Default)]
pub(crate) struct PlayerInput;

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}

/// Will be removed when descendants have been initialized.
#[derive(Component, Reflect)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct CollisionGeometry {
    /// Type of collider.
    mode: ColliderConstructor,
    rigid_body: RigidBody,
}

impl Default for CollisionGeometry {
    fn default() -> Self {
        Self {
            mode: ColliderConstructor::TrimeshFromMesh,
            rigid_body: RigidBody::Static,
        }
    }
}

/// Sets up colliders etc.
fn on_new_scene(
    _: On<SceneInstanceReady>,
    mut commands: Commands,
    q: Query<(Entity, &CollisionGeometry)>,
) {
    for (e, collision_geometry) in q {
        commands.entity(e).insert((
            ColliderConstructorHierarchy::new(collision_geometry.mode.clone()),
            collision_geometry.rigid_body,
        ));
        commands.entity(e).remove::<CollisionGeometry>();
    }
}

////! Loads and renders a glTF file as a scene.
//
//use bevy::{
//    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
//    prelude::*,
//};
//use std::f32::consts::*;
//use bevy::camera_controller::free_camera;
//use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
//use avian3d::prelude::*;
//use bevy_enhanced_input::prelude::*;
//use bevy_ahoy::prelude::*;
//
//fn main() {
//    App::new()
//        .insert_resource(DirectionalLightShadowMap { size: 4096 })
//        .add_plugins(DefaultPlugins)
//        // free camera
//        .add_plugins(free_camera::FreeCameraPlugin)
//        // inspector
//        .add_plugins(EguiPlugin::default())
//        //.add_plugins(WorldInspectorPlugin::new())
//        // avian physics
//        .add_plugins(PhysicsPlugins::default())
//        .add_plugins(avian3d::debug_render::PhysicsDebugPlugin)
//
//        .add_plugins(EnhancedInputPlugin)
//        .add_plugins(AhoyPlugins::default())
//
//        .add_systems(Startup, setup)
//        .add_systems(Startup, spawn_player)
//        //.add_systems(Startup, (
//        //            setup,
//        //            setup_physics
//        //    ).chain()
//        //)
//        //.add_systems(Update, setup_physics)
//        .add_systems(Update, setup_physics)
//        .add_systems(Update, animate_light_direction)
//        .run();
//}
//
//fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//    //commands.spawn((
//    //    Camera3d::default(),
//    //    Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
//    //    EnvironmentMapLight {
//    //        diffuse_map: asset_server.load("pisa_diffuse_rgb9e5_zstd.ktx2"),
//    //        specular_map: asset_server.load("pisa_specular_rgb9e5_zstd.ktx2"),
//    //        intensity: 250.0,
//    //        ..default()
//    //    },
//    //    //free_camera::FreeCamera::default(),
//    //));
//
//    commands.spawn((
//        DirectionalLight {
//            shadows_enabled: true,
//            ..default()
//        },
//        // This is a relatively small scene, so use tighter shadow
//        // cascade bounds than the default for better quality.
//        // We also adjusted the shadow map to be larger since we're
//        // only using a single cascade.
//        CascadeShadowConfigBuilder {
//            num_cascades: 1,
//            maximum_distance: 1.6,
//            ..default()
//        }
//        .build(),
//    ));
//    commands.spawn(SceneRoot(asset_server.load(
//        GltfAssetLabel::Scene(0).from_asset("Fox.gltf"),
//    )));
//    commands.spawn(
//        (
//            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Platform.glb"),)),
//            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
//        )
//    );
//
//    // Some ball or something
//    commands.spawn((
//        //RigidBody::Dynamic,
//        Collider::sphere(0.5),
//        // Overwrite default collider color (optional)
//        DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
//    ));
//}
//
//fn setup_a_rigid_body(mut commands: Commands, query: Query<(Entity, &Name, &ChildOf), Added<Mesh3d>>, parents: Query<&Name>) {
//    for (entity, _name, parent) in query {
//        if let Ok(parent_name) = parents.get(parent.0) {
//            match parent_name.as_str() {
//                "Suzanne" => {
//                    commands.entity(entity).insert((
//                         RigidBody::Dynamic,
//                         Restitution::new(0.7)
//                    ));
//                },
//                //"Ground" => {
//                //    commands.entity(entity).insert(RigidBody::Static);
//                //},
//                _ => ()
//            }
//        }
//    }
//}
//
//#[derive(Component)]
//struct PlayerInput;
//
//fn spawn_player(mut commands: Commands) {
//    // Spawn the player entity
//    let player = commands
//        .spawn((
//            // The character controller configuration
//            CharacterController::default(),
//            Transform::from_xyz(0.0, 20.0, 0.0),
//            // Configure inputs
//            PlayerInput,
//            actions!(PlayerInput[
//                (
//                    Action::<Movement>::new(),
//                    DeadZone::default(),
//                    Bindings::spawn((
//                        Cardinal::wasd_keys(),
//                        Axial::left_stick()
//                    ))
//                ),
//                (
//                    Action::<Jump>::new(),
//                    bindings![KeyCode::Space,  GamepadButton::South],
//                ),
//                (
//                    Action::<Crouch>::new(),
//                    bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger],
//                ),
//                (
//                    Action::<RotateCamera>::new(),
//                    Scale::splat(0.04),
//                    Bindings::spawn((
//                        Spawn(Binding::mouse_motion()),
//                        Axial::right_stick()
//                    ))
//                ),
//            ]),
//        ))
//        .id();
//
//    // Spawn the camera
//    commands.spawn((
//        Camera3d::default(),
//        // Enable the optional builtin camera controller
//        CharacterControllerCameraOf::new(player),
//    ));
//}
//
////fn setup_physics(
////        mut commands: Commands,
////        meshes: Res<Assets<Mesh>>,
////        q_children: Query<(Entity, &Mesh3d, &ChildOf), Without<Collider>>,
////        q_parents: Query<&Name>,
////    ) {
////    for (entity, mesh, parent) in q_children.iter() {
////        //println!("{name}: has a mesh!!! 🥰🥰🥰");
////        if let Ok(parent_name) = q_parents.get(parent.0) {
////            if parent_name.as_str() == "Suzanne" || parent_name.as_str() == "Ground" {
////                let mesha = meshes.get(&mesh.0);
////                match mesha {
////                    Some(mesha) => {
////                        //let mayb_collido = Collider::from_bevy_mesh(mesha, &ComputedColliderShape::TriMesh(TriMeshFlags::FIX_INTERNAL_EDGES));
////                        //match mayb_collido {
////                        //    Some(collido) => {
////                        //        println!("ADDING COLLIDER 🍓");
////                        //        commands.entity(entity).insert((
////                        //            collido,
////                        //        ));
////                        //        if parent_name.as_str() == "Suzanne" {
////                        //            commands.entity(entity).insert((
////                        //                RigidBody::Dynamic,
////                        //                Restitution::coefficient(0.7),
////                        //                //Transform::from_xyz(0.0, 4.0, 0.0),
////                        //            ));
////                        //        }
////                        //    },
////                        //    None => todo!(),
////                        //}
////                    },
////                    None => todo!(),
////                }
////            }
////
////        }
////    }
////}
//
//fn animate_light_direction(
//    time: Res<Time>,
//    mut query: Query<&mut Transform, With<DirectionalLight>>,
//) {
//    for mut transform in &mut query {
//        transform.rotation = Quat::from_euler(
//            EulerRot::ZYX,
//            0.0,
//            time.elapsed_secs() * PI / 5.0,
//            -FRAC_PI_4,
//        );
//    }
//}
