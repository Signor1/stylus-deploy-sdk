# Stylus Deploy SDK Documentation

Welcome to the Stylus Deploy SDK documentation! This comprehensive guide will help you deploy Arbitrum Stylus contracts from your frontend without needing Rust compilation.

## 📚 Documentation Structure

### Getting Started

- [Quick Start Guide](./guides/quickstart.md) - Get up and running in 5 minutes
- [Installation](./guides/installation.md) - Detailed installation instructions
- [Architecture Overview](./architecture/overview.md) - Understanding the system design

### Guides

- [Deploying Tokens](./guides/deploy-token.md) - Deploy ERC-20 tokens
- [Deploying NFTs](./guides/deploy-nft.md) - Deploy ERC-721 NFT collections
- [Custom Templates](./guides/custom-templates.md) - Create your own templates
- [React Integration](./guides/react-integration.md) - Using with React and wagmi

### Architecture

- [System Overview](./architecture/overview.md) - High-level architecture
- [Template System](./architecture/templates.md) - How templates work
- [Factory Pattern](./architecture/factory.md) - Universal deployer design
- [Registry System](./architecture/registry.md) - Template registry

### API Reference

- [SDK API](./api/sdk.md) - Core SDK classes and methods
- [React Hooks](./api/react-hooks.md) - React hooks API
- [Types](./api/types.md) - TypeScript type definitions
- [Constants](./api/constants.md) - Network configs and constants

### Examples

- [Basic Token Deployment](./examples/token-deployment.md)
- [NFT Collection](./examples/nft-collection.md)
- [Multi-Template App](./examples/multi-template.md)
- [Integration Patterns](./examples/integration-patterns.md)

### Advanced Topics

- [Gas Optimization](./advanced/gas-optimization.md)
- [Security Best Practices](./advanced/security.md)
- [Custom Networks](./advanced/custom-networks.md)
- [Template Creation](./advanced/template-creation.md)

### Troubleshooting

- [Common Issues](./troubleshooting/common-issues.md)
- [Error Codes](./troubleshooting/error-codes.md)
- [FAQ](./troubleshooting/faq.md)

## 🎯 Quick Links

- **GitHub Repository**: [stylus-deploy-sdk](https://github.com/signor1/stylus-deploy-sdk)
- **NPM Package**: [@stylus-deploy/sdk](https://www.npmjs.com/package/@stylus-deploy/sdk)
- **Demo Application**: [Live Demo](#) (Coming soon)
- **Discord Community**: [Join us](#) (Coming soon)

## 🚀 Quick Example

```typescript
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';

// Deploy an ERC-20 token in 3 lines
const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia',
});

const result = await deployer.deployFromProxy(TEMPLATE_IDS.ERC20, {
  name: 'My Token',
  symbol: 'MTK',
  decimals: 18,
  totalSupply: '1000000',
});

console.log(`Token deployed at: ${result.address}`);
```

## 📖 Contributing to Documentation

Found an error or want to improve the docs? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing to documentation.

## 📄 License

MIT License - see [LICENSE](../LICENSE) for details.
