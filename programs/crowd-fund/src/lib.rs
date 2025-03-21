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
        msg!("donate {}", amount);
        proccess_donation_record(ctx, amount)
    }

    pub fn withdraw(ctx: Context<DonationWithdrawal>) -> Result<()> {
        msg!("withdrawl token");
        process_donation_withdrawal(ctx)
    }
    
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        msg!("refund");
        proccess_refund(ctx)
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        msg!("finalize");
        proccess_finalize(ctx)
    }

    pub fn set_merkle_root(ctx: Context<SetMerkleRoot>, merkle_root: [u8; 32]) -> Result<()> {
        msg!("save merkle root");
        proccess_merkle_root(ctx, merkle_root)
    }
    
    pub fn reward_claim(ctx: Context<RewardClaim>, proof: Vec<[u8; 32]>) -> Result<()> {
        msg!("reward claim");
        proccess_reward_claim(ctx, proof)
    }
}

