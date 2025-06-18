use bevy::prelude::*;

fn main() {
    App::new().add_plugins((DefaultPlugins, HelloPlugin)).run();
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Grant Wiswell".to_string())));
    commands.spawn((Person, Name("Chloe Crichton".to_string())));
    commands.spawn((Person, Name("Ty Lee Wiswell".to_string())));
    commands.spawn((Person, Name("Mai Wiswell".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // Update our timer with the time elapsed since the last update.
    // If that caused the timer to finish, we say hello to everyone.
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("Hello {0}!", name.0)
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Chloe Crichton" {
            name.0 = "Chloe Wiswell".to_string();
            break;
        }
    }
}
