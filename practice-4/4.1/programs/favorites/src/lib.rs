use anchor_lang::prelude::*;

declare_id!("APGdTJxX4sxKYd6G6sKSW3WiGUrTYhhS8cz6GLYDpYQy");

// Anchor programs always use
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

// What we will put inside the Favorites PDA
#[account]
#[derive(InitSpace)]
pub struct Favorites {
    pub number: u64,
    
    #[max_len(50)]
    pub color: String,

    pub user: Pubkey,
}


// When people call the set_favorites instruction, they will need to provide the accounts that will
// be modified. This keeps Solana fast!
#[derive(Accounts)]
pub struct SetFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = ANCHOR_DISCRIMINATOR_SIZE + Favorites::INIT_SPACE,
        seeds = [b"favorites", user.key().as_ref()],
        bump,
    )]
    pub favorites: Account<'info, Favorites>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        has_one = user,
    )]
    pub favorites: Account<'info, Favorites>,
}

// Our Solana program!
#[program]
pub mod favorites {
    use super::*;

    // Our instruction handler! It sets the user's favorite number and color
    pub fn set_favorites(context: Context<SetFavorites>, number: u64, color: String) -> Result<()> {
        let user_public_key = context.accounts.user.key();
        msg!("Greetings from {}", context.program_id);
        msg!(
            "User {}'s favorite number is {} and favorite color is: {}",
            user_public_key,
            number,
            color
        );

        context
            .accounts
            .favorites
            .set_inner(Favorites { number, color, user: context.accounts.user.key() });
            
        Ok(())
    }
        
    pub fn update_favorites(context: Context<UpdateFavorites>, number: Option<u64>, color: Option<String>) -> Result<()> {
        match number {
            Some(n) => context.accounts.favorites.number = n,
            None => {}
        }

        match color {
            Some(c) => context.accounts.favorites.color = c,
            None => {}
        }

        Ok(())
    }

    
}
