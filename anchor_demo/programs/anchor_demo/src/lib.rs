use anchor_lang::prelude::*;

declare_id!("6tD1SdiNAyQT4oZkTAqmueRUETPMkkhs4N7bNhHWGSu9");

#[program]
pub mod anchor_demo {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing counter: {:?}", ctx.program_id);

        ctx.accounts.counter.count = 0;

        Ok(())
    }

    pub fn math_operation(ctx: Context<MathOperation>, operation: Operation) -> Result<()> {
        msg!(
            "Addition on counter. Current count: {}",
            ctx.accounts.counter.count
        );

        ctx.accounts.counter.count = operation.execute(ctx.accounts.counter.count);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(init, space = 8 + Counter::INIT_SPACE, payer = owner, seeds = [b"counter"], bump)]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MathOperation<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

#[account]
#[derive(InitSpace, Default)]
pub struct Counter {
    pub count: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Operation {
    Add(u64),
    Sub(u64),
    Mul(u64),
    Div(u64),
}

impl Operation {
    pub fn execute(&self, operand: u64) -> u64 {
        match self {
            Operation::Add(amount) => operand.checked_add(*amount).unwrap(),
            Operation::Sub(amount) => operand.checked_sub(*amount).unwrap(),
            Operation::Mul(amount) => amount.checked_mul(*amount).unwrap(),
            Operation::Div(amount) => operand.checked_div(*amount).unwrap(),
        }
    }
}
