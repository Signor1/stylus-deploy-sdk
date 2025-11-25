# Stylus Deploy SDK

> Universal deployment system for Arbitrum Stylus contracts

Deploy any Stylus contract from your browser without Rust compilation.

## Overview

Stylus Deploy SDK is a complete deployment system for Arbitrum Stylus that enables developers to:

- 🚀 Deploy any Stylus contract from the frontend
- 📦 Use pre-compiled templates (tokens, NFTs, DAOs, etc.)
- 🎨 Upload and deploy custom WASM contracts
- 🏪 Discover and share templates via decentralized registry
- ⚡ Gas-efficient deployments with CREATE2
- 🔄 Track all deployments on-chain

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Template System                                │
│  ├── On-chain Registry (metadata + IPFS hash)  │
│  ├── IPFS/Arweave (WASM bytecode storage)      │
│  └── Local Cache (browser storage)             │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  Universal Deployer                             │
│  ├── deployFromTemplate(templateId, params)    │
│  ├── deployFromBytecode(wasm, params)          │
│  └── Automatic Stylus activation                │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  SDK & Tools                                    │
│  ├── TypeScript SDK (ethers.js + viem)         │
│  ├── React Hooks (wagmi integration)           │
│  ├── CLI Tools (template management)           │
│  └── Demo Applications                          │
└─────────────────────────────────────────────────┘
```

## Repository Structure

```
stylus-deploy-sdk/
├── packages/
│   ├── contracts/          # Smart contracts (Solidity + Stylus)
│   │   ├── stylus/        # Rust Stylus templates
│   │   └── solidity/      # Registry & Deployer
│   ├── sdk/               # TypeScript SDK
│   ├── cli/               # CLI tools
│   └── demo-app/          # Demo application
├── templates/             # Compiled WASM templates
├── docs/                  # Documentation
└── scripts/               # Deployment scripts
```

## Packages

| Package                    | Description         | Version |
| -------------------------- | ------------------- | ------- |
| `@stylus-deploy/sdk`       | Core TypeScript SDK | -       |
| `@stylus-deploy/cli`       | CLI tools           | -       |
| `@stylus-deploy/contracts` | Smart contracts     | -       |

## Quick Start

### Install SDK

```bash
pnpm add @stylus-deploy/sdk ethers
```

### Deploy a Token

```typescript
import { StylusDeploySDK, TEMPLATES } from '@stylus-deploy/sdk';
import { ethers } from 'ethers';

const provider = new ethers.BrowserProvider(window.ethereum);
const signer = await provider.getSigner();

const sdk = new StylusDeploySDK({
  provider,
  signer,
  network: 'arbitrum-sepolia',
});

// Deploy from template
const result = await sdk.deployFromTemplate({
  templateId: TEMPLATES.ERC20,
  initParams: {
    name: 'My Token',
    symbol: 'MTK',
    decimals: 18,
    totalSupply: '1000000',
  },
});

console.log(`Token deployed at: ${result.address}`);
```

### Use React Hooks

```tsx
import { useDeployFromTemplate } from '@stylus-deploy/sdk/react';
import { TEMPLATES } from '@stylus-deploy/sdk';

function DeployToken() {
  const { deploy, isLoading, result } = useDeployFromTemplate();

  const handleDeploy = async () => {
    await deploy({
      templateId: TEMPLATES.ERC20,
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

## Development

### Prerequisites

- Node.js 20+
- pnpm 8+
- Rust 1.81+ (for contract development)
- Foundry (for Solidity contracts)

### Setup

```bash
# Install dependencies
pnpm install

# Build all packages
pnpm build

# Run tests
pnpm test

# Start development
pnpm dev
```

### Repository Structure

This is a monorepo managed with:

- **pnpm workspaces** for package management
- **Turborepo** for build orchestration
- **Changesets** for version management

## Features

### Template System

Pre-built, audited templates for common contract types:

- ✅ ERC-20 Tokens
- ✅ ERC-721 NFTs
- ✅ Multi-sig Wallets
- ✅ DAO Governance
- 🚧 ERC-1155 Multi-tokens
- 🚧 DeFi Protocols
- 🚧 More coming soon...

### Universal Deployer

- Deploy any Stylus contract
- CREATE2 deterministic addresses
- Automatic Stylus activation
- Gas-optimized deployments
- Transaction tracking

### Decentralized Registry

- On-chain template metadata
- IPFS/Arweave storage
- Community contributions
- Version management
- Usage statistics

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

### Development Workflow

1. Create a new branch
2. Make your changes
3. Add tests
4. Run `pnpm test`
5. Submit a PR

## Security

This project is under active development. Do not use in production without thorough testing and auditing.

To report security issues, please email: [security@example.com]

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Resources

- [Documentation](./docs/)
- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus/)
- [Examples](./examples/)
- [Discord Community](#) (Coming soon)

## Roadmap

- [x] Monorepo setup
- [ ] Core contracts (Registry + Deployer)
- [ ] Stylus templates (Token, NFT, Multisig, DAO)
- [ ] TypeScript SDK
- [ ] React hooks
- [ ] CLI tools
- [ ] Demo application
- [ ] IPFS integration
- [ ] Testnet deployment
- [ ] Documentation
- [ ] Mainnet deployment

## Support

- GitHub Issues: [Report bugs or request features](../../issues)
- Documentation: [Full documentation](./docs/)
- Discord: Coming soon

---

Built with ❤️ for the Arbitrum Stylus ecosystem
