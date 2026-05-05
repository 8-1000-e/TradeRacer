use bolt_lang::*;
use game_config::GameConfig;

declare_id!("9KvG9Htsnew3Tsvt7462bxc93HjJ6g3nK6XAjeN1Uupu");

/// Closes the lobby and switches the game to Playing. Sets the game timer.
#[system]
pub mod start_game {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: require!(game_config.status == 0, GameError::GameNotWaiting).
        // TODO: require!(now >= game_config.lobby_end, GameError::LobbyNotOver) — or allow
        //       the host to short-circuit if active_players >= MAX_PLAYERS.
        // TODO: set status = 1 (Playing), game_end = now + shared::GAME_DURATION_SEC.
        // TODO: (optional) read pyth price via shared::read_pyth_price and emit msg!()
        //       for clients — we don't store a game-wide entry_price (per-position only).
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub game_config: GameConfig,
    }
}
