use bevy::{prelude::*, window::WindowResolution};
use rand::random_range;

pub const BASE: Color = Color::srgb(0.117647059, 0.117647059, 0.180392157);
pub const TEXT: Color = Color::srgb(0.80392, 0.839215, 0.956863);
pub const GREEN: Color = Color::srgb(0.6510, 0.8902, 0.631372549);
pub const RED: Color = Color::srgb(0.9529, 0.54510, 0.658824);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    mode: bevy::window::WindowMode::Windowed,
                    resolution: WindowResolution::new(1000., 1000.).with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
            PongPlugin,
        ))
        .insert_resource(ClearColor(BASE))
        .run();
}

#[derive(Component, Default)]
struct Paddle {
    input_direction: InputDirection,
    player: u8,
}

#[derive(Component)]
struct Controlling(bool);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct InputDirection(Vec2);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Drag(f32);

/// Position in screen space
/// (0.0, 0.0) = Bottom left
/// (1.0, 1.0) = Top Right
#[derive(Component)]
struct Position(Vec2);

/// Scale as % of screen in the x and y axis
/// (100, 100) is a rect that fills the window exactly
#[derive(Component)]
struct Scale(Vec2);

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
enum GamePhase {
    // #[default]
    Starting,
    #[default]
    Rally,
    Score,
}

pub struct PongPlugin;

impl Plugin for PongPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PreUpdate, (handle_keyboard_input, handle_gamepad_input));
        app.add_systems(
            Update,
            (apply_paddle_input, apply_drag, apply_velocity)
                .run_if(in_state(GamePhase::Rally))
                .chain(),
        );
        app.add_systems(PostUpdate, (position_translation, scale_to_window).chain());
        app.init_state::<GameState>();
        app.add_sub_state::<GamePhase>();
    }
}

fn setup(mut commands: Commands) {
    // Spawn Camera
    commands.spawn(Camera2d);

    let paddle_sprite = Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 });

    // Spawn Paddles
    commands.spawn((
        Paddle {
            player: 1,
            ..default()
        },
        paddle_sprite.clone(),
        Position(Vec2 { x: 0.1, y: 0.5 }),
        Scale(Vec2 { x: 4.0, y: 20.0 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Speed(9.0),
        Drag(0.05),
    ));

    commands.spawn((
        Paddle {
            player: 2,
            ..default()
        },
        paddle_sprite,
        Position(Vec2 { x: 0.9, y: 0.5 }),
        Scale(Vec2 { x: 4.0, y: 20.0 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Speed(9.0),
        Drag(0.05),
    ));

    // Spawn Ball
    commands.spawn((
        Ball,
        Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 }),
        Position(Vec2 { x: 0.5, y: 0.5 }),
        Scale(Vec2 { x: 4.0, y: 4.0 }),
        Speed(20.0),
        Velocity(Vec2 {
            x: if random_range(0..1) == 0 { -1.0 } else { 1.0 },
            y: random_range(-0.8..0.8),
        }),
    ));
}

fn position_translation(mut query: Query<(&Position, &mut Transform)>, window: Single<&Window>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.0.x * window.width() - (window.width() / 2.0);
        transform.translation.y = position.0.y * window.height() - (window.height() / 2.0);
    }
}

fn scale_to_window(mut query: Query<(&Scale, &mut Transform)>, window: Single<&Window>) {
    for (scale, mut transform) in query.iter_mut() {
        transform.scale.x = scale.0.x / 100.0 * window.width();
        transform.scale.y = scale.0.y / 100.0 * window.height();
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &Speed, &mut Position)>) {
    for (velocity, speed, mut position) in query.iter_mut() {
        position.0 += velocity.0 * speed.0 / 100.0 * time.delta_secs();
    }
}

fn apply_drag(time: Res<Time>, mut query: Query<(&mut Velocity, &Drag)>) {
    for (mut velocity, drag) in query.iter_mut() {
        if drag.0 == 0.0 {
            continue;
        }
        let vel = velocity.0;
        velocity.0 -= vel * (drag.0.clamp(0.0, 1.0));
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Paddle>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for mut paddle in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            match game_state.get() {
                GameState::Playing => next_state.set(GameState::Paused),
                GameState::Paused => next_state.set(GameState::Playing),
            };
        }

        if paddle.player == 1 {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                paddle.input_direction.0.y = 1.0;
            } else if keyboard_input.pressed(KeyCode::ArrowDown) {
                paddle.input_direction.0.y = -1.0;
            } else {
                paddle.input_direction.0.y = 0.0;
            }
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                paddle.input_direction.0.x = -1.0;
            } else if keyboard_input.pressed(KeyCode::ArrowRight) {
                paddle.input_direction.0.x = 1.0;
            } else {
                paddle.input_direction.0.x = 0.0;
            }
        }
    }
}

fn handle_gamepad_input(gamepads: Query<&Gamepad>, mut query: Query<&mut Paddle>) {
    for gamepad in gamepads {
        for mut paddle in query.iter_mut() {
            if paddle.player != 2 {
                continue;
            }
            if gamepad.pressed(GamepadButton::DPadUp) {
                paddle.input_direction.0.y = 1.0;
            } else if gamepad.pressed(GamepadButton::DPadDown) {
                paddle.input_direction.0.y = -1.0;
            } else {
                paddle.input_direction.0.y = 0.0;
            }
            if gamepad.pressed(GamepadButton::DPadLeft) {
                paddle.input_direction.0.x = -1.0;
            } else if gamepad.pressed(GamepadButton::DPadRight) {
                paddle.input_direction.0.x = 1.0;
            } else {
                paddle.input_direction.0.x = 0.0;
            }
        }
    }
}

fn apply_paddle_input(mut query: Query<(&Paddle, &Speed, &Position, &mut Velocity)>) {
    for (paddle, speed, position, mut velocity) in &mut query {
        if paddle.input_direction.0.y == 0.0 {
            continue;
        }

        velocity.0.y = paddle.input_direction.0.y * speed.0;

        if (position.0.y > 0.88 && velocity.0.y > 0.0)
            || (position.0.y < 0.12 && velocity.0.y < 0.0)
        {
            velocity.0.y = 0.0;
        }
    }
}
