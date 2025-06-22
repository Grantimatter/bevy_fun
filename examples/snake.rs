use bevy::{
    input::{gamepad::GamepadEvent, keyboard},
    prelude::*,
    render::view::WindowSurfaces,
    window::WindowResolution,
    winit::WinitWindows,
};
use rand::{prelude::*, random_range, rng};

#[allow(warnings)]
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_from_array([
            0.117647059,
            0.117647059,
            0.180392157,
        ])))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1000., 1000.).with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
            SnakePlugin,
        ))
        .run();
}

pub struct SnakePlugin;

#[derive(Event)]
struct GrowthEvent;

#[derive(Event)]
struct GameOverEvent;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, spawn_snake).chain());
        app.add_systems(Update, handle_keyboard_input);
        app.add_systems(Update, handle_gamepad_input);
        app.add_systems(PostUpdate, (position_translation, size_scaling));
        app.add_systems(
            FixedUpdate,
            (move_snake, eat_food, grow_snake, spawn_food, game_over).chain(),
        );
        app.insert_resource(Time::<Fixed>::from_seconds(0.1));
        app.insert_resource(SnakeSegments::default());
        app.insert_resource(LastTailPosition::default());
        app.add_event::<GrowthEvent>();
        app.add_event::<GameOverEvent>();
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default, Resource)]
struct LastTailPosition(Option<Position>);

#[derive(Component)]
struct Velocity {
    x: i16,
    y: i16,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Up => Self::Down,
            Direction::Down => Self::Up,
            Direction::Left => Self::Right,
            Direction::Right => Self::Left,
        }
    }

    fn from_velocity(velocity: &Velocity) -> Self {
        match velocity {
            Velocity { x: 0, y: 1 } => Self::Up,
            Velocity { x: 0, y: -1 } => Self::Down,
            Velocity { x: -1, y: 0 } => Self::Left,
            Velocity { x: 1, y: 0 } => Self::Right,
            _ => Self::Right,
        }
    }
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(PartialEq, Component, Copy, Clone)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(Component)]
struct Food;

// TODO Change to resource
#[derive(Component)]
struct Grid {
    width: i16,
    height: i16,
}

fn setup(mut commands: Commands) {
    let grid = Grid {
        width: 12,
        height: 12,
    };

    // Spawn Camera
    commands.spawn(Camera2d);

    // Spawn Grid
    commands.spawn(grid);
}

fn spawn_snake(mut commands: Commands, grid: Single<&Grid>, mut segments: ResMut<SnakeSegments>) {
    let sprite = Sprite::from_color(
        Color::srgb(0.80392, 0.839215, 0.956863),
        Vec2 { x: (1.0), y: (1.0) },
    );
    let mut x_vel: i16 = random_range(-1..1);
    let y_vel: i16 = if x_vel.abs() > 0 {
        0
    } else {
        random_range(-1..1)
    };

    if x_vel == 0 && y_vel == 0 {
        x_vel = 1;
    }

    let velocity = Velocity { x: x_vel, y: y_vel };
    *segments = SnakeSegments(vec![
        commands
            .spawn((
                SnakeHead {
                    direction: Direction::from_velocity(&velocity),
                },
                Transform::from_xyz(0.0, 0.0, -1.0),
                Size::square(1.0),
                Position {
                    x: &grid.width / 2,
                    y: &grid.height / 2,
                },
                sprite,
                SnakeSegment,
            ))
            .id(),
    ])
}

fn spawn_food(mut commands: Commands, grid: Single<&Grid>, query: Query<&Food>) {
    if query.iter().count() == 0 {
        let sprite = Sprite::from_color(
            Color::srgb(0.9529, 0.54510, 0.658824),
            Vec2 { x: 0.8, y: 0.8 },
        );
        commands.spawn((
            Food,
            sprite,
            Size::square(1.0),
            Position {
                x: random_range(0..grid.width),
                y: random_range(0..grid.height),
            },
        ));
    }
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.7294, 0.7608, 0.8706), Vec2 { x: 1.0, y: 1.0 }),
            Transform::from_xyz(0.0, 0.0, -1.0),
            SnakeSegment,
            position,
            Size::square(1.0),
        ))
        .id()
}

fn move_snake(
    grid: Single<&Grid>,
    segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>, // query: Query<(&mut Position, &Velocity), With<SnakeHead>>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Up => head_pos.y += 1,
            Direction::Down => head_pos.y -= 1,
            Direction::Left => head_pos.x -= 1,
            Direction::Right => head_pos.x += 1,
        };

        if segment_positions.contains(&head_pos) {
            game_over_writer.write(GameOverEvent);
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        for mut position in positions {
            if position.x < 0 {
                position.x = grid.width - 1;
            } else if position.x > grid.width - 1 {
                position.x = 0;
            }
            if position.y < 0 {
                position.y = grid.height - 1;
            } else if position.y > grid.height - 1 {
                position.y = 0;
            }
        }

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
    grid: Single<&Grid>,
) {
    if reader.read().next().is_some() {
        for entity in food.iter().chain(segments.iter()) {
            commands.entity(entity).despawn();
        }
        spawn_snake(commands, grid, segments_res);
    }
}

fn grow_snake(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.read().next().is_some() {
        segments
            .0
            .push(spawn_segment(commands, last_tail_position.0.unwrap()))
    }
}

fn eat_food(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (entity, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(entity).despawn();
                growth_writer.write(GrowthEvent);
            }
        }
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut SnakeHead)>,
) {
    for mut head in query.iter_mut() {
        if (keyboard_input.pressed(KeyCode::ArrowUp)) && head.direction != Direction::Down {
            head.direction = Direction::Up;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) && head.direction != Direction::Up {
            head.direction = Direction::Down;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) && head.direction != Direction::Right {
            head.direction = Direction::Left;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) && head.direction != Direction::Left {
            head.direction = Direction::Right;
        }
    }
}

fn handle_gamepad_input(
    // mut gamepad_events: EventReader<GamepadEvent>,
    gamepads: Query<&Gamepad>,
    // button_inputs: Res<ButtonInput<GamepadButton>>,
    mut query: Query<&mut SnakeHead>,
) {
    for gamepad in &gamepads {
        for mut head in query.iter_mut() {
            if gamepad.just_pressed(GamepadButton::DPadUp) && head.direction != Direction::Down {
                head.direction = Direction::Up;
            }
            if gamepad.just_pressed(GamepadButton::DPadDown) && head.direction != Direction::Up {
                head.direction = Direction::Down;
            }
            if gamepad.just_pressed(GamepadButton::DPadLeft) && head.direction != Direction::Right {
                head.direction = Direction::Left;
            }
            if gamepad.just_pressed(GamepadButton::DPadRight) && head.direction != Direction::Left {
                head.direction = Direction::Right;
            }
        }
    }
    // if let Some(event) = gamepad_events.read().last() {
    //     match event {
    //         GamepadEvent::Connection(gamepad_connection_event) => (),
    //         GamepadEvent::Button(button_event) => {
    //             info!("{:?}",)
    //         }
    //         GamepadEvent::Axis(axis_event) => {}
    //     }
    // }
}

fn size_scaling(
    window: Single<&Window>,
    grid: Single<&Grid>,
    mut query: Query<(&Size, &mut Transform)>,
) {
    for (size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            size.width / grid.width as f32 * window.width() as f32,
            size.height / grid.height as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(
    window: Single<&Window>,
    grid: Single<&Grid>,
    mut query: Query<(&Position, &mut Transform)>,
) {
    fn convert(position: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        position / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    for (position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(position.x as f32, window.width() as f32, grid.width as f32),
            convert(
                position.y as f32,
                window.height() as f32,
                grid.height as f32,
            ),
            transform.translation.z,
        );
    }
}
