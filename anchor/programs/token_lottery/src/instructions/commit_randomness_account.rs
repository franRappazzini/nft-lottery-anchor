use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessAccountData;

use crate::{CustomErrors, TokenLottery};

#[derive(Accounts)]
pub struct CommitRandomness<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: This account is checked by Switchboard
    pub randomness_account: UncheckedAccount<'info>,
}

pub fn commit_randomness_account(ctx: Context<CommitRandomness>) -> Result<()> {
    // https://docs.switchboard.xyz/product-documentation/randomness/tutorials/solana-svm
    // Load clock to check data from the future
    let clock = Clock::get()?;
    let token_lottery = &ctx.accounts.token_lottery;

    if token_lottery.authority != ctx.accounts.signer.key() {
        return Err(CustomErrors::UnauthorizedAction.into());
    }

    // Update token_lottery's randomness_account
    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account.data.borrow()).unwrap();

    if randomness_data.seed_slot != clock.slot - 1 {
        msg!("seed_slot: {}", randomness_data.seed_slot);
        msg!("slot: {}", clock.slot);
        return Err(CustomErrors::RandomnessAlreadyRevealed.into());
    }

    Ok(())
}
