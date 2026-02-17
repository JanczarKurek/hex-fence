use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use bevy::prelude::*;

use crate::app_state::{AppPhase, GameConfig};
use crate::game::actions::{ActionSource, GameActionApplied, GameActionRequest};
use crate::game::state::GameAction;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetConfig::from_env())
            .insert_resource(NetRuntime::default())
            .add_systems(Startup, mark_local_connected_if_needed)
            .add_systems(
                Update,
                (
                    reconfigure_network_runtime,
                    poll_network_events,
                    send_local_actions_over_network,
                    send_start_game_from_host_on_enter,
                ),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetMode {
    Local,
    Host,
    Client,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct NetConfig {
    pub mode: NetMode,
    pub address: String,
    pub local_player_index: usize,
}

impl NetConfig {
    fn from_env() -> Self {
        let mode = match std::env::var("GIERECZKA_NET_MODE")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase()
            .as_str()
        {
            "host" => NetMode::Host,
            "client" => NetMode::Client,
            _ => NetMode::Local,
        };

        let address =
            std::env::var("GIERECZKA_NET_ADDR").unwrap_or_else(|_| "127.0.0.1:4000".to_string());
        let default_player = if mode == NetMode::Client { 1 } else { 0 };
        let local_player_index = std::env::var("GIERECZKA_NET_LOCAL_PLAYER")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(default_player);

        Self {
            mode,
            address,
            local_player_index,
        }
    }
}

#[derive(Resource, Default)]
pub struct NetRuntime {
    active_config: Option<NetConfig>,
    outgoing: Option<Sender<NetMessage>>,
    incoming: Option<Mutex<Receiver<NetEvent>>>,
    pub connected: bool,
    start_sent: bool,
}

impl NetRuntime {
    pub fn can_control_player(&self, config: &NetConfig, player_index: usize) -> bool {
        match config.mode {
            NetMode::Local => true,
            NetMode::Host | NetMode::Client => {
                self.connected && config.local_player_index == player_index
            }
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum NetMessage {
    StartGame(GameConfig),
    Action(GameAction),
}

#[derive(Debug)]
enum NetEvent {
    Connected,
    Disconnected,
    Message(NetMessage),
}

fn mark_local_connected_if_needed(net_config: Res<NetConfig>, mut runtime: ResMut<NetRuntime>) {
    if matches!(net_config.mode, NetMode::Local) {
        runtime.connected = true;
        runtime.active_config = Some(net_config.clone());
    }
}

fn reconfigure_network_runtime(net_config: Res<NetConfig>, mut runtime: ResMut<NetRuntime>) {
    if runtime.active_config.as_ref() == Some(net_config.as_ref()) {
        return;
    }

    runtime.outgoing = None;
    runtime.incoming = None;
    runtime.connected = false;
    runtime.start_sent = false;
    runtime.active_config = Some(net_config.clone());

    if matches!(net_config.mode, NetMode::Local) {
        runtime.connected = true;
        return;
    }

    let (to_thread_tx, to_thread_rx) = mpsc::channel::<NetMessage>();
    let (to_game_tx, to_game_rx) = mpsc::channel::<NetEvent>();
    let mode = net_config.mode;
    let address = net_config.address.clone();

    thread::spawn(move || match mode {
        NetMode::Host => {
            let Ok(listener) = TcpListener::bind(&address) else {
                let _ = to_game_tx.send(NetEvent::Disconnected);
                return;
            };

            let Ok((stream, _)) = listener.accept() else {
                let _ = to_game_tx.send(NetEvent::Disconnected);
                return;
            };

            run_socket(stream, to_thread_rx, to_game_tx);
        }
        NetMode::Client => {
            let Ok(stream) = TcpStream::connect(&address) else {
                let _ = to_game_tx.send(NetEvent::Disconnected);
                return;
            };

            run_socket(stream, to_thread_rx, to_game_tx);
        }
        NetMode::Local => {}
    });

    runtime.outgoing = Some(to_thread_tx);
    runtime.incoming = Some(Mutex::new(to_game_rx));
}

fn run_socket(stream: TcpStream, outbound: Receiver<NetMessage>, inbound: Sender<NetEvent>) {
    let _ = inbound.send(NetEvent::Connected);

    let Ok(read_stream) = stream.try_clone() else {
        let _ = inbound.send(NetEvent::Disconnected);
        return;
    };
    let writer_stream = stream;

    let read_inbound = inbound.clone();
    thread::spawn(move || {
        let reader = BufReader::new(read_stream);
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };

            if line.trim().is_empty() {
                continue;
            }

            let Ok(message) = serde_json::from_str::<NetMessage>(&line) else {
                continue;
            };
            let _ = read_inbound.send(NetEvent::Message(message));
        }
        let _ = read_inbound.send(NetEvent::Disconnected);
    });

    let mut writer = writer_stream;
    loop {
        match outbound.recv() {
            Ok(message) => {
                let Ok(serialized) = serde_json::to_string(&message) else {
                    continue;
                };
                if writer.write_all(serialized.as_bytes()).is_err() {
                    break;
                }
                if writer.write_all(b"\n").is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let _ = inbound.send(NetEvent::Disconnected);
}

fn poll_network_events(
    mut runtime: ResMut<NetRuntime>,
    net_config: Res<NetConfig>,
    mut game_config: ResMut<GameConfig>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    mut action_requests: EventWriter<GameActionRequest>,
) {
    let Some(incoming) = &runtime.incoming else {
        return;
    };
    let Ok(incoming) = incoming.lock() else {
        runtime.connected = false;
        return;
    };
    let mut events = Vec::new();

    loop {
        match incoming.try_recv() {
            Ok(event) => events.push(event),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                events.push(NetEvent::Disconnected);
                break;
            }
        }
    }
    drop(incoming);

    for event in events {
        match event {
            NetEvent::Connected => runtime.connected = true,
            NetEvent::Disconnected => {
                runtime.connected = false;
                runtime.start_sent = false;
            }
            NetEvent::Message(NetMessage::StartGame(config)) => {
                if matches!(net_config.mode, NetMode::Client) {
                    *game_config = config;
                    next_phase.set(AppPhase::InGame);
                }
            }
            NetEvent::Message(NetMessage::Action(action)) => {
                action_requests.write(GameActionRequest {
                    source: ActionSource::Remote,
                    action,
                });
            }
        }
    }
}

fn send_local_actions_over_network(
    net_config: Res<NetConfig>,
    runtime: Res<NetRuntime>,
    mut applied_actions: EventReader<GameActionApplied>,
) {
    if matches!(net_config.mode, NetMode::Local) || !runtime.connected {
        return;
    }

    let Some(outgoing) = &runtime.outgoing else {
        return;
    };

    for applied in applied_actions.read() {
        if !matches!(applied.source, ActionSource::Local) {
            continue;
        }

        let _ = outgoing.send(NetMessage::Action(applied.action));
    }
}

fn send_start_game_from_host_on_enter(
    net_config: Res<NetConfig>,
    mut runtime: ResMut<NetRuntime>,
    phase: Res<State<AppPhase>>,
    game_config: Res<GameConfig>,
) {
    if *phase.get() != AppPhase::InGame {
        runtime.start_sent = false;
        return;
    }

    if !matches!(net_config.mode, NetMode::Host) || !runtime.connected || runtime.start_sent {
        return;
    }

    if let Some(outgoing) = &runtime.outgoing {
        let _ = outgoing.send(NetMessage::StartGame(*game_config));
        runtime.start_sent = true;
    }
}
