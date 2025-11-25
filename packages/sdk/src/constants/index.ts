// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

/**
 * Constants for Stylus Deploy SDK
 */

import type { Address } from 'viem';
import type { NetworkConfig } from '../types';

/**
 * Known template IDs
 */
export const TEMPLATE_IDS = {
  ERC20: '1',
  ERC721: '2',
  ERC1155: '3',
  MULTISIG: '4',
  DAO: '5',
} as const;

/**
 * ArbWasm precompile address
 */
export const ARB_WASM_ADDRESS: Address =
  '0x0000000000000000000000000000000000000071';

/**
 * Default gas limits for different operations
 */
export const DEFAULT_GAS_LIMITS = {
  PROXY_DEPLOY: 150_000n,
  TEMPLATE_DEPLOY: 500_000n,
  WASM_ACTIVATION: 50_000_000n,
  INITIALIZATION: 100_000n,
} as const;

/**
 * Network configurations
 */
export const NETWORKS: Record<string, NetworkConfig> = {
  'arbitrum-one': {
    chainId: 42161,
    name: 'Arbitrum One',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
    explorerUrl: 'https://arbiscan.io',
    deployerAddress: '0x0000000000000000000000000000000000000000', // TODO: Update after deployment
    registryAddress: '0x0000000000000000000000000000000000000000', // TODO: Update after deployment
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18,
    },
  },
  'arbitrum-sepolia': {
    chainId: 421614,
    name: 'Arbitrum Sepolia',
    rpcUrl: 'https://sepolia-rollup.arbitrum.io/rpc',
    explorerUrl: 'https://sepolia.arbiscan.io',
    deployerAddress: '0x0000000000000000000000000000000000000000', // TODO: Update after deployment
    registryAddress: '0x0000000000000000000000000000000000000000', // TODO: Update after deployment
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18,
    },
  },
} as const;

/**
 * Error codes
 */
export const ERROR_CODES = {
  INVALID_CONFIG: 'INVALID_CONFIG',
  INVALID_TEMPLATE: 'INVALID_TEMPLATE',
  DEPLOYMENT_FAILED: 'DEPLOYMENT_FAILED',
  ACTIVATION_FAILED: 'ACTIVATION_FAILED',
  INITIALIZATION_FAILED: 'INITIALIZATION_FAILED',
  NETWORK_ERROR: 'NETWORK_ERROR',
  INSUFFICIENT_GAS: 'INSUFFICIENT_GAS',
  WASM_TOO_LARGE: 'WASM_TOO_LARGE',
} as const;

/**
 * Maximum WASM size (24KB for Ethereum)
 */
export const MAX_WASM_SIZE = 24_576; // 24 * 1024 bytes
