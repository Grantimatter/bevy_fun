use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_camera, spawn_square))
        .add_systems(
            Update,
            (
                move_square,
                update_velocity,
                (update_grounded, apply_gravity).chain(),
            ),
        )
        .run();
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Grounded(bool);

#[derive(Component)]
struct PhysicalProps {
    bounciness: f32,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_square(mut commands: Commands) {
    let sprite = Sprite::from_color(
        Color::srgb(1.0, 0.7, 0.7),
        Vec2 {
            x: (50.0),
            y: (50.0),
        },
    );
    commands.spawn((
        sprite,
        Velocity { x: 0.0, y: 0.0 },
        Grounded(false),
        PhysicalProps { bounciness: 0.8 },
    ));
}

fn move_square(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn update_velocity(mut query: Query<(&mut Velocity, &Grounded, &PhysicalProps), With<Transform>>) {
    for (mut velocity, grounded, physical_props) in &mut query {
        if grounded.0 {
            velocity.y = -velocity.y * physical_props.bounciness;
        }
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Velocity, &Grounded), With<Transform>>) {
    for (mut velocity, grounded) in &mut query {
        if !grounded.0 {
            velocity.y -= 9.81 * 75.0 * time.delta_secs();
        }
    }
}

fn update_grounded(mut query: Query<(&mut Grounded, &Velocity, &Transform)>) {
    for (mut grounded, velocity, transform) in &mut query {
        grounded.0 = transform.translation.y <= -515.0 && velocity.y < 0.0;
    }
}
