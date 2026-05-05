use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;

declare_id!("ChETGKhsRzTynoskTMnaPUfyB2GUmCoE228Gv6FoczT9");

/// Combined "tick + close" system. Called once per player either by:
///   - a cranker (every PNL_UPDATE_INTERVAL_SEC) to refresh unrealized PnL
///   - the player themselves with `"close": 1` to realize PnL and go flat
///
/// Args (JSON):
///   - "close": 1 to close the position, omit/0 to just refresh unrealized PnL
///
/// remaining_accounts[last]: Pyth Lazer SOL/USD account.
///
/// PnL math (signed i128 to avoid overflow):
///   long  unrealized = (current_price - entry_price) * position_size / entry_price
///   short unrealized = (entry_price - current_price) * position_size / entry_price
///
/// Liquidation: if balance + unrealized <= 0 the player dies (alive=false),
/// position is force-closed at zero balance, realized_pnl absorbs the loss.
#[system]
pub mod close_position {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> {
        // TODO: require!(game_config.status == 1, GameError::GameNotPlaying).
        // TODO: read current_price via shared::read_pyth_price(remaining_accounts.last()).
        // TODO: if player_state.position == POS_FLAT { return Ok(...) } — nothing to do.
        // TODO: compute signed unrealized PnL using i128 (see header math).
        // TODO: write player_state.unrealized_pnl = unrealized as i64.

        // ─── Liquidation check (runs even on a non-close tick) ───
        // TODO: let net = (player_state.balance as i128) + unrealized;
        //       if net <= 0 → force-close: balance = 0, position = POS_FLAT, leverage=0,
        //       entry_price=0, position_size=0, unrealized_pnl=0,
        //       realized_pnl = realized_pnl.saturating_add(unrealized as i64),
        //       alive = false. Return early.

        // ─── Voluntary close (parse "close": 1) ───
        // TODO: if shared::parse_json_u64(&_args_p, b"close") == 1 {
        //         margin = position_size / leverage as u64;
        //         payout = (margin as i128) + unrealized;     // signed
        //         balance = payout.max(0) as u64;             // can't go below 0
        //         realized_pnl += unrealized as i64;
        //         position = POS_FLAT, leverage = 0, entry_price = 0,
        //         position_size = 0, unrealized_pnl = 0;
        //         if balance == 0 { alive = false; }
        //       }

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player_state: PlayerState,
        pub game_config: GameConfig,
    }
}
