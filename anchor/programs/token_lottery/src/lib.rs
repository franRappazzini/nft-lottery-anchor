use anchor_lang::prelude::*;

pub mod constants;
pub mod instructions;
pub mod states;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("53gj5e26yvjTr9ho36adEhkxVXXgDDYxjdWRhtUJBE7x");

#[program]
pub mod token_lottery {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        start_time: i64,
        end_time: i64,
        ticket_price: u64,
    ) -> Result<()> {
        initialize_config_account::initialize_config(ctx, start_time, end_time, ticket_price)
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        initialize_lottery_account::initialize_lottery(ctx)
    }
}
