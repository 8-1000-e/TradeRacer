use bolt_lang::*;
use game_config::GameConfig;
use shared::GAME_DURATION_SEC;
use shared::GameError;

declare_id!("9KvG9Htsnew3Tsvt7462bxc93HjJ6g3nK6XAjeN1Uupu");

/// Closes the lobby and switches the game to Playing. Sets the game timer.
#[system]
pub mod start_game {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> 
    {
        require!(ctx.accounts.game_config.status == 0, GameError::GameNotWaiting);
        let now = Clock::get()?.unix_timestamp;
        require!(now >= ctx.accounts.game_config.min_start_time, GameError::LobbyNotOver);
        ctx.accounts.game_config.status = 1;
        ctx.accounts.game_config.game_end = now + GAME_DURATION_SEC;
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub game_config: GameConfig,
    }
}
