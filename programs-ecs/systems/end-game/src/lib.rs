use bolt_lang::*;
use game_config::GameConfig;
use player_registry::PlayerRegistry;
use leaderboard::{Leaderboard, LeaderboardEntry, MAX_LEADERBOARD};
use shared::*;

declare_id!("Hx4rWEBiQb1z41gByya3yvDCjZYcvkdF946AzQ6mbaru");

/// Bolt prepends one AccountInfo per #[system_input] component to remaining_accounts
/// before our extras. This system has 3 components, so PlayerState PDAs start at
/// index 3.
const NUM_COMPONENTS: usize = 3;

/// PlayerState byte layout (8-byte Anchor discriminator + fields, little-endian):
///   8..40   authority    (Pubkey, 32)
///   40..72  owner        (Pubkey, 32)
///   72      alive        (u8: 0/1)
///   73..81  balance      (u64)
///   81      position     (u8)
///   82      leverage     (u8)
///   83..91  entry_price  (u64)
///   91..99  position_size(u64)
///   99..107 liq_price    (u64)
///   107..115 realized_pnl(i64)
///   115..123 unrealized_pnl(i64)
const PS_AUTHORITY: usize = 8;
const PS_ALIVE: usize = 72;
const PS_BALANCE: usize = 73;
const PS_LEVERAGE: usize = 82;
const PS_POSITION_SIZE: usize = 91;
const PS_REALIZED_PNL: usize = 107;
const PS_UNREALIZED_PNL: usize = 115;
const PS_MIN_LEN: usize = 123;

/// Finalizes a game: sets status=Finished and writes the sorted leaderboard.
///
/// remaining_accounts (after the 3 component-program slots Bolt prepends):
/// every PlayerState PDA in the registry, in the order
/// player_registry.player_states[0..count].
#[system]
pub mod end_game {

    pub fn execute(ctx: Context<Components>, _args_p: Vec<u8>) -> Result<Components>
    {
        require!(ctx.accounts.game_config.status == 1, GameError::GameNotPlaying);
        let now = Clock::get()?.unix_timestamp;
        require!(now >= ctx.accounts.game_config.game_end, GameError::GameNotOver);

        let count = (ctx.accounts.player_registry.count as usize).min(MAX_LEADERBOARD);
        // Heap-allocated so we don't burn ~1.3 KB of stack at MAX_LEADERBOARD=20.
        let mut entries: Vec<LeaderboardEntry> = Vec::with_capacity(count);

        for i in 0..count {
            let acc = &ctx.remaining_accounts[NUM_COMPONENTS + i];
            let data = acc.try_borrow_data()?;
            if data.len() < PS_MIN_LEN { continue; }

            let mut authority = [0u8; 32];
            authority.copy_from_slice(&data[PS_AUTHORITY..PS_AUTHORITY + 32]);

            let alive = data[PS_ALIVE] != 0;
            let balance = u64::from_le_bytes(data[PS_BALANCE..PS_BALANCE + 8].try_into().unwrap());
            let leverage = data[PS_LEVERAGE];
            let position_size = u64::from_le_bytes(
                data[PS_POSITION_SIZE..PS_POSITION_SIZE + 8].try_into().unwrap()
            );
            let realized_pnl = i64::from_le_bytes(
                data[PS_REALIZED_PNL..PS_REALIZED_PNL + 8].try_into().unwrap()
            );
            let unrealized_pnl = i64::from_le_bytes(
                data[PS_UNREALIZED_PNL..PS_UNREALIZED_PNL + 8].try_into().unwrap()
            );
            drop(data);

            // Net worth = free cash + locked margin + flying PnL
            let margin = if leverage > 0 { (position_size / leverage as u64) as i64 } else { 0 };
            let net_worth = (balance as i64)
                .saturating_add(margin)
                .saturating_add(unrealized_pnl);

            entries.push(LeaderboardEntry {
                pubkey: authority,
                net_worth,
                balance,
                unrealized_pnl,
                realized_pnl,
                alive,
            });
        }

        // Insertion sort, net_worth DESC. Tie-breaker: alive ranks above dead.
        let filled = entries.len();
        for i in 1..filled {
            let mut j = i;
            while j > 0 {
                let a = &entries[j - 1];
                let b = &entries[j];
                let swap = b.net_worth > a.net_worth
                    || (b.net_worth == a.net_worth && b.alive && !a.alive);
                if !swap { break; }
                entries.swap(j - 1, j);
                j -= 1;
            }
        }

        for i in 0..filled {
            ctx.accounts.leaderboard.entries[i] = entries[i];
        }
        ctx.accounts.leaderboard.count = filled as u8;
        ctx.accounts.game_config.status = 2;

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub game_config: GameConfig,
        pub player_registry: PlayerRegistry,
        pub leaderboard: Leaderboard,
    }
}
