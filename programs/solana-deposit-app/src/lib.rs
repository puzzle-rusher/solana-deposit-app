use anchor_lang::prelude::*;

declare_id!("A9Lef4z6JBNzZoaQJT722eVuJR8GK5WqSLmgbNaJsacX");

#[program]
pub mod solana_deposit_app {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let user = &ctx.accounts.user;

        // CPI к SystemProgram: переводим лампорты
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: user.to_account_info(),
            to: user_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let user_account_info = ctx.accounts.user_account.to_account_info();
        let user_info = ctx.accounts.user.to_account_info();

        require!(
            **user_account_info.lamports.borrow() >= amount,
            ErrorCode::InsufficientFunds
        );

        **user_info.lamports.borrow_mut() += amount;
        **user_account_info.lamports.borrow_mut() -= amount;

        Ok(())
    }

    pub fn get_user_balance(ctx: Context<GetBalance>) -> Result<u64> {
        let balance = ctx.accounts.user_account.to_account_info().lamports();
        Ok(balance)
    }
}

// Контекст для `deposit`
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [b"user_account", user.key().as_ref()],
        bump,
        payer = user,
        space = 8,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

// Контекст для `withdraw`
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_account", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetBalance<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user_account", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
}

#[account]
pub struct UserAccount;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient balance")]
    InsufficientFunds,
    #[msg("Overflow")]
    Overflow,
}
