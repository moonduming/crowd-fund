use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{error::ErrorCode, state::Crowdfund};



#[derive(Accounts)]
pub struct DonationWithdrawal<'info> {
    #[account(mut)]
    pub singer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [singer.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    #[account(
        init_if_needed,
        payer = singer,
        associated_token::mint = mint,
        associated_token::authority = singer,
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
    let crowdfund_account_to_info = ctx.accounts.crowdfund_account.to_account_info();
    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    let now = Clock::get()?.unix_timestamp;

    if now < crowdfund_account.start_time {
        return Err(ErrorCode::NoStared.into());
    };

    if crowdfund_account.state != 1 || crowdfund_account.is_withdrawals  {
        return Err(ErrorCode::WithdrawalNotAllowed.into());
    };

    if crowdfund_account.raised_amount < crowdfund_account.target_amount {
        return Err(ErrorCode::NotReaching.into());
    };
    
    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"campaign", 
        mint_key.as_ref(),
        &[ctx.bumps.campaign_token_account]
    ]];

    let cpi_account = TransferChecked {
        from: ctx.accounts.campaign_token_account.to_account_info(),
        to: ctx.accounts.withdraw_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: crowdfund_account_to_info
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        cpi_account, 
        signer_seeds
    );

    transfer_checked(cpi_ctx, crowdfund_account.target_amount, ctx.accounts.mint.decimals)?;

    crowdfund_account.is_withdrawals = true;

    Ok(())
}
