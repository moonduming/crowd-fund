use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{state::{CampaignState, Crowdfund, DonationRecord}, error::ErrorCode};


#[derive(Accounts)]
pub struct RewardClaim<'info> {
    #[account(mut)]
    pub donor: Signer<'info>,
    pub maker: SystemAccount<'info>,

    // pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [maker.key().as_ref()],
        bump
    )]
    pub crowdfund_account: Account<'info, Crowdfund>,

    #[account(
        mut,
        seeds = [donor.key().as_ref()],
        bump
    )]
    pub donation_record_account: Account<'info, DonationRecord>,

    // #[account(
    //     mut,
    //     associated_token::mint = mint,
    //     associated_token::authority = donor,
    //     associated_token::token_program = token_program
    // )]
    // pub donor_mint_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
    // pub token_program: Interface<'info, TokenInterface>
}


pub fn proccess_reward_claim(ctx: Context<RewardClaim>, proof: Vec<[u8; 32]>) -> Result<()> {
    let crowdfund_account = &ctx.accounts.crowdfund_account;
    // 判断众筹是否成功
    require!(crowdfund_account.state == CampaignState::Success as u8, ErrorCode::CampaignNotSuccessful);

    let donation_record = &mut ctx.accounts.donation_record_account;
    msg!("donor amount: {}", donation_record.amount);
    // 构造叶子节点：
    // 拼接 donor 的公钥（32字节）和 donation_record.amount（8字节小端）
    let leaf_input = format!("{}-{}", donation_record.donor, donation_record.amount);
    let leaf = hash::hash(leaf_input.as_bytes()).to_bytes();
    
    // 使用proof 验证 Merkle Root
    let mut computed_root = leaf;
    for sibling in proof.iter() {
        let mut combined = Vec::with_capacity(64);
        if computed_root <= *sibling {
            combined.extend_from_slice(&computed_root);
            combined.extend_from_slice(sibling);
        } else {
           combined.extend_from_slice(sibling);
           combined.extend_from_slice(&computed_root); 
        }
        computed_root = hash::hash(&combined).to_bytes();
    }

    msg!("computed_root: {:?}", computed_root);
    msg!("merkle_root: {:?}", crowdfund_account.merkle_root);

    // 与链上存储的 Merkle Root 进行比较
    require!(computed_root == crowdfund_account.merkle_root, ErrorCode::InvalidMerkleProof);

    Ok(())
}
