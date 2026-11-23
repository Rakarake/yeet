use avian3d::prelude::*;
use bevy::{
    core_pipeline::Skybox, dev_tools::{self, infinite_grid::InfiniteGridPlugin}, gltf::GltfPlugin, image::ImageFilterMode, input::common_conditions::input_just_pressed, light::AtmosphereEnvironmentMapLight, log::{self, LogPlugin}, pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion}, prelude::*, window::{CursorGrabMode, CursorOptions}
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, reply, make_layer};
use clap::Parser;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component, Default, Clone)]
pub(crate) struct PlayerInput;

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player = commands.spawn_scene(bsn! {
        Name::new("Player")
        // Add the character controller configuration. We'll use the default settings for now.
        CharacterController::default()
        // The KCC currently behaves best when using a cylinder
        Collider::cylinder(0.7, 1.8)
        Transform::from_xyz(0.0, 20.0, 0.0)
        // Configure inputs. The actions `Movement`, `Jump`, etc. are provided by Ahoy, you just need to bind them.
        PlayerInput
        ////actions!(PlayerInput[
        Actions<PlayerInput>[
            Action::<Movement>::new()
            // Normalize the input vector
            DeadZone
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
                //Cardinal::wasd_keys(),
                //Axial::left_stick()
            ]
            --
            Action::<Jump>::new()
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<Crouch>::new()
            Bindings [
                Binding(KeyCode::ControlLeft)
                --
                Binding(GamepadButton::LeftTrigger2)
            ]
            --
            Action::<RotateCamera>::new()
            Bindings [
                Binding::mouse_motion()
                ~Scale::splat(0.07)
                --
                //~Negate::all()
                {preset::axial::new(
                    (GamepadAxis::RightStickX, Scale::splat(100.0), Negate::x()),
                    (GamepadAxis::RightStickY, Scale::splat(100.0), Negate::x()),
                )}
            ]

        ]
    }).id();

    // skybox
    let skybox_handle = asset_server.load("skybox.png");

    // Can't use bsn cuz CameraOf doesn't implement Template
    commands.spawn((
        Camera3d::default(),
        // Need to be off for ScreenSpaceAmbientOcclusion
        Msaa::Sample4,
        //Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        //ScreenSpaceAmbientOcclusion::default(),
        //Skybox {
        //    image: assets.load("skybox.png").clone(),
        //    brightness: 1000.0,
        //    ..default()
        //},
        //AtmosphereSettings::default(),
        //AtmosphereEnvironmentMapLight::default(),
        Projection::from(PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..default()
        }),
        // Enable the optional builtin camera controller
        CharacterControllerCameraOf::new(player),
        Skybox {
            image: Some(skybox_handle.clone()),
            brightness: 1000.0,
            ..Default::default()
        },
        GeneratedEnvironmentMapLight {
            environment_map: skybox_handle.clone(),
            intensity: 100.0,
            ..Default::default()
        }
    ));

    commands.insert_resource(crate::Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}

