use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;

declare_id!("HbiVhxLoCFQ2uYzAAKXisSpiyEdve38bvva5v97nTwVw");

/// Player opens a long or short SOL position with leverage.
///
/// Args (JSON):
///   - "direction": 1=Long, 2=Short (POS_LONG / POS_SHORT)
///   - "leverage":  one of shared::LEVERAGE_TIERS
///   - "margin":    lamports of balance to put up as margin
///                  (notional position_size = margin × leverage)
///
/// remaining_accounts[last]: Pyth Lazer SOL/USD account for entry price.
#[system]
pub mod open_position {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: require!(game_config.status == 1, GameError::GameNotPlaying).
        // TODO: require!(player_state.alive, GameError::PlayerDead).
        // TODO: require!(player_state.position == shared::POS_FLAT, GameError::PositionAlreadyOpen).
        // TODO: parse direction / leverage / margin via shared::parse_json_*.
        // TODO: require!(direction == POS_LONG || direction == POS_SHORT, InvalidDirection).
        // TODO: require!(LEVERAGE_TIERS.contains(&leverage), InvalidLeverage).
        // TODO: require!(margin > 0 && margin <= player_state.balance, InsufficientBalance).
        // TODO: read entry_price from ctx.remaining_accounts last (Pyth Lazer).
        // TODO: write player_state: position=direction, leverage, entry_price,
        //       position_size = margin × leverage as u64. Subtract margin from balance.
        //       (Margin is reclaimed by close-position when "close": 1 is passed.)
        // TODO: anti-spam: bump player_state.last_action_slot = Clock::get()?.slot.
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player_state: PlayerState,
        pub game_config: GameConfig,
    }
}
