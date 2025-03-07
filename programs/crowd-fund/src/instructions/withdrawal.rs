use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}
};

use crate::{error::ErrorCode, state::{Crowdfund, CampaignState}};


#[event]
pub struct WithdrawMade {
    pub withdrawer: Pubkey,
    pub amount: u64
}


#[derive(Accounts)]
pub struct DonationWithdrawal<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [withdrawer.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program
    )]
    pub withdraw_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = crowdfund_account,
        seeds = [b"campaign", mint.key().as_ref()],
        bump
    )]
    pub campaign_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

pub fn process_donation_withdrawal(ctx: Context<DonationWithdrawal>) -> Result<()> {
    let crowdfund_account = &ctx.accounts.crowdfund_account;
    let now = Clock::get()?.unix_timestamp;

    require!(now >= crowdfund_account.start_time, ErrorCode::NoStared);

    require!(crowdfund_account.state == CampaignState::Success as u8, ErrorCode::WithdrawalNotAllowed);
    require!(!crowdfund_account.is_withdrawals, ErrorCode::WithdrawalNotAllowed);

    require!(crowdfund_account.raised_amount >= crowdfund_account.target_amount, ErrorCode::NotReaching);

    msg!("withdraw_token_account key: {}", ctx.accounts.withdraw_token_account.key());
    
    let withdrawer_key = ctx.accounts.withdrawer.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        withdrawer_key.as_ref(),
        &[ctx.bumps.crowdfund_account]
    ]];

    let cpi_account = TransferChecked {
        from: ctx.accounts.campaign_token_account.to_account_info(),
        to: ctx.accounts.withdraw_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.crowdfund_account.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        cpi_account, 
        signer_seeds
    );

    transfer_checked(cpi_ctx, crowdfund_account.raised_amount, ctx.accounts.mint.decimals)?;

    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    crowdfund_account.is_withdrawals = true;

    msg!("Withdrawal of target amount {} succeeded.", crowdfund_account.target_amount);

    emit!(WithdrawMade {
        withdrawer: withdrawer_key,
        amount: crowdfund_account.raised_amount
    });
    
    Ok(())
}