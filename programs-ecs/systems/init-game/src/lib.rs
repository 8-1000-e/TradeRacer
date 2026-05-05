use bolt_lang::*;
use game_config::GameConfig;

declare_id!("9KGctbjYwgE3jJhb265BQ4GXDbVX7tnSptCLH56Agdtk");

/// Creates a fresh lobby. Caller is the host that just spun up the game PDA.
///
/// Lifecycle: init-game → (players spawn-player) → start-game → (open/close + close-position) → end-game.
#[system]
pub mod init_game {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: read Clock::get()?.unix_timestamp into `now`.
        // TODO: set status = 0 (Waiting), active_players = 0.
        // TODO: set start_time = now, lobby_end = now + shared::LOBBY_DURATION_SEC.
        // TODO: zero out game_end (set when start-game fires).
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub game_config: GameConfig,
    }
}
