// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

/**
 * Utility functions for Stylus Deploy SDK
 */

import type { Address } from 'viem';
import { isAddress as isViemAddress } from 'viem';

/**
 * Validate Ethereum address
 */
export function isValidAddress(address: string): address is Address {
  return isViemAddress(address);
}

/**
 * Generate random salt for CREATE2
 */
export function generateSalt(): string {
  const randomBytes = new Uint8Array(32);
  crypto.getRandomValues(randomBytes);
  return (
    '0x' +
    Array.from(randomBytes)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('')
  );
}

/**
 * Format gas amount to readable string
 */
export function formatGas(gas: bigint): string {
  return gas.toLocaleString();
}

/**
 * Format address for display (0x1234...5678)
 */
export function formatAddress(address: Address, chars: number = 4): string {
  return `${address.slice(0, chars + 2)}...${address.slice(-chars)}`;
}

/**
 * Sleep utility
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry utility
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxAttempts: number = 3,
  delayMs: number = 1000
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      if (attempt < maxAttempts) {
        await sleep(delayMs * attempt);
      }
    }
  }

  throw lastError;
}
