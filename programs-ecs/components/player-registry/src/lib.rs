use bolt_lang::*;

declare_id!("Dzi4us11W4QCSD5Erx2vfR15GyuMZn6S4Djr5HfPVVBm");

pub const MAX_PLAYERS: usize = 10;

/// Mirrors red-light's registry: parallel arrays of player authority pubkeys
/// and their corresponding PlayerState component PDAs.
#[component(delegate)]
pub struct PlayerRegistry {
    pub players: [[u8; 32]; MAX_PLAYERS],
    pub player_states: [[u8; 32]; MAX_PLAYERS],
    pub count: u8,
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        Self {
            players: [[0u8; 32]; MAX_PLAYERS],
            player_states: [[0u8; 32]; MAX_PLAYERS],
            count: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
