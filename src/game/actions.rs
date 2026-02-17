use bevy::prelude::*;

use super::audio::GameSoundEvent;
use super::components::Pawn;
use super::fence;
use super::selection::PawnSelection;
use super::state::{ActionOutcome, AppliedAction, GameAction, TurnState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionSource {
    Local,
    Remote,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameActionRequest {
    pub source: ActionSource,
    pub action: GameAction,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameActionApplied {
    pub source: ActionSource,
    pub action: GameAction,
}

pub fn apply_game_action_requests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut requests: EventReader<GameActionRequest>,
    mut applied_events: EventWriter<GameActionApplied>,
    mut sound_events: EventWriter<GameSoundEvent>,
    mut turn_state: ResMut<TurnState>,
    mut selection: ResMut<PawnSelection>,
    mut pawn_query: Query<(&Pawn, &mut Transform)>,
) {
    for request in requests.read() {
        let Ok(applied) = turn_state.try_apply_action(request.action) else {
            continue;
        };

        match applied {
            AppliedAction::Moved {
                player,
                target,
                outcome,
            } => {
                for (pawn, mut transform) in &mut pawn_query {
                    if pawn.player_index == player {
                        let world = target.to_world();
                        transform.translation = Vec3::new(world.x, world.y, 2.0);
                        break;
                    }
                }
                sound_events.write(GameSoundEvent::MovePawn);
                if matches!(outcome, ActionOutcome::Won(_)) {
                    sound_events.write(GameSoundEvent::Win);
                }
            }
            AppliedAction::FencePlaced { player, edges } => {
                let color = turn_state.players[player].pawn_color;
                fence::spawn_fence_segments(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &edges,
                    color,
                );
                sound_events.write(GameSoundEvent::MovePawn);
            }
        }

        if matches!(request.source, ActionSource::Local) {
            selection.current_selected = false;
        }

        applied_events.write(GameActionApplied {
            source: request.source,
            action: request.action,
        });
    }
}
