import * as anchor from "@coral-xyz/anchor";
import { Wallet, Provider } from "@coral-xyz/anchor";

export const MAX_UINT64 = "18446744073709551615";
export const ONE_SOL = anchor.web3.LAMPORTS_PER_SOL;

const EMPTY_SPACE = 0;

export const createSystemAccount = async ({
  provider,
  payer,
}: {
  provider: Provider;
  payer: Wallet;
}): Promise<anchor.web3.Keypair> => {
  const newAccountKeypair = anchor.web3.Keypair.generate();
  const lamports = await provider.connection.getMinimumBalanceForRentExemption(
    EMPTY_SPACE
  );

  const tx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: newAccountKeypair.publicKey,
      lamports,
      space: EMPTY_SPACE,
      programId: anchor.web3.SystemProgram.programId,
    }),
    anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: newAccountKeypair.publicKey,
      lamports: 10 * ONE_SOL,
    })
  );

  await provider.sendAndConfirm(tx, [payer.payer, newAccountKeypair]);

  return newAccountKeypair;
};

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
