use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use bevy::prelude::*;

use crate::app_state::{AppPhase, GameConfig, RematchRequested, StartRematch};
use crate::game::actions::{ActionSource, GameActionApplied, GameActionRequest};
use crate::game::state::GameAction;
use crate::settings::{AppSettings, LastNetMode};

pub struct NetworkPlugin;

type PeerId = u64;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetConfig::from_env())
            .insert_resource(NetRuntime::default())
            .insert_resource(NetLobbyState::default())
            .add_event::<NetUiCommand>()
            .add_systems(
                Startup,
                (apply_saved_network_settings, mark_local_connected_if_needed).chain(),
            )
            .add_systems(
                Update,
                (
                    reconfigure_network_runtime,
                    handle_network_ui_commands,
                    handle_rematch_requests,
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
    outgoing: Option<Sender<NetCommand>>,
    incoming: Option<Mutex<Receiver<NetEvent>>>,
    pub connected: bool,
    pub connected_peers: usize,
    pub peer_assignments: HashMap<PeerId, Option<usize>>,
    start_sent: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct NetLobbyState {
    pub config: GameConfig,
    pub host_slot: Option<usize>,
    pub remote_slots: Vec<usize>,
}

impl Default for NetLobbyState {
    fn default() -> Self {
        Self {
            config: GameConfig::default(),
            host_slot: Some(0),
            remote_slots: Vec::new(),
        }
    }
}

#[derive(Event, Clone, Debug)]
pub enum NetUiCommand {
    HostSyncLobby {
        config: GameConfig,
        host_slot: Option<usize>,
        remote_slots: Vec<usize>,
    },
    SelectLocalSlot(Option<usize>),
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

    pub fn claimed_remote_slots(&self) -> Vec<usize> {
        let mut slots = self
            .peer_assignments
            .values()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        slots.sort_unstable();
        slots.dedup();
        slots
    }

    pub fn request_reconnect(&mut self) {
        self.active_config = None;
        self.connected = false;
        self.connected_peers = 0;
        self.peer_assignments.clear();
        self.start_sent = false;
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum NetMessage {
    StartGame {
        config: GameConfig,
        local_player_index: Option<usize>,
    },
    LobbySync {
        config: GameConfig,
        host_slot: Option<usize>,
        remote_slots: Vec<usize>,
    },
    AssignedSlot(Option<usize>),
    LobbySelectSlot(Option<usize>),
    Action(GameAction),
    RematchRequest,
}

#[derive(Clone, Debug)]
enum NetCommand {
    Broadcast(NetMessage),
    BroadcastExcept {
        excluded_peer: PeerId,
        message: NetMessage,
    },
    SendToPeer {
        peer_id: PeerId,
        message: NetMessage,
    },
}

#[derive(Debug)]
enum NetEvent {
    Connected,
    Disconnected,
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    Message {
        peer_id: Option<PeerId>,
        message: NetMessage,
    },
}

#[derive(Debug)]
enum SocketEvent {
    Message {
        peer_id: PeerId,
        message: NetMessage,
    },
    Disconnected(PeerId),
}

fn mark_local_connected_if_needed(net_config: Res<NetConfig>, mut runtime: ResMut<NetRuntime>) {
    if matches!(net_config.mode, NetMode::Local) {
        runtime.connected = true;
        runtime.active_config = Some(net_config.clone());
        runtime.connected_peers = 0;
        runtime.peer_assignments.clear();
    }
}

fn sanitize_slot(slot: Option<usize>, player_count: usize) -> Option<usize> {
    slot.filter(|value| *value < player_count)
}

fn normalize_lobby(lobby: &mut NetLobbyState) {
    lobby.host_slot = sanitize_slot(lobby.host_slot, lobby.config.player_count);
    lobby
        .remote_slots
        .retain(|slot| *slot < lobby.config.player_count && Some(*slot) != lobby.host_slot);
    lobby.remote_slots.sort_unstable();
    lobby.remote_slots.dedup();
}

fn normalize_peer_assignments(
    runtime: &mut NetRuntime,
    lobby: &NetLobbyState,
) -> Vec<(PeerId, Option<usize>)> {
    let mut updates = Vec::new();

    for (peer_id, slot) in &mut runtime.peer_assignments {
        if let Some(value) = *slot
            && !lobby.remote_slots.contains(&value)
        {
            *slot = None;
            updates.push((*peer_id, None));
        }
    }

    let mut seen = HashMap::<usize, PeerId>::new();
    let mut collisions = Vec::new();
    for (peer_id, slot) in &runtime.peer_assignments {
        if let Some(value) = slot {
            if let Some(other) = seen.insert(*value, *peer_id) {
                collisions.push(other.max(*peer_id));
            }
        }
    }

    for peer_id in collisions {
        if let Some(slot) = runtime.peer_assignments.get_mut(&peer_id)
            && slot.take().is_some()
        {
            updates.push((peer_id, None));
        }
    }

    updates.extend(auto_assign_available_slots(runtime, lobby));

    updates
}

fn auto_assign_available_slots(
    runtime: &mut NetRuntime,
    lobby: &NetLobbyState,
) -> Vec<(PeerId, Option<usize>)> {
    let mut available_slots = lobby.remote_slots.clone();
    let claimed_slots = runtime.claimed_remote_slots();
    available_slots.retain(|slot| !claimed_slots.contains(slot));

    let mut waiting_peers = runtime
        .peer_assignments
        .iter()
        .filter_map(|(peer_id, slot)| slot.is_none().then_some(*peer_id))
        .collect::<Vec<_>>();
    waiting_peers.sort_unstable();

    let mut updates = Vec::new();
    for peer_id in waiting_peers {
        let Some(slot) = available_slots.first().copied() else {
            break;
        };
        available_slots.remove(0);
        runtime.peer_assignments.insert(peer_id, Some(slot));
        updates.push((peer_id, Some(slot)));
    }

    updates
}

fn send_lobby_sync(outgoing: &Sender<NetCommand>, lobby: &NetLobbyState) {
    let _ = outgoing.send(NetCommand::Broadcast(NetMessage::LobbySync {
        config: lobby.config,
        host_slot: lobby.host_slot,
        remote_slots: lobby.remote_slots.clone(),
    }));
}

fn send_assigned_slot(outgoing: &Sender<NetCommand>, peer_id: PeerId, slot: Option<usize>) {
    let _ = outgoing.send(NetCommand::SendToPeer {
        peer_id,
        message: NetMessage::AssignedSlot(slot),
    });
}

fn apply_saved_network_settings(app_settings: Res<AppSettings>, mut net_config: ResMut<NetConfig>) {
    if has_env_network_override() {
        return;
    }

    net_config.mode = match app_settings.network.mode {
        LastNetMode::Host => NetMode::Host,
        LastNetMode::Client => NetMode::Client,
    };
    net_config.address = app_settings.network.address.clone();
    net_config.local_player_index = app_settings.network.local_player_index;
}

fn reconfigure_network_runtime(
    net_config: Res<NetConfig>,
    mut runtime: ResMut<NetRuntime>,
    mut lobby: ResMut<NetLobbyState>,
) {
    if runtime
        .active_config
        .as_ref()
        .map(|config| (config.mode, config.address.as_str()))
        == Some((net_config.mode, net_config.address.as_str()))
    {
        runtime.active_config = Some(net_config.clone());
        return;
    }

    runtime.outgoing = None;
    runtime.incoming = None;
    runtime.connected = false;
    runtime.connected_peers = 0;
    runtime.peer_assignments.clear();
    runtime.start_sent = false;
    runtime.active_config = Some(net_config.clone());

    if matches!(net_config.mode, NetMode::Local) {
        runtime.connected = true;
        *lobby = NetLobbyState::default();
        return;
    }

    let (to_thread_tx, to_thread_rx) = mpsc::channel::<NetCommand>();
    let (to_game_tx, to_game_rx) = mpsc::channel::<NetEvent>();
    let mode = net_config.mode;
    let address = net_config.address.clone();

    thread::spawn(move || match mode {
        NetMode::Host => run_host_manager(address, to_thread_rx, to_game_tx),
        NetMode::Client => run_client_manager(address, to_thread_rx, to_game_tx),
        NetMode::Local => {}
    });

    runtime.outgoing = Some(to_thread_tx);
    runtime.incoming = Some(Mutex::new(to_game_rx));
    lobby.host_slot = Some(0);
    lobby.remote_slots.clear();
}

fn handle_network_ui_commands(
    mut net_config: ResMut<NetConfig>,
    mut runtime: ResMut<NetRuntime>,
    mut lobby: ResMut<NetLobbyState>,
    mut commands: EventReader<NetUiCommand>,
) {
    if matches!(net_config.mode, NetMode::Local) {
        return;
    }

    let mut should_sync = false;
    let mut peer_updates = Vec::new();

    for command in commands.read() {
        match command {
            NetUiCommand::HostSyncLobby {
                config,
                host_slot,
                remote_slots,
            } => {
                if !matches!(net_config.mode, NetMode::Host) {
                    continue;
                }
                lobby.config = *config;
                lobby.host_slot = *host_slot;
                lobby.remote_slots = remote_slots.clone();
                normalize_lobby(&mut lobby);
                peer_updates = normalize_peer_assignments(&mut runtime, &lobby);
                should_sync = true;
            }
            NetUiCommand::SelectLocalSlot(slot) => match net_config.mode {
                NetMode::Host => {
                    lobby.host_slot = *slot;
                    normalize_lobby(&mut lobby);
                    peer_updates = normalize_peer_assignments(&mut runtime, &lobby);
                    should_sync = true;
                }
                NetMode::Client => {
                    net_config.local_player_index = slot.unwrap_or(usize::MAX);
                    if let Some(outgoing) = &runtime.outgoing {
                        let _ = outgoing
                            .send(NetCommand::Broadcast(NetMessage::LobbySelectSlot(*slot)));
                    }
                }
                NetMode::Local => {}
            },
        }
    }

    if !matches!(net_config.mode, NetMode::Host) || !runtime.connected {
        return;
    }

    let Some(outgoing) = &runtime.outgoing else {
        return;
    };

    for (peer_id, slot) in peer_updates {
        send_assigned_slot(outgoing, peer_id, slot);
    }

    if should_sync {
        send_lobby_sync(outgoing, &lobby);
    }
}

fn run_host_manager(address: String, outbound: Receiver<NetCommand>, inbound: Sender<NetEvent>) {
    let Ok(listener) = TcpListener::bind(&address) else {
        let _ = inbound.send(NetEvent::Disconnected);
        return;
    };
    if listener.set_nonblocking(true).is_err() {
        let _ = inbound.send(NetEvent::Disconnected);
        return;
    }

    let (socket_tx, socket_rx) = mpsc::channel::<SocketEvent>();
    let mut peers: HashMap<PeerId, Sender<NetMessage>> = HashMap::new();
    let mut next_peer_id: PeerId = 1;
    let _ = inbound.send(NetEvent::Connected);

    loop {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let peer_id = next_peer_id;
                    next_peer_id += 1;
                    let sender = spawn_socket_endpoint(peer_id, stream, socket_tx.clone());
                    peers.insert(peer_id, sender);
                    let _ = inbound.send(NetEvent::PeerConnected(peer_id));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    let _ = inbound.send(NetEvent::Disconnected);
                    return;
                }
            }
        }

        loop {
            match socket_rx.try_recv() {
                Ok(SocketEvent::Message { peer_id, message }) => {
                    let _ = inbound.send(NetEvent::Message {
                        peer_id: Some(peer_id),
                        message,
                    });
                }
                Ok(SocketEvent::Disconnected(peer_id)) => {
                    peers.remove(&peer_id);
                    let _ = inbound.send(NetEvent::PeerDisconnected(peer_id));
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = inbound.send(NetEvent::Disconnected);
                    return;
                }
            }
        }

        loop {
            match outbound.try_recv() {
                Ok(NetCommand::Broadcast(message)) => {
                    broadcast_to_peers(&peers, None, &message);
                }
                Ok(NetCommand::BroadcastExcept {
                    excluded_peer,
                    message,
                }) => {
                    broadcast_to_peers(&peers, Some(excluded_peer), &message);
                }
                Ok(NetCommand::SendToPeer { peer_id, message }) => {
                    if let Some(sender) = peers.get(&peer_id) {
                        let _ = sender.send(message);
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn run_client_manager(address: String, outbound: Receiver<NetCommand>, inbound: Sender<NetEvent>) {
    let Ok(stream) = TcpStream::connect(&address) else {
        let _ = inbound.send(NetEvent::Disconnected);
        return;
    };

    let (socket_tx, socket_rx) = mpsc::channel::<SocketEvent>();
    let sender = spawn_socket_endpoint(0, stream, socket_tx);
    let _ = inbound.send(NetEvent::Connected);

    loop {
        loop {
            match socket_rx.try_recv() {
                Ok(SocketEvent::Message { message, .. }) => {
                    let _ = inbound.send(NetEvent::Message {
                        peer_id: None,
                        message,
                    });
                }
                Ok(SocketEvent::Disconnected(_)) => {
                    let _ = inbound.send(NetEvent::Disconnected);
                    return;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = inbound.send(NetEvent::Disconnected);
                    return;
                }
            }
        }

        loop {
            match outbound.try_recv() {
                Ok(NetCommand::Broadcast(message))
                | Ok(NetCommand::BroadcastExcept { message, .. })
                | Ok(NetCommand::SendToPeer { message, .. }) => {
                    if sender.send(message).is_err() {
                        let _ = inbound.send(NetEvent::Disconnected);
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn spawn_socket_endpoint(
    peer_id: PeerId,
    stream: TcpStream,
    inbound: Sender<SocketEvent>,
) -> Sender<NetMessage> {
    let (outbound_tx, outbound_rx) = mpsc::channel::<NetMessage>();

    thread::spawn(move || {
        let Ok(read_stream) = stream.try_clone() else {
            let _ = inbound.send(SocketEvent::Disconnected(peer_id));
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
                let _ = read_inbound.send(SocketEvent::Message { peer_id, message });
            }
            let _ = read_inbound.send(SocketEvent::Disconnected(peer_id));
        });

        let mut writer = writer_stream;
        while let Ok(message) = outbound_rx.recv() {
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

        let _ = inbound.send(SocketEvent::Disconnected(peer_id));
    });

    outbound_tx
}

fn broadcast_to_peers(
    peers: &HashMap<PeerId, Sender<NetMessage>>,
    excluded_peer: Option<PeerId>,
    message: &NetMessage,
) {
    for (peer_id, sender) in peers {
        if Some(*peer_id) == excluded_peer {
            continue;
        }
        let _ = sender.send(message.clone());
    }
}

fn poll_network_events(
    mut runtime: ResMut<NetRuntime>,
    mut net_config: ResMut<NetConfig>,
    mut lobby: ResMut<NetLobbyState>,
    mut game_config: ResMut<GameConfig>,
    phase: Res<State<AppPhase>>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    mut rematch_events: EventWriter<StartRematch>,
    mut action_requests: EventWriter<GameActionRequest>,
) {
    let Some(incoming) = &runtime.incoming else {
        return;
    };
    let Ok(incoming) = incoming.lock() else {
        runtime.connected = false;
        runtime.connected_peers = 0;
        runtime.peer_assignments.clear();
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
            NetEvent::Connected => {
                runtime.connected = true;
                if matches!(net_config.mode, NetMode::Host)
                    && let Some(outgoing) = &runtime.outgoing
                {
                    normalize_lobby(&mut lobby);
                    send_lobby_sync(outgoing, &lobby);
                }
            }
            NetEvent::Disconnected => {
                runtime.connected = false;
                runtime.connected_peers = 0;
                runtime.peer_assignments.clear();
                runtime.start_sent = false;
            }
            NetEvent::PeerConnected(peer_id) => {
                runtime.connected_peers += 1;
                runtime.peer_assignments.insert(peer_id, None);
                if matches!(net_config.mode, NetMode::Host) {
                    let peer_updates = auto_assign_available_slots(&mut runtime, &lobby);
                    if let Some(outgoing) = &runtime.outgoing {
                        for (peer_id, slot) in peer_updates {
                            send_assigned_slot(outgoing, peer_id, slot);
                        }
                    }
                    if let Some(outgoing) = &runtime.outgoing {
                        send_lobby_sync(outgoing, &lobby);
                    }
                }
            }
            NetEvent::PeerDisconnected(peer_id) => {
                runtime.connected_peers = runtime.connected_peers.saturating_sub(1);
                runtime.peer_assignments.remove(&peer_id);
                runtime.start_sent = false;
                if matches!(net_config.mode, NetMode::Host) {
                    let peer_updates = auto_assign_available_slots(&mut runtime, &lobby);
                    if let Some(outgoing) = &runtime.outgoing {
                        for (peer_id, slot) in peer_updates {
                            send_assigned_slot(outgoing, peer_id, slot);
                        }
                    }
                    if let Some(outgoing) = &runtime.outgoing {
                        send_lobby_sync(outgoing, &lobby);
                    }
                }
            }
            NetEvent::Message {
                peer_id: _,
                message:
                    NetMessage::StartGame {
                        config,
                        local_player_index,
                    },
            } => {
                if matches!(net_config.mode, NetMode::Client) {
                    *game_config = config;
                    net_config.local_player_index = local_player_index.unwrap_or(usize::MAX);
                    if *phase.get() == AppPhase::InGame {
                        rematch_events.write(StartRematch);
                    } else {
                        next_phase.set(AppPhase::InGame);
                    }
                }
            }
            NetEvent::Message {
                peer_id: _,
                message:
                    NetMessage::LobbySync {
                        config,
                        host_slot,
                        remote_slots,
                    },
            } => {
                if matches!(net_config.mode, NetMode::Client) {
                    lobby.config = config;
                    lobby.host_slot = host_slot;
                    lobby.remote_slots = remote_slots;
                    normalize_lobby(&mut lobby);
                }
            }
            NetEvent::Message {
                peer_id: _,
                message: NetMessage::AssignedSlot(slot),
            } => {
                if matches!(net_config.mode, NetMode::Client) {
                    net_config.local_player_index = slot.unwrap_or(usize::MAX);
                }
            }
            NetEvent::Message {
                peer_id: Some(peer_id),
                message: NetMessage::LobbySelectSlot(slot),
            } => {
                if matches!(net_config.mode, NetMode::Host) {
                    let requested = sanitize_slot(slot, lobby.config.player_count)
                        .filter(|value| lobby.remote_slots.contains(value));
                    let claimed_elsewhere =
                        runtime
                            .peer_assignments
                            .iter()
                            .any(|(other_peer_id, other_slot)| {
                                *other_peer_id != peer_id && *other_slot == requested
                            });
                    let assigned = if claimed_elsewhere { None } else { requested };
                    runtime.peer_assignments.insert(peer_id, assigned);
                    let peer_updates = auto_assign_available_slots(&mut runtime, &lobby);
                    if let Some(outgoing) = &runtime.outgoing {
                        send_assigned_slot(outgoing, peer_id, assigned);
                        for (peer_id, slot) in peer_updates {
                            send_assigned_slot(outgoing, peer_id, slot);
                        }
                        send_lobby_sync(outgoing, &lobby);
                    }
                }
            }
            NetEvent::Message {
                peer_id: Some(peer_id),
                message: NetMessage::RematchRequest,
            } => {
                if matches!(net_config.mode, NetMode::Host)
                    && *phase.get() == AppPhase::InGame
                    && runtime.connected
                {
                    if let Some(outgoing) = &runtime.outgoing {
                        send_start_messages(outgoing, &runtime, &game_config);
                    }
                    runtime.start_sent = true;
                    rematch_events.write(StartRematch);
                } else {
                    let _ = peer_id;
                }
            }
            NetEvent::Message {
                peer_id: Some(peer_id),
                message: NetMessage::Action(action),
            } => {
                if matches!(net_config.mode, NetMode::Host) {
                    if let Some(outgoing) = &runtime.outgoing {
                        let _ = outgoing.send(NetCommand::BroadcastExcept {
                            excluded_peer: peer_id,
                            message: NetMessage::Action(action),
                        });
                    }
                    action_requests.write(GameActionRequest {
                        source: ActionSource::Remote,
                        action,
                    });
                }
            }
            NetEvent::Message {
                peer_id: None,
                message: NetMessage::RematchRequest,
            } => {}
            NetEvent::Message {
                peer_id: None,
                message: NetMessage::Action(action),
            } => {
                action_requests.write(GameActionRequest {
                    source: ActionSource::Remote,
                    action,
                });
            }
            NetEvent::Message {
                peer_id: None,
                message: NetMessage::LobbySelectSlot(_),
            } => {}
        }
    }
}

fn send_start_messages(
    outgoing: &Sender<NetCommand>,
    runtime: &NetRuntime,
    game_config: &GameConfig,
) {
    for (peer_id, slot) in &runtime.peer_assignments {
        let _ = outgoing.send(NetCommand::SendToPeer {
            peer_id: *peer_id,
            message: NetMessage::StartGame {
                config: *game_config,
                local_player_index: *slot,
            },
        });
    }
}

fn handle_rematch_requests(
    net_config: Res<NetConfig>,
    mut runtime: ResMut<NetRuntime>,
    game_config: Res<GameConfig>,
    mut requests: EventReader<RematchRequested>,
    mut rematch_events: EventWriter<StartRematch>,
) {
    let mut requested = false;
    for _ in requests.read() {
        requested = true;
    }
    if !requested {
        return;
    }

    match net_config.mode {
        NetMode::Local => {
            rematch_events.write(StartRematch);
        }
        NetMode::Host => {
            if !runtime.connected {
                return;
            }
            if let Some(outgoing) = &runtime.outgoing {
                send_start_messages(outgoing, &runtime, &game_config);
            }
            runtime.start_sent = true;
            rematch_events.write(StartRematch);
        }
        NetMode::Client => {
            if !runtime.connected {
                return;
            }
            if let Some(outgoing) = &runtime.outgoing {
                let _ = outgoing.send(NetCommand::Broadcast(NetMessage::RematchRequest));
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

        let _ = outgoing.send(NetCommand::Broadcast(NetMessage::Action(applied.action)));
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
        send_start_messages(outgoing, &runtime, &game_config);
        runtime.start_sent = true;
    }
}

fn has_env_network_override() -> bool {
    std::env::var_os("GIERECZKA_NET_MODE").is_some()
        || std::env::var_os("GIERECZKA_NET_ADDR").is_some()
        || std::env::var_os("GIERECZKA_NET_LOCAL_PLAYER").is_some()
}
