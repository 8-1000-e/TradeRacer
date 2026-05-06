use bolt_lang::*;

declare_id!("Dzi4us11W4QCSD5Erx2vfR15GyuMZn6S4Djr5HfPVVBm");

pub const MAX_PLAYERS: usize = 20;

/// Mirrors red-light's registry: parallel arrays of player authority pubkeys
/// and their corresponding PlayerState component PDAs.
///
/// Stored as `Vec` rather than fixed arrays so Borsh can deserialize
/// element-by-element on the stack (32 bytes temp per element) instead of
/// allocating the whole 20-slot array at once. This keeps the auto-generated
/// `update` / `update_with_session` and the systems that read this component
/// inside the BPF 4 KB stack frame budget at MAX_PLAYERS=20.
///
/// `Default` pre-fills both Vecs with `MAX_PLAYERS` zeroed entries so systems
/// can keep using positional writes (`players[idx] = …`) — same semantics as
/// the original fixed arrays. `count` still tracks how many slots are in use.
#[component(delegate)]
pub struct PlayerRegistry {
    #[max_len(MAX_PLAYERS)]
    pub players: Vec<[u8; 32]>,
    #[max_len(MAX_PLAYERS)]
    pub player_states: Vec<[u8; 32]>,
    pub count: u8,
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        Self {
            players: vec![[0u8; 32]; MAX_PLAYERS],
            player_states: vec![[0u8; 32]; MAX_PLAYERS],
            count: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
