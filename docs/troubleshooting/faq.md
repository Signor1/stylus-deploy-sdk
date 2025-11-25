# Frequently Asked Questions (FAQ)

Common questions about Stylus Deploy SDK.

## General Questions

### What is Stylus Deploy SDK?

Stylus Deploy SDK is a TypeScript library that enables you to deploy Arbitrum Stylus (WASM-based) smart contracts directly from your frontend application, without needing to set up Rust compilation tooling.

### Why use Stylus instead of Solidity?

Stylus contracts:

- Are written in Rust, C, or C++ (memory-safe languages)
- Have significantly lower gas costs (10-100x cheaper)
- Can leverage existing libraries from those ecosystems
- Are faster to execute

### Do I need to know Rust to use this SDK?

No! You can deploy from pre-built templates without writing any Rust code. Only template creators need Rust knowledge.

### What chains are supported?

Currently:

- **Arbitrum One** (Mainnet) - Production deployments
- **Arbitrum Sepolia** (Testnet) - Testing and development
- **Arbitrum Orbit chains** - Custom L3 chains (configurable)

### Is this production-ready?

The SDK is in active development. Use testnet for now. Production deployment coming Q1 2026.

## Deployment Questions

### How much does deployment cost?

Deployment costs depend on the method:

| Method   | Gas Cost  | USD (at 0.1 gwei) |
| -------- | --------- | ----------------- |
| Proxy    | ~150k gas | $1-2              |
| Template | ~500k gas | $5-15             |

These are estimates. Actual costs vary with network congestion and gas prices.

### Can I deploy custom contracts?

Yes! Three options:

1. **Use existing templates** - Deploy pre-built contracts (easiest)
2. **Create your own template** - Write Rust, compile to WASM, register
3. **Deploy raw WASM** - Upload WASM bytecode directly

### What's the difference between proxy and template deployment?

**Proxy Deployment:**

- Uses EIP-1167 minimal proxy pattern
- Very cheap (~$1-2)
- Points to shared template contract
- Best for standardized contracts (tokens, NFTs)

**Template Deployment:**

- Deploys full WASM bytecode
- More expensive (~$5-15)
- Fully customizable
- Best for unique contract logic

### Can I customize templates?

With proxy deployment: Only initialization parameters (name, symbol, etc.)

With template deployment: Full customization possible by modifying WASM before deployment.

### How long does deployment take?

- Proxy: ~15-30 seconds
- Template: ~30-60 seconds
- Varies based on network congestion

### Can I predict the deployment address?

Yes! Use `predictAddress()` with CREATE2:

```typescript
const salt = generateSalt();
const address = await deployer.predictAddress(bytecode, salt);
// Deploy with same salt to get this address
```

## Technical Questions

### Do I need a backend server?

No! Everything runs in the browser:

- Templates stored on IPFS (decentralized)
- Deployment happens via smart contracts
- SDK handles all the complexity

### How are templates stored?

Templates are stored in two places:

1. **IPFS/Arweave** - WASM bytecode (decentralized)
2. **On-chain Registry** - Metadata (template ID, name, author, IPFS hash)

### What happens if IPFS is slow?

The SDK implements several optimizations:

- Multiple gateway fallbacks
- Local caching (browser storage)
- CDN for popular templates (coming soon)

### Can I use this with Next.js?

Yes! Works with:

- Next.js (App Router or Pages Router)
- Create React App
- Vite
- Remix
- Any React framework

### Does this work with TypeScript?

Yes! The SDK is written in TypeScript with full type definitions.

### Can I use this with ethers.js?

The SDK internally uses viem, but you can convert between ethers and viem:

```typescript
import { ethers } from 'ethers';
import { createPublicClient, http } from 'viem';

// Convert ethers provider to viem
const provider = new ethers.BrowserProvider(window.ethereum);
const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http(provider.connection.url),
});
```

### How do I test locally?

1. Use Arbitrum Sepolia testnet (recommended)
2. Run local Arbitrum node (advanced):

```bash
# Coming soon: Local development environment
stylus-deploy dev
```

## Security Questions

### Is it safe to deploy from the frontend?

Yes, when done correctly:

- Private keys never leave your wallet
- Transactions are signed locally
- SDK only prepares transaction data
- You approve each transaction in your wallet

### How do I verify deployed contracts?

Contracts are automatically verifiable:

1. Check on [Arbiscan](https://arbiscan.io)
2. Template source code is public
3. WASM bytecode matches template hash

### What if a template is malicious?

Protection measures:

- Templates registered on-chain (transparent)
- Community review process (coming soon)
- Author reputation system (coming soon)
- Always review template source code before deploying

### Can deployed contracts be upgraded?

Templates themselves are immutable. However:

- You can implement upgradeable patterns in your contract
- New versions can be published as separate templates
- Consider using proxy patterns for upgradeability

## Integration Questions

### How do I integrate with wagmi?

Use the React hooks:

```typescript
import { useDeployFromProxy } from '@stylus-deploy/sdk/react';

function MyComponent() {
  const { deploy, isLoading } = useDeployFromProxy();
  // ... use the hook
}
```

Full guide: [React Integration](../guides/react-integration.md)

### Can I use this in a mobile app?

Yes, through:

- React Native with WalletConnect
- Mobile web browsers with wallet apps
- Dedicated mobile wallet integration

### How do I handle errors?

All errors are typed and descriptive:

```typescript
try {
  await deploy(...);
} catch (error) {
  switch (error.code) {
    case 'INSUFFICIENT_FUNDS':
      // Handle low balance
      break;
    case 'NETWORK_ERROR':
      // Handle network issues
      break;
    default:
      // Handle other errors
  }
}
```

See: [Error Handling Guide](../guides/error-handling.md)

### Can I customize gas prices?

Yes:

```typescript
await deployer.deployFromProxy(templateId, params, {
  maxFeePerGas: parseGwei('0.1'),
  maxPriorityFeePerGas: parseGwei('0.01'),
});
```

## Template Questions

### How do I create a custom template?

Full guide: [Creating Templates](../advanced/template-creation.md)

Quick overview:

1. Write contract in Rust using stylus-sdk
2. Compile to WASM
3. Upload to IPFS
4. Register in template registry

### Can I monetize my templates?

Template monetization features coming soon:

- Optional deployment fees
- Creator attribution
- Revenue sharing

### How do I update a template?

Templates are immutable. To update:

1. Create new version with updated code
2. Register as new template
3. Deprecate old version (optional)
4. Users can migrate to new version

### What makes a good template?

Good templates:

- Are well-documented
- Have clear initialization parameters
- Follow security best practices
- Include test coverage
- Have example usage code

## Troubleshooting

### Deployment fails with no error message

Check:

1. Wallet has enough ETH
2. Connected to correct network
3. Template exists and is active
4. Transaction on block explorer for detailed error

### Transactions are pending forever

Possible causes:

- Low gas price (increase it)
- Network congestion (wait or retry)
- Wallet nonce issues (restart wallet)

### TypeScript errors in my IDE

Make sure:

```bash
npm install --save-dev typescript @types/node
```

And check tsconfig.json includes:

```json
{
  "compilerOptions": {
    "skipLibCheck": true,
    "moduleResolution": "bundler"
  }
}
```

## Getting Help

Still have questions?

- 📖 [Read the Docs](../README.md)
- 🐛 [Report Issues](https://github.com/signor1/stylus-deploy-sdk/issues)
- 💬 [Join Discord](#) (Coming soon)
- 📧 Email: support@example.com (Coming soon)

## Contributing

Want to contribute?

- [Contributing Guidelines](../../CONTRIBUTING.md)
- [Code of Conduct](../../CODE_OF_CONDUCT.md)

---

**Didn't find your answer?** [Ask a question](https://github.com/signor1/stylus-deploy-sdk/discussions)
