use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Crowdfunding has not started")]
    NoStared,

    #[msg("Target amount must be greater than zero.")]
    InvalidTargetAmount,

    #[msg("The campaign has expired.")]
    CampaignExpired,

    #[msg("Start time must be earlier than end time.")]
    InvalidTimeRange,

    #[msg("Refund not allowed in the current state.")]
    RefundNotAllowed,

    #[msg("Withdrawal not allowed in the current state.")]
    WithdrawalNotAllowed,

    #[msg("Target amount not reached")]
    NotReaching,

    #[msg("Donation record has already been refunded.")]
    AlreadyRefunded,

    #[msg("Donation amount must be greater than zero.")]
    InvalidDonationAmount,

    #[msg("Arithmetic overflow occurred.")]
    Overflow,
}
