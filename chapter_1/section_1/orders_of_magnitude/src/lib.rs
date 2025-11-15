use bevy::prelude::*;
use rhysics_common::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(default_window_plugin(
            "Chapter 1.1 - Orders of Magnitude"
        )))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.3, 0.4)))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (apply_velocity, move_ball).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn camera
    commands.spawn(Camera2d);
    
    log::info!("Camera spawned!");
    
    // Spawn a simple colored circle in the center

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(30.)),
        Ball,
        Velocity(Vec2::new(0.5, -0.5).normalize() * 100.0)
    ));

    commands.spawn((
        Text::new("Hello Bevy!"),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        },
    ));
}


fn move_ball(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ball_velocity: Single<&mut Velocity, With<Ball>>,
    time: Res<Time>,
) {
    let mut direction: Vec2 = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    ball_velocity.0 = direction * 10000.0 * time.delta_secs();
}


fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}