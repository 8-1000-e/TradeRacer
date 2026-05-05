use bolt_lang::prelude::*;

declare_id!("7mUMr33noPhfFnnQJfY6BwziCAWnevAKHv2AnJ65d1B4");

#[program]
pub mod trade_fight {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
