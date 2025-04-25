use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },

};

use crate::Offer;

#[derive(Accounts)]
pub struct CloseOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    // InterfaceAccount показує що цей аккаунт є аккаунтом вказаного типу
    // умова mint::token_program = token_program перевіряє що аккаунт token_mint_a має 
    // мінт створений програмою token_program
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a, 
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        close = maker, // аккаунт в кінці закривається і гроші повертаються мейкеру
        has_one = maker, // поле maker в аккаунті offer повинно бути таким же як поле maker в інструкції
        has_one = token_mint_a,
    )]
    pub offer: Account<'info, Offer>,

    // аккаунт куди будуть переслані токени, 
    // управляти цим аккаунтом може попредньо створений PDA
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
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


pub fn refund_and_close_vault(
    ctx: Context<CloseOffer>
) -> Result<()> {

    // в якості підписанта транзакції виступає PDA offer
    // він не має приватного ключа, АЛЕ
    // програма може підписувати транзакції за всі своє PDA аккаунти
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"offer",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes()[..],
        &[ctx.accounts.offer.bump],
    ]];

    // переказ токену а з волту до мейкера
    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
        to: ctx.accounts.maker_token_account_a.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
    };

    // new_with_signer - створює CPI контекст з підписантом
    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), // токен програма
        accounts,
        &signer_seeds,
    );
    transfer_checked(
        cpi_context,
        ctx.accounts.vault.amount,
        ctx.accounts.token_mint_a.decimals,
    )?;

    // закриття волту, щоб закрити волт також необхідно підпис PDA offer
    let accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
    };

    // оскілки волт це аккунт створений токен програмою, то й видаляти його 
    // буде токен програма
    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(cpi_context)
}




