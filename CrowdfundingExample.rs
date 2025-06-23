use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn initialize_campaign(
        ctx: Context<InitializeCampaign>,
        title: String,
        description: String,
        target_amount: u64,
        duration_days: u64,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let clock = Clock::get()?;

        // Input validation
        require!(title.len() <= 100, CrowdfundingError::TitleTooLong);
        require!(description.len() <= 500, CrowdfundingError::DescriptionTooLong);
        require!(target_amount > 0, CrowdfundingError::InvalidTargetAmount);
        require!(duration_days > 0 && duration_days <= 365, CrowdfundingError::InvalidDuration);

        campaign.creator = ctx.accounts.creator.key();
        campaign.title = title;
        campaign.description = description;
        campaign.target_amount = target_amount;
        campaign.current_amount = 0;
        campaign.start_time = clock.unix_timestamp;
        campaign.end_time = clock.unix_timestamp + (duration_days as i64 * 24 * 60 * 60);
        campaign.is_successful = false;
        campaign.is_withdrawn = false;
        campaign.contributors_count = 0;

        emit!(CampaignCreated {
            campaign: campaign.key(),
            creator: campaign.creator,
            target_amount: campaign.target_amount,
            end_time: campaign.end_time,
        });

        Ok(())
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let contribution = &mut ctx.accounts.contribution;
        let clock = Clock::get()?;

        // Check if campaign is active
        require!(clock.unix_timestamp < campaign.end_time, CrowdfundingError::CampaignEnded);
        require!(amount > 0, CrowdfundingError::InvalidContributionAmount);
        require!(!campaign.is_withdrawn, CrowdfundingError::CampaignAlreadyWithdrawn);

        // Check if we don't exceed the target
        let new_total = campaign.current_amount
            .checked_add(amount)
            .ok_or(CrowdfundingError::AmountOverflow)?;
        
        require!(new_total <= campaign.target_amount, CrowdfundingError::ExceedsTarget);

        // Transfer tokens to campaign vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.contributor_token_account.to_account_info(),
            to: ctx.accounts.campaign_vault.to_account_info(),
            authority: ctx.accounts.contributor.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update contribution state
        if contribution.amount == 0 {
            // New contributor
            contribution.contributor = ctx.accounts.contributor.key();
            contribution.campaign = campaign.key();
            campaign.contributors_count += 1;
        }
        
        contribution.amount = contribution.amount
            .checked_add(amount)
            .ok_or(CrowdfundingError::AmountOverflow)?;
        
        campaign.current_amount = new_total;

        // Check if target has been reached
        if campaign.current_amount >= campaign.target_amount {
            campaign.is_successful = true;
        }

        emit!(ContributionMade {
            campaign: campaign.key(),
            contributor: ctx.accounts.contributor.key(),
            amount,
            total_raised: campaign.current_amount,
        });

        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let clock = Clock::get()?;

        // Check permissions
        require!(
            campaign.creator == ctx.accounts.creator.key(),
            CrowdfundingError::UnauthorizedWithdrawal
        );

        // Check withdrawal conditions
        require!(
            campaign.is_successful || clock.unix_timestamp >= campaign.end_time,
            CrowdfundingError::WithdrawalConditionsNotMet
        );

        require!(!campaign.is_withdrawn, CrowdfundingError::AlreadyWithdrawn);

        let amount_to_withdraw = ctx.accounts.campaign_vault.amount;
        require!(amount_to_withdraw > 0, CrowdfundingError::NoFundsToWithdraw);

        // Seeds for PDA vault
        let campaign_key = campaign.key();
        let seeds = &[
            b"vault",
            campaign_key.as_ref(),
            &[ctx.bumps.campaign_vault],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer funds to campaign creator
        let cpi_accounts = Transfer {
            from: ctx.accounts.campaign_vault.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: ctx.accounts.campaign_vault.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, amount_to_withdraw)?;

        campaign.is_withdrawn = true;

        emit!(FundsWithdrawn {
            campaign: campaign.key(),
            creator: campaign.creator,
            amount: amount_to_withdraw,
        });

        Ok(())
    }

    pub fn refund_contribution(ctx: Context<RefundContribution>) -> Result<()> {
        let campaign = &ctx.accounts.campaign;
        let contribution = &mut ctx.accounts.contribution;
        let clock = Clock::get()?;

        // Check refund conditions
        require!(
            clock.unix_timestamp >= campaign.end_time,
            CrowdfundingError::CampaignStillActive
        );
        
        require!(!campaign.is_successful, CrowdfundingError::CampaignWasSuccessful);
        require!(contribution.amount > 0, CrowdfundingError::NoContributionToRefund);

        let refund_amount = contribution.amount;

        // Seeds for PDA vault
        let campaign_key = campaign.key();
        let seeds = &[
            b"vault",
            campaign_key.as_ref(),
            &[ctx.bumps.campaign_vault],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer refund to contributor
        let cpi_accounts = Transfer {
            from: ctx.accounts.campaign_vault.to_account_info(),
            to: ctx.accounts.contributor_token_account.to_account_info(),
            authority: ctx.accounts.campaign_vault.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, refund_amount)?;

        contribution.amount = 0;

        emit!(ContributionRefunded {
            campaign: campaign.key(),
            contributor: ctx.accounts.contributor.key(),
            amount: refund_amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct InitializeCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::SIZE,
        seeds = [b"campaign", creator.key().as_ref(), title.as_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    
    #[account(
        init,
        payer = creator,
        token::mint = mint,
        token::authority = campaign_vault,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub campaign_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    
    #[account(
        init_if_needed,
        payer = contributor,
        space = Contribution::SIZE,
        seeds = [b"contribution", campaign.key().as_ref(), contributor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,
    
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub campaign_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub contributor_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub contributor: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub campaign_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RefundContribution<'info> {
    pub campaign: Account<'info, Campaign>,
    
    #[account(
        mut,
        seeds = [b"contribution", campaign.key().as_ref(), contributor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,
    
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub campaign_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub contributor_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub contributor: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Campaign {
    pub creator: Pubkey,           // 32 bytes
    pub title: String,             // 4 + 100 bytes
    pub description: String,       // 4 + 500 bytes
    pub target_amount: u64,        // 8 bytes
    pub current_amount: u64,       // 8 bytes
    pub start_time: i64,           // 8 bytes
    pub end_time: i64,             // 8 bytes
    pub is_successful: bool,       // 1 byte
    pub is_withdrawn: bool,        // 1 byte
    pub contributors_count: u32,   // 4 bytes
}

impl Campaign {
    pub const SIZE: usize = 8 + 32 + 4 + 100 + 4 + 500 + 8 + 8 + 8 + 8 + 1 + 1 + 4;
}

#[account]
pub struct Contribution {
    pub contributor: Pubkey,       // 32 bytes
    pub campaign: Pubkey,          // 32 bytes
    pub amount: u64,               // 8 bytes
}

impl Contribution {
    pub const SIZE: usize = 8 + 32 + 32 + 8;
}

#[event]
pub struct CampaignCreated {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub target_amount: u64,
    pub end_time: i64,
}

#[event]
pub struct ContributionMade {
    pub campaign: Pubkey,
    pub contributor: Pubkey,
    pub amount: u64,
    pub total_raised: u64,
}

#[event]
pub struct FundsWithdrawn {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ContributionRefunded {
    pub campaign: Pubkey,
    pub contributor: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum CrowdfundingError {
    #[msg("Campaign title is too long (max 100 characters)")]
    TitleTooLong,
    
    #[msg("Campaign description is too long (max 500 characters)")]
    DescriptionTooLong,
    
    #[msg("Invalid target amount")]
    InvalidTargetAmount,
    
    #[msg("Invalid campaign duration (1-365 days)")]
    InvalidDuration,
    
    #[msg("Campaign has already ended")]
    CampaignEnded,
    
    #[msg("Invalid contribution amount")]
    InvalidContributionAmount,
    
    #[msg("Contribution exceeds campaign target")]
    ExceedsTarget,
    
    #[msg("Amount overflow")]
    AmountOverflow,
    
    #[msg("Unauthorized withdrawal")]
    UnauthorizedWithdrawal,
    
    #[msg("Withdrawal conditions not met")]
    WithdrawalConditionsNotMet,
    
    #[msg("Funds already withdrawn")]
    AlreadyWithdrawn,
    
    #[msg("No funds to withdraw")]
    NoFundsToWithdraw,
    
    #[msg("Campaign is still active")]
    CampaignStillActive,
    
    #[msg("Campaign was successful")]
    CampaignWasSuccessful,
    
    #[msg("No contribution to refund")]
    NoContributionToRefund,
    
    #[msg("Campaign funds already withdrawn")]
    CampaignAlreadyWithdrawn,
}