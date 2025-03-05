use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    #[msg("Crowdfunding has not started")]
    NoStared,

    #[msg("The campaign has expired.")]
    CampaignExpired,

    #[msg("Refund not allowed in the current state.")]
    RefundNotAllowed,

    #[msg("Withdrawal not allowed in the current state.")]
    WithdrawalNotAllowed,

    #[msg("Target amount not reached")]
    NotReaching,

    #[msg("Donation record has already been refunded.")]
    AlreadyRefunded
}

