use anchor_lang::prelude::*;

use crate::state::{Crowdfund, CampaignState};

#[derive(Accounts)]
pub struct Finalize<'info> {
    pub make: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [make.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    pub system_program: Program<'info, System>
}

pub fn proccess_finalize(ctx: Context<Finalize>) -> Result<()> {
    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    let now = Clock::get()?.unix_timestamp;

    if now > crowdfund_account.end_time {
        if crowdfund_account.raised_amount >= crowdfund_account.target_amount {
            crowdfund_account.state = CampaignState::Success as u8;
        } else {
            crowdfund_account.state = CampaignState::Fail as u8;
        }
    } else {
        if crowdfund_account.raised_amount >= crowdfund_account.target_amount {
            crowdfund_account.state = CampaignState::Success as u8;
        }
    }

    Ok(())
}