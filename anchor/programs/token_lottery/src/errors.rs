use anchor_lang::error_code;

#[error_code]
pub enum CustomErrors {
    #[msg("Out of time to buy the ticket.")]
    OutOfTime,
    #[msg("Unauthorized action.")]
    UnauthorizedAction,
    #[msg("Randomness already revealed.")]
    RandomnessAlreadyRevealed,
    #[msg("Randomness not resolved.")]
    RandomnessNotResolved,
    #[msg("Incorrect randomness account.")]
    IncorrectRandomness,
    #[msg("Winner already chosen.")]
    WinnerChosen,
    #[msg("Winner not chosen yet.")]
    WinnerNotChosen,
    #[msg("Token not verified.")]
    TokenNotVerified,
    #[msg("Incorrect ticket.")]
    IncorrectTicket,
    #[msg("No tickets yet.")]
    NoTicket,
}
