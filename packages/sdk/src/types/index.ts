// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

/**
 * Core type definitions for Stylus Deploy SDK
 */

import type { Address, Hash, TransactionReceipt } from 'viem';

/**
 * Supported deployment methods
 */
export type DeploymentMethod = 'proxy' | 'template' | 'hybrid' | 'direct';

/**
 * Template categories
 */
export type TemplateCategory =
  | 'token'
  | 'nft'
  | 'defi'
  | 'dao'
  | 'gaming'
  | 'identity'
  | 'storage'
  | 'oracle'
  | 'bridge'
  | 'custom';

/**
 * Supported networks
 */
export type Network = 'arbitrum-one' | 'arbitrum-sepolia' | 'arbitrum-orbit';

/**
 * Template metadata
 */
export interface Template {
  /** Unique template ID */
  id: string;
  /** Template name */
  name: string;
  /** Template version */
  version: string;
  /** Template category */
  category: TemplateCategory;
  /** Template description */
  description: string;
  /** Template author address */
  author: Address;
  /** IPFS CID for WASM bytecode */
  ipfsCID?: string;
  /** On-chain template address */
  onChainAddress?: Address;
  /** Number of deployments */
  deploymentCount: number;
  /** Whether proxy deployment is supported */
  supportsProxy: boolean;
  /** Whether template deployment is supported */
  supportsTemplate: boolean;
  /** Recommended deployment method */
  recommendedMethod: DeploymentMethod;
}

/**
 * Deployment configuration
 */
export interface DeploymentConfig {
  /** Deployment method */
  method: DeploymentMethod;
  /** Template ID (for proxy/template deployment) */
  templateId?: string;
  /** Initialization parameters */
  initParams?: Record<string, any>;
  /** Salt for CREATE2 deployment */
  salt?: string;
  /** Gas limit */
  gasLimit?: bigint;
  /** Maximum fee per gas (EIP-1559) */
  maxFeePerGas?: bigint;
  /** Maximum priority fee per gas (EIP-1559) */
  maxPriorityFeePerGas?: bigint;
}

/**
 * Deployment result
 */
export interface DeploymentResult {
  /** Whether deployment was successful */
  success: boolean;
  /** Deployed contract address */
  address: Address;
  /** Transaction hash */
  transactionHash: Hash;
  /** Deployment method used */
  method: DeploymentMethod;
  /** Gas used */
  gasUsed: bigint;
  /** WASM bytecode size (bytes) */
  wasmSize?: number;
  /** Block number */
  blockNumber: bigint;
  /** Timestamp */
  timestamp: number;
  /** Transaction receipt */
  receipt: TransactionReceipt;
  /** Error message (if failed) */
  error?: string;
}

/**
 * Deployment info (from registry)
 */
export interface DeploymentInfo {
  /** Deployed contract address */
  contractAddress: Address;
  /** Deployer address */
  creator: Address;
  /** Template ID (if deployed from template) */
  templateId?: string;
  /** WASM hash */
  wasmHash?: Hash;
  /** Deployment method */
  method: DeploymentMethod;
  /** Deployment timestamp */
  timestamp: number;
  /** Additional metadata */
  metadata?: string;
}

/**
 * Network configuration
 */
export interface NetworkConfig {
  /** Chain ID */
  chainId: number;
  /** Network name */
  name: string;
  /** RPC URL */
  rpcUrl: string;
  /** Block explorer URL */
  explorerUrl: string;
  /** Universal deployer contract address */
  deployerAddress: Address;
  /** Template registry contract address */
  registryAddress: Address;
  /** Native currency */
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
}

/**
 * SDK configuration
 */
export interface StylusDeployConfig {
  /** Network to deploy on */
  network: Network;
  /** Custom network configuration */
  customNetwork?: NetworkConfig;
  /** RPC URL override */
  rpcUrl?: string;
  /** Custom deployer address */
  deployerAddress?: Address;
  /** Custom registry address */
  registryAddress?: Address;
}

/**
 * Validation error
 */
export interface ValidationError {
  field: string;
  message: string;
  code?: string;
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}
