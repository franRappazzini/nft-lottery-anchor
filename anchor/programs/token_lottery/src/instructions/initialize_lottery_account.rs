use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        self,
        mpl_token_metadata::types::{Creator, DataV2},
        CreateMasterEditionV3, Metadata, SignMetadata,
    },
    token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{NAME, SYMBOL, URI};

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = collection_mint_account,
        mint::freeze_authority = collection_mint_account,
        seeds = [b"mint_account".as_ref()],
        bump,
    )]
    pub mint_account: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: Validate address by deriving pda (https://github.com/solana-developers/program-examples/blob/main/tokens/nft-minter/anchor/programs/nft-minter/src/lib.rs)
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint_account.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint_account.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub edition_account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"collection_mint".as_ref()],
        bump,
        token::mint = mint_account,
        token::authority = collection_mint_account
    )]
    pub collection_mint_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>, // para hacer la metadata del token
}

pub fn initialize_lottery_account(ctx: Context<InitializeLottery>) -> Result<()> {
    let signer_seeds_collection_mint: &[&[&[u8]]] = &[&[
        b"collection_mint".as_ref(),
        &[ctx.bumps.collection_mint_account],
    ]];

    msg!("Creating mint account..");

    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_account.to_account_info(),
                to: ctx.accounts.collection_mint_account.to_account_info(),
                authority: ctx.accounts.collection_mint_account.to_account_info(),
            },
            signer_seeds_collection_mint,
        ),
        1,
    )?;

    msg!("Creating metadata account");

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                mint_authority: ctx.accounts.collection_mint_account.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                update_authority: ctx.accounts.collection_mint_account.to_account_info(),
            },
            signer_seeds_collection_mint,
        ),
        DataV2 {
            name: NAME.to_string(),
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            // collection: Some(Collection {
            //     verified: false,
            //     key: ctx.accounts.collection_mint_account.key(),
            // }),
            collection: None,
            seller_fee_basis_points: 0,
            creators: Some(vec![
                Creator {
                    address: ctx.accounts.collection_mint_account.key(),
                    share: 50,
                    verified: false,
                },
                Creator {
                    address: ctx.accounts.signer.key(),
                    share: 50,
                    verified: false,
                },
            ]),
            uses: None,
        },
        true,
        true,
        Some(metadata::mpl_token_metadata::types::CollectionDetails::V1 { size: 0 }),
    )?;

    msg!("Creating master edition account");

    metadata::create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.edition_account.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                mint_authority: ctx.accounts.collection_mint_account.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                update_authority: ctx.accounts.collection_mint_account.to_account_info(),
            },
            signer_seeds_collection_mint,
        ),
        None,
    )?;

    msg!("Verifying collection");

    metadata::sign_metadata(CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        SignMetadata {
            creator: ctx.accounts.collection_mint_account.to_account_info(),
            metadata: ctx.accounts.metadata_account.to_account_info(),
        },
        signer_seeds_collection_mint,
    ))?;

    Ok(())
}
