import * as anchor from "@coral-xyz/anchor";
import * as fs from "fs";
import * as sb from "@switchboard-xyz/on-demand";

import { before, describe, it } from "node:test";

import { Program } from "@coral-xyz/anchor";
import SbIdl from "../switchboard_idl.json";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { TokenLottery } from "anchor/target/types/token_lottery";
import dotenv from "dotenv";

// import reader from "readline-sync";

dotenv.config();
console.clear();

describe("Token Lottery Program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet as anchor.Wallet;
  const wallet2 = new anchor.Wallet(anchor.web3.Keypair.generate());

  anchor.setProvider(provider);

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;

  let sbProgram: anchor.Program<anchor.Idl>;
  // const sbProgram = new anchor.Program(SbIdl as anchor.Idl, provider);
  const rngKp = anchor.web3.Keypair.generate();

  // to set the switchboard json
  before(async () => {
    const switchboardIDL = (await anchor.Program.fetchIdl(sb.ON_DEMAND_MAINNET_PID, {
      connection: new anchor.web3.Connection("https://api.mainnet-beta.solana.com"),
    })) as anchor.Idl;

    console.log("switchboard idl", switchboardIDL);

    sbProgram = new anchor.Program(switchboardIDL, provider);

    fs.writeFileSync("switchboard_idl.json", JSON.stringify(switchboardIDL, null, 2));
    console.log("switchboard idl saved");
  });

  const buyTicket = async (/* wallet: anchor.Wallet */) => {
    const buyTx = await program.methods
      .buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
        // buyer: wallet.publicKey,
      })
      // .signers([wallet.payer])
      // .rpc({ skipPreflight: true });
      .instruction();

    const computeInstruction = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300000,
    });

    const computePrice = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1,
    });

    const latestBlockchain = await provider.connection.getLatestBlockhash();
    const tx = new anchor.web3.Transaction({
      feePayer: wallet.publicKey,
      blockhash: latestBlockchain.blockhash,
      lastValidBlockHeight: latestBlockchain.lastValidBlockHeight,
    })
      .add(buyTx)
      .add(computeInstruction)
      .add(computePrice);

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

    console.log(wallet.publicKey.toBase58());
    console.log(wallet2.publicKey.toBase58());

    await buyTicket(/* wallet */);
    await buyTicket(/* wallet2 */);
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();

    const sbQueue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"); // mainnet
    // const sbQueue = new anchor.web3.PublicKey(SbIdl.address); // localnet

    const queueAccount = new sb.Queue(sbProgram, sbQueue);
    console.log("Queue account", sbQueue.toString());

    try {
      await queueAccount.loadData();
    } catch (err) {
      console.log("Queue account not found:", err);
      process.exit(1);
    }

    const [randomnessAccount, randomnessIx] = await sb.Randomness.create(sbProgram, rngKp, sbQueue);
    console.log("Created randomness account..");
    console.log("Randomness account", randomnessAccount.pubkey.toBase58());
    console.log("rkp account", rngKp.publicKey.toBase58());

    const createRandomnessTx = await sb.asV0Tx({
      connection: provider.connection,
      ixs: [randomnessIx],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const blockhashContext = await provider.connection.getLatestBlockhashAndContext();

    const createRandomnessSignature = await provider.connection.sendTransaction(createRandomnessTx);
    await provider.connection.confirmTransaction({
      signature: createRandomnessSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log(
      "Transaction Signature for randomness account creation: ",
      createRandomnessSignature
    );

    const commitIx = await randomnessAccount.commitIx(sbQueue);

    const commitRandomnessIx = await program.methods
      .commitRandomness()
      .accounts({ randomnessAccount: randomnessAccount.pubkey })
      .instruction();
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
