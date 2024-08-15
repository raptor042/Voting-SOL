use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock::Clock, sysvar::Sysvar, system_instruction};

declare_id!("DZSvsU8uyfaq2E93wyQF6RmwW7WdQWMiZRLecLA71dki");

#[program]
pub mod voting_dapp {
    use super::*;

    pub fn start_election(ctx: Context<StartElection>, name: String, description: String, candidates: [String; 4], duration: u8) -> Result<()> {
        require!(name.len() <= Election::MAX_NAME_LEN, Errors::MaxStrLen);
        require!(description.len() <= Election::MAX_DESC_LEN, Errors::MaxStrLen);
        require!(candidates.len() <= Election::MAX_CANDIDATES, Errors::MaxNoOfCandidatesExceeded);

        msg!("Initializing the {} election for {} hours.", name, duration);
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let election = &mut ctx.accounts.election;

        election.creator = ctx.accounts.creator.key();
        election.name = name;
        election.description = description;
        election.candidates = candidates;
        election.candidate_votes = [0, 0, 0, 0];
        election.duration = duration;
        election.started_at = timestamp;
        election.has_ended = false;
        election.winner = None;

        msg!("Started the election.");

        Ok(())
    }

    pub fn vote(ctx: Context<Voting>, name: String, candidate: String, amount: u64) -> Result<()> {
        msg!("Initializing your vote for the {} election.", name);
        let election = &mut ctx.accounts.election;
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let duration = (timestamp - election.started_at) / (60 * 60);

        let mut result = false;
        let mut index = 0;
        for i in &election.candidates {
            if i.as_str() == candidate.as_str() {
                result = true;
            } else {
                index += 1;
            }
        }
        require!(result, Errors::CandidateDoesNotExist);
        require!(duration <= election.duration.into(), Errors::ElectionHasEnded);
        
        election.candidate_votes[index] += amount;

        let lamports_transfer_instruction = system_instruction::transfer(
            ctx.accounts.voter.key,
            ctx.accounts.creator.key,
            amount
        );
        anchor_lang::solana_program::program::invoke_signed(
            &lamports_transfer_instruction,
            &[
                ctx.accounts.voter.to_account_info(),
                ctx.accounts.creator.clone(),
                ctx.accounts.system_program.to_account_info()
            ],
            &[]
        ).unwrap();

        msg!("Voted successfully.");
        
        Ok(())
    }

    pub fn close_election(ctx: Context<CloseElection>, name: String) -> Result<()> {
        msg!("Closing the {} election.", name);
        let election = &ctx.accounts.election;
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let duration = (timestamp - election.started_at) / (60 * 60);

        require!(duration > election.duration.into(), Errors::ElectionHasNotEnded);

        msg!("Election has been closed.");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, description: String, candidates: [String; 4])]
pub struct StartElection<'info> {
    #[account(
        init,
        payer = creator,
        space = Election::INIT_SPACE + name.len() + description.len() + candidates[0].len() + candidates[1].len() + candidates[2].len() + candidates[3].len(),
        seeds = [b"election", name.as_bytes()],
        bump
    )]
    pub election: Account<'info, Election>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Voting<'info> {
    #[account(
        mut,
        seeds = [b"election", name.as_bytes()],
        bump
    )]
    pub election: Account<'info, Election>,
    #[account(mut)]
    pub voter: Signer<'info>,
    /// CHECK: Creator's address
    #[account(mut)]
    pub creator: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CloseElection<'info> {
    #[account(
        mut,
        seeds = [b"election", name.as_bytes()],
        bump,
        close = creator
    )]
    pub election: Account<'info, Election>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Election {
    creator: Pubkey,
    name: String,
    description: String,
    candidates: [String; 4],
    candidate_votes: [u64; 4],
    duration: u8,
    started_at: i64,
    has_ended: bool,
    winner: Option<Pubkey>
}

impl Space for Election {
    const INIT_SPACE: usize = 8 + 32 + 4 + 4 + (4 * 4) + (8 * 4) + 1 + 8 + 1 + (1 + 32);
}

impl Election {
    const MAX_CANDIDATES: usize = 4;

    const MAX_NAME_LEN: usize = 12;

    const MAX_DESC_LEN: usize = 48;
}

#[error_code]
pub enum Errors {
    #[msg("Election has ended")]
    ElectionHasEnded,

    #[msg("Election has not ended")]
    ElectionHasNotEnded,

    #[msg("String is too long")]
    MaxStrLen,

    #[msg("Exceeds max number of candidates")]
    MaxNoOfCandidatesExceeded,

    #[msg("Candidate does not exist")]
    CandidateDoesNotExist,
}