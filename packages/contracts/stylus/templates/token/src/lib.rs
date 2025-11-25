// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

//! ERC-20 Token Template for Stylus
//!
//! This template provides a comprehensive ERC-20 token implementation
//! with additional features like minting, burning, and pausable operations.

#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::string::String;
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    alloy_sol_types::sol,
    evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageU8},
};

// Solidity ABI for events
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Paused(address account);
    event Unpaused(address account);
}

// Storage layout for the ERC-20 token
sol_storage! {
    #[entrypoint]
    pub struct Erc20Token {
        // Token metadata
        StorageString name;
        StorageString symbol;
        StorageU8 decimals;

        // Token state
        StorageU256 total_supply;
        StorageMap<Address, U256> balances;
        StorageMap<Address, StorageMap<Address, U256>> allowances;

        // Initialization and control
        StorageBool initialized;
        StorageAddress owner;
        StorageBool paused;
    }
}

#[external]
impl Erc20Token {
    /// Initialize the token (called once by the deployer)
    pub fn initialize(
        &mut self,
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: U256,
        owner: Address,
    ) -> Result<(), Vec<u8>> {
        // Ensure not already initialized
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }

        // Set metadata
        self.name.set_str(&name);
        self.symbol.set_str(&symbol);
        self.decimals.set(decimals);

        // Mint initial supply to owner
        self.total_supply.set(total_supply);
        self.balances.setter(owner).set(total_supply);
        self.owner.set(owner);
        self.paused.set(false);

        // Mark as initialized
        self.initialized.set(true);

        // Emit Transfer event from zero address
        evm::log(Transfer {
            from: Address::ZERO,
            to: owner,
            value: total_supply,
        });

        Ok(())
    }

    /// Get the token name
    pub fn name(&self) -> Result<String, Vec<u8>> {
        Ok(self.name.get_string())
    }

    /// Get the token symbol
    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        Ok(self.symbol.get_string())
    }

    /// Get the number of decimals
    pub fn decimals(&self) -> Result<u8, Vec<u8>> {
        Ok(self.decimals.get())
    }

    /// Get the total supply
    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_supply.get())
    }

    /// Get the balance of an account
    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(account))
    }

    /// Transfer tokens to another account
    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool, Vec<u8>> {
        self.require_not_paused()?;

        let from = msg::sender();

        // Validate recipient
        if to == Address::ZERO {
            return Err(b"Transfer to zero address".to_vec());
        }

        // Check balance
        let from_balance = self.balances.get(from);
        if from_balance < amount {
            return Err(b"Insufficient balance".to_vec());
        }

        // Update balances
        self.balances.setter(from).set(from_balance - amount);
        let to_balance = self.balances.get(to);
        self.balances.setter(to).set(to_balance + amount);

        // Emit event
        evm::log(Transfer {
            from,
            to,
            value: amount,
        });

        Ok(true)
    }

    /// Approve spender to spend tokens
    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool, Vec<u8>> {
        self.require_not_paused()?;

        let owner = msg::sender();

        // Validate spender
        if spender == Address::ZERO {
            return Err(b"Approve to zero address".to_vec());
        }

        // Set allowance
        self.allowances.setter(owner).setter(spender).set(amount);

        // Emit event
        evm::log(Approval {
            owner,
            spender,
            value: amount,
        });

        Ok(true)
    }

    /// Get the allowance for a spender
    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        Ok(self.allowances.getter(owner).get(spender))
    }

    /// Transfer tokens from one account to another
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<bool, Vec<u8>> {
        self.require_not_paused()?;

        let spender = msg::sender();

        // Validate recipient
        if to == Address::ZERO {
            return Err(b"Transfer to zero address".to_vec());
        }

        // Check allowance
        let current_allowance = self.allowances.getter(from).get(spender);
        if current_allowance < amount {
            return Err(b"Insufficient allowance".to_vec());
        }

        // Check balance
        let from_balance = self.balances.get(from);
        if from_balance < amount {
            return Err(b"Insufficient balance".to_vec());
        }

        // Update allowance
        self.allowances
            .setter(from)
            .setter(spender)
            .set(current_allowance - amount);

        // Update balances
        self.balances.setter(from).set(from_balance - amount);
        let to_balance = self.balances.get(to);
        self.balances.setter(to).set(to_balance + amount);

        // Emit event
        evm::log(Transfer {
            from,
            to,
            value: amount,
        });

        Ok(true)
    }

    /// Increase allowance for a spender
    pub fn increase_allowance(
        &mut self,
        spender: Address,
        added_value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.require_not_paused()?;

        let owner = msg::sender();
        let current_allowance = self.allowances.getter(owner).get(spender);
        let new_allowance = current_allowance + added_value;

        self.allowances.setter(owner).setter(spender).set(new_allowance);

        evm::log(Approval {
            owner,
            spender,
            value: new_allowance,
        });

        Ok(true)
    }

    /// Decrease allowance for a spender
    pub fn decrease_allowance(
        &mut self,
        spender: Address,
        subtracted_value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.require_not_paused()?;

        let owner = msg::sender();
        let current_allowance = self.allowances.getter(owner).get(spender);

        if current_allowance < subtracted_value {
            return Err(b"Allowance below zero".to_vec());
        }

        let new_allowance = current_allowance - subtracted_value;
        self.allowances.setter(owner).setter(spender).set(new_allowance);

        evm::log(Approval {
            owner,
            spender,
            value: new_allowance,
        });

        Ok(true)
    }

    /// Mint new tokens (owner only)
    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        self.require_not_paused()?;

        if to == Address::ZERO {
            return Err(b"Mint to zero address".to_vec());
        }

        // Update balance
        let current_balance = self.balances.get(to);
        self.balances.setter(to).set(current_balance + amount);

        // Update total supply
        let current_supply = self.total_supply.get();
        self.total_supply.set(current_supply + amount);

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            value: amount,
        });

        Ok(())
    }

    /// Burn tokens from caller's balance
    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        let from = msg::sender();
        let current_balance = self.balances.get(from);

        if current_balance < amount {
            return Err(b"Burn amount exceeds balance".to_vec());
        }

        // Update balance
        self.balances.setter(from).set(current_balance - amount);

        // Update total supply
        let current_supply = self.total_supply.get();
        self.total_supply.set(current_supply - amount);

        evm::log(Transfer {
            from,
            to: Address::ZERO,
            value: amount,
        });

        Ok(())
    }

    /// Burn tokens from an address using allowance
    pub fn burn_from(&mut self, from: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        let spender = msg::sender();

        // Check allowance
        let current_allowance = self.allowances.getter(from).get(spender);
        if current_allowance < amount {
            return Err(b"Burn amount exceeds allowance".to_vec());
        }

        // Check balance
        let current_balance = self.balances.get(from);
        if current_balance < amount {
            return Err(b"Burn amount exceeds balance".to_vec());
        }

        // Update allowance
        self.allowances
            .setter(from)
            .setter(spender)
            .set(current_allowance - amount);

        // Update balance
        self.balances.setter(from).set(current_balance - amount);

        // Update total supply
        let current_supply = self.total_supply.get();
        self.total_supply.set(current_supply - amount);

        evm::log(Transfer {
            from,
            to: Address::ZERO,
            value: amount,
        });

        Ok(())
    }

    /// Pause the contract (owner only)
    pub fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.require_owner()?;

        if self.paused.get() {
            return Ok(());
        }

        self.paused.set(true);

        evm::log(Paused {
            account: msg::sender(),
        });

        Ok(())
    }

    /// Unpause the contract (owner only)
    pub fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.require_owner()?;

        if !self.paused.get() {
            return Ok(());
        }

        self.paused.set(false);

        evm::log(Unpaused {
            account: msg::sender(),
        });

        Ok(())
    }

    /// Get the owner address
    pub fn owner(&self) -> Result<Address, Vec<u8>> {
        Ok(self.owner.get())
    }

    /// Check if contract is paused
    pub fn is_paused(&self) -> Result<bool, Vec<u8>> {
        Ok(self.paused.get())
    }

    /// Transfer ownership (owner only)
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        self.require_owner()?;

        if new_owner == Address::ZERO {
            return Err(b"New owner is zero address".to_vec());
        }

        self.owner.set(new_owner);
        Ok(())
    }
}

/// Internal helper methods
impl Erc20Token {
    /// Require that caller is the owner
    fn require_owner(&self) -> Result<(), Vec<u8>> {
        if msg::sender() != self.owner.get() {
            return Err(b"Caller is not owner".to_vec());
        }
        Ok(())
    }

    /// Require that contract is not paused
    fn require_not_paused(&self) -> Result<(), Vec<u8>> {
        if self.paused.get() {
            return Err(b"Contract is paused".to_vec());
        }
        Ok(())
    }
}
