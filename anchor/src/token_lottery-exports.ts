// Here we export some useful types and functions for interacting with the Anchor program.
import { AnchorProvider, Program } from "@coral-xyz/anchor";

import { PublicKey } from "@solana/web3.js";
import type { TokenLottery } from "../target/types/token_lottery";
import TokenLotteryIDL from "../target/idl/token_lottery.json";

// Re-export the generated IDL and type
export { TokenLottery, TokenLotteryIDL };

// The programId is imported from the program IDL.
export const TOKEN_LOTTERY_PROGRAM_ID = new PublicKey(TokenLotteryIDL.address);

// This is a helper function to get the TokenLottery Anchor program.
export function getTokenLotteryProgram(provider: AnchorProvider) {
  return new Program(TokenLotteryIDL as TokenLottery, provider);
}
