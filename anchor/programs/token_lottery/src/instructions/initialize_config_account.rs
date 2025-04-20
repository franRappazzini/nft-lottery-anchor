use anchor_lang::prelude::*;

use crate::{TokenLottery, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = TokenLottery::INIT_SPACE + ANCHOR_DISCRIMINATOR,
        seeds = [b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_config_account(
    ctx: Context<InitializeConfig>,
    start_time: u64,
    end_time: u64,
    ticket_price: u64,
) -> Result<()> {
    ctx.accounts.token_lottery.set_inner(TokenLottery {
        bump: ctx.bumps.token_lottery,
        winner: 0,
        winner_chosed: false,
        start_time,
        end_time,
        lottery_pot_amount: 0,
        total_tickets: 0,
        ticket_price,
        authority: ctx.accounts.signer.key(),
        randomness_account: Pubkey::default(),
    });
    Ok(())
}
