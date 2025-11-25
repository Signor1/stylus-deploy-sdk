// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

import { describe, it, expect } from 'vitest';
import {
  isValidAddress,
  generateSalt,
  formatGas,
  formatAddress,
} from '../utils';

describe('Utils', () => {
  describe('isValidAddress', () => {
    it('should validate valid addresses', () => {
      expect(isValidAddress('0x0000000000000000000000000000000000000000')).toBe(
        true
      );
      expect(isValidAddress('0x1234567890123456789012345678901234567890')).toBe(
        true
      );
    });

    it('should reject invalid addresses', () => {
      expect(isValidAddress('invalid')).toBe(false);
      expect(isValidAddress('0x123')).toBe(false);
      expect(isValidAddress('')).toBe(false);
    });
  });

  describe('generateSalt', () => {
    it('should generate valid salt', () => {
      const salt = generateSalt();
      expect(salt).toMatch(/^0x[0-9a-f]{64}$/);
    });

    it('should generate unique salts', () => {
      const salt1 = generateSalt();
      const salt2 = generateSalt();
      expect(salt1).not.toBe(salt2);
    });
  });

  describe('formatGas', () => {
    it('should format gas amounts', () => {
      expect(formatGas(150000n)).toBe('150,000');
      expect(formatGas(1000000n)).toBe('1,000,000');
    });
  });

  describe('formatAddress', () => {
    it('should format addresses', () => {
      const address = '0x1234567890123456789012345678901234567890' as const;
      expect(formatAddress(address)).toBe('0x1234...7890');
      expect(formatAddress(address, 6)).toBe('0x123456...567890');
    });
  });
});
