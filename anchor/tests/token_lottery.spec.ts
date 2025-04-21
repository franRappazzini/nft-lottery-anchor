import * as anchor from "@coral-xyz/anchor";
import * as sb from "@switchboard-xyz/on-demand";

import { before, describe, it } from "node:test";

import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { TokenLottery } from "../target/types/token_lottery";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("NFT Token Lottery Program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;
  let sbProgram: Program<anchor.Idl>;
  const rngKp = anchor.web3.Keypair.generate();

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );

  before(async () => {
    const sbIdl = (await anchor.Program.fetchIdl(sb.ON_DEMAND_MAINNET_PID, {
      connection: new anchor.web3.Connection("https://api.mainnet-beta.solana.com"),
    })) as anchor.Idl;

    console.log("Switchboard IDL:", sbIdl);

    sbProgram = new anchor.Program(sbIdl, provider);
  });

  async function buyTicket() {
    const buyTicketIx = await program.methods
      .buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    // aumentamos el limite de compute units porque pasa los 200000 por default
    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300000,
    });

    const priorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1,
    });

    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    })
      .add(buyTicketIx)
      .add(computeIx)
      .add(priorityIx);

    const signature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [wallet.payer], {
      skipPreflight: true,
    });
    console.log("Buy ticket signature:", signature);
  }

  it("Should initialize config and lottery", async () => {
    const slot = await connection.getSlot();

    const collectionMint = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("collection_mint")],
      program.programId
    )[0];

    const metadata = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), collectionMint.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID
    )[0];

    const masterEdition = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )[0];

    const initConfigIx = await program.methods
      .initializeConfig(new anchor.BN(0), new anchor.BN(slot + 10), new anchor.BN(10000))
      .instruction();

    // PARA EL metadata_program_account ES NECESARIO DESCARGAR EL PROGRAMA DE MAINNET Y EJECUTARLO LOCAL
    // https://solana.com/es/developers/cookbook/development/using-mainnet-accounts-programs
    /*
      # solana program dump -u <source cluster> <address of account to fetch> <destination file name/path>
      solana program dump -u m PROGRAM_ID NAME.so

      # solana-test-validator --bpf-program <address to load the program to> <path to program file> --reset
      solana-test-validator --bpf-program PROGRAM_ID NAME.so --reset
    */
    const initLotteryIx = await program.methods
      .initializeLottery()
      .accounts({
        // masterEdition: masterEdition,
        // metadata: metadata,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    })
      .add(initConfigIx)
      .add(initLotteryIx);

    const signature = await anchor.web3.sendAndConfirmTransaction(connection, tx, [wallet.payer], {
      skipPreflight: true,
    });
    console.log("Initialize signature:", signature);
  });

  it("Should buy tickets", async () => {
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
  });

  it("Should commit and reveal a winner", async () => {
    const sbQueue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"); // mainnet

    const queueAccount = new sb.Queue(sbProgram, sbQueue);
    console.log("Queue account:", sbQueue.toString());

    try {
      await queueAccount.loadData();
    } catch (err) {
      console.log("Queue account not found:", err);
      process.exit(1);
    }

    const [randomness, ix] = await sb.Randomness.create(sbProgram, rngKp, sbQueue);

    console.log("Created randomness account..");
    console.log("Randomness account", randomness.pubkey.toBase58());
    console.log("rkp account", rngKp.publicKey.toBase58());

    const createRandomnessTx = await sb.asV0Tx({
      connection: connection,
      ixs: [ix],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const blockhashContext = await connection.getLatestBlockhashAndContext();
    const createRandomnessSignature = await connection.sendTransaction(createRandomnessTx);

    await connection.confirmTransaction({
      signature: createRandomnessSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log(
      "Transaction Signature for randomness account creation:",
      createRandomnessSignature
    );

    console.log("randomness account:", randomness.pubkey.toBase58());
    console.log("Randomness account data:", await randomness.loadData());
    console.log("sbQueue:", sbQueue.toBase58());

    const commitIx = await randomness.commitIx(sbQueue);
    console.log("Commit randomness ix:", commitIx);

    const commitRandomnessIx = await program.methods
      .commitRandomness()
      .accounts({ randomnessAccount: randomness.pubkey })
      .instruction();

    const commitTx = await sb.asV0Tx({
      connection: sbProgram.provider.connection,
      ixs: [commitRandomnessIx, commitIx],
      payer: wallet.publicKey,
      signers: [wallet.payer],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    console.log("Commit randomness tx:", commitTx);

    const commitSignature = await connection.sendTransaction(commitTx);
    await connection.confirmTransaction({
      signature: commitSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log("commitRandomness tx signature:", commitSignature);

    const sbRevealIx = await randomness.revealIx();
    const revealIx = await program.methods.revealWinner().instruction();
    const revealTx = await sb.asV0Tx({
      connection: sbProgram.provider.connection,
      ixs: [sbRevealIx, revealIx],
      payer: wallet.publicKey,
      signers: [wallet.payer],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const revealSignature = await connection.sendTransaction(revealTx);
    await connection.confirmTransaction({
      signature: commitSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log("revealWinner tx signature", revealSignature);
  });

  it("Should claim tokens", async () => {
    const tokenLotteryAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token_lottery")],
      program.programId
    )[0];

    const lotteryConfig = await program.account.tokenLottery.fetch(tokenLotteryAddress);
    console.log("Lottery config:", lotteryConfig);

    const tokenAccounts = await connection.getParsedTokenAccountsByOwner(wallet.publicKey, {
      programId: TOKEN_PROGRAM_ID,
    });
    tokenAccounts.value.forEach(async (account) => {
      console.log("Token account mint:", account.account.data.parsed.info.mint);
      console.log("Token account address:", account.pubkey.toBase58());
    });

    const winnerAccount = anchor.web3.PublicKey.findProgramAddressSync(
      [new anchor.BN(lotteryConfig.winner).toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    console.log("Winner account:", winnerAccount.toBase58());

    const winnerTokenAddress = getAssociatedTokenAddressSync(winnerAccount, wallet.publicKey);
    console.log("Winner token address:", winnerTokenAddress.toBase58());

    const claimIx = await program.methods
      .claimTokens()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    const claimTx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    }).add(claimIx);

    const claimTokensSignature = await anchor.web3.sendAndConfirmTransaction(
      connection,
      claimTx,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log(claimTokensSignature);
  });
});
