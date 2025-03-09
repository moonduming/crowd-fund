use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}
};

use crate::{
    error::ErrorCode, 
    state::{Crowdfund, DonationRecord, CampaignState}
};


#[event]
pub struct RefundMade {
    pub refunder: Pubkey,
    pub payee: Pubkey,
    pub amount: u64
}


#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub donor: SystemAccount<'info>,

    pub weekly_planner: SystemAccount<'info>,
    
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [weekly_planner.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    #[account(
        mut,
        seeds = [donor.key().as_ref()],
        bump
    )]
    pub donation_record_account: Account<'info, DonationRecord>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = donor,
        associated_token::token_program = token_program
    )]
    pub donation_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = crowdfund_account,
        associated_token::token_program = token_program
    )]
    pub campaign_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

pub fn proccess_refund(ctx: Context<Refund>) -> Result<()> {
    let crowdfund_account = &ctx.accounts.crowdfund_account;
    
    require!(
        crowdfund_account.get_state() == Some(CampaignState::Fail),
        ErrorCode::RefundNotAllowed
    );

    let donation_record_account = &mut ctx.accounts.donation_record_account;
    require!(
        !donation_record_account.is_refunded,
        ErrorCode::AlreadyRefunded
    );

    let weekly_planner_key = ctx.accounts.weekly_planner.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        weekly_planner_key.as_ref(),
        &[ctx.bumps.crowdfund_account]
    ]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.campaign_token_account.to_account_info(),
        to: ctx.accounts.donation_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.crowdfund_account.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        cpi_accounts, 
        signer_seeds
    );

    transfer_checked(cpi_ctx, donation_record_account.amount, ctx.accounts.mint.decimals)?;

    donation_record_account.is_refunded = true;

    emit!(RefundMade {
        refunder: ctx.accounts.weekly_planner.key(),
        payee: ctx.accounts.donor.key(),
        amount: donation_record_account.amount
    });

    Ok(())
}