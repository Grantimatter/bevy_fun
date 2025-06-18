use bevy::{
    color::palettes::css::GREEN,
    prelude::*,
    winit::{UpdateMode::Continuous, WinitSettings},
};
use bevy_quinnet::{
    client::{
        QuinnetClient, certificate::CertificateVerificationMode,
        connection::ClientEndpointConfiguration,
    },
    server::{QuinnetServer, ServerEndpointConfiguration, certificate::CertificateRetrievalMode},
};
use bevy_replicon::prelude::*;
use bevy_replicon_quinnet::{ChannelsConfigurationExt, RepliconQuinnetPlugins};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    net::{IpAddr, Ipv6Addr},
};

const PORT: u16 = 5000;

fn main() {
    App::new()
        .init_resource::<Cli>()
        .insert_resource(WinitSettings {
            focused_mode: Continuous,
            unfocused_mode: Continuous,
        })
        .add_plugins((
            DefaultPlugins,
            RepliconPlugins,
            RepliconQuinnetPlugins,
            HelloPlugin,
        ))
        .run();
}

#[derive(Parser, PartialEq, Resource)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    network_mode: Option<NetworkMode>,
}

// #[derive(Parser, PartialEq, Resource)]
#[derive(Subcommand, PartialEq)]
enum NetworkMode {
    SinglePlayer,
    Server {
        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
    Client {
        #[arg(short, long, default_value_t = Ipv6Addr::LOCALHOST.into())]
        ip: IpAddr,

        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
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
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            info!("hello {}!", name.0)
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
        app.add_systems(
            Startup,
            (read_cli.map(Result::unwrap), add_people, spawn_camera),
        );
        app.add_systems(Update, (update_people, greet_people).chain());
        app.add_systems(Update, (draw_boxes));
    }
}

fn read_cli(
    mut commands: Commands,
    cli: Res<Cli>,
    channels: Res<RepliconChannels>,
    mut server: ResMut<QuinnetServer>,
    mut client: ResMut<QuinnetClient>,
) -> Result<(), Box<dyn Error>> {
    match &cli.network_mode {
        Some(mode) => match mode {
            NetworkMode::SinglePlayer => start_singleplayer(commands),
            NetworkMode::Server { port } => start_server(server, channels, *port, commands),
            NetworkMode::Client { ip, port } => {
                start_client(*ip, *port, channels, commands, client)
            }
        },
        None => start_singleplayer(commands),
    }
    Ok(())
}

#[derive(Component, Deref, Deserialize, Serialize, Default)]
#[require(BoxPosition, Replicated)]
struct PlayerBox {
    color: Color,
}

#[derive(Component, Deserialize, Serialize, Deref, DerefMut, Default)]
struct BoxPosition(Vec2);

#[derive(Component, Clone, Copy, Deref)]
struct BoxOwner(Entity);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn draw_boxes(mut gizmos: Gizmos, boxes: Query<(&BoxPosition, &PlayerBox)>) {
    for (position, player) in &boxes {
        gizmos.rect(
            Vec3::new(position.x, position.y, 0.0),
            Vec2::ONE * 50.0,
            player.color,
        );
    }
}

fn start_server(
    mut server: ResMut<QuinnetServer>,
    channels: Res<RepliconChannels>,
    port: u16,
    mut commands: Commands,
) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_ip(Ipv6Addr::LOCALHOST, port),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: Ipv6Addr::LOCALHOST.to_string(),
            },
            channels.server_configs(),
        )
        .unwrap();

    commands.spawn((
        Text::new("Server"),
        TextFont {
            font_size: 30.0,
            ..Default::default()
        },
        TextColor::WHITE,
    ));

    commands.spawn((
        PlayerBox {
            color: GREEN.into(),
        },
        BoxOwner(SERVER),
    ));
}

fn start_client(
    ip: IpAddr,
    port: u16,
    channels: Res<RepliconChannels>,
    mut commands: Commands,
    mut client: ResMut<QuinnetClient>,
) {
    client
        .open_connection(
            ClientEndpointConfiguration::from_ips(ip, port, Ipv6Addr::UNSPECIFIED, 0),
            CertificateVerificationMode::SkipVerification,
            channels.client_configs(),
        )
        .unwrap();
    commands.spawn((
        Text("Client".into()),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor::WHITE,
    ));

    info!("Client started. Connected to {}", ip)
}

fn spawn_clients(trigger: Trigger<OnAdd, ConnectedClient>, mut commands: Commands) {
    let mut hasher = DefaultHasher::new();
    trigger.entity().index().hash(&mut hasher);
    let hash = hasher.finish();

    let r = ((hash >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hash >> 8) & 0xFF) as f32 / 255.0;
    let b = (hash & 0xFF) as f32 / 255.0;

    info!("Spawning box for `{}`", trigger.entity());
    commands.spawn((
        PlayerBox {
            color: Color::srgb(r, g, b),
        },
        BoxOwner(trigger.entity()),
    ));
}

fn despawn_clients(
    trigger: Trigger<OnRemove, ConnectedClient>,
    mut commands: Commands,
    boxes: Query<(Entity, &BoxOwner)>,
) {
    let (entity, _) = boxes
        .iter()
        .find(|(_, &owner)| *owner == trigger.entity())
        .expect("all clients should have entites");
    commands.entity(entity).despawn();
}

fn start_singleplayer(mut commands: Commands) {
    info!("Starting Singleplayer!");
    commands.spawn((
        PlayerBox {
            color: GREEN.into(),
        },
        BoxOwner(SERVER),
    ));
}
