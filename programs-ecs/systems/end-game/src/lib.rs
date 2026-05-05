use bolt_lang::*;
use game_config::GameConfig;
use player_registry::PlayerRegistry;
use leaderboard::Leaderboard;

declare_id!("Hx4rWEBiQb1z41gByya3yvDCjZYcvkdF946AzQ6mbaru");

/// Finalizes a game: sets status=Finished and writes the sorted leaderboard.
///
/// remaining_accounts: every PlayerState PDA in the registry, in the order
/// player_registry.player_states[0..count]. The system reads each one to compute
/// final net_worth = balance + unrealized_pnl.
#[system]
pub mod end_game {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: require!(game_config.status == 1, GameError::GameNotPlaying).
        // TODO: require!(now >= game_config.game_end, GameError::GameNotOver).

        // TODO: iterate ctx.remaining_accounts in registry order:
        //       for each PlayerState PDA, deserialize and pull
        //       (authority, alive, balance, unrealized_pnl, realized_pnl).
        //       Build a LeaderboardEntry: pubkey=authority bytes,
        //       net_worth = balance as i64 + unrealized_pnl,
        //       realized_pnl, alive.

        // TODO: sort entries by net_worth DESC (stack-only — registry max is 10,
        //       insertion sort is fine). Tie-breaker: alive players above dead.

        // TODO: write into leaderboard.entries[0..count], leaderboard.count = count.

        // TODO: set game_config.status = 2 (Finished).

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub game_config: GameConfig,
        pub player_registry: PlayerRegistry,
        pub leaderboard: Leaderboard,
    }
}
