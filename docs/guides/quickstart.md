# Quick Start Guide

Get started with Stylus Deploy SDK in 5 minutes!

## Prerequisites

- Node.js 20+ installed
- A wallet with Arbitrum Sepolia ETH ([Get testnet ETH](https://faucet.quicknode.com/arbitrum/sepolia))
- Basic knowledge of TypeScript/React

## Installation

```bash
npm install @stylus-deploy/sdk ethers
# or
pnpm add @stylus-deploy/sdk ethers
```

## Your First Deployment

### Option 1: Vanilla JavaScript/TypeScript

```typescript
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';
import { createPublicClient, createWalletClient, http } from 'viem';
import { arbitrumSepolia } from 'viem/chains';
import { privateKeyToAccount } from 'viem/accounts';

// 1. Set up clients
const account = privateKeyToAccount('0x...');

const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http(),
});

const walletClient = createWalletClient({
  account,
  chain: arbitrumSepolia,
  transport: http(),
});

// 2. Create deployer
const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia',
});

// 3. Deploy a token
const result = await deployer.deployFromProxy(TEMPLATE_IDS.ERC20, {
  name: 'My First Token',
  symbol: 'MFT',
  decimals: 18,
  totalSupply: '1000000',
});

console.log('Token deployed!');
console.log('Address:', result.address);
console.log('Transaction:', result.transactionHash);
console.log('Gas used:', result.gasUsed.toString());
```

### Option 2: React + wagmi

```tsx
import { useDeployFromProxy } from '@stylus-deploy/sdk/react';
import { TEMPLATE_IDS } from '@stylus-deploy/sdk';
import { useAccount } from 'wagmi';

function DeployTokenButton() {
  const { address } = useAccount();
  const { deploy, isLoading, result, error } = useDeployFromProxy();

  const handleDeploy = async () => {
    try {
      await deploy({
        templateId: TEMPLATE_IDS.ERC20,
        initParams: {
          name: 'My First Token',
          symbol: 'MFT',
          decimals: 18,
          totalSupply: '1000000',
        },
      });
    } catch (err) {
      console.error('Deployment failed:', err);
    }
  };

  return (
    <div>
      <button onClick={handleDeploy} disabled={isLoading || !address}>
        {isLoading ? 'Deploying...' : 'Deploy Token'}
      </button>
      {error && <p className="error">{error.message}</p>}
      {result && (
        <div className="success">
          <p>Token deployed at: {result.address}</p>
          <a href={`https://sepolia.arbiscan.io/address/${result.address}`}>
            View on Explorer
          </a>
        </div>
      )}
    </div>
  );
}
```

## Understanding the Result

After deployment, you'll receive a `DeploymentResult` object:

```typescript
{
  success: true,
  address: '0x1234...5678',        // Your deployed contract
  transactionHash: '0xabcd...',    // Transaction hash
  method: 'proxy',                 // Deployment method used
  gasUsed: 150000n,                // Gas consumed
  blockNumber: 12345678n,          // Block number
  timestamp: 1234567890,           // Unix timestamp
  receipt: { ... }                 // Full transaction receipt
}
```

## Next Steps

### 1. Verify Your Contract

Visit [Arbiscan Sepolia](https://sepolia.arbiscan.io/) and search for your contract address to see it on the blockchain.

### 2. Interact with Your Token

```typescript
// Get token info
const token = new Contract(result.address, ERC20_ABI, walletClient);
const name = await token.name();
const balance = await token.balanceOf(address);

console.log(`Token name: ${name}`);
console.log(`Your balance: ${balance.toString()}`);
```

### 3. Deploy Other Contract Types

Try deploying different templates:

```typescript
// Deploy an NFT collection
const nft = await deployer.deployFromProxy(TEMPLATE_IDS.ERC721, {
  name: 'My NFT Collection',
  symbol: 'MNFT',
  baseURI: 'ipfs://...',
  maxSupply: 10000,
});

// Deploy a DAO
const dao = await deployer.deployFromProxy(TEMPLATE_IDS.DAO, {
  name: 'My DAO',
  votingPeriod: 7 * 24 * 3600, // 7 days
  quorum: 4, // 4%
});
```

## Common Issues

### "Unsupported network" Error

Make sure you're connected to Arbitrum Sepolia:

```typescript
// Check your network
const chainId = await publicClient.getChainId();
console.log('Chain ID:', chainId); // Should be 421614 for Sepolia
```

### "Insufficient funds" Error

Get testnet ETH from [Arbitrum Sepolia Faucet](https://faucet.quicknode.com/arbitrum/sepolia).

### TypeScript Errors

Make sure you have the correct types installed:

```bash
npm install --save-dev @types/node
```

## Learning Resources

- [Deploy NFTs Guide](./deploy-nft.md)
- [React Integration Guide](./react-integration.md)
- [API Reference](../api/sdk.md)
- [Example Applications](../examples/)

## Get Help

- GitHub Issues: [Report a bug](https://github.com/signor1/stylus-deploy-sdk/issues)
- Discord: [Join our community](#) (Coming soon)
- Twitter: [@StylusDeploySDK](#) (Coming soon)

---

**Congratulations! 🎉** You've deployed your first Stylus contract!
