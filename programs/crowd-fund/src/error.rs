use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    #[msg("Crowdfunding has not started")]
    NoStared,

    #[msg("Donation amount is too large")]
    AmountTooLarge,

    #[msg("The donation amount did not reach the set value")]
    NotReaching,

    #[msg("Duplicate extraction is prohibited.")]
    RepeatedExtraction,

    #[msg("Donation in progress")]
    RefundFailed,

    #[msg("Refund completed")]
    Refunded,
}

