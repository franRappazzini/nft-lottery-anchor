use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, metadata::{self, mpl_token_metadata::types::{Creator, DataV2}, Metadata}, token_interface::{self, Mint, TokenAccount, TokenInterface}};

use crate::{CustomErrors, TokenLottery, NAME, SYMBOL, URI};

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,    
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        init,
        payer = buyer,
        mint::decimals = 0,
        mint::authority = collection_mint_account,
        mint::freeze_authority = collection_mint_account,
        mint::token_program  =  token_program,      
        seeds = [token_lottery.total_tickets.to_le_bytes().as_ref()],
        bump
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = buyer,
        associated_token::mint = ticket_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Validate address by deriving pda (https://github.com/solana-developers/program-examples/blob/main/tokens/nft-minter/anchor/programs/nft-minter/src/lib.rs)
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            ticket_mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub ticket_metadata_account: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub ticket_master_edition: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda (https://github.com/solana-developers/program-examples/blob/main/tokens/nft-minter/anchor/programs/nft-minter/src/lib.rs)
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint_account.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint_account.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump
    )]
    pub collection_mint_account: InterfaceAccount<'info, TokenAccount>,
    
    pub associated_token_program : Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program : Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent : Sysvar<'info, Rent>,
}

pub fn buy_ticket_lottery(ctx: Context<BuyTicket>) -> Result<()> {
    // checkear token_lottery times
    let clock = Clock::get()?;

    let token_lottery = &ctx.accounts.token_lottery;

    if clock.slot < token_lottery.start_time || clock.slot > token_lottery.end_time {
        return Err(CustomErrors::OutOfTime.into())
    }
        
    msg!("Sending SOL to token_lottery account..");

    system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.token_lottery.to_account_info()
            }
        ), 
        ctx.accounts.token_lottery.ticket_price
    )?;    

    ctx.accounts.token_lottery.lottery_pot_amount += ctx.accounts.token_lottery.ticket_price;
    
    msg!("Sending NFT Ticket to {:?}...", ctx.accounts.buyer.key());

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"collection_mint".as_ref(),
        &[ctx.bumps.collection_mint_account]
    ]];
    
    // token_interface::transfer_checked(
    //     CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         token_interface::TransferChecked {
    //             from: ctx.accounts.collection_mint_account.to_account_info(),
    //             mint: ctx.accounts.ticket_mint.to_account_info(),
    //             to: ctx.accounts.destination.to_account_info(),
    //             authority: ctx.accounts.collection_mint_account.to_account_info()
    //         },
    //         signer_seeds),
    //     1,
    //     0
    // )?;
    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::MintTo {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.collection_mint_account.to_account_info()
            },
            signer_seeds
        ),
        1
    )?;

    msg!("Creating metadata to new Ticket...");

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(), 
            metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.ticket_metadata_account.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint_account.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                update_authority: ctx.accounts.collection_mint_account.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info() 
            }, 
            signer_seeds
        ),
        DataV2{
            collection: None,
            creators: Some(vec![Creator{
                address: ctx.accounts.collection_mint_account.key(),
                share: 100,
                verified: true
            }]),
            name: format!("{}{}", NAME, ctx.accounts.token_lottery.total_tickets),
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: 0,
            uses: None,
        },
        true,
        true,
        None  // only to collections
    )?;

    msg!("Creating master edition to new NFT Ticket");

    metadata::create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            metadata::CreateMasterEditionV3 {
                edition: ctx.accounts.ticket_master_edition.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint_account.to_account_info(),
                update_authority: ctx.accounts.collection_mint_account.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                metadata: ctx.accounts.ticket_metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            },
            signer_seeds
        ),
        Some(0)
    )?;

    // no set_and_verify_collection porque acabamos de agregar un nuevo nft a la collection
    metadata::set_and_verify_sized_collection_item(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            metadata::SetAndVerifySizedCollectionItem {
                metadata: ctx.accounts.ticket_metadata_account.to_account_info(),
                collection_authority: ctx.accounts.collection_mint_account.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                update_authority: ctx.accounts.collection_mint_account.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                collection_mint: ctx.accounts.collection_mint_account.to_account_info()
            },
            signer_seeds
        ),
        None
    )?;

 
    // sumar ticket en token_lottery
    ctx.accounts.token_lottery.total_tickets +=  1;

    Ok(())
}
