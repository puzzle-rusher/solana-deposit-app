use anchor_lang::prelude::*;

declare_id!("A9Lef4z6JBNzZoaQJT722eVuJR8GK5WqSLmgbNaJsacX");

#[program]
pub mod solana_deposit_app {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user_vault = &mut ctx.accounts.user_vault;
        let user = &ctx.accounts.user;

        let cpi_transfer = anchor_lang::system_program::Transfer {
            from: user.to_account_info(),
            to: user_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_transfer);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let user_vault_info = ctx.accounts.user_vault.to_account_info();
        let user_info = ctx.accounts.user.to_account_info();

        require!(
            **user_vault_info.lamports.borrow() >= amount,
            ErrorCode::InsufficientFunds
        );

        let user_key = ctx.accounts.user.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"user_vault", user_key.as_ref(), &[ctx.bumps.user_vault]]];

        let transfer_accounts = anchor_lang::system_program::Transfer {
            from: user_vault_info,
            to: user_info,
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_accounts,
        ).with_signer(signer_seeds);
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn get_user_balance(ctx: Context<GetBalance>) -> Result<u64> {
        let balance = ctx.accounts.user_vault.to_account_info().lamports();
        Ok(balance)
    }
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump
    )]
    pub user_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetBalance<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user_vault", user.key().as_ref()],
        bump
    )]
    pub user_vault: SystemAccount<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient balance")]
    InsufficientFunds,
    #[msg("Overflow")]
    Overflow,
}
