use crate::instructions::CurveLaunchpadError;
use crate::state::Global;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(
        mut,
        seeds = [Global::SEED_PREFIX],
        bump,
    )]
    global: Box<Account<'info, Global>>,

    user: Signer<'info>,

    new_authority: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn set_authority(ctx: Context<SetAuthority>) -> Result<()> {
    let global = &mut ctx.accounts.global;

    //confirm program is initialized
    require!(global.initialized, CurveLaunchpadError::NotInitialized);

    //confirm user is the authority
    require!(
        global.authority == *ctx.accounts.user.key,
        CurveLaunchpadError::InvalidAuthority
    );

    global.authority = *ctx.accounts.new_authority.key;

    Ok(())
}
