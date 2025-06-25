use bevy::{log::tracing::Instrument, prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new(),
        ))
        .insert_resource(ClearColor(BASE))
        .run();
}

#[derive(Component, Default)]
/// This is a paddle
struct Paddle {
    input_direction: InputDirection,
    player: u8,
}

#[derive(Component)]
struct Ball;

#[derive(Component, Default)]
#[require(Transform)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct InputDirection(Vec2);

#[derive(Component, Default)]
#[require(Transform)]
struct Speed(f32);

#[derive(Component)]
#[require(Transform)]
struct Drag(f32);

#[derive(Component)]
#[require(Transform, Position, Velocity, Scale)]
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

pub enum Edge {
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

/// Scale as % of screen in the x and y axis
/// (100, 100) is a rect that fills the window exactly
#[derive(Component, Default)]
#[require(Transform)]
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
            (
                apply_paddle_input,
                apply_drag,
                apply_velocity,
                handle_box_collisions,
            )
                .run_if(in_state(GamePhase::Rally))
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            (
                position_translation,
                scale_to_window,
                draw_box_collider_gizmos,
            )
                .chain(),
        );
        app.init_state::<GameState>();
        app.add_sub_state::<GamePhase>();
    }
}

fn setup(mut commands: Commands) {
    // Spawn Camera
    commands.spawn(Camera2d);

    let paddle_sprite = Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 });

    // Spawn Barriers
    commands.spawn((
        Position(Vec2 { x: 50.0, y: 102.5 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Scale(Vec2 { x: 100.0, y: 5.0 }),
        BoxCollider {
            kinematic: true,
            ..Default::default()
        },
    ));

    commands.spawn((
        Position(Vec2 { x: 50.0, y: -2.5 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Scale(Vec2 { x: 100.0, y: 5.0 }),
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
        paddle_sprite.clone(),
        Position(Vec2 { x: 10.0, y: 50.0 }),
        Scale(Vec2 { x: 4.0, y: 20.0 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Speed(80.0),
        Drag(0.05),
        BoxCollider {
            kinematic: true,
            ..Default::default()
        },
    ));

    commands.spawn((
        Name::new("Right Paddle"),
        Paddle {
            player: 2,
            ..default()
        },
        paddle_sprite,
        Position(Vec2 { x: 90.0, y: 50.0 }),
        Scale(Vec2 { x: 4.0, y: 20.0 }),
        Velocity(Vec2 { x: 0.0, y: 0.0 }),
        Speed(80.0),
        Drag(0.05),
        BoxCollider {
            kinematic: true,
            ..Default::default()
        },
    ));

    // Spawn Ball
    spawn_ball(commands);
}

fn spawn_ball(mut commands: Commands) {
    commands.spawn((
        Name::new("Ball"),
        Ball,
        Sprite::from_color(TEXT, Vec2 { x: 1.0, y: 1.0 }),
        Position(Vec2 { x: 50.0, y: 50.0 }),
        Scale(Vec2 { x: 4.0, y: 4.0 }),
        Speed(40.0),
        Velocity(Vec2 {
            x: if random_range(0..=1) == 0 {
                -40.0
            } else {
                40.0
            },
            y: random_range(-1.0..1.0) * 40.0,
        }),
        BoxCollider {
            friction: 0.5,
            ..Default::default()
        },
    ));
}

fn position_translation(mut query: Query<(&Position, &mut Transform)>, window: Single<&Window>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.0.x * 0.01 * window.width() - (window.width() / 2.0);
        transform.translation.y = position.0.y * 0.01 * window.height() - (window.height() / 2.0);
    }
}

fn scale_to_window(mut query: Query<(&Scale, &mut Transform)>, window: Single<&Window>) {
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

fn handle_box_collisions(
    // mut colliders: ParamSet<(
    //     Query<(Entity, &mut Velocity, &mut Position, &BoxCollider, &Scale)>,
    //     Query<(Entity, &Position, &BoxCollider, &Scale)>,
    // )>,
    time: Res<Time>,
    mut query: Query<(
        NameOrEntity,
        &mut Velocity,
        &mut Position,
        &BoxCollider,
        &Scale,
    )>,
) {
    fn edge_pos(pos: &Position, scale: &Scale, direction: Edge) -> f32 {
        match direction {
            Edge::Top => pos.0.y + scale.0.y / 2.0,
            Edge::Bottom => pos.0.y - scale.0.y / 2.0,
            Edge::Left => pos.0.x - scale.0.x / 2.0,
            Edge::Right => pos.0.x + scale.0.x / 2.0,
        }
    }

    let mut combos = query.iter_combinations_mut();
    while let Some(
        [
            (entity_1, mut velocity_1, mut position_1, collider_1, scale_1),
            (entity_2, mut velocity_2, mut position_2, collider_2, scale_2),
        ],
    ) = combos.fetch_next()
    {
        if entity_1.entity == entity_2.entity || (collider_1.kinematic && collider_2.kinematic) {
            continue;
        }

        if edge_pos(&position_1, &scale_1, Edge::Right) + (velocity_1.0.x * time.delta_secs())
            > edge_pos(&position_2, &scale_2, Edge::Left) + (velocity_2.0.x * time.delta_secs())
            && edge_pos(&position_1, &scale_1, Edge::Left) + (velocity_1.0.x * time.delta_secs())
                < edge_pos(&position_2, &scale_2, Edge::Right)
                    + (velocity_2.0.x * time.delta_secs())
            && edge_pos(&position_1, &scale_1, Edge::Top)
                > edge_pos(&position_2, &scale_2, Edge::Bottom)
            && edge_pos(&position_1, &scale_1, Edge::Bottom)
                < edge_pos(&position_2, &scale_2, Edge::Top)
        {
            if !collider_1.kinematic {
                velocity_1.0.x *= -1.0;
                velocity_1.0.y += velocity_2.0.y
                    * ((collider_1.friction + collider_2.friction) / 2.0).clamp(0.0, 1.0);
            }
            if !collider_2.kinematic {
                velocity_2.0.x *= -1.0;
                velocity_2.0.y += velocity_1.0.y
                    * ((collider_2.friction + collider_1.friction) / 2.0).clamp(0.0, 1.0);
            }
        }

        if edge_pos(&position_1, &scale_1, Edge::Right)
            > edge_pos(&position_2, &scale_2, Edge::Left)
            && edge_pos(&position_1, &scale_1, Edge::Left)
                < edge_pos(&position_2, &scale_2, Edge::Right)
            && edge_pos(&position_1, &scale_1, Edge::Top) + (velocity_1.0.y * time.delta_secs())
                > edge_pos(&position_2, &scale_2, Edge::Bottom)
                    + (velocity_2.0.y * time.delta_secs())
            && edge_pos(&position_1, &scale_1, Edge::Bottom) + (velocity_1.0.y * time.delta_secs())
                < edge_pos(&position_2, &scale_2, Edge::Top) + (velocity_2.0.y * time.delta_secs())
        {
            if !collider_1.kinematic {
                velocity_1.0.y *= -1.0;
                velocity_1.0.x += velocity_2.0.x
                    * ((collider_1.friction + collider_2.friction) / 2.0).clamp(0.0, 1.0);
            }
            if !collider_2.kinematic {
                velocity_2.0.y *= -1.0;
                velocity_2.0.x += velocity_1.0.x
                    * ((collider_2.friction + collider_1.friction) / 2.0).clamp(0.0, 1.0);
            }
        }
    }
}

fn draw_box_collider_gizmos(mut gizmos: Gizmos, query: Query<(&Transform, &BoxCollider)>) {
    for (transform, collider) in query {
        //     gizmos.rect_2d(
        //         Isometry2d::from_translation(transform.translation.truncate()),
        //         transform.scale.truncate(),
        //         if collider.kinematic {
        //             Color::srgb(1.0, 0.0, 0.0)
        //         } else {
        //             Color::srgb(0.0, 0.0, 1.0)
        //         },
        //     );
    }
}
