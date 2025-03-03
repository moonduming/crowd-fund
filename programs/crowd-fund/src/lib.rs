use anchor_lang::prelude::*;

declare_id!("H5NDgHeJkob5QMnH5V4BkPBeTjrjwKAvpeTUDvZWWFXP");

#[program]
pub mod crowd_fund {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
