use anchor_lang::prelude::*;

mod state;
mod instructions;
mod error;

use instructions::*;


declare_id!("H5NDgHeJkob5QMnH5V4BkPBeTjrjwKAvpeTUDvZWWFXP");

#[program]
pub mod crowd_fund {

    use super::*;

    pub fn campaign(
        ctx: Context<InitCrowdfund>,
        name: String,
        target_amount: u64,
        start_time: i64,
        end_time: i64
    ) -> Result<()> {
        msg!("Intialize Campaign");
        proccess_crowdfund(ctx, name, target_amount, start_time, end_time)
    }

    pub fn donation(ctx: Context<InitDonationRecord>, amount: u64) -> Result<()> {
        proccess_donation_record(ctx, amount)
    }

    pub fn withdraw(ctx: Context<DonationWithdrawal>) -> Result<()> {
        process_donation_withdrawal(ctx)
    }
    
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        proccess_refund(ctx)
    }
}

