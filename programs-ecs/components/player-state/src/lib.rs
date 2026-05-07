use bolt_lang::*;

declare_id!("Ba9QeK5PB6bF8fkfA64pyd4p3fkd6dco8tmEf2yToBtb");

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
    /// Leverage multiplier for the open position. u16 instead of u8 because
    /// the aggressive tier set goes up to 5000× — see shared::LEVERAGE_TIERS.
    pub leverage: u16,
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
    /// Unix-second timestamp at which the current position was opened.
    /// Set by `open-position` from `Clock::get()?.unix_timestamp`,
    /// cleared back to 0 by every close branch in `close-position`.
    /// Powers the chart's entry dot on the front + the `openedAt` column
    /// in the `trade_fight_trade` history table — survives reconnects
    /// because it lives on-chain.
    pub opened_at: i64,
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
            opened_at: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
