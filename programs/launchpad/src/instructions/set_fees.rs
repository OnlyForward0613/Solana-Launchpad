//! SetFees instruction handler

use {
    crate::{
        error::LaunchpadError,
        state::{
            launchpad::Launchpad,
            multisig::{AdminInstruction, Multisig},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"multisig"], bump = multisig.load()?.bump)]
    pub multisig: AccountLoader<'info, Multisig>,

    #[account(mut, seeds = [b"launchpad"], bump = launchpad.bump)]
    pub launchpad: Box<Account<'info, Launchpad>>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetFeesParams {
    pub new_auction_fee: Fee,
    pub auction_update_fee: Fee,
    pub invalid_bid_fee: Fee,
    pub trade_fee: Fee,
}

pub fn set_fees<'info>(
    ctx: Context<'_, '_, '_, 'info, SetFees<'info>>,
    params: &SetFeesParams,
) -> Result<u8> {
    // validate signatures
    let mut multisig = ctx.accounts.multisig.load_mut()?;

    let signatures_left = multisig.sign_multisig(
        &ctx.accounts.admin,
        &Multisig::get_account_infos(&ctx)[1..],
        &Multisig::get_instruction_data(AdminInstruction::SetFees, params)?,
    )?;
    if signatures_left > 0 {
        msg!(
            "Instruction has been signed but more signatures are required: {}",
            signatures_left
        );
        return Ok(signatures_left);
    }

    // update permissions
    let launchpad = ctx.accounts.launchpad.as_mut();
    launchpad.fees.new_auction = params.new_auction_fee;
    launchpad.fees.auction_update = params.auction_update_fee;
    launchpad.fees.invalid_bid = params.invalid_bid_fee;
    launchpad.fees.trade = params.trade_fee;

    if !launchpad.validate() {
        err!(LaunchpadError::InvalidLaunchpadConfig)
    } else {
        Ok(0)
    }
}