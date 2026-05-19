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
                toggle_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                //capture_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                //release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
            ),
        )
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
    //for n in q {
    //    println!("{:?}", n);
    //}
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // Spawn the player
    let player = commands
        .spawn((
            Name::new("Player"),
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

fn toggle_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = !cursor.visible;
    cursor.grab_mode = if cursor.visible {CursorGrabMode::None} else {CursorGrabMode::Locked};
}

