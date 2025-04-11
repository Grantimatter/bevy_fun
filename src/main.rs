use bevy::prelude::*;
// use bevy_replicon::prelude::*;

const PORT: u16 = 5000;

fn main() {
    App::new()
        // .init_resource::<Cli>()
        // .insert_resource(WinitSettings {
        //     focused_mode: Continuous,
        //     unfocused_mode: Continuous,
        // })
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        // .add_plugins(RepliconPlugins)
        // .add_plugins(QuinnetClientPlugin::defualt())
        .run();
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
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0)
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

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}
