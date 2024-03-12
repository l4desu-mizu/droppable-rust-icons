use bevy::prelude::*;
use bevy::window::{Cursor, WindowLevel, WindowMode};
use bevy_rapier3d::prelude::*;
use rand::{Rng, thread_rng};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust Icon Dropper".to_string(),
                    transparent: true,
                    cursor: Cursor {
                        hit_test: false,
                        ..default()
                    },
                    mode: WindowMode::BorderlessFullscreen,
                    window_level: WindowLevel::AlwaysOnTop,
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Gears(100))
        .add_systems(Startup,(
            add_camera
                .before(spawn_transparent_plane),
            spawn_transparent_plane.before(spawn_gears),
            spawn_gears
        ))
        .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 10.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::Y * 10.0),
        ..default()
    });
}

#[derive(Resource)]
struct Gears(u32);

#[derive(Component)]
struct Ground;

fn spawn_transparent_plane(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: mesh.add(Plane3d::new(Vec3::Y).mesh()),
            material: materials.add(StandardMaterial::from(Color::NONE)),
            ..default()
        },
        Ground,
        Collider::cuboid(25.0, 0.1, 25.0),
        RigidBody::Fixed,
    ));
}

fn spawn_gears(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gears: Res<Gears>,
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    ground: Query<&GlobalTransform, With<Ground>>,
) {
    let mut rng = thread_rng();
    for _ in 0..gears.0 {
        let windows_res = window
            .single().resolution.clone();
        let cursor_position = Vec2::new(
            rng.gen_range(0.0..windows_res.width()),
            rng.gen_range(0.0..windows_res.height()),
        );
        println!("{:?}", cursor_position);
        let (cam, transform) = camera.single();
        let ground = ground.single();
        let ground_plane = Plane3d::new(Vec3::Y);
        let ray = cam
            .viewport_to_world(transform, cursor_position)
            .expect("Your viewport is misconfigured, mate.");
        let distance = ray
            .intersect_plane(ground.translation(), ground_plane)
            .expect("These should intersect");
        let point = ray.get_point(distance);
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("rust_cog.gltf#Scene0"),
                transform: Transform::from_translation(point + Vec3::Y * 10.0),
                ..default()
            },
            Collider::cylinder(0.5, 1.0),
            RigidBody::Dynamic,
        ));
    }
}
