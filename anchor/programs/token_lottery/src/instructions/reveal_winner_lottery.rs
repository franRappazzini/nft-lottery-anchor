use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessAccountData;

use crate::{CustomErrors, TokenLottery};

#[derive(Accounts)]
pub struct RevealWinner<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
        has_one = authority @ CustomErrors::UnauthorizedAction,
        has_one = randomness_account @ CustomErrors::IncorrectRandomness,
        constraint = !token_lottery.winner_chosed @ CustomErrors::WinnerChosen
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK This account is checked by Switchboard
    pub randomness_account: UncheckedAccount<'info>,
}

pub fn reveal_winner_lottery(ctx: Context<RevealWinner>) -> Result<()> {
    // Load clock to check data from the future
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    require!(clock.slot < token_lottery.end_time, CustomErrors::OutOfTime);

    // Parsing the oracle's scroll Call the switchboard on-demand parse function to get the randomness data
    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account.data.borrow()).unwrap();

    // Call the switchboard on-demand get_value function to get the revealed random value
    let revealed_random_value = randomness_data
        .get_value(&clock)
        .map_err(|_| CustomErrors::RandomnessNotResolved)?;

    msg!("revealed_random_value: {:?}", revealed_random_value);

    let winner = revealed_random_value[0] as u64 % token_lottery.total_tickets;

    msg!("Winner: {}", winner);

    token_lottery.winner = winner;
    token_lottery.winner_chosed = true;

    Ok(())
}
