//! Loads and renders a glTF file as a scene.

use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::*;
use bevy::camera_controller::free_camera;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use avian3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        // free camera
        .add_plugins(free_camera::FreeCameraPlugin)
        // inspector
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // avian physics
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(avian3d::debug_render::PhysicsDebugPlugin)

        .add_systems(Startup, setup)
        //.add_systems(Startup, (
        //            setup,
        //            setup_physics
        //    ).chain()
        //)
        //.add_systems(Update, setup_physics)
        .add_systems(Update, setup_physics)
        .add_systems(Update, animate_light_direction)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
            ..default()
        },
        free_camera::FreeCamera::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("Fox.gltf"),
    )));
    commands.spawn(
        (
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Platform.glb"),)),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
        )
    );

    // Some ball or something
    commands.spawn((
        //RigidBody::Dynamic,
        Collider::sphere(0.5),
        // Overwrite default collider color (optional)
        DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
    ));
}

fn setup_physics(mut commands: Commands, query: Query<(Entity, &Name, &ChildOf), Added<Mesh3d>>, parents: Query<&Name>) {
    for (entity, name, parent) in query {
        if let Ok(parent_name) = parents.get(parent.0) {
            match parent_name.as_str() {
                "Suzanne" => {
                    commands.entity(entity).insert(RigidBody::Dynamic);
                },
                "Ground" => {
                    commands.entity(entity).insert(RigidBody::Static);
                    println!("🌱🌱🌱🌱");
                },
                _ => ()
            }
        }
        println!("addeddddsd!!!🚴🚴🚴");
    }
}

//fn setup_physics(
//        mut commands: Commands,
//        meshes: Res<Assets<Mesh>>,
//        q_children: Query<(Entity, &Mesh3d, &ChildOf), Without<Collider>>,
//        q_parents: Query<&Name>,
//    ) {
//    for (entity, mesh, parent) in q_children.iter() {
//        //println!("{name}: has a mesh!!! 🥰🥰🥰");
//        if let Ok(parent_name) = q_parents.get(parent.0) {
//            if parent_name.as_str() == "Suzanne" || parent_name.as_str() == "Ground" {
//                let mesha = meshes.get(&mesh.0);
//                match mesha {
//                    Some(mesha) => {
//                        //let mayb_collido = Collider::from_bevy_mesh(mesha, &ComputedColliderShape::TriMesh(TriMeshFlags::FIX_INTERNAL_EDGES));
//                        //match mayb_collido {
//                        //    Some(collido) => {
//                        //        println!("ADDING COLLIDER 🍓");
//                        //        commands.entity(entity).insert((
//                        //            collido,
//                        //        ));
//                        //        if parent_name.as_str() == "Suzanne" {
//                        //            commands.entity(entity).insert((
//                        //                RigidBody::Dynamic,
//                        //                Restitution::coefficient(0.7),
//                        //                //Transform::from_xyz(0.0, 4.0, 0.0),
//                        //            ));
//                        //        }
//                        //    },
//                        //    None => todo!(),
//                        //}
//                    },
//                    None => todo!(),
//                }
//            }
//
//        }
//    }
//}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}
