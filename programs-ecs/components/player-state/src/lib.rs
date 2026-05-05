use bolt_lang::*;

declare_id!("DTQz7WWMC6wpnBNMwujh9BUeHrezBcXfsrUM1KsdBL73");

/// Per-player trading account. Balance is fake USD (USDC-equivalent) handed out
/// on spawn — dies (alive=false) when balance + unrealized PnL hits 0.
/// All dollar amounts (balance, margin, position_size, entry_price, liq_price,
/// realized_pnl, unrealized_pnl) are USD with 8 decimals — same unit as the
/// Pyth Lazer SOL/USD feed, so PnL math needs no unit conversion.
#[component(delegate)]
pub struct PlayerState {
    pub authority: Pubkey,
    pub owner: Pubkey,

    /// false once the player gets liquidated (balance ≤ 0)
    pub alive: bool,

    // ─── Trading state ───
    /// Cash balance in fake USD (8 decimals). Updated by close-position on
    /// close / liquidation; the part locked as margin lives in position_size.
    pub balance: u64,

    /// Position direction: 0=Flat, 1=Long, 2=Short. See shared::POS_*.
    pub position: u8,
    /// Leverage multiplier for the open position (1, 2, 5, 10, 25, 50).
    pub leverage: u8,
    /// Pyth raw price at which the position was opened (0 if flat).
    pub entry_price: u64,
    /// Notional position size in fake USD (8 decimals) = margin × leverage.
    /// Margin (USD reserved from balance) = position_size / leverage.
    pub position_size: u64,
    /// Pyth raw price at which the position auto-liquidates (0 if flat).
    /// Long: entry × (1 − 1/leverage). Short: entry × (1 + 1/leverage).
    pub liq_price: u64,

    /// Cumulative realized PnL across closed positions (fake USD, 8 decimals, signed).
    pub realized_pnl: i64,
    /// Most recent unrealized PnL snapshot (fake USD, 8 decimals, signed),
    /// recomputed by close-position on each tick.
    pub unrealized_pnl: i64,

}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(),
            owner: Pubkey::default(),
            alive: false,
            balance: 0,
            position: 0,
            leverage: 0,
            entry_price: 0,
            position_size: 0,
            liq_price: 0,
            realized_pnl: 0,
            unrealized_pnl: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
