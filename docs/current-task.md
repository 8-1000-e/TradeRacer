# Current Task

## What's done

The on-chain layer of trade-fight is **feature-complete for the v0 game loop**:

- 4 components ([programs-ecs/components/](../programs-ecs/components/)):
  `game-config`, `player-state`, `player-registry`, `leaderboard`
- 7 systems ([programs-ecs/systems/](../programs-ecs/systems/)):
  `init-game` → `spawn-player` → `start-game` → `open-position` ⇄
  `close-position` (= tick + voluntary close + auto-liquidation) +
  `refresh-leaderboard` (cranker-driven) → `end-game`
- Shared lib at [programs-ecs/libs/shared/src/lib.rs](../programs-ecs/libs/shared/src/lib.rs):
  `read_pyth_price` (offset 73), `parse_json_*`, `GameError` enum, all
  game constants in USD-with-8-decimals
- `bolt build` is green — see [setup.md](setup.md) for the dependency
  patches needed to actually build

The cockpit-DA frontend is on the front-dev side, branch
`trade-fight-cockpit` — race-track minimap, leaderboard pills, tx
explosion toasts on liquidation, live SOL/USD chart with Pyth Lazer
feed via `useSolPrice`. All running off mock state.

## What's next

### 1. Tests (highest priority)

[tests/](../tests/) only has the default Bolt scaffold (`tests/trade-fight.ts`
+ a Mocha config in `Anchor.toml`).

Write a happy-path integration test that:

1. `init-game` → assert `game_config.status == 0`, `lobby_end` set to
   `now + LOBBY_DURATION_SEC`
2. `spawn-player` × 2 → assert `player_registry.count == 2`,
   `active_players == 2`, balance = `STARTING_BALANCE`
3. `start-game` (after `lobby_end`) → assert `status == 1`,
   `game_end` set to `now + GAME_DURATION_SEC`
4. `open-position` (LONG 10x, margin 100 USD) →
   assert `position == POS_LONG`, `entry_price` ≈ Pyth price,
   `liq_price = entry × 0.9`, `position_size = 100 × 10 USD`,
   `balance` debited
5. `close-position` (with `"close":1`) → assert `position == POS_FLAT`,
   `realized_pnl` updated, `balance` credited
6. `refresh-leaderboard` (with all PlayerState PDAs as
   `remaining_accounts[3..]`) → assert `leaderboard.entries`
   sorted by `net_worth DESC`
7. `end-game` (after `game_end`) → assert `status == 2`,
   leaderboard finalized

Iteration tests:
- Liquidation: open at 50x, simulate price moving 2.5% → close-position
  should auto-trigger force-close, `realized_pnl -= margin`
- Voluntary close at PnL > 0: balance should go up by
  `margin + unrealized`

Pyth feed in tests: pass the **devnet** Pyth Lazer SOL/USD PDA
(`ENYwebBThHzmzwPLAQvCucUTsjyfBSZdD9ViXksS4jPu`) as the last extra
account on every system call that needs the price.

### 2. Cranker

A small TS script (probably under `app/` or a new `crank/` dir) that:

```
loop forever:
  for each alive player with an open position:
    invoke close-position with no "close" arg (= tick mode)
  invoke refresh-leaderboard with all PlayerState PDAs as remaining_accounts
  sleep 1s
```

This is what feeds `unrealized_pnl` updates to clients (so the cockpit's
HULL bar / Live PnL stat reflects market moves) and triggers
liquidations when `margin + unrealized ≤ 0`. Without it, players' PnL
freezes between trades and the leaderboard never re-sorts.

### 3. Frontend on-chain wiring

In [front-dev/src/components/games/trade-fight/](file:///Users/emile/Documents/TNTX/front-dev/src/components/games/trade-fight/):

- **Lobby**: replace mock `players` state in `trade-fight.tsx` with
  socket-driven lobby state (mirror `red-light` lib/lobby-api pattern)
- **Game**: replace mock `position` / `balance` / `history` in
  `components/game.tsx` with subscriptions to the on-chain PlayerState
  + GameConfig + Leaderboard PDAs
- **Open / close**: wire LONG / SHORT hold-to-launch + EJECT button
  to `openPosition` / `closePosition` via bolt-sdk's `ApplySystem`
- **Real tx hashes**: replace `randomTxHash()` in `TxToast` with the
  signature returned by the bolt-sdk call

The cranker should run in a separate process (Node.js worker), NOT in
the React frontend.

### 4. Deployment

Anchor.toml's `[programs.localnet]` is wired with all 11 program IDs.
Run `anchor keys sync` after first build if any IDs drift, then
`bolt deploy --provider.cluster devnet` (or use MagicBlock ER for the
real-time game loop).

## How to run

```bash
cd /Users/emile/Documents/TNTX/trade-fight
bolt build      # see docs/setup.md for the dependency patches
bolt test       # default Mocha scaffold — placeholder
```

Frontend (separate repo):
```bash
cd /Users/emile/Documents/TNTX/front-dev
git checkout trade-fight-cockpit
npm run dev     # localhost:3000 → click PLAY on the trade-fight card
```

## Open questions

- **Liquidation source-of-truth**: currently the cranker triggers
  liquidation by calling close-position. If the cranker is down for
  N seconds and price moves >2% on a 50x position, the player is
  un-liquidated longer than they should be. Acceptable for v0; needs
  thought for v1 (oracle-based attestations? on-chain timer bound?).
- **Round timing**: `LOBBY_DURATION_SEC = 60`, `GAME_DURATION_SEC = 5×60`.
  Re-tune after first internal playtest.
- **Max players**: 10 (red-light precedent). Bumping requires resizing
  PlayerRegistry / Leaderboard fixed arrays + their account space.
