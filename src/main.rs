use avian3d::prelude::*;
use bevy::{
    core_pipeline::Skybox, dev_tools::{self, infinite_grid::InfiniteGridPlugin}, gltf::GltfPlugin, image::ImageFilterMode, input::common_conditions::input_just_pressed, light::AtmosphereEnvironmentMapLight, log::{self, LogPlugin}, pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion}, prelude::*, render::render_resource::{TextureViewDescriptor, TextureViewDimension}, window::{CursorGrabMode, CursorOptions}
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, reply, make_layer};
use clap::Parser;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod player;

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
            DefaultPlugins.set(GltfPlugin {
                default_sampler: bevy::image::ImageSamplerDescriptor {
                    min_filter: ImageFilterMode::Linear,
                    mag_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    anisotropy_clamp: 16,
                    ..Default::default()
                },
                ..Default::default()
            }).set(LogPlugin {
                //level: log::Level::INFO,
                //filter: "error,capture_bevy_logs=info".to_owned(),
                custom_layer: make_layer,
                ..Default::default()
            }),
            PhysicsPlugins::default(),
            InfiniteGridPlugin,
            //PhysicsDebugPlugin,
            EnhancedInputPlugin,
            AhoyPlugins::default(),
            ConsolePlugin,
            bevy_skein::SkeinPlugin::default(),
            WorldInspectorPlugin::new(),
        ))
        .add_input_context::<player::PlayerInput>()
        .add_systems(
            Startup,
            (
                setup,
                player::spawn_player,
            )
        )
        .add_systems(
            Update,
            (
                test,
                toggle_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                //capture_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                //release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                asset_loaded,
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
fn test(mut commands: Commands, world: &World, q_child: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>)>, q: Query<(Entity, &Name, &Children)>) {
    //for (e, n, c) in q {
    //    if n.as_str() == "Cube.194" {
    //        println!("{:#?}",   world.inspect_entity(e).unwrap()
    //                     .map(|info| info.name())
    //                     .collect::<Vec<_>>());
    //        println!("{:#?}", c);
    //        for child in c.iter() {
    //            let (e_child, n_child, mut material) = q_child.get(child).unwrap();
    //            println!("{:#?}", n_child);
    //            println!("{:#?}", world.inspect_entity(e_child).unwrap()
    //                         .map(|info| info.name())
    //                         .collect::<Vec<_>>());

    //        }
    //    }
    //}
    //for n in q {
    //    println!("{:?}", n);
    //}
}

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        info!("Swapping Cubemap");
        let mut image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            let layers = image.height() / image.width();
            image
                .reinterpret_stacked_2d_as_array(layers)
                .expect("asset should be 2d texture and height will always be evenly divisible with the given layers");
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = Some(cubemap.image_handle.clone());
        }

        cubemap.is_loaded = true;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {// mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    commands.spawn(dev_tools::infinite_grid::InfiniteGrid);

    // Spawn a directional light
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(vec3(1.0, -2.0, -2.0), Vec3::Y),
        DirectionalLight {
            illuminance: 35000.0,
            //shadows_enabled: true,
            ..default()
        },
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 1.0,
        ..default()
    });

    // TODO look at bevy::scene root?
    let mesh_scene =
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("main2.glb")));
    commands.spawn((
        mesh_scene,
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
    ));
}

fn toggle_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = !cursor.visible;
    cursor.grab_mode = if cursor.visible {CursorGrabMode::None} else {CursorGrabMode::Locked};
}

