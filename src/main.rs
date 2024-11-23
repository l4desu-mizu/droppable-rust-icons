use bevy::prelude::*;
use bevy::window::{CursorOptions, WindowLevel, WindowMode};
use avian3d::prelude::*;
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
                    cursor_options: CursorOptions {
                        hit_test: false,
                        ..default()
                    },
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    window_level: WindowLevel::AlwaysOnTop,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default()
            //RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .add_event::<DropNow>()
        .init_state::<DropState>()
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Gears(50))
        .insert_resource(StartReceiver(receiver))
        .insert_resource(Timed(Timer::new(
            Duration::from_millis(5),
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (add_camera, spawn_transparent_plane))
        .add_systems(
            Update,
            (
                handle_state_change.run_if(in_state(DropState::Disabled)),
                send_event.run_if(in_state(DropState::Enabled)),
            ),
        )
        .add_observer(spawn_gears)
        .run();
}

#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
enum DropState {
    Enabled,
    #[default]
    Disabled,
}

#[derive(Resource)]
struct StartReceiver<T>(Receiver<T>);

#[derive(Event)]
struct DropNow;

#[derive(Resource)]
struct Timed(Timer);

fn handle_state_change(
    mut next_state: ResMut<NextState<DropState>>,
    start_receiver: Res<StartReceiver<bool>>,
) {
    if let Ok(_start) = start_receiver.0.try_recv() {
        next_state.set(DropState::Enabled);
    }
}

fn send_event(
    mut timer: ResMut<Timed>,
    time: Res<Time>,
    mut gears: ResMut<Gears>,
    mut commands: Commands,
) {
    if gears.0 > 0 && timer.0.tick(time.delta()).just_finished() {
        commands.trigger(DropNow);
        gears.0 -= 1;
    }
}

fn add_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 10.0, 10.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera3d::default()
    ));
    commands.spawn((
        Transform::from_translation(Vec3::Y * 10.0),
        PointLight::default()
    ));
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
        Mesh3d(mesh.add(Plane3d::new(Vec3::Y, Vec2::splat(100.0)).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::NONE))),
        Ground,
        Collider::cuboid(plane_size / 2.0, 0.1, plane_size / 2.0),
        RigidBody::Static,
    ));
}

fn spawn_gears(
    _: Trigger<DropNow>,
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
    let ground_plane = InfinitePlane3d::new(Vec3::Y);
    let ray = cam
        .viewport_to_world(transform, drop_position)
        .expect("Your viewport is misconfigured, mate.");
    let distance = ray
        .intersect_plane(ground.translation(), ground_plane)
        .expect("These should intersect");
    let point = ray.get_point(distance);
    commands.spawn((
        SceneRoot(asset_server.load("rust_cog.gltf#Scene0")),
        Transform::from_translation(point + Vec3::Y * 10.0),
        Collider::cylinder(0.5, 1.0),
        RigidBody::Dynamic,
    ));
}
