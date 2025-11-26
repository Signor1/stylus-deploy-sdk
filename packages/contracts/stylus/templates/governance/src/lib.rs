// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

//! DAO Governance Template for Stylus
//!
//! This template provides a comprehensive DAO governance system with:
//! - Proposal creation and management
//! - Voting mechanisms (for/against/abstain)
//! - Timelock for executed proposals
//! - Quorum requirements
//! - Token-based or membership-based voting power

#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    alloy_sol_types::sol,
    evm, msg,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

// Solidity ABI for events
sol! {
    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        string description,
        uint256 startBlock,
        uint256 endBlock
    );

    event VoteCast(
        address indexed voter,
        uint256 indexed proposalId,
        uint8 support,
        uint256 weight,
        string reason
    );

    event ProposalExecuted(uint256 indexed proposalId, address indexed executor);

    event ProposalCancelled(uint256 indexed proposalId, address indexed canceller);

    event ProposalQueued(
        uint256 indexed proposalId,
        uint256 eta
    );

    event VotingPeriodUpdated(uint256 oldPeriod, uint256 newPeriod);

    event TimelockPeriodUpdated(uint256 oldPeriod, uint256 newPeriod);

    event QuorumUpdated(uint256 oldQuorum, uint256 newQuorum);

    event MemberAdded(address indexed member, uint256 votingPower);

    event MemberRemoved(address indexed member);

    event VotingPowerUpdated(address indexed member, uint256 oldPower, uint256 newPower);
}

// Proposal state enum
#[derive(Copy, Clone, PartialEq)]
pub enum ProposalState {
    Pending = 0,
    Active = 1,
    Succeeded = 2,
    Defeated = 3,
    Queued = 4,
    Executed = 5,
    Cancelled = 6,
}

// Vote type enum
#[derive(Copy, Clone, PartialEq)]
pub enum VoteType {
    Against = 0,
    For = 1,
    Abstain = 2,
}

// Action structure for proposal execution
sol_storage! {
    pub struct ProposalAction {
        address target;
        uint256 value;
        StorageVec<u8> calldata;
    }
}

// Vote record structure
sol_storage! {
    pub struct VoteRecord {
        bool has_voted;
        uint8 vote_type; // 0 = Against, 1 = For, 2 = Abstain
        uint256 weight;
    }
}

// Proposal structure
sol_storage! {
    pub struct Proposal {
        uint256 id;
        address proposer;
        StorageString description;

        // Actions to execute
        StorageVec<ProposalAction> actions;

        // Voting
        uint256 start_block;
        uint256 end_block;
        uint256 for_votes;
        uint256 against_votes;
        uint256 abstain_votes;
        StorageMap<Address, VoteRecord> votes;

        // Execution
        uint256 eta; // Estimated time of execution (after timelock)
        bool executed;
        bool cancelled;

        // Metadata
        uint256 created_at;
    }
}

// Storage layout for the governance contract
sol_storage! {
    #[entrypoint]
    pub struct DAOGovernance {
        // Governance parameters
        StorageString name;
        StorageU256 voting_period; // in blocks
        StorageU256 timelock_period; // in seconds
        StorageU256 quorum; // minimum votes required (percentage * 100, e.g., 4000 = 40%)
        StorageU256 proposal_threshold; // minimum voting power to create proposal

        // Proposals
        StorageVec<Proposal> proposals;
        StorageU256 proposal_count;

        // Members and voting power
        StorageMap<Address, U256> voting_power;
        StorageMap<Address, bool> is_member;
        StorageVec<Address> members;
        StorageU256 total_voting_power;

        // Governance controls
        StorageBool initialized;
        StorageAddress admin;
        StorageBool paused;
    }
}

#[external]
impl DAOGovernance {
    /// Initialize the governance contract
    pub fn initialize(
        &mut self,
        name: String,
        voting_period: U256,
        timelock_period: U256,
        quorum: U256,
        proposal_threshold: U256,
        admin: Address,
    ) -> Result<(), Vec<u8>> {
        // Ensure not already initialized
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }

        // Validate parameters
        if voting_period.is_zero() {
            return Err(b"Invalid voting period".to_vec());
        }

        if quorum > U256::from(10000) {
            return Err(b"Quorum too high (max 100%)".to_vec());
        }

        if admin == Address::ZERO {
            return Err(b"Invalid admin address".to_vec());
        }

        // Set parameters
        self.name.set_str(&name);
        self.voting_period.set(voting_period);
        self.timelock_period.set(timelock_period);
        self.quorum.set(quorum);
        self.proposal_threshold.set(proposal_threshold);
        self.admin.set(admin);
        self.paused.set(false);
        self.proposal_count.set(U256::ZERO);
        self.total_voting_power.set(U256::ZERO);

        // Mark as initialized
        self.initialized.set(true);

        Ok(())
    }

    /// Get governance name
    pub fn get_name(&self) -> Result<String, Vec<u8>> {
        Ok(self.name.get_string())
    }

    /// Get voting period (in blocks)
    pub fn get_voting_period(&self) -> Result<U256, Vec<u8>> {
        Ok(self.voting_period.get())
    }

    /// Get timelock period (in seconds)
    pub fn get_timelock_period(&self) -> Result<U256, Vec<u8>> {
        Ok(self.timelock_period.get())
    }

    /// Get quorum threshold (percentage * 100)
    pub fn get_quorum(&self) -> Result<U256, Vec<u8>> {
        Ok(self.quorum.get())
    }

    /// Get proposal threshold
    pub fn get_proposal_threshold(&self) -> Result<U256, Vec<u8>> {
        Ok(self.proposal_threshold.get())
    }

    /// Get total proposal count
    pub fn get_proposal_count(&self) -> Result<U256, Vec<u8>> {
        Ok(self.proposal_count.get())
    }

    /// Get total voting power
    pub fn get_total_voting_power(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_voting_power.get())
    }

    /// Get voting power of an address
    pub fn get_voting_power(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.voting_power.get(account))
    }

    /// Check if address is a member
    pub fn is_member(&self, account: Address) -> Result<bool, Vec<u8>> {
        Ok(self.is_member.get(account))
    }

    /// Get all members
    pub fn get_members(&self) -> Result<Vec<Address>, Vec<u8>> {
        let mut result = Vec::new();
        for i in 0..self.members.len() {
            if let Some(member) = self.members.get(i) {
                result.push(member);
            }
        }
        Ok(result)
    }

    /// Get member count
    pub fn get_member_count(&self) -> Result<U256, Vec<u8>> {
        Ok(U256::from(self.members.len()))
    }

    /// Add member with voting power (admin only)
    pub fn add_member(&mut self, member: Address, power: U256) -> Result<(), Vec<u8>> {
        self.require_admin()?;
        self.require_not_paused()?;

        if member == Address::ZERO {
            return Err(b"Invalid member address".to_vec());
        }

        if power.is_zero() {
            return Err(b"Voting power must be positive".to_vec());
        }

        if self.is_member.get(member) {
            return Err(b"Already a member".to_vec());
        }

        // Add member
        self.members.push(member);
        self.is_member.setter(member).set(true);
        self.voting_power.setter(member).set(power);

        // Update total voting power
        let total = self.total_voting_power.get();
        self.total_voting_power.set(total + power);

        evm::log(MemberAdded {
            member,
            votingPower: power,
        });

        Ok(())
    }

    /// Remove member (admin only)
    pub fn remove_member(&mut self, member: Address) -> Result<(), Vec<u8>> {
        self.require_admin()?;
        self.require_not_paused()?;

        if !self.is_member.get(member) {
            return Err(b"Not a member".to_vec());
        }

        // Get member's voting power
        let power = self.voting_power.get(member);

        // Remove member
        let member_count = self.members.len();
        for i in 0..member_count {
            if let Some(current_member) = self.members.get(i) {
                if current_member == member {
                    // Swap with last element and pop
                    if i < member_count - 1 {
                        if let Some(last_member) = self.members.get(member_count - 1) {
                            self.members.setter(i).unwrap().set(last_member);
                        }
                    }
                    self.members.pop();
                    break;
                }
            }
        }

        self.is_member.setter(member).set(false);
        self.voting_power.setter(member).set(U256::ZERO);

        // Update total voting power
        let total = self.total_voting_power.get();
        self.total_voting_power.set(total - power);

        evm::log(MemberRemoved { member });

        Ok(())
    }

    /// Update member voting power (admin only)
    pub fn update_voting_power(
        &mut self,
        member: Address,
        new_power: U256,
    ) -> Result<(), Vec<u8>> {
        self.require_admin()?;
        self.require_not_paused()?;

        if !self.is_member.get(member) {
            return Err(b"Not a member".to_vec());
        }

        if new_power.is_zero() {
            return Err(b"Voting power must be positive".to_vec());
        }

        let old_power = self.voting_power.get(member);

        // Update voting power
        self.voting_power.setter(member).set(new_power);

        // Update total voting power
        let total = self.total_voting_power.get();
        if new_power > old_power {
            self.total_voting_power.set(total + (new_power - old_power));
        } else {
            self.total_voting_power.set(total - (old_power - new_power));
        }

        evm::log(VotingPowerUpdated {
            member,
            oldPower: old_power,
            newPower: new_power,
        });

        Ok(())
    }

    /// Update voting period (admin only)
    pub fn set_voting_period(&mut self, new_period: U256) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        if new_period.is_zero() {
            return Err(b"Invalid voting period".to_vec());
        }

        let old_period = self.voting_period.get();
        self.voting_period.set(new_period);

        evm::log(VotingPeriodUpdated {
            oldPeriod: old_period,
            newPeriod: new_period,
        });

        Ok(())
    }

    /// Update timelock period (admin only)
    pub fn set_timelock_period(&mut self, new_period: U256) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        let old_period = self.timelock_period.get();
        self.timelock_period.set(new_period);

        evm::log(TimelockPeriodUpdated {
            oldPeriod: old_period,
            newPeriod: new_period,
        });

        Ok(())
    }

    /// Update quorum (admin only)
    pub fn set_quorum(&mut self, new_quorum: U256) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        if new_quorum > U256::from(10000) {
            return Err(b"Quorum too high (max 100%)".to_vec());
        }

        let old_quorum = self.quorum.get();
        self.quorum.set(new_quorum);

        evm::log(QuorumUpdated {
            oldQuorum: old_quorum,
            newQuorum: new_quorum,
        });

        Ok(())
    }

    /// Get admin address
    pub fn get_admin(&self) -> Result<Address, Vec<u8>> {
        Ok(self.admin.get())
    }

    /// Check if contract is paused
    pub fn is_paused(&self) -> Result<bool, Vec<u8>> {
        Ok(self.paused.get())
    }

    /// Pause the contract (admin only)
    pub fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        if self.paused.get() {
            return Ok(());
        }

        self.paused.set(true);
        Ok(())
    }

    /// Unpause the contract (admin only)
    pub fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        if !self.paused.get() {
            return Ok(());
        }

        self.paused.set(false);
        Ok(())
    }

    /// Transfer admin rights (admin only)
    pub fn transfer_admin(&mut self, new_admin: Address) -> Result<(), Vec<u8>> {
        self.require_admin()?;

        if new_admin == Address::ZERO {
            return Err(b"Invalid admin address".to_vec());
        }

        self.admin.set(new_admin);
        Ok(())
    }
}

/// Internal helper methods
impl DAOGovernance {
    /// Require that caller is admin
    fn require_admin(&self) -> Result<(), Vec<u8>> {
        if msg::sender() != self.admin.get() {
            return Err(b"Caller is not admin".to_vec());
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

    /// Require that caller is a member
    fn require_member(&self) -> Result<(), Vec<u8>> {
        if !self.is_member.get(msg::sender()) {
            return Err(b"Caller is not a member".to_vec());
        }
        Ok(())
    }

    /// Get current block number
    fn current_block(&self) -> U256 {
        U256::from(evm::block_number())
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> U256 {
        U256::from(evm::block_timestamp())
    }
}
