use anchor_lang::error_code;

#[error_code]
pub enum CustomErrors {
    #[msg("Out of time to buy the ticket.")]
    OutOfTime,
}
