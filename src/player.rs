use avian3d::prelude::*;
use bevy::{
    core_pipeline::Skybox, dev_tools::{self, infinite_grid::InfiniteGridPlugin}, gltf::GltfPlugin, image::ImageFilterMode, input::common_conditions::input_just_pressed, light::AtmosphereEnvironmentMapLight, log::{self, LogPlugin}, pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion}, prelude::*, window::{CursorGrabMode, CursorOptions}
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, reply, make_layer};
use clap::Parser;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use avian_pickup::prelude::*;

#[derive(Component, Default, Clone)]
pub(crate) struct PlayerInput;

#[derive(Component, Default, Clone)]
pub(crate) struct PlayerNoclip;

#[derive(Component, Default, Clone)]
pub struct Noclip {
    input_dir: Vec2,
    old_filter: u32,
    old_gravity: f32,
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "noclip")]
pub struct NoclipCommand;


pub fn noclip_command(mut commands: Commands, mut log: ConsoleCommand<NoclipCommand>, q: Query<(Entity, &mut CharacterController, Option<&Noclip>)>) {
    if let Some(Ok(_)) = log.take() {
        for (e, mut c, noclip) in q {
            if let Some(noclip) = noclip {
                log.reply("noclip off");
                c.filter.mask = LayerMask(noclip.old_filter);
                c.gravity = noclip.old_gravity;
                commands.entity(e).remove::<Noclip>();
            } else {
                log.reply("noclip on");
                info!("filtz: {:?}", c.filter);
                let noclip = Noclip {
                    old_filter: c.filter.mask.0,
                    old_gravity: c.gravity,
                    ..default()
                };
                commands.entity(e).insert(noclip);
                c.filter.mask = LayerMask::NONE;
                c.gravity = 0.0;
            }
        }
    }
}

pub fn noclip_update(q: Query<(&mut Noclip, &mut CharacterControllerState, &mut LinearVelocity)>) {
    for (mut noclip, c, mut vel) in q {
        let mut dir = c.orientation * Vec3::new(noclip.input_dir.x, 0.0, - noclip.input_dir.y);
        if dir != Vec3::ZERO {
            dir = dir.normalize();
        }
        *vel = LinearVelocity(vel.move_towards(dir, 200.0) * 10.0);
        noclip.input_dir = Vec2::ZERO;
    }
}

pub fn noclip_movement(movement: On<Fire<Movement>>, q: Query<&mut Noclip>) {
    for mut noclip in q {
        noclip.input_dir = movement.value;
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player = commands.spawn_scene(bsn! {
        Name::new("Player")
        // Add the character controller configuration. We'll use the default settings for now.
        CharacterController {
            speed: 7.0,
        }
        // The KCC currently behaves best when using a cylinder
        Collider::cylinder(0.7, 1.8)
        Transform::from_xyz(0.0, 20.0, 0.0)
        // Configure inputs. The actions `Movement`, `Jump`, etc. are provided by Ahoy, you just need to bind them.
        ////actions!(PlayerInput[
        PlayerInput
        Actions<PlayerInput>[
            Action::<Movement>::new()
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
            --
            Action::<PullObject>::new()
            Bindings [
                Binding(KeyCode::KeyE)
            ]
            --
            Action::<ThrowObject>::new()
            Bindings [
                Binding(KeyCode::KeyQ)
            ]
        ]
    }).id();

    // skybox
    let skybox_handle = asset_server.load("skybox.png");
    let skybox_blurred_handle = asset_server.load("skybox-small-blurred.png");

    // Can't use bsn cuz CameraOf doesn't implement Template
    use bevy_ahoy::prelude::*;
    commands.spawn((
        Camera3d::default(),
        PickupConfig {
            interaction_distance: 300.0,
            ..Default::default()
        },
        //AvianPickupActor::default(),
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
        EnvironmentMapLight {
            diffuse_map: skybox_blurred_handle.clone(),
            specular_map: skybox_handle.clone(),
            intensity: 50.0,
            ..Default::default()
        },
        //GeneratedEnvironmentMapLight {
        //    environment_map: skybox_handle.clone(),
        //    intensity: 100.0,
        //    ..Default::default()
        //}
        //
        //GlobalAmbientLight {
        //    color: Color::srgb_u8(210, 220, 240),
        //    brightness: 1000.0,
        //    ..default()
        //},
    ));

    commands.insert_resource(crate::Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
        blurred_image_handle: skybox_blurred_handle,
    });
}

