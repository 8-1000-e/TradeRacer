use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;
use shared::*;

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

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> 
    {
        require!(ctx.accounts.game_config.status == 1, GameError::GameNotPlaying);
        require!(ctx.accounts.player_state.alive, GameError::PlayerDead);
        require!(ctx.accounts.player_state.position == POS_FLAT, GameError::PositionAlreadyOpen);

        ctx.accounts.player_state.position = parse_json_u64(&_args_p, "position")? as u8;
        ctx.accounts.player_state.leverage = parse_json_u64(&_args_p, "leverage")? as u8;
        ctx.accounts.player_state.position_size = parse_json_u64(&_args_p, "margin")? * ctx.accounts.player_state.leverage as u64;
        ctx.accounts.player_state.balance -= ctx.accounts.player_state.position_size;
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
