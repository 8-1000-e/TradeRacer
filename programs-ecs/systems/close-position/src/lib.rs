use bolt_lang::*;
use game_config::GameConfig;
use player_state::PlayerState;
use shared::*;

declare_id!("3Xqf5Aj2VrYAmUfwS88CfjKTn4UDoXHCFgri6MjBfNEh");

/// Anchor event emitted whenever a position closes — manual EJECT
/// (`{"close":1}` from the player's session signer) OR forced auto-
/// liquidation (cranker tick crosses `liq_price`). Logged on the
/// system's program ID so clients can `onLogs(closePositionSystemId)`,
/// parse the `"Program data: <base64>"` line, decode and react in
/// real time.
///
/// All fields snapshot the BEFORE-state of the position so the consumer
/// gets the entry / leverage / size that just closed (the close branches
/// zero them right after this emit). `opened_at` lets the consumer
/// compute holding duration without a separate fetch.
///
/// `exit_price` = the actual Pyth Lazer price the system read inside
/// the TX — this is the EXACT trigger price for liquidations (no off-
/// chain approximation). `realized_pnl_delta` is signed: -margin for
/// liquidations (saturated), `unrealized_at_close` for manual closes.
///
/// `reason` encodes how the position closed:
///   0 = manual (player hit EJECT)
///   1 = liquidated (cranker tick crossed liq_price)
///   2 = expired (settled at end-game by the back's finishMatch)
#[event]
pub struct PositionClosed {
    pub player: Pubkey,
    pub side: u8,             // 1 = LONG, 2 = SHORT (mirrors POS_*)
    pub leverage: u16,
    pub entry_price: u64,
    pub exit_price: u64,
    pub margin: u64,
    pub realized_pnl_delta: i64,
    pub opened_at: i64,
    pub reason: u8,
}

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
        // Recovery path: if a previous open landed with entry_price=0 (e.g.
        // Pyth returned 0 right after an ER restart, before the require!
        // guard in open-position was deployed), every subsequent unrealized
        // math would divide by zero and the cranker would spam
        // ProgramFailedToComplete forever. Detect that bricked state here,
        // refund the locked margin (the position never had a real entry,
        // so it's not a real loss), and force-close back to FLAT.
        if ctx.accounts.player_state.position != POS_FLAT
            && ctx.accounts.player_state.entry_price == 0
        {
            let lev = ctx.accounts.player_state.leverage;
            if lev > 0 {
                let margin = ctx.accounts.player_state.position_size / lev as u64;
                ctx.accounts.player_state.balance =
                    ctx.accounts.player_state.balance.saturating_add(margin);
            }
            ctx.accounts.player_state.position = POS_FLAT;
            ctx.accounts.player_state.leverage = 0;
            ctx.accounts.player_state.entry_price = 0;
            ctx.accounts.player_state.liq_price = 0;
            ctx.accounts.player_state.position_size = 0;
            ctx.accounts.player_state.unrealized_pnl = 0;
            ctx.accounts.player_state.opened_at = 0;
            return Ok(ctx.accounts);
        }
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
                // Snapshot the pre-zero fields so the emitted event still
                // carries side / leverage / entry / opened_at — the writes
                // below wipe them all back to FLAT.
                let ev_owner = ctx.accounts.player_state.owner;
                let ev_side = ctx.accounts.player_state.position;
                let ev_leverage = ctx.accounts.player_state.leverage;
                let ev_entry = ctx.accounts.player_state.entry_price;
                let ev_opened_at = ctx.accounts.player_state.opened_at;
                ctx.accounts.player_state.realized_pnl = ctx.accounts.player_state.realized_pnl.saturating_sub(margin as i64);
                ctx.accounts.player_state.position = POS_FLAT;
                ctx.accounts.player_state.leverage = 0;
                ctx.accounts.player_state.entry_price = 0;
                ctx.accounts.player_state.liq_price = 0;
                ctx.accounts.player_state.position_size = 0;
                ctx.accounts.player_state.unrealized_pnl = 0;
                ctx.accounts.player_state.opened_at = 0;
                if ctx.accounts.player_state.balance == 0
                {
                    ctx.accounts.player_state.alive = false;
                }
                emit!(PositionClosed {
                    player: ev_owner,
                    side: ev_side,
                    leverage: ev_leverage,
                    entry_price: ev_entry,
                    exit_price: current_price,
                    margin,
                    // Saturating sub mirrors the realized_pnl write above —
                    // for any sane match scale `margin` fits an i64 so this
                    // is just `-margin` in practice.
                    realized_pnl_delta: -(margin as i64),
                    opened_at: ev_opened_at,
                    reason: 1, // 1 = liquidated
                });
                return Ok(ctx.accounts);
            }


            //manual close
            if parse_json_u64(&_args_p, b"close") == 1 && ctx.accounts.player_state.alive
            {
                let margin = ctx.accounts.player_state.position_size / ctx.accounts.player_state.leverage as u64;
                let payout = (margin as i128) + unrealized;
                let ev_owner = ctx.accounts.player_state.owner;
                let ev_side = ctx.accounts.player_state.position;
                let ev_leverage = ctx.accounts.player_state.leverage;
                let ev_entry = ctx.accounts.player_state.entry_price;
                let ev_opened_at = ctx.accounts.player_state.opened_at;
                ctx.accounts.player_state.balance += payout.max(0) as u64;
                ctx.accounts.player_state.realized_pnl = ctx.accounts.player_state.realized_pnl.saturating_add(unrealized as i64);
                ctx.accounts.player_state.position = POS_FLAT;
                ctx.accounts.player_state.leverage = 0;
                ctx.accounts.player_state.entry_price = 0;
                ctx.accounts.player_state.liq_price = 0;
                ctx.accounts.player_state.position_size = 0;
                ctx.accounts.player_state.unrealized_pnl = 0;
                ctx.accounts.player_state.opened_at = 0;
                emit!(PositionClosed {
                    player: ev_owner,
                    side: ev_side,
                    leverage: ev_leverage,
                    entry_price: ev_entry,
                    exit_price: current_price,
                    margin,
                    realized_pnl_delta: unrealized as i64,
                    opened_at: ev_opened_at,
                    reason: 0, // 0 = manual
                });
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
