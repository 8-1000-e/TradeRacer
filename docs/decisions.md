# Decisions

## 2026-05-05 — Game spec: 5-min battle royale, leverage 2× to 50×

**Decision**: Each round: every player gets a starting fake balance, opens
ONE long or short SOL position with leverage from the tier list
[2, 5, 10, 25, 50], and the highest PnL after 5 minutes wins. Liquidation
when `margin + unrealized ≤ 0`.

**Context**: User wants a battle-royale trading game inspired by red-light
(MagicBlock ER + Bolt ECS), with a per-tick price feed driving live PnL.

**Rationale**: Familiar perp trading mechanics (any DEX user gets it
instantly), short rounds keep matches snappy, the leverage tier list is
broad enough that "go big or go home" plays exist alongside conservative
ones.

**Alternatives considered**:
- Free leverage input (any integer): rejected — encourages 1000× cheese
  and complicates the liq-price math (rounding) without adding gameplay.
- No fixed round duration (last-man-standing): rejected — predictability
  matters for spectators and the leaderboard.

---

## 2026-05-05 — Bolt ECS over plain Anchor

**Decision**: Build on Bolt ECS (`bolt-lang 0.2.6` from `magicblock-labs/bolt`
git main). Components/systems pattern, with components/* + systems/* +
libs/shared.

**Context**: red-light (sister project) is on Bolt and runs cleanly on
MagicBlock's Ephemeral Rollups. Same infra means we can reuse the cranker
pattern, the websocket layer, and the front-end Bolt SDK integration.

**Rationale**: ECS gives us first-class entity composition (Player +
GameConfig as separate components on the same World), and the Bolt SDK's
`ApplySystem` handles delegation/CPI plumbing automatically.

**How to apply**: `bolt init`'d the scaffold, kept the structure
(`programs/`, `programs-ecs/components/*`, `programs-ecs/systems/*`).
Added a libs/* sibling for shared code (Pyth read + JSON parsers + errors)
— see [setup.md](setup.md) for the workspace patch needed to make it build.

---

## 2026-05-06 — All values in USD with 8 decimals (NOT SOL lamports)

**Decision**: `balance`, `margin`, `position_size`, `entry_price`,
`liq_price`, `realized_pnl`, `unrealized_pnl` — every dollar-denominated
field on PlayerState — uses `USD_DECIMALS = 8`. Starting balance =
`STARTING_BALANCE = 250_000_000_000` (= $2,500).

**Context**: Initial spec was "give each player 10 fake SOL on spawn"
(`STARTING_BALANCE = 10_000_000_000` lamports). But the PnL formula is
`unrealized = (price − entry) × size / entry`, where `price` is Pyth
Lazer USD-with-8-decimals. Mixing lamports (for size) and USD (for price)
in that formula yields meaningless numbers.

**Rationale**: Pyth Lazer's raw `u64` is already 8-decimal USD. By making
all balance/margin/size fields use the same scale, PnL math becomes
single-unit (USD) and never needs a SOL→USD conversion. Real perps work
this way too — margin and size are quoted in the quote currency (USD/USDC),
not the underlying.

**Alternatives considered**:
- Store balance in SOL lamports, do an on-the-fly USD conversion in
  every PnL calc: rejected — adds rounding error per-tick and double the
  arithmetic on-chain.
- Use 6 decimals (USDC standard): rejected — the Pyth feed is 8 decimals,
  and matching it avoids one rescaling.

**How to apply**: PlayerState comments + shared/lib.rs constants both
say "fake USD (8 decimals)". UI shows "USDC" but the on-chain unit is
abstract.

---

## 2026-05-06 — Pre-compute `liq_price` at open-position, not per-tick

**Decision**: When `open-position` runs, immediately compute and store
`liq_price = entry × (leverage−1)/leverage` (long) or
`entry × (leverage+1)/leverage` (short) on PlayerState. close-position
then just compares `current_price` to `liq_price` for the trigger
(no PnL recomputation needed for the liq decision).

**Context**: An earlier draft of close-position computed
`unrealized_pnl` every tick THEN checked
`if margin + unrealized ≤ 0`. That works but does the same arithmetic
twice (once for PnL display, once for liq trigger). The trigger is a
cheap price comparison if we know the liq price up-front.

**Rationale**: Reduces compute-per-tick (cranker calls close-position
once per player per second — pre-computed liq is meaningfully cheaper).
Also conceptually cleaner: liq is a price-level event, not a math event.

**How to apply**: see [open-position](../programs-ecs/systems/open-position/src/lib.rs)
for the formula and [close-position](../programs-ecs/systems/close-position/src/lib.rs)
for the comparison-only check.

---

## 2026-05-06 — `close-position` is a merged "tick + close" system

**Decision**: One system named `close-position` handles three modes:
1. **Tick** (default, no JSON arg): refresh `unrealized_pnl` for display
2. **Voluntary close** (`{"close":1}` arg): realize PnL, credit balance, reset to FLAT
3. **Auto-liquidation** (when `margin + unrealized ≤ 0` regardless of arg): force-close

**Context**: Initial design had `update-pnl` (tick) + `close-position`
(voluntary close) as two separate systems. User asked to merge them after
realizing the cranker would call both per player per tick anyway, doubling
the transaction count.

**Rationale**: One system per player per tick (cranker calls close-position
once with no `close` flag) handles both PnL refresh AND liquidation.
Voluntary close is just the same system with a different arg from the
player. Halves the cranker tx count.

**Trade-off**: One bigger system instead of two smaller ones. Code is
slightly more branchy (the if-close-else-tick split inside `execute`),
but the deployment surface and cranker's coordination cost are both lower.

---

## 2026-05-06 — `refresh-leaderboard` is a separate system, not merged with close-position

**Decision**: `refresh-leaderboard` is its own system with components
[`game_config`, `player_registry`, `leaderboard`] and `remaining_accounts`
of all PlayerState PDAs. The cranker calls it ONCE per tick, AFTER all
the per-player close-position calls.

**Context**: Could have folded leaderboard refresh into close-position
(each tick, after computing the player's PnL, sort and write the
leaderboard). But close-position only has access to ONE PlayerState
(the player it's ticking). It can't see all 10 players to sort them.

**Rationale**: refresh-leaderboard reads ALL PlayerState PDAs in one
go, computes net_worth = balance + margin + unrealized for each, sorts
desc, writes to Leaderboard component. One sorted snapshot per tick is
all clients need. Decoupled from per-player ticks.

**How to apply**: cranker pattern is `for each player: close-position;
then refresh-leaderboard`. The leaderboard reads then become a single
PDA fetch on the front-end.

---

## (append future decisions below — never edit existing entries)
