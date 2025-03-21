use anchor_lang::prelude::*;

// Define an enum for campaign state
#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum CampaignState {
    Active = 0,   // in progress
    Success = 1,  // success
    Fail = 2,     // fail
}

impl CampaignState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CampaignState::Active),
            1 => Some(CampaignState::Success),
            2 => Some(CampaignState::Fail),
            _ => None,
        }
    }
}

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
    pub state: u8,
    pub is_withdrawals: bool,
    pub merkle_root: [u8; 32]
}

impl Crowdfund {
    pub fn get_state(&self) -> Option<CampaignState> {
        CampaignState::from_u8(self.state)
    }
}



#[account]
#[derive(InitSpace)]
pub struct DonationRecord {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub is_refunded: bool,
}
