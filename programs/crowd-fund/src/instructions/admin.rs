use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{error::ErrorCode, state::{CampaignState, Crowdfund}};


#[event]
pub struct CrowdfundInitialized {
    pub campaign: Pubkey,
    pub owner: Pubkey,
    pub target_amount: u64,
}

#[derive(Accounts)]
pub struct InitCrowdfund<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = crowdfund_account,
        seeds = [b"campaign", mint.key().as_ref()],
        bump
    )]
    pub campaign_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + Crowdfund::INIT_SPACE,
        seeds = [payer.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>
}

pub fn proccess_crowdfund(
    ctx: Context<InitCrowdfund>, 
    name: String, 
    target_amount: u64,
    start_time: i64,
    end_time: i64
) -> Result<()> {
    // Validate inputs
    require!(target_amount > 0, ErrorCode::InvalidTargetAmount);
    require!(start_time < end_time, ErrorCode::InvalidTimeRange);

    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    crowdfund_account.owner = ctx.accounts.payer.key();
    crowdfund_account.name = name;
    crowdfund_account.escrow_account = ctx.accounts.campaign_token_account.key();
    crowdfund_account.target_amount = target_amount;
    crowdfund_account.raised_amount = 0;
    crowdfund_account.start_time = start_time;
    crowdfund_account.end_time = end_time;
    crowdfund_account.state = CampaignState::Active as u8;
    crowdfund_account.is_withdrawals = false;

    msg!("Crowdfund initialized for owner: {} with target amount: {}", ctx.accounts.payer.key(), target_amount);

    // Emit an event to notify that the crowdfund has been initialized
    emit!(CrowdfundInitialized {
        campaign: crowdfund_account.key(),
        owner: ctx.accounts.payer.key(),
        target_amount,
    });

    Ok(())
}
