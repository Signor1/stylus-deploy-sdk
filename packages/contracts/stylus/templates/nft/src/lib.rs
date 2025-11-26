// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

//! ERC-721 NFT Template for Stylus
//!
//! This template provides a comprehensive ERC-721 NFT implementation
//! with minting, burning, enumeration, and metadata URI support.

#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    alloy_sol_types::sol,
    evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

// ERC-721 events
sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
}

// Storage layout for the ERC-721 NFT
sol_storage! {
    #[entrypoint]
    pub struct Erc721Nft {
        // NFT metadata
        StorageString name;
        StorageString symbol;
        StorageString base_uri;

        // Token ownership and approvals
        StorageMap<U256, Address> owners;
        StorageMap<U256, Address> token_approvals;
        StorageMap<Address, StorageMap<Address, bool>> operator_approvals;

        // Balance tracking
        StorageMap<Address, U256> balances;

        // Token enumeration
        StorageVec<U256> all_tokens;
        StorageMap<U256, U256> all_tokens_index;
        StorageMap<Address, StorageVec<U256>> owned_tokens;
        StorageMap<U256, U256> owned_tokens_index;

        // Token metadata URIs (optional override)
        StorageMap<U256, StorageString> token_uris;

        // Minting control
        StorageU256 next_token_id;
        StorageU256 max_supply;

        // Initialization and control
        StorageBool initialized;
        StorageAddress owner;
        StorageBool paused;
    }
}

#[external]
impl Erc721Nft {
    /// Initialize the NFT collection
    pub fn initialize(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
        max_supply: U256,
        owner: Address,
    ) -> Result<(), Vec<u8>> {
        // Ensure not already initialized
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }

        // Set metadata
        self.name.set_str(&name);
        self.symbol.set_str(&symbol);
        self.base_uri.set_str(&base_uri);

        // Set minting parameters
        self.next_token_id.set(U256::from(1)); // Start from token ID 1
        self.max_supply.set(max_supply);

        // Set owner
        self.owner.set(owner);
        self.paused.set(false);

        // Mark as initialized
        self.initialized.set(true);

        Ok(())
    }

    /// Get the NFT collection name
    pub fn name(&self) -> Result<String, Vec<u8>> {
        Ok(self.name.get_string())
    }

    /// Get the NFT collection symbol
    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        Ok(self.symbol.get_string())
    }

    /// Get the base URI for token metadata
    pub fn base_uri(&self) -> Result<String, Vec<u8>> {
        Ok(self.base_uri.get_string())
    }

    /// Get total supply of minted tokens
    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(U256::from(self.all_tokens.len()))
    }

    /// Get maximum supply
    pub fn max_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.max_supply.get())
    }

    /// Get balance of an account
    pub fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>> {
        if owner == Address::ZERO {
            return Err(b"Query for zero address".to_vec());
        }
        Ok(self.balances.get(owner))
    }

    /// Get owner of a token
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Vec<u8>> {
        let owner = self.owners.get(token_id);
        if owner == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }
        Ok(owner)
    }

    /// Get token URI
    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        // Check if token exists
        if self.owners.get(token_id) == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }

        // Check for token-specific URI override
        let token_uri = self.token_uris.get(token_id);
        if !token_uri.is_empty() {
            return Ok(token_uri.get_string());
        }

        // Return base URI + token ID
        let base = self.base_uri.get_string();
        let token_id_str = token_id.to_string();
        Ok(format!("{}{}", base, token_id_str))
    }

    /// Transfer NFT from caller to recipient
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        // Check if caller is authorized
        if !self.is_approved_or_owner(msg::sender(), token_id)? {
            return Err(b"Not authorized".to_vec());
        }

        self._transfer(from, to, token_id)?;
        Ok(())
    }

    /// Safely transfer NFT (same as transferFrom for Stylus)
    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.transfer_from(from, to, token_id)
    }

    /// Approve an address to manage a specific token
    pub fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        let owner = self.owner_of(token_id)?;

        if to == owner {
            return Err(b"Approve to current owner".to_vec());
        }

        let caller = msg::sender();
        if caller != owner && !self.is_approved_for_all(owner, caller)? {
            return Err(b"Not authorized".to_vec());
        }

        self._approve(to, token_id)?;
        Ok(())
    }

    /// Get approved address for a token
    pub fn get_approved(&self, token_id: U256) -> Result<Address, Vec<u8>> {
        if self.owners.get(token_id) == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }
        Ok(self.token_approvals.get(token_id))
    }

    /// Set approval for all tokens
    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        let caller = msg::sender();
        if operator == caller {
            return Err(b"Approve to caller".to_vec());
        }

        self.operator_approvals
            .setter(caller)
            .setter(operator)
            .set(approved);

        evm::log(ApprovalForAll {
            owner: caller,
            operator,
            approved,
        });

        Ok(())
    }

    /// Check if operator is approved for all tokens of owner
    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> Result<bool, Vec<u8>> {
        Ok(self.operator_approvals.getter(owner).get(operator))
    }

    /// Mint a new NFT (owner only)
    pub fn mint(&mut self, to: Address) -> Result<U256, Vec<u8>> {
        self.require_owner()?;
        self.require_not_paused()?;

        if to == Address::ZERO {
            return Err(b"Mint to zero address".to_vec());
        }

        // Check max supply
        let current_supply = U256::from(self.all_tokens.len());
        let max_supply = self.max_supply.get();
        if max_supply > U256::ZERO && current_supply >= max_supply {
            return Err(b"Max supply reached".to_vec());
        }

        // Get next token ID
        let token_id = self.next_token_id.get();
        self.next_token_id.set(token_id + U256::from(1));

        self._mint(to, token_id)?;
        Ok(token_id)
    }

    /// Mint multiple NFTs (owner only)
    pub fn mint_batch(&mut self, to: Address, quantity: U256) -> Result<Vec<U256>, Vec<u8>> {
        self.require_owner()?;
        self.require_not_paused()?;

        if to == Address::ZERO {
            return Err(b"Mint to zero address".to_vec());
        }

        let mut token_ids = Vec::new();
        let quantity_u32 = quantity.to::<u32>();

        for _ in 0..quantity_u32 {
            let token_id = self.mint(to)?;
            token_ids.push(token_id);
        }

        Ok(token_ids)
    }

    /// Burn an NFT
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.require_not_paused()?;

        // Check if caller is authorized
        if !self.is_approved_or_owner(msg::sender(), token_id)? {
            return Err(b"Not authorized".to_vec());
        }

        self._burn(token_id)?;
        Ok(())
    }

    /// Set token URI (owner only)
    pub fn set_token_uri(&mut self, token_id: U256, uri: String) -> Result<(), Vec<u8>> {
        self.require_owner()?;

        if self.owners.get(token_id) == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }

        self.token_uris.setter(token_id).set_str(&uri);
        Ok(())
    }

    /// Set base URI (owner only)
    pub fn set_base_uri(&mut self, base_uri: String) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        self.base_uri.set_str(&base_uri);
        Ok(())
    }

    /// Get token by index
    pub fn token_by_index(&self, index: U256) -> Result<U256, Vec<u8>> {
        let index_u32 = index.to::<u32>();
        if index_u32 >= self.all_tokens.len() {
            return Err(b"Index out of bounds".to_vec());
        }
        Ok(self.all_tokens.get(index_u32).unwrap())
    }

    /// Get token of owner by index
    pub fn token_of_owner_by_index(&self, owner: Address, index: U256) -> Result<U256, Vec<u8>> {
        let owned_tokens = self.owned_tokens.getter(owner);
        let index_u32 = index.to::<u32>();

        if index_u32 >= owned_tokens.len() {
            return Err(b"Index out of bounds".to_vec());
        }

        Ok(owned_tokens.get(index_u32).unwrap())
    }

    /// Pause the contract (owner only)
    pub fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        if self.paused.get() {
            return Ok(());
        }
        self.paused.set(true);
        Ok(())
    }

    /// Unpause the contract (owner only)
    pub fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.require_owner()?;
        if !self.paused.get() {
            return Ok(());
        }
        self.paused.set(false);
        Ok(())
    }

    /// Get owner address
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
impl Erc721Nft {
    /// Internal transfer function
    fn _transfer(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        // Verify ownership
        let owner = self.owners.get(token_id);
        if owner != from {
            return Err(b"Transfer from incorrect owner".to_vec());
        }

        if to == Address::ZERO {
            return Err(b"Transfer to zero address".to_vec());
        }

        // Clear approvals
        self._approve(Address::ZERO, token_id)?;

        // Update balances
        let from_balance = self.balances.get(from);
        self.balances.setter(from).set(from_balance - U256::from(1));

        let to_balance = self.balances.get(to);
        self.balances.setter(to).set(to_balance + U256::from(1));

        // Update ownership
        self.owners.setter(token_id).set(to);

        // Update enumeration
        self._remove_token_from_owner_enumeration(from, token_id)?;
        self._add_token_to_owner_enumeration(to, token_id);

        evm::log(Transfer { from, to, tokenId: token_id });

        Ok(())
    }

    /// Internal mint function
    fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        if to == Address::ZERO {
            return Err(b"Mint to zero address".to_vec());
        }

        if self.owners.get(token_id) != Address::ZERO {
            return Err(b"Token already minted".to_vec());
        }

        // Update balance
        let balance = self.balances.get(to);
        self.balances.setter(to).set(balance + U256::from(1));

        // Set owner
        self.owners.setter(token_id).set(to);

        // Add to enumeration
        self._add_token_to_all_tokens_enumeration(token_id);
        self._add_token_to_owner_enumeration(to, token_id);

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            tokenId: token_id,
        });

        Ok(())
    }

    /// Internal burn function
    fn _burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        let owner = self.owners.get(token_id);
        if owner == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }

        // Clear approvals
        self._approve(Address::ZERO, token_id)?;

        // Update balance
        let balance = self.balances.get(owner);
        self.balances.setter(owner).set(balance - U256::from(1));

        // Clear ownership
        self.owners.setter(token_id).set(Address::ZERO);

        // Clear token URI if exists
        if !self.token_uris.get(token_id).is_empty() {
            self.token_uris.setter(token_id).erase();
        }

        // Remove from enumeration
        self._remove_token_from_owner_enumeration(owner, token_id)?;
        self._remove_token_from_all_tokens_enumeration(token_id)?;

        evm::log(Transfer {
            from: owner,
            to: Address::ZERO,
            tokenId: token_id,
        });

        Ok(())
    }

    /// Internal approve function
    fn _approve(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.token_approvals.setter(token_id).set(to);

        let owner = self.owners.get(token_id);
        evm::log(Approval {
            owner,
            approved: to,
            tokenId: token_id,
        });

        Ok(())
    }

    /// Check if spender is owner or approved
    fn is_approved_or_owner(&self, spender: Address, token_id: U256) -> Result<bool, Vec<u8>> {
        let owner = self.owners.get(token_id);
        if owner == Address::ZERO {
            return Err(b"Token does not exist".to_vec());
        }

        Ok(spender == owner
            || self.token_approvals.get(token_id) == spender
            || self.operator_approvals.getter(owner).get(spender))
    }

    /// Add token to all tokens enumeration
    fn _add_token_to_all_tokens_enumeration(&mut self, token_id: U256) {
        let index = self.all_tokens.len();
        self.all_tokens.push(token_id);
        self.all_tokens_index.setter(token_id).set(U256::from(index));
    }

    /// Remove token from all tokens enumeration
    fn _remove_token_from_all_tokens_enumeration(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        let last_token_index = self.all_tokens.len() - 1;
        let token_index = self.all_tokens_index.get(token_id).to::<u32>();

        if token_index != last_token_index {
            let last_token_id = self.all_tokens.get(last_token_index).unwrap();
            self.all_tokens.setter(token_index).unwrap().set(last_token_id);
            self.all_tokens_index.setter(last_token_id).set(U256::from(token_index));
        }

        self.all_tokens.pop();
        self.all_tokens_index.setter(token_id).set(U256::ZERO);

        Ok(())
    }

    /// Add token to owner enumeration
    fn _add_token_to_owner_enumeration(&mut self, to: Address, token_id: U256) {
        let owned_tokens = self.owned_tokens.setter(to);
        let index = owned_tokens.len();
        owned_tokens.push(token_id);
        self.owned_tokens_index.setter(token_id).set(U256::from(index));
    }

    /// Remove token from owner enumeration
    fn _remove_token_from_owner_enumeration(&mut self, from: Address, token_id: U256) -> Result<(), Vec<u8>> {
        let owned_tokens = self.owned_tokens.setter(from);
        let last_token_index = owned_tokens.len() - 1;
        let token_index = self.owned_tokens_index.get(token_id).to::<u32>();

        if token_index != last_token_index {
            let last_token_id = owned_tokens.get(last_token_index).unwrap();
            owned_tokens.setter(token_index).unwrap().set(last_token_id);
            self.owned_tokens_index.setter(last_token_id).set(U256::from(token_index));
        }

        owned_tokens.pop();
        self.owned_tokens_index.setter(token_id).set(U256::ZERO);

        Ok(())
    }

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
