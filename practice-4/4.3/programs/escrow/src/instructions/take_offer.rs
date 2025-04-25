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
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,

    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker, // аккаунт в кінці закривається і гроші повертаються мейкеру
        has_one = maker, // поле maker в аккаунті offer повинно бути таким же як поле maker в інструкції
        has_one = token_mint_a,
        has_one = token_mint_b,
        // закоментовано тому що аккаунт offer не може зсилатись на самого себе
        // seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        // bump = offer.bump
    )]
    offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


pub fn send_wanted_tokens_to_maker(ctx: &Context<TakeOffer>) -> Result<()> {
    let transfer_accounts = TransferChecked {
        from: ctx.accounts.taker_token_account_b.to_account_info(),
        mint: ctx.accounts.token_mint_b.to_account_info(),
        to: ctx.accounts.maker_token_account_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );
    transfer_checked(
        cpi_context,
        ctx.accounts.offer.token_b_wanted_amount,
        ctx.accounts.token_mint_b.decimals,
    )
}


pub fn withdraw_and_close_vault(ctx: Context<TakeOffer>) -> Result<()> {
    // в якості підписанта транзакції виступає PDA offer
    // він не має приватного ключа, АЛЕ
    // програма може підписувати транзакції за всі своє PDA аккаунти
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"offer",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes()[..],
        &[ctx.accounts.offer.bump],
    ]];

    // переказ токену а з волту до тейкера
    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
        to: ctx.accounts.taker_token_account_a.to_account_info(),
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
        destination: ctx.accounts.taker.to_account_info(),
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


