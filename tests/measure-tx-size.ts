/**
 * Measure how many player PlayerState PDAs we can stuff in `remaining_accounts`
 * of a `refresh-leaderboard` ApplySystem transaction without busting the
 * 1232-byte legacy TX limit.
 *
 * Run: yarn ts-node tests/measure-tx-size.ts
 */
import {
  PublicKey,
  Keypair,
  Transaction,
  SystemProgram,
  TransactionInstruction,
  Message,
  VersionedTransaction,
  TransactionMessage,
} from "@solana/web3.js";
import { ApplySystem } from "@magicblock-labs/bolt-sdk";
import * as anchor from "@coral-xyz/anchor";

const TX_MAX = 1232;

// Hardcoded program IDs from Anchor.toml so we don't need a live workspace.
const REFRESH_LEADERBOARD = new PublicKey(
  "4tp1GQbdrhqw45jwwdZPG5e1Njhhbd5tVXg94cRdxdRo",
);
const GAME_CONFIG = new PublicKey(
  "D3AZXBim4wY9p2vt4ysHzL1M692ErQ4KtsnvWXS11YA7",
);
const PLAYER_REGISTRY = new PublicKey(
  "Dzi4us11W4QCSD5Erx2vfR15GyuMZn6S4Djr5HfPVVBm",
);
const LEADERBOARD = new PublicKey(
  "EYcpWDjusuacuFrcz4JKnNDU78gsPpJYcmyrGZ2s9qz",
);

const FAKE_BLOCKHASH = "11111111111111111111111111111111";

function randomPubkey(): PublicKey {
  return Keypair.generate().publicKey;
}

async function buildTx(playerCount: number, payer: PublicKey): Promise<{
  legacyBytes: number;
  v0NoLutBytes: number;
}> {
  // World/entity PDAs are placeholders for sizing — exact value doesn't matter
  // as long as we have the right number of accounts.
  const worldPda = randomPubkey();
  const entity = randomPubkey();

  const extraAccounts = Array.from({ length: playerCount }, () => ({
    pubkey: randomPubkey(),
    isSigner: false,
    isWritable: false,
  }));

  const apply = await ApplySystem({
    authority: payer,
    world: worldPda,
    systemId: REFRESH_LEADERBOARD,
    entities: [
      {
        entity,
        components: [
          { componentId: GAME_CONFIG },
          { componentId: PLAYER_REGISTRY },
          { componentId: LEADERBOARD },
        ],
      },
    ],
    extraAccounts,
  });

  const ixs = apply.transaction.instructions;

  // Legacy
  const legacyTx = new Transaction().add(...ixs);
  legacyTx.feePayer = payer;
  legacyTx.recentBlockhash = FAKE_BLOCKHASH;
  // serializeMessage gives bytes; +1 (sig count) + 64 (signature)
  const legacyBytes = legacyTx.serializeMessage().length + 1 + 64;

  // v0 (without LUT)
  const msg = new TransactionMessage({
    payerKey: payer,
    recentBlockhash: FAKE_BLOCKHASH,
    instructions: ixs,
  }).compileToV0Message();
  const v0Tx = new VersionedTransaction(msg);
  // signatures slot is empty placeholder; size includes 1 byte count + N×64
  const v0NoLutBytes = v0Tx.serialize().length;

  return { legacyBytes, v0NoLutBytes };
}

(async () => {
  const payer = Keypair.generate().publicKey;
  console.log("players | legacy bytes | v0 bytes | fits?");
  console.log("--------|--------------|----------|------");
  let lastFitting = 0;
  for (let n = 0; n <= 40; n++) {
    let r;
    try {
      r = await buildTx(n, payer);
    } catch (e) {
      console.log(`${String(n).padStart(7)} | ERROR: ${(e as Error).message}`);
      continue;
    }
    const fits = r.legacyBytes <= TX_MAX && r.v0NoLutBytes <= TX_MAX;
    console.log(
      `${String(n).padStart(7)} | ${String(r.legacyBytes).padStart(12)} | ${String(r.v0NoLutBytes).padStart(8)} | ${fits ? "OK" : "TOO BIG"}`,
    );
    if (fits) lastFitting = n;
    if (!fits && n > 5) {
      console.log(`\nMax players that fits: ${lastFitting}`);
      break;
    }
  }
})();
