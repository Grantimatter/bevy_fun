use std::ops::RangeInclusive;

use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::WindowResolution,
};
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

#[derive(Resource)]
struct StartupTimer(Timer);

#[derive(Component, Default)]
#[require(Position,
    Shape = Shape(Vec2 { x: 4.0, y: 20.0 }),
    Velocity,
    Speed = Speed(80.0),
    Drag = Drag(0.05),
    BoxCollider = BoxCollider {kinematic: true, ..default()},
    Sprite = Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 })
)]
struct Paddle {
    input_direction: InputDirection,
    player: u8,
}

#[derive(Component)]
#[require(
    Name = Name::new("Ball"),
    Position = Position(Vec2 { x: 50.0, y: 50.0 }),
    Shape = Shape(Vec2 { x: 4.0, y: 4.0 }),
    Velocity,
    Speed = Speed(40.0),
    BoxCollider = BoxCollider {kinematic: false, friction: 0.5},
    Sprite = Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 }),
)]
struct Ball;

#[derive(Component, Default)]
#[require(Transform)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct InputDirection(Vec2);

#[derive(Component, Default)]
#[require(Transform)]
struct Speed(f32);

#[derive(Component, Default)]
#[require(Transform, Velocity)]
struct Drag(f32);

#[derive(Component)]
#[require(Transform, Position, Velocity, Shape)]
struct BoxCollider {
    kinematic: bool,
    friction: f32,
}

impl Default for BoxCollider {
    fn default() -> BoxCollider {
        BoxCollider {
            kinematic: false,
            friction: 0.1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Collision {
    Top,
    Bottom,
    Left,
    Right,
}

/// Position in screen space
/// (0.0, 0.0) = Bottom left
/// (100.0, 100.0) = Top Right
#[derive(Component, Default)]
#[require(Transform)]
struct Position(Vec2);

/// Shape as % of screen in the x and y axis
/// (100, 100) is a rect that fills the window exactly
#[derive(Component, Default)]
#[require(Transform)]
struct Shape(Vec2);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ai;

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum DebugMode {
    #[default]
    None,
    Debug,
}

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
enum GamePhase {
    #[default]
    Starting,
    Rally,
    Scoring,
}

#[derive(Component)]
enum Scorer {
    Player,
    Ai,
}

#[derive(Event)]
struct ScoredEvent(Scorer);

#[derive(Resource, Default)]
struct Score {
    player: u32,
    ai: u32,
}

#[derive(Component, Default)]
#[require(
    Text = Text::new("0"),
    TextColor = TextColor(TEXT),
    TextFont = TextFont {font_size: 60.0, ..default() },
    TextLayout = TextLayout::new_with_justify(JustifyText::Center),
    Scorer = Scorer::Player,
)]
struct ScoreCard;

pub struct PongPlugin;

impl Plugin for PongPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartupTimer(Timer::from_seconds(2.0, TimerMode::Once)));
        app.add_systems(Startup, setup);
        app.add_systems(PreUpdate, (handle_keyboard_input, handle_gamepad_input));
        app.add_systems(
            Update,
            (
                apply_paddle_input,
                apply_drag,
                apply_velocity,
                handle_collisions,
                handle_ai_paddle,
                kill_offscreen,
                detect_scoring,
                reset_ball,
                update_score,
                update_score_display,
            )
                .run_if(in_state(GamePhase::Rally))
                .chain(),
        );
        app.add_systems(Update, (game_startup).run_if(in_state(GamePhase::Starting)));
        app.add_systems(PostUpdate, (position_translation, scale_to_window).chain());
        app.add_systems(
            PostUpdate,
            (draw_box_collider_gizmos).run_if(in_state(DebugMode::Debug)),
        );
        app.init_state::<GameState>();
        app.add_sub_state::<GamePhase>();
        app.init_state::<DebugMode>();
        app.add_event::<ScoredEvent>();
        app.init_resource::<Score>();
    }
}

fn setup(mut commands: Commands) {
    // Spawn Camera
    commands.spawn(Camera2d);

    // Spawn Barriers
    commands.spawn((
        Position(Vec2 { x: 50.0, y: 102.5 }),
        Shape(Vec2 { x: 100.0, y: 5.0 }),
        BoxCollider {
            kinematic: true,
            ..Default::default()
        },
    ));

    commands.spawn((
        Position(Vec2 { x: 50.0, y: -2.5 }),
        Shape(Vec2 { x: 100.0, y: 5.0 }),
        BoxCollider {
            kinematic: true,
            ..Default::default()
        },
    ));

    // Spawn Paddles
    commands.spawn((
        Name::new("Left Paddle"),
        Paddle {
            player: 1,
            ..default()
        },
        Position(Vec2 { x: 10.0, y: 50.0 }),
    ));

    commands.spawn((
        Name::new("Right Paddle"),
        Ai,
        Paddle {
            player: 2,
            ..default()
        },
        Position(Vec2 { x: 90.0, y: 50.0 }),
    ));

    commands.spawn((
        ScoreCard,
        Scorer::Player,
        Node {
            top: Val::Percent(10.0),
            left: Val::Percent(20.0),
            ..default()
        },
    ));

    commands.spawn((
        ScoreCard,
        Scorer::Ai,
        Node {
            top: Val::Percent(10.0),
            left: Val::Percent(80.0),
            ..default()
        },
    ));

    // Spawn Ball
    spawn_ball(commands);
}

fn spawn_ball(mut commands: Commands) {
    commands.spawn((
        Ball,
        Velocity(Vec2 {
            x: if random_range(0..=1) == 0 {
                -40.0
            } else {
                40.0
            },
            y: random_range(-1.0..1.0) * 40.0,
        }),
    ));
}

fn position_translation(mut query: Query<(&Position, &mut Transform)>, window: Single<&Window>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.0.x * 0.01 * window.width() - (window.width() / 2.0);
        transform.translation.y = position.0.y * 0.01 * window.height() - (window.height() / 2.0);
    }
}

fn scale_to_window(mut query: Query<(&Shape, &mut Transform)>, window: Single<&Window>) {
    for (scale, mut transform) in query.iter_mut() {
        transform.scale.x = scale.0.x * 0.01 * window.width();
        transform.scale.y = scale.0.y * 0.01 * window.height();
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Position)>) {
    for (velocity, mut position) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_secs();
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

fn handle_ai_paddle(
    mut ai_paddle: Query<&mut Position, (With<Ai>, With<Paddle>, Without<Ball>)>,
    ball: Single<&Position, With<Ball>>,
) {
    for mut position in &mut ai_paddle {
        position.0.y = ball.0.y;
    }
}

fn handle_keyboard_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Paddle>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        spawn_ball(commands);
    }
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

        if (position.0.y > 88.0 && velocity.0.y > 0.0)
            || (position.0.y < 12.0 && velocity.0.y < 0.0)
        {
            velocity.0.y = 0.0;
        }
    }
}

fn collide_with_side(this: Aabb2d, other: Aabb2d) -> Option<Collision> {
    if !this.intersects(&other) {
        return None;
    }

    let closest = other.closest_point(this.center());
    let offset = this.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    other_things: Query<(&Position, &Velocity, &Shape), Without<Ball>>,
) {
    for (mut ball_velocity, ball_position, ball_shape) in &mut ball {
        for (position, velocity, shape) in &other_things {
            if let Some(collision) = collide_with_side(
                Aabb2d::new(ball_position.0, ball_shape.0 / 2.0),
                Aabb2d::new(position.0, shape.0 / 2.0),
            ) {
                match collision {
                    Collision::Top | Collision::Bottom => ball_velocity.0.y *= -1.0,
                    Collision::Left | Collision::Right => {
                        ball_velocity.0.x *= -1.0;
                        ball_velocity.0.y += velocity.0.y * 0.3;
                    }
                }
            }
        }
    }
}

fn draw_box_collider_gizmos(mut gizmos: Gizmos, query: Query<(&Transform, &BoxCollider)>) {
    for (transform, collider) in query {
        gizmos.rect_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            transform.scale.truncate(),
            if collider.kinematic {
                Color::srgb(1.0, 0.0, 0.0)
            } else {
                Color::srgb(0.0, 0.0, 1.0)
            },
        );
    }
}

fn game_startup(
    time: Res<Time>,
    mut timer: ResMut<StartupTimer>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        next_state.set(GamePhase::Rally);
    }
}

fn kill_offscreen(mut commands: Commands, query: Query<(Entity, &Position)>) {
    for (entity, position) in query {
        // Kill entity if it is over 100 units from the center of the screen
        if position.0.distance(Vec2 { x: 50.0, y: 50.0 }) > 80.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn detect_scoring(
    mut ball: Single<&mut Position, With<Ball>>,
    mut events: EventWriter<ScoredEvent>,
) {
    if ball.0.x > 100.0 {
        events.write(ScoredEvent(Scorer::Player));
    } else if ball.0.x < 0.0 {
        events.write(ScoredEvent(Scorer::Ai));
    }
}

fn reset_ball(
    mut commands: Commands,
    mut balls: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<ScoredEvent>,
) {
    for event in events.read() {
        for (mut position, mut velocity) in balls.iter_mut() {
            position.0 = Vec2::new(50.0, 50.0);
            match event.0 {
                Scorer::Player => velocity.0 = Vec2::new(1.0, random_range(-1.0..=1.0)) * 40.0,
                Scorer::Ai => velocity.0 = Vec2::new(-1.0, random_range(-1.0..=1.0)) * 40.0,
            }
        }
    }
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<ScoredEvent>) {
    for event in events.read() {
        match event.0 {
            Scorer::Player => score.player += 1,
            Scorer::Ai => score.ai += 1,
        }
    }
}

fn update_score_display(score: Res<Score>, mut query: Query<(&mut Text, &Scorer)>) {
    if !score.is_changed() {
        return;
    }
    for (mut text, scorer) in &mut query {
        match scorer {
            Scorer::Player => text.0 = score.player.to_string(),
            Scorer::Ai => text.0 = score.ai.to_string(),
        }
    }
}
