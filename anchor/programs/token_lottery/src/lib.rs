use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use states::*;

declare_id!("53gj5e26yvjTr9ho36adEhkxVXXgDDYxjdWRhtUJBE7x");

#[program]
pub mod token_lottery {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        start_time: u64,
        end_time: u64,
        ticket_price: u64,
    ) -> Result<()> {
        initialize_config_account::initialize_config_account(
            ctx,
            start_time,
            end_time,
            ticket_price,
        )
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        initialize_lottery_account::initialize_lottery_account(ctx)
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        buy_ticket_lottery::buy_ticket_lottery(ctx)
    }

    pub fn commit_randomness(ctx: Context<CommitRandomness>) -> Result<()> {
        commit_randomness_account::commit_randomness_account(ctx)
    }

    pub fn reveal_winner(ctx: Context<RevealWinner>) -> Result<()> {
        reveal_winner_lottery::reveal_winner_lottery(ctx)
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        claim_tokens_lottery::claim_tokens_lottery(ctx)
    }
}
