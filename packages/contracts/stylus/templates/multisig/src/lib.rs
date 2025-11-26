// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

//! Multisig Wallet Template for Stylus
//!
//! This template provides a comprehensive multisig wallet implementation
//! that requires multiple owner confirmations to execute transactions.

#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    alloy_sol_types::sol,
    call::{call, Call},
    evm, msg,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256, StorageVec},
};

// Solidity ABI for events
sol! {
    event OwnerAdded(address indexed owner);
    event OwnerRemoved(address indexed owner);
    event RequirementChanged(uint256 required);
    event TransactionSubmitted(uint256 indexed txId, address indexed submitter, address indexed to, uint256 value);
    event TransactionConfirmed(uint256 indexed txId, address indexed owner);
    event ConfirmationRevoked(uint256 indexed txId, address indexed owner);
    event TransactionExecuted(uint256 indexed txId, address indexed executor);
    event TransactionFailed(uint256 indexed txId, address indexed executor);
    event Deposit(address indexed sender, uint256 value);
}

// Transaction structure
sol_storage! {
    pub struct Transaction {
        address destination;
        uint256 value;
        StorageVec<u8> data;
        bool executed;
        StorageMap<Address, bool> confirmations;
        uint256 confirmation_count;
    }
}

// Storage layout for the multisig wallet
sol_storage! {
    #[entrypoint]
    pub struct MultisigWallet {
        // Owner management
        StorageVec<Address> owners;
        StorageMap<Address, bool> is_owner;
        StorageU256 required_confirmations;

        // Transaction management
        StorageVec<Transaction> transactions;

        // Initialization
        StorageBool initialized;
    }
}

#[external]
impl MultisigWallet {
    /// Initialize the multisig wallet (called once by the deployer)
    pub fn initialize(
        &mut self,
        owners: Vec<Address>,
        required: U256,
    ) -> Result<(), Vec<u8>> {
        // Ensure not already initialized
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }

        // Validate inputs
        if owners.is_empty() {
            return Err(b"Owners required".to_vec());
        }

        let required_u256 = required;
        if required_u256.is_zero() || required_u256 > U256::from(owners.len()) {
            return Err(b"Invalid required confirmations".to_vec());
        }

        // Add owners
        for owner in owners.iter() {
            if *owner == Address::ZERO {
                return Err(b"Invalid owner address".to_vec());
            }

            if self.is_owner.get(*owner) {
                return Err(b"Duplicate owner".to_vec());
            }

            self.owners.push(*owner);
            self.is_owner.setter(*owner).set(true);

            evm::log(OwnerAdded { owner: *owner });
        }

        self.required_confirmations.set(required);
        self.initialized.set(true);

        evm::log(RequirementChanged { required });

        Ok(())
    }

    /// Submit a new transaction
    pub fn submit_transaction(
        &mut self,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<U256, Vec<u8>> {
        self.require_owner()?;

        if to == Address::ZERO {
            return Err(b"Invalid destination".to_vec());
        }

        // Create new transaction
        let tx_id = U256::from(self.transactions.len());
        let mut new_tx = self.transactions.grow();

        new_tx.destination.set(to);
        new_tx.value.set(value);

        // Store data
        for byte in data.iter() {
            new_tx.data.push(*byte);
        }

        new_tx.executed.set(false);
        new_tx.confirmation_count.set(U256::ZERO);

        evm::log(TransactionSubmitted {
            txId: tx_id,
            submitter: msg::sender(),
            to,
            value,
        });

        // Auto-confirm by submitter
        self._confirm_transaction(tx_id)?;

        Ok(tx_id)
    }

    /// Confirm a transaction
    pub fn confirm_transaction(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        self._confirm_transaction(tx_id)
    }

    /// Revoke confirmation for a transaction
    pub fn revoke_confirmation(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        self.require_not_executed(tx_id)?;

        let sender = msg::sender();

        // Check if already confirmed
        let tx = self.get_transaction_mut(tx_id)?;
        if !tx.confirmations.get(sender) {
            return Err(b"Transaction not confirmed".to_vec());
        }

        // Revoke confirmation
        tx.confirmations.setter(sender).set(false);
        let current_count = tx.confirmation_count.get();
        tx.confirmation_count.set(current_count - U256::from(1));

        evm::log(ConfirmationRevoked {
            txId: tx_id,
            owner: sender,
        });

        Ok(())
    }

    /// Execute a confirmed transaction
    pub fn execute_transaction(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        self.require_not_executed(tx_id)?;
        self.require_confirmed(tx_id)?;

        // Mark as executed before external call (reentrancy protection)
        {
            let tx = self.get_transaction_mut(tx_id)?;
            tx.executed.set(true);
        }

        // Get transaction details (separate scope to avoid borrow issues)
        let (destination, value, data) = {
            let tx = self.get_transaction_ref(tx_id)?;
            let destination = tx.destination.get();
            let value = tx.value.get();

            // Read data from storage
            let mut data_vec = Vec::new();
            for i in 0..tx.data.len() {
                if let Some(byte) = tx.data.get(i) {
                    data_vec.push(byte);
                }
            }

            (destination, value, data_vec)
        };

        // Execute external call
        let call_result = unsafe {
            call(
                Call::new_in(self)
                    .value(value),
                destination,
                &data,
            )
        };

        let executor = msg::sender();

        match call_result {
            Ok(_) => {
                evm::log(TransactionExecuted {
                    txId: tx_id,
                    executor,
                });
                Ok(())
            }
            Err(_) => {
                // Mark as not executed if call failed
                let tx = self.get_transaction_mut(tx_id)?;
                tx.executed.set(false);

                evm::log(TransactionFailed {
                    txId: tx_id,
                    executor,
                });

                Err(b"Transaction execution failed".to_vec())
            }
        }
    }

    /// Get transaction details
    pub fn get_transaction(
        &self,
        tx_id: U256,
    ) -> Result<(Address, U256, Vec<u8>, bool, U256), Vec<u8>> {
        let tx = self.get_transaction_ref(tx_id)?;

        let destination = tx.destination.get();
        let value = tx.value.get();
        let executed = tx.executed.get();
        let confirmation_count = tx.confirmation_count.get();

        // Read data from storage
        let mut data = Vec::new();
        for i in 0..tx.data.len() {
            if let Some(byte) = tx.data.get(i) {
                data.push(byte);
            }
        }

        Ok((destination, value, data, executed, confirmation_count))
    }

    /// Get total number of transactions
    pub fn get_transaction_count(&self) -> Result<U256, Vec<u8>> {
        Ok(U256::from(self.transactions.len()))
    }

    /// Get number of confirmations for a transaction
    pub fn get_confirmation_count(&self, tx_id: U256) -> Result<U256, Vec<u8>> {
        let tx = self.get_transaction_ref(tx_id)?;
        Ok(tx.confirmation_count.get())
    }

    /// Check if transaction is confirmed by owner
    pub fn is_confirmed_by(&self, tx_id: U256, owner: Address) -> Result<bool, Vec<u8>> {
        let tx = self.get_transaction_ref(tx_id)?;
        Ok(tx.confirmations.get(owner))
    }

    /// Get list of all owners
    pub fn get_owners(&self) -> Result<Vec<Address>, Vec<u8>> {
        let mut owners = Vec::new();
        for i in 0..self.owners.len() {
            if let Some(owner) = self.owners.get(i) {
                owners.push(owner);
            }
        }
        Ok(owners)
    }

    /// Check if address is an owner
    pub fn is_owner(&self, address: Address) -> Result<bool, Vec<u8>> {
        Ok(self.is_owner.get(address))
    }

    /// Get required confirmations
    pub fn get_required_confirmations(&self) -> Result<U256, Vec<u8>> {
        Ok(self.required_confirmations.get())
    }

    /// Add a new owner (requires multisig confirmation via transaction)
    pub fn add_owner(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        self.require_wallet()?;

        if owner == Address::ZERO {
            return Err(b"Invalid owner address".to_vec());
        }

        if self.is_owner.get(owner) {
            return Err(b"Owner already exists".to_vec());
        }

        self.owners.push(owner);
        self.is_owner.setter(owner).set(true);

        evm::log(OwnerAdded { owner });

        Ok(())
    }

    /// Remove an owner (requires multisig confirmation via transaction)
    pub fn remove_owner(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        self.require_wallet()?;

        if !self.is_owner.get(owner) {
            return Err(b"Not an owner".to_vec());
        }

        let owner_count = self.owners.len();
        if U256::from(owner_count) <= self.required_confirmations.get() {
            return Err(b"Cannot remove owner: would break requirement".to_vec());
        }

        // Find and remove owner
        for i in 0..owner_count {
            if let Some(current_owner) = self.owners.get(i) {
                if current_owner == owner {
                    // Swap with last element and pop
                    if i < owner_count - 1 {
                        if let Some(last_owner) = self.owners.get(owner_count - 1) {
                            self.owners.setter(i).unwrap().set(last_owner);
                        }
                    }
                    self.owners.pop();
                    break;
                }
            }
        }

        self.is_owner.setter(owner).set(false);

        evm::log(OwnerRemoved { owner });

        Ok(())
    }

    /// Replace an owner (requires multisig confirmation via transaction)
    pub fn replace_owner(
        &mut self,
        old_owner: Address,
        new_owner: Address,
    ) -> Result<(), Vec<u8>> {
        self.require_wallet()?;

        if new_owner == Address::ZERO {
            return Err(b"Invalid new owner address".to_vec());
        }

        if !self.is_owner.get(old_owner) {
            return Err(b"Old owner does not exist".to_vec());
        }

        if self.is_owner.get(new_owner) {
            return Err(b"New owner already exists".to_vec());
        }

        // Find and replace owner
        for i in 0..self.owners.len() {
            if let Some(owner) = self.owners.get(i) {
                if owner == old_owner {
                    self.owners.setter(i).unwrap().set(new_owner);
                    break;
                }
            }
        }

        self.is_owner.setter(old_owner).set(false);
        self.is_owner.setter(new_owner).set(true);

        evm::log(OwnerRemoved { owner: old_owner });
        evm::log(OwnerAdded { owner: new_owner });

        Ok(())
    }

    /// Change the required confirmations (requires multisig confirmation via transaction)
    pub fn change_requirement(&mut self, required: U256) -> Result<(), Vec<u8>> {
        self.require_wallet()?;

        let owner_count = U256::from(self.owners.len());
        if required.is_zero() || required > owner_count {
            return Err(b"Invalid required confirmations".to_vec());
        }

        self.required_confirmations.set(required);

        evm::log(RequirementChanged { required });

        Ok(())
    }

    /// Receive ETH deposits
    #[payable]
    pub fn deposit(&self) -> Result<(), Vec<u8>> {
        evm::log(Deposit {
            sender: msg::sender(),
            value: msg::value(),
        });
        Ok(())
    }

    /// Get wallet balance
    pub fn get_balance(&self) -> Result<U256, Vec<u8>> {
        Ok(self.balance())
    }
}

/// Internal helper methods
impl MultisigWallet {
    /// Require that caller is an owner
    fn require_owner(&self) -> Result<(), Vec<u8>> {
        if !self.is_owner.get(msg::sender()) {
            return Err(b"Not an owner".to_vec());
        }
        Ok(())
    }

    /// Require that caller is the wallet itself (for owner management)
    fn require_wallet(&self) -> Result<(), Vec<u8>> {
        if msg::sender() != self.address() {
            return Err(b"Only wallet can call this".to_vec());
        }
        Ok(())
    }

    /// Require that transaction exists and is not executed
    fn require_not_executed(&self, tx_id: U256) -> Result<(), Vec<u8>> {
        let tx = self.get_transaction_ref(tx_id)?;
        if tx.executed.get() {
            return Err(b"Transaction already executed".to_vec());
        }
        Ok(())
    }

    /// Require that transaction is confirmed
    fn require_confirmed(&self, tx_id: U256) -> Result<(), Vec<u8>> {
        let tx = self.get_transaction_ref(tx_id)?;
        let confirmation_count = tx.confirmation_count.get();
        let required = self.required_confirmations.get();

        if confirmation_count < required {
            return Err(b"Transaction not confirmed".to_vec());
        }
        Ok(())
    }

    /// Internal method to confirm a transaction
    fn _confirm_transaction(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        self.require_not_executed(tx_id)?;

        let sender = msg::sender();

        // Check if already confirmed
        let tx = self.get_transaction_mut(tx_id)?;
        if tx.confirmations.get(sender) {
            return Err(b"Transaction already confirmed".to_vec());
        }

        // Add confirmation
        tx.confirmations.setter(sender).set(true);
        let current_count = tx.confirmation_count.get();
        tx.confirmation_count.set(current_count + U256::from(1));

        evm::log(TransactionConfirmed {
            txId: tx_id,
            owner: sender,
        });

        Ok(())
    }

    /// Get transaction reference (immutable)
    fn get_transaction_ref(&self, tx_id: U256) -> Result<&Transaction, Vec<u8>> {
        let index = tx_id.to::<usize>();
        if index >= self.transactions.len() {
            return Err(b"Transaction does not exist".to_vec());
        }
        Ok(self.transactions.getter(index).unwrap())
    }

    /// Get transaction reference (mutable)
    fn get_transaction_mut(&mut self, tx_id: U256) -> Result<&mut Transaction, Vec<u8>> {
        let index = tx_id.to::<usize>();
        if index >= self.transactions.len() {
            return Err(b"Transaction does not exist".to_vec());
        }
        Ok(self.transactions.setter(index).unwrap())
    }

    /// Get contract balance
    fn balance(&self) -> U256 {
        U256::from(self.address().balance())
    }

    /// Get contract address
    fn address(&self) -> Address {
        Address::from(evm::contract_address().0)
    }
}
