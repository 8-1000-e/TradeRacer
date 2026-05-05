use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;
use player_registry::PlayerRegistry;

declare_id!("FnRjhEaucBXa3ZNZtosKuCjdxnG4VaVe4MCNZa2HxDzB");

/// Joins a player to the lobby. Hands out the starting fake-SOL balance.
///
/// remaining_accounts[last]: player owner pubkey (mirrors red-light pattern).
#[system]
pub mod spawn_player {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: require!(game_config.status == 0, GameError::GameNotWaiting).
        // TODO: require!(active_players < shared::MAX_PLAYERS, GameError::TooManyPlayers).
        // TODO: set player_state.authority = *ctx.accounts.authority.key.
        // TODO: set player_state.owner = *ctx.remaining_accounts.last().key.
        // TODO: hand out starting balance: player_state.balance = shared::STARTING_BALANCE.
        // TODO: zero trading state: position=POS_FLAT, leverage=0, entry_price=0,
        //       position_size=0, realized_pnl=0, unrealized_pnl=0.
        // TODO: alive = true.
        // TODO: append player_state PDA bytes into player_registry.player_states[count],
        //       authority into player_registry.players[count]; bump count + active_players.
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player_state: PlayerState,
        pub game_config: GameConfig,
        pub player_registry: PlayerRegistry,
    }
}
