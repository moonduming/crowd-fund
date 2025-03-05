use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct Crowdfund {
    pub owner: Pubkey,
    pub escrow_account: Pubkey,
    #[max_len(50)]
    pub name: String,
    pub start_time: i64,
    pub end_time: i64,
    pub target_amount: u64,
    pub raised_amount: u64,
    // 0: in progress 1: success 2: fail
    pub state: u8
}


#[account]
#[derive(InitSpace)]
pub struct DonationRecord {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub is_refunded: bool
}
