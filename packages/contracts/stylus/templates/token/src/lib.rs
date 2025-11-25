// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

//! ERC-20 Token Template for Stylus
//!
//! This template provides a standard ERC-20 token implementation
//! that can be deployed via the Universal Deployer.

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

        // Initialization guard
        StorageBool initialized;
        StorageAddress owner;
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

    /// Get the owner address
    pub fn owner(&self) -> Result<Address, Vec<u8>> {
        Ok(self.owner.get())
    }
}
