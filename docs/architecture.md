# Architecture

## On-chain layer (Bolt ECS)

The program is structured as a single **World** containing one **Game entity**
that holds the four singleton components. Each connected player additionally
gets a **Player entity** holding their own `PlayerState`.

```
World
├─ Game entity
│   ├─ GameConfig         (status, lobby_end, game_end, max_players, …)
│   ├─ PlayerRegistry     (count, active_players, fixed-size player_pdas[10])
│   └─ Leaderboard        (entries[10], updated_at, finalized)
└─ Player entity (×N)
    └─ PlayerState        (authority, balance, position, leverage, entry_price,
                           position_size, liq_price, realized/unrealized_pnl)
```

All dollar-denominated fields use **USD with 8 decimals** to match the Pyth
Lazer raw `u64`. See [decisions.md](decisions.md).

## System flow

```
                          ┌─ open-position ←─┐
init-game → spawn-player → start-game        │
                          └─ close-position ←┘   (called every tick by cranker)
                                              │
                                              └─ refresh-leaderboard (cranker, 1×/tick)
                                              │
                                              └─ end-game (when now ≥ game_end)
```

| System | Components touched | Pyth needed? | Args |
|---|---|---|---|
| `init-game` | GameConfig | No | `{}` |
| `spawn-player` | PlayerRegistry, PlayerState | No | `{}` |
| `start-game` | GameConfig | No | `{}` |
| `open-position` | PlayerState, GameConfig | Yes | `{"direction":0\|1,"leverage":2\|5\|…,"margin":<u64>}` |
| `close-position` | PlayerState, GameConfig | Yes | `{}` (tick) or `{"close":1}` (voluntary) |
| `refresh-leaderboard` | GameConfig, PlayerRegistry, Leaderboard | No | `{}` + all PlayerState PDAs in `remaining_accounts` |
| `end-game` | GameConfig, PlayerRegistry, Leaderboard | No | `{}` + all PlayerState PDAs in `remaining_accounts` |

## Cross-cutting patterns

**Pyth account passing**: every system that needs the price expects the Pyth
Lazer SOL/USD account at `remaining_accounts.last()`. The on-chain helper
[`read_pyth_price`](../programs-ecs/libs/shared/src/lib.rs) parses 8 LE bytes
at offset 73. Devnet PDA: `ENYwebBThHzmzwPLAQvCucUTsjyfBSZdD9ViXksS4jPu`.

**Iterating PlayerState PDAs (refresh-leaderboard, end-game)**: Bolt prepends
`NUM_COMPONENTS` slots to `remaining_accounts` (so the framework can resolve
the components declared in the system's `#[arguments]`). Extra accounts the
client passes start at index `NUM_COMPONENTS`. We deserialize each
PlayerState by raw byte offsets (faster than full `try_deserialize` and avoids
pulling the component crate as a dependency for the system that aggregates
across players).

**Liquidation trigger**: pre-computed `liq_price` field on PlayerState (set
in open-position, checked in close-position) — a price comparison rather than
re-running the PnL formula every tick. See [decisions.md](decisions.md).

**Net worth for ranking**: `balance + margin + unrealized_pnl`. Computed in
both `refresh-leaderboard` (for the live ranking) and `end-game` (for the
final ranking).

## Off-chain pieces

- **Cranker** (TS, not yet implemented): every ~1s, calls `close-position`
  per alive player with no `close` arg (= tick mode), then calls
  `refresh-leaderboard` once with all PlayerState PDAs as
  `remaining_accounts`. This is what drives live PnL + auto-liquidation.
- **Frontend** (separate repo `front-dev`, branch `trade-fight-cockpit`):
  cockpit-DA Next.js app. Currently renders mock state. On-chain wiring
  via `bolt-sdk`'s `ApplySystem` is the next FE task.

## Dependencies

The repo follows the **bolt-lang from git main + session-keys local patch +
Cargo.lock pinned from red-light** recipe — see [setup.md](setup.md). This
sidesteps the rustc-1.84/edition2024 build conflict that affects
out-of-the-box `bolt init` projects.
