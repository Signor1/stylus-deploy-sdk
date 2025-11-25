// Copyright (c) 2025 Signordev
// SPDX-License-Identifier: MIT

/**
 * Stylus Deploy SDK
 *
 * Universal deployment system for Arbitrum Stylus contracts
 */

// Core exports
export { UniversalDeployer } from './core/deployer';
export { TemplateManager } from './core/template-manager';

// Type exports
export type {
  DeploymentMethod,
  TemplateCategory,
  Network,
  Template,
  DeploymentConfig,
  DeploymentResult,
  DeploymentInfo,
  NetworkConfig,
  StylusDeployConfig,
  ValidationError,
  ValidationResult,
} from './types';

// Constants
export {
  TEMPLATE_IDS,
  ARB_WASM_ADDRESS,
  DEFAULT_GAS_LIMITS,
  NETWORKS,
  ERROR_CODES,
  MAX_WASM_SIZE,
} from './constants';

// Utils
export * from './utils';
