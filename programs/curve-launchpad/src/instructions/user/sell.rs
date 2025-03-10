use crate::{
    amm, calculate_fee,
    state::{BondingCurve, Global},
    CurveLaunchpadError, TradeEvent,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[event_cpi]
#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(
        seeds = [Global::SEED_PREFIX],
        bump,
    )]
    global: Box<Account<'info, Global>>,

    /// CHECK: Using global state to validate fee_recipient account
    #[account(mut)]
    fee_recipient: AccountInfo<'info>,

    mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX, mint.to_account_info().key.as_ref()],
        bump,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    bonding_curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    user_token_account: Box<Account<'info, TokenAccount>>,

    system_program: Program<'info, System>,

    token_program: Program<'info, Token>,
}

// https://solana.com/developers/cookbook/programs/transfer-sol
fn transfer_lamports(
    from_account: &AccountInfo,
    to_account: &AccountInfo,
    amount_of_lamports: u64,
) -> Result<()> {
    // Does the from account have enough lamports to transfer?
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(CurveLaunchpadError::InsufficientSOL.into());
    }
    // Debit from_account and credit to_account
    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;
    Ok(())
}

pub fn sell(ctx: Context<Sell>, token_amount: u64, min_sol_output: u64) -> Result<()> {
    //confirm program is initialized
    require!(
        ctx.accounts.global.initialized,
        CurveLaunchpadError::NotInitialized
    );

    //confirm program is not paused
    require!(
        !ctx.accounts.global.paused,
        CurveLaunchpadError::ProgramIsPaused
    );

    //check if bonding curve is complete
    require!(
        !ctx.accounts.bonding_curve.complete,
        CurveLaunchpadError::BondingCurveComplete,
    );

    //confirm user has enough tokens
    require!(
        ctx.accounts.user_token_account.amount >= token_amount,
        CurveLaunchpadError::InsufficientTokens,
    );

    //invalid fee recipient
    require!(
        ctx.accounts.fee_recipient.key == &ctx.accounts.global.fee_recipient,
        CurveLaunchpadError::InvalidFeeRecipient,
    );

    //confirm bonding curve has enough tokens
    require!(
        ctx.accounts.bonding_curve_token_account.amount >= token_amount,
        CurveLaunchpadError::InsufficientTokens,
    );

    require!(token_amount > 0, CurveLaunchpadError::MinSell,);

    let mut amm = amm::AMM::new(
        ctx.accounts.bonding_curve.virtual_sol_reserves as u128,
        ctx.accounts.bonding_curve.virtual_token_reserves as u128,
        ctx.accounts.bonding_curve.real_sol_reserves as u128,
        ctx.accounts.bonding_curve.real_token_reserves as u128,
        ctx.accounts.global.initial_virtual_token_reserves as u128,
    );

    let sell_result = amm.apply_sell(token_amount as u128).unwrap();
    let fee = calculate_fee(sell_result.sol_amount, ctx.accounts.global.fee_basis_points);

    //the fee is subtracted from the sol amount to confirm the user minimum sol output is met
    let sell_amount_minus_fee = sell_result.sol_amount - fee;

    //confirm min sol output is greater than sol output
    require!(
        sell_amount_minus_fee >= min_sol_output,
        CurveLaunchpadError::MinSOLOutputNotReached,
    );

    //transfer SPL
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info().clone(),
        to: ctx
            .accounts
            .bonding_curve_token_account
            .to_account_info()
            .clone(),
        authority: ctx.accounts.user.to_account_info().clone(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &[],
        ),
        sell_result.token_amount,
    )?;

    //transfer SOL back to user
    let from_account = &ctx.accounts.bonding_curve.to_account_info();
    let user = &ctx.accounts.user;
    let fee_recipient = &ctx.accounts.fee_recipient;

    transfer_lamports(from_account, user, sell_result.sol_amount)?;
    transfer_lamports(from_account, fee_recipient, fee)?;

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    bonding_curve.real_token_reserves = amm.real_token_reserves as u64;
    bonding_curve.real_sol_reserves = amm.real_sol_reserves as u64;
    bonding_curve.virtual_token_reserves = amm.virtual_token_reserves as u64;
    bonding_curve.virtual_sol_reserves = amm.virtual_sol_reserves as u64;

    emit!(TradeEvent {
        mint: *ctx.accounts.mint.to_account_info().key,
        sol_amount: sell_result.sol_amount,
        token_amount: sell_result.token_amount,
        is_buy: false,
        user: *ctx.accounts.user.key,
        timestamp: Clock::get()?.unix_timestamp,
        virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
        virtual_token_reserves: bonding_curve.virtual_token_reserves,
        real_sol_reserves: bonding_curve.real_sol_reserves,
        real_token_reserves: bonding_curve.real_token_reserves,
        finished_bounding_curve: bonding_curve.real_sol_reserves == 0,
    });

    Ok(())
}
