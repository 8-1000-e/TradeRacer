use bolt_lang::*;

declare_id!("DTQz7WWMC6wpnBNMwujh9BUeHrezBcXfsrUM1KsdBL73");

/// Per-player trading account. Balance is "fake SOL" handed out on spawn —
/// dies (alive=false) when balance + unrealized PnL hits 0.
#[component(delegate)]
pub struct PlayerState {
    pub authority: Pubkey,
    pub owner: Pubkey,

    /// false once the player gets liquidated (balance ≤ 0)
    pub alive: bool,

    // ─── Trading state ───
    /// Cash balance in lamports (fake SOL). Updated by close-position on close /
    /// liquidation; locked while a position is open.
    pub balance: u64,

    /// Position direction: 0=Flat, 1=Long, 2=Short. See shared::POS_*.
    pub position: u8,
    /// Leverage multiplier for the open position (1, 2, 5, 10, 25, 50).
    pub leverage: u8,
    /// Pyth raw price at which the position was opened (0 if flat).
    pub entry_price: u64,
    /// Notional position size in lamports = margin × leverage.
    /// Margin (lamports of balance reserved) = position_size / leverage.
    pub position_size: u64,

    /// Cumulative realized PnL across closed positions (lamports, can be negative).
    pub realized_pnl: i64,
    /// Most recent unrealized PnL snapshot (lamports), recomputed by close-position.
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
            realized_pnl: 0,
            unrealized_pnl: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
