use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{Offer, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
#[instruction(id: u64)] // показує що це параметр інструкції, в іншому випадку id буде сприйматись як аккаунт
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    // InterfaceAccount показує що цей аккаунт є аккаунтом вказаного типу
    // умова mint::token_program = token_program перевіряє що аккаунт token_mint_a має 
    // мінт створений програмою token_program
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a, 
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    // аккаунт куди будуть переслані токени, 
    // управляти цим аккаунтом може попредньо створений PDA
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // АДРЕСИ ПРОГРАМ
    // потрібно для виклику функцій AssociatedToken програми (я думаю що це програма для взаємодії з токен аккаунтами)
    pub associated_token_program: Program<'info, AssociatedToken>,
    // потрібно для виклику функцій токен програми
    pub token_program: Interface<'info, TokenInterface>,
    // потрібно тому що створюются аккаунти
    pub system_program: Program<'info, System>,
}

pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    let transfer_accounts = TransferChecked {
        from: context.accounts.maker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        to: context.accounts.vault.to_account_info(),
        authority: context.accounts.maker.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        context.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    transfer_checked(
        cpi_context,
        token_a_offered_amount,
        context.accounts.token_mint_a.decimals,
    )
}


pub fn save_offer(context: Context<MakeOffer>, id: u64, token_b_wanted_amount: u64) -> Result<()> {
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_b_wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}


