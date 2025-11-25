# @stylus-deploy/sdk

> TypeScript SDK for deploying Arbitrum Stylus contracts

## Installation

```bash
npm install @stylus-deploy/sdk ethers
# or
pnpm add @stylus-deploy/sdk ethers
```

## Quick Start

### Basic Usage

```typescript
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';
import { createPublicClient, createWalletClient, http } from 'viem';
import { arbitrumSepolia } from 'viem/chains';

// Create clients
const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http(),
});

const walletClient = createWalletClient({
  chain: arbitrumSepolia,
  transport: http(),
});

// Create deployer
const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia',
});

// Deploy a token from template
const result = await deployer.deployFromProxy(TEMPLATE_IDS.ERC20, {
  name: 'My Token',
  symbol: 'MTK',
  decimals: 18,
  totalSupply: '1000000',
});

console.log(`Token deployed at: ${result.address}`);
```

### React Hooks

```tsx
import { useDeployFromTemplate } from '@stylus-deploy/sdk/react';
import { TEMPLATE_IDS } from '@stylus-deploy/sdk';

function DeployToken() {
  const { deploy, isLoading, result } = useDeployFromTemplate();

  const handleDeploy = async () => {
    await deploy({
      templateId: TEMPLATE_IDS.ERC20,
      initParams: {
        name: 'My Token',
        symbol: 'MTK',
        decimals: 18,
        totalSupply: '1000000',
      },
    });
  };

  return (
    <div>
      <button onClick={handleDeploy} disabled={isLoading}>
        {isLoading ? 'Deploying...' : 'Deploy Token'}
      </button>
      {result && <p>Deployed at: {result.address}</p>}
    </div>
  );
}
```

## API Reference

### UniversalDeployer

Main class for deploying Stylus contracts.

#### Methods

- `deployFromProxy(templateId, initParams, options?)` - Deploy from proxy (fast, cheap)
- `deployFromTemplate(template, initParams, options?)` - Deploy from template (flexible)
- `deployFromBytecode(wasmBytecode, initData?, options?)` - Deploy custom WASM
- `predictAddress(bytecode, salt)` - Predict deployment address
- `estimateGas(config)` - Estimate gas cost
- `getDeploymentInfo(address)` - Get deployment information
- `getUserDeployments(address)` - Get user's deployments

### TemplateManager

Manage and discover templates.

#### Methods

- `loadTemplate(templateId)` - Load template by ID
- `loadFromIPFS(cid)` - Load template from IPFS
- `listTemplates(category?)` - List all templates
- `searchTemplates(query)` - Search templates

## Template IDs

Pre-defined template IDs:

```typescript
import { TEMPLATE_IDS } from '@stylus-deploy/sdk';

TEMPLATE_IDS.ERC20; // ERC-20 token
TEMPLATE_IDS.ERC721; // ERC-721 NFT
TEMPLATE_IDS.ERC1155; // ERC-1155 multi-token
TEMPLATE_IDS.MULTISIG; // Multi-sig wallet
TEMPLATE_IDS.DAO; // DAO governance
```

## Networks

Supported networks:

- `arbitrum-one` - Arbitrum mainnet
- `arbitrum-sepolia` - Arbitrum Sepolia testnet
- `arbitrum-orbit` - Custom Arbitrum Orbit chains

## Development

```bash
# Install dependencies
pnpm install

# Build
pnpm build

# Test
pnpm test

# Type check
pnpm typecheck
```

## License

MIT
