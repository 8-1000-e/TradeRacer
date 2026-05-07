use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;
use shared::*;

declare_id!("EroraNpFUnP8WcKXPnLD4HE2cnAj5YNyQo8NKWsmqYLy");

/// Player opens a long or short SOL position with leverage.
///
/// Args (JSON):
///   - "direction": 1=Long, 2=Short (POS_LONG / POS_SHORT)
///   - "leverage":  one of shared::LEVERAGE_TIERS
///   - "margin":    fake USD (8 decimals) of balance to put up as margin
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

        let position = parse_json_u64(&_args_p, b"position") as u8;
        let leverage = parse_json_u64(&_args_p, b"leverage") as u16;
        let margin   = parse_json_u64(&_args_p, b"margin");

        require!(position == POS_LONG || position == POS_SHORT, GameError::InvalidDirection);
        require!(LEVERAGE_TIERS.contains(&leverage), GameError::InvalidLeverage);
        require!(margin > 0 && margin <= ctx.accounts.player_state.balance, GameError::InsufficientBalance);
        let position_size = margin.checked_mul(leverage as u64).ok_or(GameError::InsufficientBalance)?;

        let entry_price = read_pyth_price(ctx.remaining_accounts.last().unwrap())?;
        // entry_price may legitimately be 0 on localnet (Pyth Lazer feed
        // isn't running) — close-position handles that case below by
        // skipping the unrealized / liq math, so we don't reject here.
        let liq_price = match position {
            POS_LONG  => entry_price - entry_price / leverage as u64,
            POS_SHORT => entry_price + entry_price / leverage as u64,
            _ => 0,
        };

        ctx.accounts.player_state.position = position;
        ctx.accounts.player_state.leverage = leverage;
        ctx.accounts.player_state.position_size = position_size;
        ctx.accounts.player_state.entry_price = entry_price;
        ctx.accounts.player_state.liq_price = liq_price;
        ctx.accounts.player_state.balance -= margin;
        ctx.accounts.player_state.unrealized_pnl = 0;
        // Stamp the open time — the front uses this for the chart entry
        // dot, and the back's history watcher copies it onto the trade
        // row so closed trades carry both endpoints (openedAt / closedAt).
        ctx.accounts.player_state.opened_at = Clock::get()?.unix_timestamp;

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player_state: PlayerState,
        pub game_config: GameConfig,
    }
}
