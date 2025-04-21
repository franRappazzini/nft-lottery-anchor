use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    metadata::{Metadata, MetadataAccount},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{CustomErrors, TokenLottery, NAME};

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = ticket_mint,
        associated_token::authority = winner,
        associated_token::token_program = token_program,
        constraint = winner_account.amount > 0 @ CustomErrors::NoTicket
    )]
    pub winner_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        // mut,
        seeds = [b"token_lottery".as_ref()],
        bump,
        constraint = token_lottery.winner_chosed @ CustomErrors::WinnerNotChosen
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        // mut,
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        // mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            ticket_mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(), 
        constraint = token_metadata.collection.as_ref().unwrap().verified @ CustomErrors::TokenNotVerified,
        constraint = token_metadata.collection.as_ref().unwrap().key ==  collection_mint.key() @ CustomErrors::IncorrectTicket,
        constraint = token_metadata.name == format!("{}{}", NAME, token_lottery.winner) @ CustomErrors::IncorrectTicket
    )]
    pub token_metadata: Account<'info, MetadataAccount>, // no UncheckedAccount porque el token_metadata account ya fue creado

    #[account(
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>, // no UncheckedAccount porque el token_metadata account ya fue creado

    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn claim_tokens_lottery(ctx: Context<ClaimTokens>) -> Result<()> {
    let pot_amount = ctx.accounts.token_lottery.lottery_pot_amount;

    // **ctx.accounts.winner.to_account_info().lamports.borrow_mut() += pot_amount;
    // **ctx.accounts.token_lottery.to_account_info().lamports.borrow_mut() -= pot_amount;

    system_program::transfer(CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.token_lottery.to_account_info(),
            to: ctx.accounts.winner.to_account_info()
        }
    ), pot_amount)?;

    ctx.accounts.token_lottery.lottery_pot_amount = 0;
    
    Ok(())
}
