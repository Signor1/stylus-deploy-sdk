# Example: Token Deployment

Complete examples of deploying ERC-20 tokens using Stylus Deploy SDK.

## Basic Token Deployment

### Vanilla TypeScript

```typescript
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';
import { createPublicClient, createWalletClient, http } from 'viem';
import { arbitrumSepolia } from 'viem/chains';
import { privateKeyToAccount } from 'viem/accounts';

async function deployToken() {
  // 1. Setup clients
  const account = privateKeyToAccount(process.env.PRIVATE_KEY as `0x${string}`);

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

  // 3. Deploy token
  console.log('Deploying token...');

  const result = await deployer.deployFromProxy(TEMPLATE_IDS.ERC20, {
    name: 'My Token',
    symbol: 'MTK',
    decimals: 18,
    totalSupply: '1000000000000000000000000', // 1M tokens
  });

  console.log('✅ Token deployed!');
  console.log('Address:', result.address);
  console.log('Transaction:', result.transactionHash);
  console.log('Gas used:', result.gasUsed.toString());
  console.log(
    'View on Arbiscan:',
    `https://sepolia.arbiscan.io/address/${result.address}`
  );

  return result;
}

// Run
deployToken().catch(console.error);
```

## React Component with wagmi

### Complete Component

```tsx
import { useState } from 'react';
import { useDeployFromProxy } from '@stylus-deploy/sdk/react';
import { TEMPLATE_IDS } from '@stylus-deploy/sdk';
import { useAccount } from 'wagmi';
import { formatEther } from 'viem';

interface TokenFormData {
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: string;
}

export function TokenDeployer() {
  const { address, isConnected } = useAccount();
  const { deploy, isLoading, result, error } = useDeployFromProxy();

  const [formData, setFormData] = useState<TokenFormData>({
    name: '',
    symbol: '',
    decimals: 18,
    totalSupply: '1000000',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!isConnected) {
      alert('Please connect your wallet first');
      return;
    }

    try {
      // Convert supply to wei
      const supplyInWei =
        BigInt(formData.totalSupply) * BigInt(10 ** formData.decimals);

      await deploy({
        templateId: TEMPLATE_IDS.ERC20,
        initParams: {
          name: formData.name,
          symbol: formData.symbol,
          decimals: formData.decimals,
          totalSupply: supplyInWei.toString(),
        },
      });
    } catch (err) {
      console.error('Deployment failed:', err);
    }
  };

  return (
    <div className="token-deployer">
      <h2>Deploy Your Token</h2>

      {!isConnected && (
        <p className="warning">Please connect your wallet to deploy</p>
      )}

      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="name">Token Name</label>
          <input
            id="name"
            type="text"
            placeholder="My Token"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="symbol">Symbol</label>
          <input
            id="symbol"
            type="text"
            placeholder="MTK"
            value={formData.symbol}
            onChange={(e) =>
              setFormData({ ...formData, symbol: e.target.value.toUpperCase() })
            }
            maxLength={11}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="decimals">Decimals</label>
          <input
            id="decimals"
            type="number"
            min="0"
            max="18"
            value={formData.decimals}
            onChange={(e) =>
              setFormData({ ...formData, decimals: parseInt(e.target.value) })
            }
            required
          />
          <small>Standard is 18 (like ETH)</small>
        </div>

        <div className="form-group">
          <label htmlFor="supply">Total Supply</label>
          <input
            id="supply"
            type="text"
            placeholder="1000000"
            value={formData.totalSupply}
            onChange={(e) =>
              setFormData({ ...formData, totalSupply: e.target.value })
            }
            required
          />
          <small>
            Supply: {formData.totalSupply} {formData.symbol}
          </small>
        </div>

        <button type="submit" disabled={isLoading || !isConnected}>
          {isLoading ? 'Deploying...' : 'Deploy Token'}
        </button>
      </form>

      {error && (
        <div className="error">
          <h3>Error</h3>
          <p>{error.message}</p>
        </div>
      )}

      {result && (
        <div className="success">
          <h3>🎉 Token Deployed Successfully!</h3>
          <dl>
            <dt>Address:</dt>
            <dd>
              <code>{result.address}</code>
              <button
                onClick={() => navigator.clipboard.writeText(result.address)}
              >
                Copy
              </button>
            </dd>

            <dt>Transaction:</dt>
            <dd>
              <a
                href={`https://sepolia.arbiscan.io/tx/${result.transactionHash}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                View on Arbiscan
              </a>
            </dd>

            <dt>Gas Used:</dt>
            <dd>{result.gasUsed.toString()} gas</dd>

            <dt>Network:</dt>
            <dd>Arbitrum Sepolia</dd>
          </dl>

          <button
            onClick={() => {
              // Add token to MetaMask
              window.ethereum?.request({
                method: 'wallet_watchAsset',
                params: {
                  type: 'ERC20',
                  options: {
                    address: result.address,
                    symbol: formData.symbol,
                    decimals: formData.decimals,
                  },
                },
              });
            }}
          >
            Add to MetaMask
          </button>
        </div>
      )}
    </div>
  );
}
```

### Styles (Optional)

```css
.token-deployer {
  max-width: 500px;
  margin: 0 auto;
  padding: 2rem;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 600;
}

.form-group input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
}

.form-group small {
  display: block;
  margin-top: 0.25rem;
  color: #666;
}

button[type='submit'] {
  width: 100%;
  padding: 1rem;
  background: #0066ff;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
}

button[type='submit']:hover {
  background: #0052cc;
}

button[type='submit']:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.error {
  margin-top: 1rem;
  padding: 1rem;
  background: #fee;
  border: 1px solid #fcc;
  border-radius: 4px;
  color: #c00;
}

.success {
  margin-top: 1rem;
  padding: 1rem;
  background: #efe;
  border: 1px solid #cfc;
  border-radius: 4px;
}

.success dl {
  margin: 1rem 0;
}

.success dt {
  font-weight: 600;
  margin-top: 0.5rem;
}

.success dd {
  margin-left: 0;
  margin-bottom: 0.5rem;
}

.success code {
  background: #f5f5f5;
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-family: monospace;
}
```

## Advanced: Batch Token Deployment

Deploy multiple tokens in sequence:

```typescript
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';

async function deployMultipleTokens(
  deployer: UniversalDeployer,
  tokens: Array<{ name: string; symbol: string; supply: string }>
) {
  const results = [];

  for (const token of tokens) {
    console.log(`Deploying ${token.name}...`);

    const result = await deployer.deployFromProxy(TEMPLATE_IDS.ERC20, {
      name: token.name,
      symbol: token.symbol,
      decimals: 18,
      totalSupply: token.supply,
    });

    results.push({
      ...token,
      address: result.address,
      txHash: result.transactionHash,
    });

    console.log(`✅ ${token.name} deployed at ${result.address}`);

    // Wait a bit between deployments
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }

  return results;
}

// Usage
const tokens = [
  { name: 'Token A', symbol: 'TKA', supply: '1000000' },
  { name: 'Token B', symbol: 'TKB', supply: '2000000' },
  { name: 'Token C', symbol: 'TKC', supply: '500000' },
];

const deployed = await deployMultipleTokens(deployer, tokens);
console.log('All tokens deployed:', deployed);
```

## Next Steps

- [Deploy NFTs](./nft-collection.md)
- [Multi-Template Application](./multi-template.md)
- [Integration Patterns](./integration-patterns.md)
