use bevy::prelude::*;
use bevy::window::{Cursor, WindowLevel, WindowMode};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::crossbeam;
use crossbeam::channel::{bounded, Receiver};
use rand::{thread_rng, Rng};
use std::io::stdin;
use std::thread;
use std::time::Duration;

fn main() {
    let (sender, receiver) = bounded::<bool>(1);
    thread::spawn(move || {
        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("Failed to read io");
        sender.send(true).expect("Failed to send start signal");
    });
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
        .add_event::<DropNow>()
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Gears(50))
        .insert_resource(Go(false))
        .insert_resource(StartReceiver(receiver))
        .insert_resource(Timed(Timer::new(
            Duration::from_millis(5),
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (add_camera, spawn_transparent_plane))
        .add_systems(
            Update,
            (
                send_event.run_if(resource_equals(Go(true))),
                trigger_run,
                spawn_gears.run_if(on_event::<DropNow>()),
            ),
        )
        .run();
}

#[derive(Resource, PartialEq)]
struct Go(bool);

#[derive(Resource)]
struct StartReceiver<T>(Receiver<T>);

#[derive(Event)]
struct DropNow;

#[derive(Resource)]
struct Timed(Timer);

fn trigger_run(mut go_res: ResMut<Go>, start_receiver: Res<StartReceiver<bool>>) {
    if let Ok(start) = start_receiver.0.try_recv() {
        go_res.0 = start;
    }
}

fn send_event(
    mut timer: ResMut<Timed>,
    time: Res<Time>,
    mut gears: ResMut<Gears>,
    mut drop_now: EventWriter<DropNow>,
) {
    if gears.0 > 0 && timer.0.tick(time.delta()).just_finished() {
        drop_now.send(DropNow);
        gears.0 -= 1;
    }
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
    let plane_size = 500.0;
    commands.spawn((
        PbrBundle {
            mesh: mesh.add(Plane3d::new(Vec3::Y).mesh()),
            material: materials.add(StandardMaterial::from(Color::NONE)),
            ..default()
        },
        Ground,
        Collider::cuboid(plane_size / 2.0, 0.1, plane_size / 2.0),
        RigidBody::Fixed,
    ));
}

fn spawn_gears(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    ground: Query<&GlobalTransform, With<Ground>>,
) {
    let mut rng = thread_rng();
    let window = window.single();
    let windows_res = window.resolution.clone();
    let drop_position = Vec2::new(
        rng.gen_range(200.0..windows_res.width() - 200.0),
        rng.gen_range(100.0..windows_res.height() - 100.0),
    );
    let (cam, transform) = camera.single();
    let ground = ground.single();
    let ground_plane = Plane3d::new(Vec3::Y);
    let ray = cam
        .viewport_to_world(transform, drop_position)
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
