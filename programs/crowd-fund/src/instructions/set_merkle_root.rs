use anchor_lang::prelude::*;

use crate::{state::{CampaignState, Crowdfund}, error::ErrorCode};



#[derive(Accounts)]
pub struct SetMerkleRoot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [authority.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    pub system_program: Program<'info, System>
}


pub fn proccess_merkle_root(ctx: Context<SetMerkleRoot>, merkle_root: [u8; 32]) -> Result<()> {
    let crowdfund_account = &mut ctx.accounts.crowdfund_account;
    // 判断众筹是否成功
    require!(crowdfund_account.state == CampaignState::Success as u8, ErrorCode::CampaignNotSuccessful);

    crowdfund_account.merkle_root = merkle_root;
    
    Ok(()) 
}
