use anchor_lang::prelude::*;

declare_id!("9nsWkWFdfLAbuX1ymmaVgHFnvFnosCm1CdxjBV9z6MPG");

#[program]
pub mod team_pool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
