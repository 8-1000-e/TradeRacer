use bolt_lang::*;

declare_id!("D3AZXBim4wY9p2vt4ysHzL1M692ErQ4KtsnvWXS11YA7");

#[component(delegate)]
pub struct GameConfig {
    /// 0 = Waiting (lobby open), 1 = Playing, 2 = Finished
    pub status: u8,
    /// Number of players currently in the lobby / game
    pub active_players: u8,
    /// Game start (lobby open) timestamp — set by init-game
    pub min_start_time: i64,
    /// Game ends at this timestamp (set by start-game = now + GAME_DURATION_SEC)
    pub game_end: i64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            status: 0,
            active_players: 0,
            min_start_time: 0,
            game_end: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
