# Debugging Notes

## bolt build fails with rustc 1.84 / edition2024 errors

### Symptoms
- `bolt init`'d project, ran `bolt build`, got errors about `edition2024`
  being unstable, errors deep in `solana-program` / `borsh` transitive deps,
  errors about `ahash` features.
- Errors do not mention bolt itself — they come from the Anchor / Solana
  crate stack pulled in as transitive deps.

### Root cause
The default `bolt-lang = "0.2.x"` from crates.io pulls a transitive set of
Solana / Anchor deps that don't compile under recent stable rustc + the
edition2024 unstable feature gate. The `session-keys` crate in particular
(used by Bolt for delegation) ships with a manifest that requests features
incompatible with current toolchains.

### Fix
Three coordinated changes, all in [`Cargo.toml`](../Cargo.toml) +
`patches/`:

1. Switch `bolt-lang` to git main:
   ```toml
   bolt-lang = { git = "https://github.com/magicblock-labs/bolt.git", branch = "main" }
   ```
2. Copy `session-keys` from red-light into `patches/session-keys/`, bump
   its version to `2.0.8`, and patch:
   ```toml
   [patch.crates-io]
   session-keys = { path = "patches/session-keys" }
   ```
3. Copy `Cargo.lock` from red-light wholesale (pins `blake3 1.5.5`,
   `indexmap 2.13.0`, etc., which compile cleanly under current toolchain).

After this, `bolt build` is green. Full recipe in [setup.md](setup.md).

### Gotcha
Don't run `cargo update` after applying the fix — it'll re-resolve the
pinned deps and re-introduce the broken versions. If a new dep needs
adding, prefer adding it to `Cargo.toml` and letting `cargo` resolve only
the new entries (or update the pinned versions one at a time and re-test).

---

## React auto-liquidation `setTimeout` never fires

### Symptoms
- Frontend (front-dev `trade-fight-cockpit`): SHORT positions are supposed
  to auto-liquidate after 4s (mock simulation). Pressing SHORT showed the
  position open, but the liquidation toast never fired.
- No console errors. The 4s timer was registered, just no-op'd at fire
  time.

### Root cause
Classic React stale-closure bug:
```tsx
const openPosition = (side) => {
  setPosition({ side, entry: livePrice, /* ... */ });   // async
  if (side === "SHORT") setTimeout(() => closePosition("liquidated"), 4000);
};

const closePosition = (reason) => {
  if (!position) return;   // ← STALE: still null at fire time!
};
```
The `setTimeout` captured the closure from the render where `position`
was still `null`. 4s later, `closePosition` ran against that stale closure
and early-returned.

### Fix
Move the timer to a `useEffect` that watches a stable identifier of the
new position, so it re-binds AFTER React has applied the state update:
```tsx
useEffect(() => {
  if (position?.side !== "SHORT") return;
  const id = setTimeout(() => closePosition("liquidated"), 4000);
  return () => clearTimeout(id);
}, [position?.openedAt]);
```

This pattern was extracted into a permanent skill at
`~/.claude/skills/brain-dump/extracted/react-settimeout-stale-state-closure/`.

---

## Bolt `remaining_accounts` indexing for cross-entity reads

### Symptoms
While building `refresh-leaderboard` (which needs to read ALL PlayerState
PDAs to sort them), early drafts indexed `remaining_accounts[0]` and got
the wrong account back — a Bolt-internal account, not the first player.

### Root cause
Bolt's `ApplySystem` machinery pre-pends `NUM_COMPONENTS` slots to
`remaining_accounts` (one per component declared in `#[arguments]` for the
system). The "extras" the client passes via `extraAccounts` start at
index `NUM_COMPONENTS`, not 0.

For `refresh-leaderboard`: 3 components declared (game_config,
player_registry, leaderboard) → `NUM_COMPONENTS = 3` → player PDAs start
at `remaining_accounts[3]`.

### Fix
Always offset by `NUM_COMPONENTS`:
```rust
const NUM_COMPONENTS: usize = 3;
for i in 0..registry.count as usize {
    let acc = &ctx.remaining_accounts[NUM_COMPONENTS + i];
    let data = acc.try_borrow_data()?;
    // parse PlayerState fields by byte offset
}
```

### Gotcha
The byte-offset parsing in [refresh-leaderboard](../programs-ecs/systems/refresh-leaderboard/src/lib.rs)
is hand-rolled — it does NOT call `PlayerState::try_deserialize`. If the
PlayerState struct layout changes (fields added/removed/reordered), the
offsets in refresh-leaderboard AND end-game must be updated. There is no
compile-time check for this — only runtime mis-reads.

---

## Unit mismatch: SOL lamports vs Pyth USD

### Symptoms
Caught mid-implementation, before any test ran. PnL formula
`unrealized = (price − entry) × size / entry` was producing nonsensical
values when `size` was in SOL lamports (10⁹ scale) and `price`/`entry`
in Pyth Lazer USD (10⁸ scale).

### Root cause
Initial spec said "give each player 10 fake SOL". `STARTING_BALANCE` was
`10_000_000_000` lamports. But every other dollar field (entry_price,
position_size, margin) ended up in Pyth USD scale (10⁸) because they
derive from the Pyth feed.

### Fix
Switched everything to **USD with 8 decimals** (matching Pyth). New
constants:
- `STARTING_BALANCE = 250_000_000_000` (= $2,500 × 10⁸)
- `USD_DECIMALS = 8`
- All PlayerState dollar fields commented "fake USD with 8 decimals"
- UI shows "USDC" but the on-chain unit is abstract

Decision recorded in [decisions.md](decisions.md) (2026-05-06 entry).

### Gotcha
The frontend label says "USDC" but the program never touches a real
USDC mint. Anyone wiring up real-asset settlement later must remember
this is purely a fake balance — there is no SPL transfer in the flow.

---

## airport-carousel `OutOfRange` errors after PICK_RANGE_PX bump

### Symptoms (back-dev, separate codebase but worked on in same session)
Players reported "OutOfRange" errors when trying to pick bags slightly
outside the claw's central pick zone.

### Fix
Bumped `PICK_RANGE_PX` from `90` → `130` in
[`back-dev/src/games/airport-carousel/match-simulator.ts`](file:///Users/emile/Documents/TNTX/back-dev/src/games/airport-carousel/match-simulator.ts).
Committed on branch `airport-pick-tolerance` with a one-line message.

### Gotcha
The "CHEAT TEST 400" debug button on the frontend (added the same session)
sends `bagId=26` (the slot that holds the 400-point bag in the test fixture).
First attempt sent `bagId=400`, which hit the `bagId >= COUNT` validation
before any distance check — yielding `InvalidBagId`, not `OutOfRange`.
Lesson: validate test fixtures match the bagId index space, not the score
space.
