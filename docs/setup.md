# Setup

## Prerequisites

- Rust + cargo (current stable)
- Solana CLI
- Anchor CLI (matching what `bolt-lang` from git main pins — currently
  Anchor `0.30.x` family)
- Bolt CLI: `cargo install --git https://github.com/magicblock-labs/bolt.git bolt-cli`
- Node.js + pnpm/npm (for tests + cranker once it exists)

## First-time bootstrap

The project was created via `bolt init`, then patched to make it actually
build under current toolchains. Anyone cloning fresh needs the following.

### 1. Cargo workspace patch

[`Cargo.toml`](../Cargo.toml) already contains:

```toml
[workspace.dependencies]
bolt-lang = { git = "https://github.com/magicblock-labs/bolt.git", branch = "main" }

[patch.crates-io]
session-keys = { path = "patches/session-keys" }
```

These two lines are the result of debugging `bolt build` failures —
DON'T remove them without re-validating. See [debugging-notes.md](debugging-notes.md).

### 2. `patches/session-keys/`

This directory is committed to the repo. It's a copy of the `session-keys`
crate from red-light, with its `Cargo.toml` `version` bumped to `2.0.8`
(to match what `bolt-lang` from git main expects). If `bolt build`
complains about a version mismatch, check the version field here.

### 3. `Cargo.lock`

Committed. Pinned from red-light. Keeps `blake3 1.5.5` + `indexmap 2.13.0`
+ other transitive deps at versions that compile cleanly under current
rustc. **Do not run `cargo update`** without re-running `bolt build`
afterwards and being prepared to revert.

## Build & test

```bash
cd /Users/emile/Documents/TNTX/trade-fight
bolt build       # compiles all components + systems
bolt test        # runs Mocha integration tests (currently default scaffold)
```

If `bolt build` fails out of nowhere, check:
1. Did anyone touch `Cargo.toml` or `Cargo.lock`?
2. Did `cargo update` run inadvertently (e.g. via an editor extension)?
3. Is `patches/session-keys/Cargo.toml` still at `version = "2.0.8"`?

## Devnet deployment (later)

```bash
anchor keys sync          # sync any drifted program IDs
bolt deploy --provider.cluster devnet
```

Or use MagicBlock Ephemeral Rollups for the real-time game loop (cranker
calling close-position every second works much better on ER than on
plain Solana — far cheaper + faster slots).

## Frontend (separate repo)

```bash
cd /Users/emile/Documents/TNTX/front-dev
git checkout trade-fight-cockpit
npm install
npm run dev      # localhost:3000 → click PLAY on the trade-fight card
```

Currently renders mock state. On-chain wiring (bolt-sdk `ApplySystem` +
PDA subscriptions) is the next FE task — see [current-task.md](current-task.md).

## Pyth Lazer feed

Devnet PDA: `ENYwebBThHzmzwPLAQvCucUTsjyfBSZdD9ViXksS4jPu`

The on-chain helper [`read_pyth_price`](../programs-ecs/libs/shared/src/lib.rs)
parses 8 LE bytes at offset 73 of this account — bypasses `pyth-sdk-solana`
entirely (cuts crate bloat + avoids version conflicts with bolt-lang).
Pattern documented in `~/.claude/skills/brain-dump/extracted/pyth-lazer-raw-read-offset-73/`.

Frontend / cranker pass this PDA via `ApplySystem`'s `extraAccounts`:

```ts
extraAccounts: [
  { pubkey: PYTH_LAZER_SOL_USD, isWritable: false, isSigner: false },
],
```

Always at the END of the extras array — the on-chain helper reads
`remaining_accounts.last()`.
