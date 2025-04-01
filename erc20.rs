use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::{prelude::*, Host};

pub trait Erc20Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

sol_storage! {
    pub struct Erc20<T> {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        uint256 total_supply;
        PhantomData<T> phantom;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl<T: Erc20Params> Erc20<T> {
    pub fn _transfer(
        &mut self,
        host: &dyn Host,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Erc20Error> {
        let mut sender_balance = self.balances.setter(from);
        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }
        sender_balance.set(old_sender_balance - value);

        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;
        to_balance.set(new_to_balance);

        // Log the transfer event
        log(host, Transfer { from, to, value });
        Ok(())
    }

    pub fn mint(&mut self, host: &dyn Host, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let new_balance = balance.get() + value;
        balance.set(new_balance);

        self.total_supply.set(self.total_supply.get() + value);

        // Log minting as a transfer from the zero address.
        log(host, Transfer { from: Address::new([0u8; 20]), to: address, value });

        Ok(())
    }

    pub fn burn(&mut self, host: &dyn Host, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from: address,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);

        self.total_supply.set(self.total_supply.get() - value);

        // Log burning as a transfer to the zero address.
        log(host, Transfer { from: address, to: Address::new([0u8; 20]), value });

        Ok(())
    }
}

#[public]  // Replaced deprecated #[external] with #[public]
impl<T: Erc20Params> Erc20<T> {
    pub fn name() -> String {
        T::NAME.into()
    }

    pub fn symbol() -> String {
        T::SYMBOL.into()
    }

    pub fn decimals() -> u8 {
        T::DECIMALS
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner)
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        let sender = self.vm().msg_sender();
        self._transfer(self.vm(), sender, to, value)?;
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        let sender = self.vm().msg_sender();
        let mut sender_allowances = self.allowances.setter(from);
        // Retrieve allowance for the sender
        let mut allowance = sender_allowances.setter(sender);
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err(Erc20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: sender,
                have: old_allowance,
                want: value,
            }));
        }

        allowance.set(old_allowance - value);
        self._transfer(self.vm(), from, to, value)?;

        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> bool {
        self.allowances.setter(self.vm().msg_sender()).insert(spender, value);
        log(self.vm(), Approval {
            owner: self.vm().msg_sender(),
            spender,
            value,
        });
        true
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }
}use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::{evm, msg, prelude::*};

pub trait Erc20Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

sol_storage! {
    pub struct Erc20<T> {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        uint256 total_supply;
        PhantomData<T> phantom;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl<T: Erc20Params> Erc20<T> {
    pub fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Erc20Error> {
        let mut sender_balance = self.balances.setter(from);
        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }
        sender_balance.set(old_sender_balance - value);

        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;
        to_balance.set(new_to_balance);

        evm::log(Transfer { from, to, value });
        Ok(())
    }

    pub fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let new_balance = balance.get() + value;
        balance.set(new_balance);

        self.total_supply.set(self.total_supply.get() + value);

        evm::log(Transfer {
            from: Address::ZERO,
            to: address,
            value,
        });

        Ok(())
    }

    pub fn burn(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from: address,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);

        self.total_supply.set(self.total_supply.get() - value);

        evm::log(Transfer {
            from: address,
            to: Address::ZERO,
            value,
        });

        Ok(())
    }
}

#[external]
impl<T: Erc20Params> Erc20<T> {
    pub fn name() -> String {
        T::NAME.into()
    }

    pub fn symbol() -> String {
        T::SYMBOL.into()
    }

    pub fn decimals() -> u8 {
        T::DECIMALS
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner)
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self._transfer(msg::sender(), to, value)?;
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        let mut sender_allowances = self.allowances.setter(from);
        let mut allowance = sender_allowances.setter(msg::sender());
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err(Erc20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: msg::sender(),
                have: old_allowance,
                want: value,
            }));
        }

        allowance.set(old_allowance - value);
        self._transfer(from, to, value)?;

        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> bool {
        self.allowances.setter(msg::sender()).insert(spender, value);
        evm::log(Approval {
            owner: msg::sender(),
            spender,
            value,
        });
        true
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }
}
