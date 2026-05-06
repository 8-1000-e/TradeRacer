## Context Recovery

IMPORTANT: At session start, read all .md files in the [/docs/](docs/) directory
to restore full project context from the previous session.

## Current State

- **Branch**: main
- **Status**: All 6 systems compile (`bolt build` green). On-chain layer is
  feature-complete for the v0 game loop. No real on-chain wiring yet on the
  frontend — front renders mocks.
- **Frontend**: in [/Users/emile/Documents/TNTX/front-dev/](file:///Users/emile/Documents/TNTX/front-dev/)
  on branch `trade-fight-cockpit` (cockpit DA + race-track minimap + leaderboard
  pills + tx toasts, all running off mock data).
- **Last updated**: 2026-05-06

## Task Progress

- [x] Components: `game-config`, `player-state`, `player-registry`, `leaderboard`
- [x] Shared lib: `read_pyth_price` (offset 73), `parse_json_*`, `GameError` enum, USD-with-8-decimals constants
- [x] Systems: `init-game`, `spawn-player`, `start-game`, `open-position`, `close-position` (merged tick + close + auto-liq), `refresh-leaderboard`, `end-game`
- [x] All amounts switched from "10 fake SOL lamports" → **USD with 8 decimals** (matches Pyth Lazer feed unit) — see [docs/decisions.md](docs/decisions.md)
- [x] Liquidation precomputed via `liq_price` field on PlayerState (set in open-position, checked in close-position) — avoids re-computing the trigger every tick
- [x] Bolt build setup wired (bolt-lang from git main + session-keys local patch + Cargo.lock pinned from red-light) — [docs/setup.md](docs/setup.md)
- [x] Frontend cockpit DA implemented in front-dev with race-track minimap + tx toasts <- previous session work
- [ ] **Tests** — `tests/` only has the default Mocha scaffold from `bolt init` <- CURRENT next step
- [ ] **Cranker** — TS script that polls `close-position` per player + `refresh-leaderboard` once per tick
- [ ] **Frontend on-chain wiring** — replace mock state in front-dev's `trade-fight.tsx` with `bolt-sdk` calls

## Key Decisions

- **All values in USD with 8 decimals** (no SOL lamports anywhere). Math stays
  in one unit, no conversions. See [docs/decisions.md](docs/decisions.md).
- **`close-position` is a merged "tick + close" system** — passed `"close":1`
  in JSON args = voluntary close; otherwise just refreshes `unrealized_pnl`.
- **Per-position liquidation, not account-wide** — when `margin + unrealized ≤ 0`,
  the position auto-closes with `realized_pnl -= margin`; balance survives so
  the player keeps trading until balance + open margin all reach 0.
- **`refresh-leaderboard` is a separate system** (not merged into close-position)
  — cranker calls it once per tick after the per-player close-position passes.
- **Bolt extras pattern**: PlayerState PDAs are passed via `remaining_accounts`
  AFTER the `NUM_COMPONENTS` slots Bolt prepends — see end-game / refresh-leaderboard
  for the iteration template.

## Programs

| Program | Address | File |
|---|---|---|
| trade_fight | `7mUMr33noPhfFnnQJfY6BwziCAWnevAKHv2AnJ65d1B4` | [programs/trade-fight/src/lib.rs](programs/trade-fight/src/lib.rs) |
| game-config | `D3AZXBim4wY9p2vt4ysHzL1M692ErQ4KtsnvWXS11YA7` | [programs-ecs/components/game-config/src/lib.rs](programs-ecs/components/game-config/src/lib.rs) |
| player-state | `DTQz7WWMC6wpnBNMwujh9BUeHrezBcXfsrUM1KsdBL73` | [programs-ecs/components/player-state/src/lib.rs](programs-ecs/components/player-state/src/lib.rs) |
| player-registry | `Dzi4us11W4QCSD5Erx2vfR15GyuMZn6S4Djr5HfPVVBm` | [programs-ecs/components/player-registry/src/lib.rs](programs-ecs/components/player-registry/src/lib.rs) |
| leaderboard | `EYcpWDjusuacuFrcz4JKnNDU78gsPpJYcmyrGZ2s9qz` | [programs-ecs/components/leaderboard/src/lib.rs](programs-ecs/components/leaderboard/src/lib.rs) |
| init-game | `9KGctbjYwgE3jJhb265BQ4GXDbVX7tnSptCLH56Agdtk` | [programs-ecs/systems/init-game/src/lib.rs](programs-ecs/systems/init-game/src/lib.rs) |
| spawn-player | `FnRjhEaucBXa3ZNZtosKuCjdxnG4VaVe4MCNZa2HxDzB` | [programs-ecs/systems/spawn-player/src/lib.rs](programs-ecs/systems/spawn-player/src/lib.rs) |
| start-game | `9KvG9Htsnew3Tsvt7462bxc93HjJ6g3nK6XAjeN1Uupu` | [programs-ecs/systems/start-game/src/lib.rs](programs-ecs/systems/start-game/src/lib.rs) |
| open-position | `HbiVhxLoCFQ2uYzAAKXisSpiyEdve38bvva5v97nTwVw` | [programs-ecs/systems/open-position/src/lib.rs](programs-ecs/systems/open-position/src/lib.rs) |
| close-position | `ChETGKhsRzTynoskTMnaPUfyB2GUmCoE228Gv6FoczT9` | [programs-ecs/systems/close-position/src/lib.rs](programs-ecs/systems/close-position/src/lib.rs) |
| refresh-leaderboard | `4tp1GQbdrhqw45jwwdZPG5e1Njhhbd5tVXg94cRdxdRo` | [programs-ecs/systems/refresh-leaderboard/src/lib.rs](programs-ecs/systems/refresh-leaderboard/src/lib.rs) |
| end-game | `Hx4rWEBiQb1z41gByya3yvDCjZYcvkdF946AzQ6mbaru` | [programs-ecs/systems/end-game/src/lib.rs](programs-ecs/systems/end-game/src/lib.rs) |
