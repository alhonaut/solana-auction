import * as anchor from "@coral-xyz/anchor";
import { Program, BN, Wallet } from "@coral-xyz/anchor";
import { NftMinter } from "../../target/types/nft_minter";
import { NFT, Creator } from "../interfaces";
import * as pda from "../pda";
import { TOKEN_METADATA_PROGRAM_ID } from "../generated";

export async function createNft({
  nftMinterProgram,
  payer,
  name,
  symbol,
  uri,
  creators,
  sellerFeeBasisPoints,
  isMutable,
  maxSupply,
}: {
  nftMinterProgram: Program<NftMinter>;
  payer: Wallet;

  name: string;
  symbol: string;
  uri: string;
  creators?: Creator[];
  sellerFeeBasisPoints: number;
  isMutable: boolean;
  maxSupply: number;
}) {
  const mint = anchor.web3.Keypair.generate();
  const owner = payer.payer;
  const ata = anchor.utils.token.associatedAddress({
    mint: mint.publicKey,
    owner: owner.publicKey,
  });
  const [metadata] = pda.findMetadataAddress({
    mint: mint.publicKey,
  });
  const [masterEdition] = pda.findEditionAddress({
    mint: mint.publicKey,
  });

  const createTokenTx = await nftMinterProgram.methods
    .createToken(
      name,
      symbol,
      uri,
      creators ?? null,
      sellerFeeBasisPoints,
      isMutable
    )
    .accounts({
      payer: payer.publicKey,
      mintAccount: mint.publicKey,
      mintAuthority: payer.publicKey,
      updateAuthority: payer.publicKey,
      metadataAccount: metadata,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
    })
    .signers([mint, payer.payer])
    .rpc();

  console.log("Transaction [Create Token]", createTokenTx);

  const mintTokenTx = await nftMinterProgram.methods
    .mintToken(new BN(maxSupply))
    .accounts({
      payer: payer.publicKey,
      mintAccount: mint.publicKey,
      mintAuthority: payer.publicKey,
      updateAuthority: payer.publicKey,
      associatedTokenAccount: ata,
      metadataAccount: metadata,
      editionAccount: masterEdition,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
    })
    .signers([mint, payer.payer])
    .rpc();
  console.log("Transaction [Mint Token]", mintTokenTx);

  const token: NFT = {
    mint,
    owner: payer.payer,
    ata,
    metadata,
    masterEdition,
  };

  return token;
}
