use anchor_lang::prelude::*;
use anchor_spl::token::Token;


declare_id!("9nsWkWFdfLAbuX1ymmaVgHFnvFnosCm1CdxjBV9z6MPG");

#[program]
pub mod team_pool {
    use super::*;
    use anchor_lang::system_program;
    use anchor_spl::token;



    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        number_of_max_members: u8,
        price: f32,
        privacy: PoolPrivacy,
        pool_code: Option<String>,
    ) -> Result<()> {
        let pool: &mut Account<Pool> = &mut ctx.accounts.pool_pda;

        pool.creator = *ctx.accounts.creator.key;
        pool.members = Vec::with_capacity(number_of_max_members as usize);
        pool.number_of_max_members = number_of_max_members;
        pool.price = price;
        pool.status = PoolStatus::Open;
        if number_of_max_members == 0 {
            return Err(ErrorCode::DivisionByZero.into());
        }
    
        // Realizar a divisão
        pool.price_per_member = price / (number_of_max_members as f32);

        if privacy == PoolPrivacy::Private {
            require!(pool_code.is_some(), ErrorCode::PoolCodeRequired);
            pool.pool_code = pool_code.unwrap();
        } else {
            pool.pool_code = String::new();
        }

        pool.privacy = privacy.clone();

        require!(number_of_max_members > 0, ErrorCode::InvalidNumberOfMaxMembers);

        emit!(PoolCreated {
            pool: pool.key(),
            creator: pool.creator,
            number_of_max_members,
            price,
            privacy
        });

        Ok(())
    }

    pub fn join_pool(ctx: Context<JoinPool>, pool_code: Option<String>) -> Result<()> {
        let pool: &mut Account<Pool> = &mut ctx.accounts.pool_pda;
        let new_member: Pubkey = ctx.accounts.new_member.key();

        require!(pool.status == PoolStatus::Open, ErrorCode::PoolNotOpen);

        let number_of_members = pool.members.len();

        require!(
            number_of_members < pool.number_of_max_members as usize,
            ErrorCode::PoolFull
        );

        if pool.privacy == PoolPrivacy::Private {
                require!(pool_code.is_some(), ErrorCode::PoolCodeRequired);
                require!(pool.pool_code == pool_code.unwrap(), ErrorCode::WrongPoolCode)
        }

        // Atualiza o status da pool para cheia, se necessário
        if pool.members.len() == pool.number_of_max_members as usize {
            pool.status = PoolStatus::Full;
        }

        emit!(MemberJoined {
            pool: pool.key(),
            member: new_member,
        });



        let amount: f32 = pool.price_per_member;
        
        let cpi_context =  CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.new_member.to_account_info().clone(),
                to: ctx.accounts.pool_vault_pda.to_account_info().clone()
            },
        );

        let amount_lamports = (amount * 1_000_000_000.0) as u64;
        system_program::transfer(cpi_context, amount_lamports)?;


        pool.vault = pool.vault + amount;

        emit!(VaultUpdated {
            pool: pool.key(),
            amount,
        });

        pool.members.push(new_member);

        Ok(())
    }

    pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool_pda;

        pool.status = PoolStatus::Closed;


        Ok(())
    }

    pub fn transfer_to_creator(
        ctx: Context<TransferToCreator>,
        amount: u64, // quantidade de tokens a ser transferida
    ) -> Result<()> {
       
        let cpi_context =  CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                authority: ctx.accounts.vault_authority.to_account_info().clone(),
                from: ctx.accounts.pool_vault_pda.to_account_info().clone(),
                to: ctx.accounts.vault_authority.to_account_info().clone()
            },
        );
        
        token::transfer(cpi_context, amount)?;
    
        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    // Derive pool PDA
    #[account(init, 
        seeds = [b"pool", creator.key().as_ref()], 
        bump, 
        payer = creator, 
        space = 8 + Pool::INIT_SPACE,
    )]
    pub pool_pda: Account<'info, Pool>,

    // Derive Vault PDA
    #[account(init, 
        seeds = [b"pool_vault", pool_pda.key().as_ref()], 
        bump, 
        payer = creator,
        space = 8 + 8,
    )]
    pub pool_vault_pda: Account<'info, PoolVault>,

    
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePool<'info> {
    #[account(mut, 
        seeds = [b"pool", pool_pda.creator.as_ref()], 
        bump, 
    )]
    pub pool_pda: Account<'info, Pool>,


    #[account(address = pool_pda.creator)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct JoinPool<'info> {
    #[account(mut, 
        seeds = [b"pool", pool_pda.creator.as_ref()], 
        bump, 
    )]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, 
        seeds = [b"pool_vault", pool_pda.key().as_ref()], 
        bump, 
    )]
    pub pool_vault_pda: Account<'info, PoolVault>,

    #[account(mut)]
    pub new_member: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferToCreator<'info> {
    #[account(mut, 
        seeds = [b"pool", pool_pda.creator.as_ref()], 
        bump, 
    )]
    pub pool_pda: Account<'info, Pool>,

    #[account(mut, 
        seeds = [b"pool_vault", pool_pda.key().as_ref()], 
        bump, 
    )]
    pub pool_vault_pda: Account<'info, PoolVault>,

    // A conta de token do criador
    #[account(address = pool_pda.creator)]
    pub vault_authority: Signer<'info>,

    // O programa de tokens SPL
    pub token_program: Program<'info, Token>,
}



#[account]
pub struct PoolVault {
    pub amount: u64,  // Espaço para armazenar o valor em lamports

}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub creator: Pubkey,
    #[max_len(50)]
    pub members: Vec<Pubkey>,
    pub number_of_max_members: u8,
    pub price: f32,
    pub price_per_member: f32,
    pub status: PoolStatus,
    #[max_len(7)]
    pub privacy: PoolPrivacy,
    #[max_len(10)]
    pub pool_code: String,
    pub vault: f32,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, InitSpace)]
pub enum PoolStatus {
    Open,
    Full,
    Closed,
    Canceled,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, InitSpace)]
pub enum PoolPrivacy {
    Private,
    Public,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The number of maximum members cannot be zero.")]
    InvalidNumberOfMaxMembers,

    #[msg("The pool is full.")]
    PoolFull,

    #[msg("The pool is not open")]
    PoolNotOpen,

    #[msg("The pool code is required")]
    PoolCodeRequired,

    #[msg("The pool code is invalid")]
    WrongPoolCode,

    #[msg("An arithmetic overflow occurred.")]
    Overflow,

    #[msg("Division by zero.")]
    DivisionByZero,


    #[msg("A conta fornecida não corresponde à conta do criador da pool.")]
    InvalidCreatorAccount,
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub creator: Pubkey,
    pub number_of_max_members: u8,
    pub price: f32,
    pub privacy: PoolPrivacy,
}

#[event]
pub struct MemberJoined {
    pub pool: Pubkey,
    pub member: Pubkey,
}

#[event]
pub struct VaultUpdated {
    pub pool: Pubkey,
    pub amount: f32,
}

