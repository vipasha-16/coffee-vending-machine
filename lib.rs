#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

mod erc20;

use alloy_primitives::{Address, U256};
use stylus_sdk::{msg, prelude::*};
use crate::erc20::{Erc20, Erc20Params, Erc20Error};

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

struct StylusTokenParams;
impl Erc20Params for StylusTokenParams {
    const NAME: &'static str = "StylusToken";
    const SYMBOL: &'static str = "STK";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
    struct StylusToken {
        #[borrow]
        Erc20<StylusTokenParams> erc20;
    }
}

#[external]
#[inherit(Erc20<StylusTokenParams>)]
impl StylusToken {
    pub fn mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(msg::sender(), value)?;
        Ok(())
    }

    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(to, value)?;
        Ok(())
    }

    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), value)?;
        Ok(())
    }
}