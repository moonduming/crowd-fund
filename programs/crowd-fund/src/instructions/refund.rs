use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{error::ErrorCode, state::{Crowdfund, DonationRecord}};


#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub crowdfund_key: SystemAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [crowdfund_key.key().as_ref()],
        bump
    )]
    pub crowfund_account: Account<'info, Crowdfund>,

    #[account(
        mut,
        seeds = [signer.key().as_ref()],
        bump
    )]
    pub dontaion_record_account: Account<'info, DonationRecord>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = donation_token_account,
        associated_token::token_program = token_program
    )]
    pub donation_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = campaign_token_account,
        seeds = [b"campaign", mint.key().as_ref()],
        bump
    )]
    pub campaign_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}


pub fn proccess_refund(ctx: Context<Refund>) -> Result<()> {
    let crowfund_account = &ctx.accounts.crowfund_account;
    if crowfund_account.state < 2 {
        return Err(ErrorCode::RefundFailed.into());
    };

    let dontaion_record_account = &mut ctx.accounts.dontaion_record_account;
    if dontaion_record_account.is_refunded {
        return Err(ErrorCode::Refunded.into());
    };

    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"campaign", 
        mint_key.as_ref(),
        &[ctx.bumps.campaign_token_account]
    ]];
    
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.campaign_token_account.to_account_info(),
        to: ctx.accounts.donation_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.campaign_token_account.to_account_info()
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        cpi_accounts, 
        signer_seeds
    );

    transfer_checked(cpi_ctx, dontaion_record_account.amount, ctx.accounts.mint.decimals)?;

    dontaion_record_account.is_refunded = true;

    Ok(())
}
