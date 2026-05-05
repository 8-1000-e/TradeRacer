use bolt_lang::*;

declare_id!("EYcpWDjusuacuFrcz4JKnNDU78gsPpJYcmyrGZ2s9qz");

pub const MAX_LEADERBOARD: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct LeaderboardEntry {
    pub pubkey: [u8; 32],
    /// Sort key: balance + locked margin + unrealized_pnl, in lamports.
    pub net_worth: i64,
    /// Free cash on hand (lamports).
    pub balance: u64,
    /// Flying PnL on the open position (lamports, signed).
    pub unrealized_pnl: i64,
    /// Realized PnL across closed trades (lamports).
    pub realized_pnl: i64,
    pub alive: bool,
}

impl Default for LeaderboardEntry {
    fn default() -> Self {
        Self {
            pubkey: [0u8; 32],
            net_worth: 0,
            balance: 0,
            unrealized_pnl: 0,
            realized_pnl: 0,
            alive: false,
        }
    }
}

#[component(delegate)]
pub struct Leaderboard {
    /// Entries sorted by net_worth descending (best PnL on top).
    pub entries: [LeaderboardEntry; MAX_LEADERBOARD],
    pub count: u8,
}

impl Default for Leaderboard {
    fn default() -> Self {
        Self {
            entries: [LeaderboardEntry::default(); MAX_LEADERBOARD],
            count: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
