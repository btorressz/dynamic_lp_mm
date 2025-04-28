use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Burn, TokenAccount, Token, Transfer};

declare_id!("57y2Lg2TEBxTvnfo5Jokj21SsAVeZyc6ijANBYXhm9bc");

#[program]
pub mod dynamic_lp_mm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        fee_bps: u16,
        band_size_bp: u16,
    ) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        v.authority = *ctx.accounts.authority.key;
        v.admins = Vec::new();
        v.treasury = ctx.accounts.treasury.key();
        v.base_mint = ctx.accounts.base_mint.key();
        v.quote_mint = ctx.accounts.quote_mint.key();
        v.share_mint = ctx.accounts.share_mint.key();
        v.base_vault = ctx.accounts.base_vault.key();
        v.quote_vault = ctx.accounts.quote_vault.key();
        v.fee_bps = fee_bps;
        v.withdraw_fee_bps = 0;
        v.band_size_bp = band_size_bp;
        v.min_deposit_amount = 0;
        v.max_total_deposit = 0;
        v.last_band = 0;
        v.rebalance_cooldown_sec = 300;
        v.last_rebalance_ts = 0;
        v.rebalance_mode = 0;
        v.paused = false;
        v.emergency_withdraw_only = false;
        v.accrued_fee_base = 0;
        v.accrued_fee_quote = 0;
        v.deposit_whitelist = Vec::new();
        v.withdraw_whitelist = Vec::new();
        v.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.paused = paused;
        Ok(())
    }

    pub fn update_fee(ctx: Context<UpdateFee>, fee_bps: u16) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.fee_bps = fee_bps;
        Ok(())
    }

    pub fn set_withdraw_fee(ctx: Context<UpdateFee>, fee_bps: u16) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.withdraw_fee_bps = fee_bps;
        Ok(())
    }

    pub fn set_rebalance_cooldown(ctx: Context<UpdateFee>, cooldown_sec: u64) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.rebalance_cooldown_sec = cooldown_sec;
        Ok(())
    }

    pub fn set_rebalance_mode(ctx: Context<UpdateFee>, mode: u8) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.rebalance_mode = mode;
        Ok(())
    }

    pub fn add_admin(ctx: Context<ModifyAdmins>, admin: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(ctx.accounts.authority.key == &v.authority, VaultError::Unauthorized);
        if !v.admins.contains(&admin) {
            v.admins.push(admin);
        }
        Ok(())
    }

    pub fn remove_admin(ctx: Context<ModifyAdmins>, admin: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(ctx.accounts.authority.key == &v.authority, VaultError::Unauthorized);
        v.admins.retain(|a| a != &admin);
        Ok(())
    }

    pub fn add_deposit_whitelist(ctx: Context<ModifyWhitelist>, user: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        if !v.deposit_whitelist.contains(&user) {
            v.deposit_whitelist.push(user);
        }
        Ok(())
    }

    pub fn remove_deposit_whitelist(ctx: Context<ModifyWhitelist>, user: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.deposit_whitelist.retain(|u| u != &user);
        Ok(())
    }

    pub fn add_withdraw_whitelist(ctx: Context<ModifyWhitelist>, user: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        if !v.withdraw_whitelist.contains(&user) {
            v.withdraw_whitelist.push(user);
        }
        Ok(())
    }

    pub fn remove_withdraw_whitelist(ctx: Context<ModifyWhitelist>, user: Pubkey) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);
        v.withdraw_whitelist.retain(|u| u != &user);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, base_amount: u64, quote_amount: u64) -> Result<()> {
        let vault_ref = &ctx.accounts.vault;
        require!(!vault_ref.paused, VaultError::VaultPaused);
        require!(!vault_ref.emergency_withdraw_only, VaultError::EmergencyMode);

        if !vault_ref.deposit_whitelist.is_empty() {
            require!(
                vault_ref.deposit_whitelist.contains(&ctx.accounts.user.key()),
                VaultError::NotWhitelisted
            );
        }
        if vault_ref.min_deposit_amount > 0 {
            require!(
                base_amount >= vault_ref.min_deposit_amount
                    && quote_amount >= vault_ref.min_deposit_amount,
                VaultError::BelowMinDeposit
            );
        }
        if vault_ref.max_total_deposit > 0 {
            require!(
                ctx.accounts
                    .base_vault
                    .amount
                    .checked_add(base_amount)
                    .unwrap()
                    <= vault_ref.max_total_deposit
                    && ctx
                        .accounts
                        .quote_vault
                        .amount
                        .checked_add(quote_amount)
                        .unwrap()
                        <= vault_ref.max_total_deposit,
                VaultError::AboveMaxTotal
            );
        }

        let supply = ctx.accounts.share_mint.supply;
        let vault_base = ctx.accounts.base_vault.amount;
        let vault_quote = ctx.accounts.quote_vault.amount;
        let shares = if supply == 0 {
            base_amount.checked_add(quote_amount).unwrap()
        } else {
            let sb = base_amount
                .checked_mul(supply)
                .unwrap()
                .checked_div(vault_base)
                .unwrap();
            let sq = quote_amount
                .checked_mul(supply)
                .unwrap()
                .checked_div(vault_quote)
                .unwrap();
            sb.min(sq)
        };

        // transfer in
        token::transfer(ctx.accounts.transfer_base_to_vault_ctx(), base_amount)?;
        token::transfer(ctx.accounts.transfer_quote_to_vault_ctx(), quote_amount)?;

        // auto‐compound: clear old fees
        {
            let mut vault = &mut ctx.accounts.vault;
            vault.accrued_fee_base = 0;
            vault.accrued_fee_quote = 0;
        }

        // prepare seeds
        let bump = ctx.accounts.vault.bump;
        let base_mint = ctx.accounts.vault.base_mint;
        let quote_mint = ctx.accounts.vault.quote_mint;

        // mint shares
        token::mint_to(
            ctx.accounts
                .mint_shares_ctx()
                .with_signer(&[&[b"vault", base_mint.as_ref(), quote_mint.as_ref(), &[bump]]]),
            shares,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, share_amount: u64) -> Result<()> {
        let vault_ref = &ctx.accounts.vault;
        require!(!vault_ref.paused, VaultError::VaultPaused);
        if vault_ref.emergency_withdraw_only == false && !vault_ref.withdraw_whitelist.is_empty() {
            require!(
                vault_ref.withdraw_whitelist.contains(&ctx.accounts.user.key()),
                VaultError::NotWhitelisted
            );
        }

        let supply = ctx.accounts.share_mint.supply;
        let vb = ctx.accounts.base_vault.amount;
        let vq = ctx.accounts.quote_vault.amount;
        let ba = share_amount.checked_mul(vb).unwrap().checked_div(supply).unwrap();
        let qa = share_amount.checked_mul(vq).unwrap().checked_div(supply).unwrap();

        let fee_b = ba
            .checked_mul(vault_ref.withdraw_fee_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();
        let fee_q = qa
            .checked_mul(vault_ref.withdraw_fee_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();
        let nb = ba.checked_sub(fee_b).unwrap();
        let nq = qa.checked_sub(fee_q).unwrap();

        token::burn(ctx.accounts.burn_shares_ctx(), share_amount)?;
        token::transfer(
            ctx.accounts
                .transfer_base_to_user_ctx()
                .with_signer(&[&[
                    b"vault",
                    vault_ref.base_mint.as_ref(),
                    vault_ref.quote_mint.as_ref(),
                    &[vault_ref.bump],
                ]]),
            nb,
        )?;
        token::transfer(
            ctx.accounts
                .transfer_quote_to_user_ctx()
                .with_signer(&[&[
                    b"vault",
                    vault_ref.base_mint.as_ref(),
                    vault_ref.quote_mint.as_ref(),
                    &[vault_ref.bump],
                ]]),
            nq,
        )?;

        // update fees and auto‐compound
        {
            let mut vault = &mut ctx.accounts.vault;
            vault.accrued_fee_base = vault.accrued_fee_base.checked_add(fee_b).unwrap();
            vault.accrued_fee_quote = vault.accrued_fee_quote.checked_add(fee_q).unwrap();
            // immediately reinvest
            vault.accrued_fee_base = 0;
            vault.accrued_fee_quote = 0;
        }

        Ok(())
    }

    pub fn rebalance(ctx: Context<Rebalance>, current_price: u64) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        require!(!v.paused, VaultError::VaultPaused);
        require!(!v.emergency_withdraw_only, VaultError::EmergencyMode);
        let now = Clock::get()?.unix_timestamp as u64;
        require!(
            now.checked_sub(v.last_rebalance_ts).unwrap() >= v.rebalance_cooldown_sec,
            VaultError::CooldownNotPassed
        );
        require!(is_admin(v, ctx.accounts.authority.key), VaultError::Unauthorized);

        let band = current_price.checked_div(v.band_size_bp as u64 * 100).unwrap_or(0);
        let old_band = v.last_band;
        if band != old_band {
            // TODO: CPI remove liquidity at old_band
            // TODO: CPI add liquidity at band

            v.accrued_fee_base = 0;
            v.accrued_fee_quote = 0;
            v.last_band = band;
            v.last_rebalance_ts = now;
            emit!(RebalanceEvent {
                old_band,
                new_band: band,
                timestamp: now,
            });
        }
        Ok(())
    }

    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        let vault_ref = &ctx.accounts.vault;
        require!(is_admin(vault_ref, ctx.accounts.authority.key), VaultError::Unauthorized);

        // snapshot fees & seeds
        let fb = vault_ref.accrued_fee_base;
        let fq = vault_ref.accrued_fee_quote;
        let bump = vault_ref.bump;
        let base_mint = vault_ref.base_mint;
        let quote_mint = vault_ref.quote_mint;

        if fb > 0 {
            token::transfer(
                ctx.accounts
                    .transfer_fee_base_ctx()
                    .with_signer(&[&[b"vault", base_mint.as_ref(), quote_mint.as_ref(), &[bump]]]),
                fb,
            )?;
        }
        if fq > 0 {
            token::transfer(
                ctx.accounts
                    .transfer_fee_quote_ctx()
                    .with_signer(&[&[b"vault", base_mint.as_ref(), quote_mint.as_ref(), &[bump]]]),
                fq,
            )?;
        }

        // clear accrued fees
        let mut vault = &mut ctx.accounts.vault;
        vault.accrued_fee_base = 0;
        vault.accrued_fee_quote = 0;

        Ok(())
    }
}

// -- Helpers, State, Events, Errors, Contexts (unchanged) --

fn is_admin(v: &Vault, key: &Pubkey) -> bool {
    *key == v.authority || v.admins.contains(key)
}

#[account]
pub struct Vault {
    pub authority:              Pubkey,
    pub admins:                 Vec<Pubkey>,
    pub treasury:               Pubkey,
    pub base_mint:              Pubkey,
    pub quote_mint:             Pubkey,
    pub share_mint:             Pubkey,
    pub base_vault:             Pubkey,
    pub quote_vault:            Pubkey,
    pub fee_bps:                u16,
    pub withdraw_fee_bps:       u16,
    pub band_size_bp:           u16,
    pub min_deposit_amount:     u64,
    pub max_total_deposit:      u64,
    pub last_band:              u64,
    pub rebalance_cooldown_sec: u64,
    pub last_rebalance_ts:      u64,
    pub rebalance_mode:         u8,
    pub paused:                 bool,
    pub emergency_withdraw_only: bool,
    pub accrued_fee_base:       u64,
    pub accrued_fee_quote:      u64,
    pub deposit_whitelist:      Vec<Pubkey>,
    pub withdraw_whitelist:     Vec<Pubkey>,
    pub bump:                   u8,
}

#[event]
pub struct RebalanceEvent {
    pub old_band:   u64,
    pub new_band:   u64,
    pub timestamp:  u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Vault is paused")] VaultPaused,
    #[msg("Emergency withdraw only")] EmergencyMode,
    #[msg("Caller not whitelisted")] NotWhitelisted,
    #[msg("Below minimum deposit")] BelowMinDeposit,
    #[msg("Above maximum total deposit")] AboveMaxTotal,
    #[msg("Cooldown not passed")] CooldownNotPassed,
    #[msg("Unauthorized")] Unauthorized,
}

#[derive(Accounts)]
#[instruction(fee_bps: u16, band_size_bp: u16)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"vault", base_mint.key().as_ref(), quote_mint.key().as_ref()],
        bump,
        space = 4096
    )]
    pub vault:         Account<'info, Vault>,
    #[account(mut)]    pub authority:   Signer<'info>,
    pub treasury:      UncheckedAccount<'info>,
    pub base_mint:     Account<'info, Mint>,
    pub quote_mint:    Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        seeds = [b"share_mint", vault.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = vault
    )]
    pub share_mint:    Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        seeds = [b"base_vault", vault.key().as_ref()],
        bump,
        token::mint = base_mint,
        token::authority = vault
    )]
    pub base_vault:    Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        seeds = [b"quote_vault", vault.key().as_ref()],
        bump,
        token::mint = quote_mint,
        token::authority = vault
    )]
    pub quote_vault:   Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent:          Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)]    pub vault:     Account<'info, Vault>,
    pub authority:     Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateFee<'info> {
    #[account(mut)]    pub vault:     Account<'info, Vault>,
    pub authority:     Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyAdmins<'info> {
    #[account(mut)]    pub vault:     Account<'info, Vault>,
    pub authority:     Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(mut)]    pub vault:     Account<'info, Vault>,
    pub authority:     Signer<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]    pub vault:         Account<'info, Vault>,
    #[account(mut, seeds = [b"share_mint", vault.key().as_ref()], bump = vault.bump)]
    pub share_mint:     Account<'info, Mint>,
    #[account(mut, seeds = [b"base_vault", vault.key().as_ref()], bump = vault.bump)]
    pub base_vault:     Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"quote_vault", vault.key().as_ref()], bump = vault.bump)]
    pub quote_vault:    Account<'info, TokenAccount>,
    #[account(mut)]    pub user:          Signer<'info>,
    #[account(mut, constraint = user_base_ata.mint == vault.base_mint)]
    pub user_base_ata:  Account<'info, TokenAccount>,
    #[account(mut, constraint = user_quote_ata.mint == vault.quote_mint)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_share_ata.mint == vault.share_mint)]
    pub user_share_ata: Account<'info, TokenAccount>,
    pub token_program:  Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn transfer_base_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_base_ata.to_account_info(),
            to: self.base_vault.to_account_info(),
            authority: self.user.to_account_info(),
        })
    }
    pub fn transfer_quote_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_quote_ata.to_account_info(),
            to: self.quote_vault.to_account_info(),
            authority: self.user.to_account_info(),
        })
    }
    pub fn mint_shares_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(self.token_program.to_account_info(), MintTo {
            mint: self.share_mint.to_account_info(),
            to: self.user_share_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        })
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]    pub vault:         Account<'info, Vault>,
    #[account(mut, seeds = [b"share_mint", vault.key().as_ref()], bump = vault.bump)]
    pub share_mint:     Account<'info, Mint>,
    #[account(mut, seeds = [b"base_vault", vault.key().as_ref()], bump = vault.bump)]
    pub base_vault:     Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"quote_vault", vault.key().as_ref()], bump = vault.bump)]
    pub quote_vault:    Account<'info, TokenAccount>,
    #[account(mut)]    pub user:          Signer<'info>,
    #[account(mut, constraint = user_base_ata.mint == vault.base_mint)]
    pub user_base_ata:  Account<'info, TokenAccount>,
    #[account(mut, constraint = user_quote_ata.mint == vault.quote_mint)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_share_ata.mint == vault.share_mint)]
    pub user_share_ata: Account<'info, TokenAccount>,
    pub token_program:  Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn burn_shares_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Burn {
            mint: self.share_mint.to_account_info(),
            from: self.user_share_ata.to_account_info(),
            authority: self.user.to_account_info(),
        })
    }
    pub fn transfer_base_to_user_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.base_vault.to_account_info(),
            to: self.user_base_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        })
    }
    pub fn transfer_quote_to_user_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.quote_vault.to_account_info(),
            to: self.user_quote_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        })
    }
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(mut)]    pub vault:         Account<'info, Vault>,
    pub authority:     Signer<'info>,
    #[account(mut, seeds = [b"base_vault", vault.key().as_ref()], bump = vault.bump)]
    pub base_vault:    Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"quote_vault", vault.key().as_ref()], bump = vault.bump)]
    pub quote_vault:   Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock:         Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)]    pub vault:             Account<'info, Vault>,
    pub authority:     Signer<'info>,
    #[account(mut, seeds = [b"base_vault", vault.key().as_ref()], bump = vault.bump)]
    pub base_vault:        Account<'info, TokenAccount>,
    #[account(mut, constraint = treasury_base_ata.owner == vault.treasury && treasury_base_ata.mint == vault.base_mint)]
    pub treasury_base_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"quote_vault", vault.key().as_ref()], bump = vault.bump)]
    pub quote_vault:       Account<'info, TokenAccount>,
    #[account(mut, constraint = treasury_quote_ata.owner == vault.treasury && treasury_quote_ata.mint == vault.quote_mint)]
    pub treasury_quote_ata:Account<'info, TokenAccount>,
    pub token_program:     Program<'info, Token>,
}

impl<'info> SweepFees<'info> {
    pub fn transfer_fee_base_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.base_vault.to_account_info(),
            to: self.treasury_base_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        })
    }
    pub fn transfer_fee_quote_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.quote_vault.to_account_info(),
            to: self.treasury_quote_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        })
    }
}
