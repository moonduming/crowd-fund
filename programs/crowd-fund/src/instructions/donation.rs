use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{error::ErrorCode, state::{CampaignState, Crowdfund, DonationRecord}};


#[event]
pub struct DonationMade {
    pub donor: Pubkey,
    pub amount: u64
}


#[derive(Accounts)]
pub struct InitDonationRecord<'info> {
    #[account(mut)]
    pub donor: Signer<'info>,

    pub maker: SystemAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [maker.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = crowdfund_account,
        associated_token::token_program = token_program
    )]
    pub campaign_token_account: InterfaceAccount<'info, TokenAccount>,


    #[account(
        init,
        payer = donor,
        space = 8 + DonationRecord::INIT_SPACE,
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

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>
}


pub fn proccess_donation_record(ctx: Context<InitDonationRecord>, amount: u64) -> Result<()> {
    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    let donation_record_account = &mut ctx.accounts.donation_record_account;

    // Check that donation amount is greater than zero
    require!(amount > 0, ErrorCode::InvalidDonationAmount);

    let now = Clock::get()?.unix_timestamp;

    if now < crowdfund_account.start_time {
        return Err(ErrorCode::NoStared.into());
    } else if now > crowdfund_account.end_time {
        return Err(ErrorCode::CampaignExpired.into());
    }

    // Check that the campaign is active using the helper method
    match crowdfund_account.get_state() {
        Some(CampaignState::Active) => {},
        _ => return Err(ErrorCode::CampaignExpired.into()),
    };

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.donation_token_account.to_account_info(),
        to: ctx.accounts.campaign_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.donor.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), 
        cpi_accounts
    );

    transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

    crowdfund_account.raised_amount = crowdfund_account.raised_amount
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    if crowdfund_account.raised_amount >= crowdfund_account.target_amount {
        msg!("raised_amount: {}", crowdfund_account.raised_amount);
        crowdfund_account.state = CampaignState::Success as u8;
    }

    donation_record_account.amount = amount;
    donation_record_account.campaign = crowdfund_account.escrow_account;
    donation_record_account.donor = ctx.accounts.donor.key();
    donation_record_account.is_refunded = false;

    msg!("Donation of {} succeeded. Total raised: {}", amount, crowdfund_account.raised_amount);


    emit!(DonationMade {
        donor: ctx.accounts.donor.key(),
        amount
    });

    Ok(())
}
