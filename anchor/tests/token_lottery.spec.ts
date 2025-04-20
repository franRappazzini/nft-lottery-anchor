import * as anchor from "@coral-xyz/anchor";

import { describe, it } from "node:test";

import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { TokenLottery } from "anchor/target/types/token_lottery";

// import { describe, it } from "node:test";

describe("Token Lottery Program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;

  const buyTicket = async () => {
    const buyTx = await program.methods
      .buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      // .rpc({ skipPreflight: true });
      .instruction();

    // const computeInstruction = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
    //   units: 300000,
    // });

    // const computePrice = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
    //   microLamports: 1,
    // });

    const latestBlockchain = await provider.connection.getLatestBlockhash();
    const tx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: latestBlockchain.blockhash,
      lastValidBlockHeight: latestBlockchain.lastValidBlockHeight,
    }).add(buyTx);
    // .add(computeInstruction)
    // .add(computePrice);

    const signature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      tx,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log("Buy ticket tx signature:", signature);
  };

  it("Should initilize configs and lottery", async () => {
    // Add your test here.
    const slot = await provider.connection.getSlot();

    const instructionTx = await program.methods
      .initializeConfig(bn(0), bn(slot + 100), bn(10000))
      .instruction();

    // PARA EL metadata_program_account ES NECESARIO DESCARGAR EL PROGRAMA DE MAINNET Y EJECUTARLO LOCAL
    // https://solana.com/es/developers/cookbook/development/using-mainnet-accounts-programs
    /*
      # solana program dump -u <source cluster> <address of account to fetch> <destination file name/path>
      solana program dump -u m PROGRAM_ID NAME.so

      # solana-test-validator --bpf-program <address to load the program to> <path to program file> --reset
      solana-test-validator --bpf-program PROGRAM_ID NAME.so --reset
    */
    const instructionLottery = await program.methods
      .initializeLottery()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const latestBlockchain = await provider.connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: latestBlockchain.blockhash,
      lastValidBlockHeight: latestBlockchain.lastValidBlockHeight,
    })
      .add(instructionTx)
      .add(instructionLottery);

    const signature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      tx,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log("tx signature:", signature);

    await buyTicket();
  });

  // it("Should initialize and config the lottery", async () => {
  //   const slot = await provider.connection.getSlot();

  //   const configTx = await program.methods
  //     .initializeConfig(bn(0), bn(slot + 100), bn(10000))
  //     // .accounts({})
  //     .rpc();

  //   console.log("Config tx signature:", configTx);

  //   const lotteryTx = await program.methods
  //     .initializeLottery()
  //     .accounts({
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .rpc();

  //   console.log("Lottery tx signature:", lotteryTx);

  //   // Fetch the account data
  //   // const lotteryAccount = await program.account.tokenLottery.fetch(program.programId);
  //   // console.log("Lottery account:", lotteryAccount);

  //   console.log("program id:", program.programId.toBase58());

  //   await buyTicket();
  // });
});

function bn(n: number) {
  return new anchor.BN(n);
}
