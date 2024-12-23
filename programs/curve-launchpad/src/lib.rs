use anchor_lang::prelude::*;

use instructions::*;

pub mod instructions;
pub mod state;
pub mod amm;

declare_id!("9fq1skqDR1ZsVwUyDwgjnF7usJKf6dXJSh6QRJ7zAeY3");

#[program]
pub mod curve_launchpad {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::initialize(ctx)
    }

    pub fn create(ctx: Context<Create>, name: String, symbol: String, uri: String) -> Result<()> {
        create::create(ctx, name, symbol, uri)
    }

    pub fn buy(ctx: Context<Buy>, token_amount: u64, max_sol_cost: u64) -> Result<()> {
        buy::buy(ctx, token_amount, max_sol_cost)
    }

    pub fn sell(ctx: Context<Sell>, token_amount: u64, min_sol_output: u64) -> Result<()> {
        sell::sell(ctx, token_amount, min_sol_output)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdraw::withdraw(ctx)
    }

    pub fn set_fee(ctx: Context<SetFee>, fee_basis_points: u64) -> Result<()> {
        set_fee::set_fee(ctx, fee_basis_points)
    }

    pub fn set_params(
        ctx: Context<SetParams>,
        fee_recipient: Pubkey,
        withdraw_authority: Pubkey,
        initial_virtual_token_reserves: u64,
        initial_virtual_sol_reserves: u64,
        initial_real_token_reserves: u64,
        inital_token_supply: u64,
        fee_basis_points: u64,
    ) -> Result<()> {
        set_params::set_params(
            ctx,
            fee_recipient,
            withdraw_authority,
            initial_virtual_token_reserves,
            initial_virtual_sol_reserves,
            initial_real_token_reserves,
            inital_token_supply,
            fee_basis_points,
        )
    }
}
