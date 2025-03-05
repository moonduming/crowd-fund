use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::state::Crowdfund;


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
    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    crowdfund_account.owner = ctx.accounts.payer.key();
    crowdfund_account.name = name;
    crowdfund_account.escrow_account = ctx.accounts.campaign_token_account.key();
    crowdfund_account.target_amount = target_amount;
    crowdfund_account.raised_amount = 0;
    crowdfund_account.start_time = start_time;
    crowdfund_account.end_time = end_time;
    crowdfund_account.is_withdrawals = false;

    Ok(())
}
