use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;
use shared::*;

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

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components> 
    {
        require!(ctx.accounts.game_config.status == 1, GameError::GameNotPlaying);
        let current_price = shared::read_pyth_price(ctx.remaining_accounts.last().unwrap())?;
        if ctx.accounts.player_state.position != POS_FLAT 
        {
            let unrealized: i128 = if ctx.accounts.player_state.position == POS_LONG 
            {
                (current_price as i128 - ctx.accounts.player_state.entry_price as i128)
                    * (ctx.accounts.player_state.position_size as i128)
                    / (ctx.accounts.player_state.entry_price as i128)
            } 
            else //SHORT
            {
                (ctx.accounts.player_state.entry_price as i128 - current_price as i128)
                    * (ctx.accounts.player_state.position_size as i128)
                    / (ctx.accounts.player_state.entry_price as i128)
            };

            ctx.accounts.player_state.unrealized_pnl = unrealized as i64;


            //liquidation check

            let liquidated = match ctx.accounts.player_state.position 
            {
                POS_LONG  => current_price <= ctx.accounts.player_state.liq_price,
                POS_SHORT => current_price >= ctx.accounts.player_state.liq_price,
                _ => false,
            };

            if liquidated 
            {
                let margin = ctx.accounts.player_state.position_size / ctx.accounts.player_state.leverage as u64;
                ctx.accounts.player_state.realized_pnl = ctx.accounts.player_state.realized_pnl.saturating_sub(margin as i64);
                ctx.accounts.player_state.position = POS_FLAT;
                ctx.accounts.player_state.leverage = 0;
                ctx.accounts.player_state.entry_price = 0;
                ctx.accounts.player_state.liq_price = 0;
                ctx.accounts.player_state.position_size = 0;
                ctx.accounts.player_state.unrealized_pnl = 0;
                if ctx.accounts.player_state.balance == 0 
                {
                    ctx.accounts.player_state.alive = false;
                }
                return Ok(ctx.accounts);
            }


            //manual close
            if parse_json_u64(&_args_p, b"close") == 1 && ctx.accounts.player_state.alive
            {
                let margin = ctx.accounts.player_state.position_size / ctx.accounts.player_state.leverage as u64;
                let payout = (margin as i128) + unrealized;
                ctx.accounts.player_state.balance += payout.max(0) as u64;
                ctx.accounts.player_state.realized_pnl = ctx.accounts.player_state.realized_pnl.saturating_add(unrealized as i64);
                ctx.accounts.player_state.position = POS_FLAT;
                ctx.accounts.player_state.leverage = 0;
                ctx.accounts.player_state.entry_price = 0;
                ctx.accounts.player_state.liq_price = 0;
                ctx.accounts.player_state.position_size = 0;
                ctx.accounts.player_state.unrealized_pnl = 0;
                if ctx.accounts.player_state.balance == 0 
                {
                    ctx.accounts.player_state.alive = false;
                }
            }
        }
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player_state: PlayerState,
        pub game_config: GameConfig,
    }
}
