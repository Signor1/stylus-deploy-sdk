# Common Issues & Solutions

This guide covers the most common issues developers encounter when using Stylus Deploy SDK.

## Installation Issues

### Issue: `Cannot find module '@stylus-deploy/sdk'`

**Cause:** Package not installed or incorrect import path.

**Solution:**

```bash
# Reinstall the package
npm install @stylus-deploy/sdk ethers

# Or with pnpm
pnpm add @stylus-deploy/sdk ethers
```

Verify import path:

```typescript
// ✅ Correct
import { UniversalDeployer } from '@stylus-deploy/sdk';

// ❌ Incorrect
import { UniversalDeployer } from 'stylus-deploy-sdk';
```

### Issue: TypeScript errors with types

**Cause:** Missing TypeScript definitions.

**Solution:**

```bash
npm install --save-dev @types/node typescript
```

Ensure your `tsconfig.json` includes:

```json
{
  "compilerOptions": {
    "moduleResolution": "bundler",
    "skipLibCheck": true
  }
}
```

## Network Issues

### Issue: "Unsupported network" error

**Cause:** Invalid network configuration or wrong chain ID.

**Solution:**

```typescript
// Verify you're using a supported network
const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia', // Must be: 'arbitrum-one' or 'arbitrum-sepolia'
});

// Check current chain ID
const chainId = await publicClient.getChainId();
console.log('Current chain:', chainId);
// Arbitrum Sepolia = 421614
// Arbitrum One = 42161
```

### Issue: RPC connection failures

**Cause:** Rate limiting or RPC endpoint issues.

**Solution:**

```typescript
// Use custom RPC endpoint
const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia',
  rpcUrl: 'https://your-custom-rpc.com', // Add custom RPC
});

// Or use Alchemy/Infura
const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http('https://arb-sepolia.g.alchemy.com/v2/YOUR_KEY'),
});
```

## Deployment Issues

### Issue: "Deployment failed" with no specific error

**Cause:** Various reasons - check transaction on block explorer.

**Solution:**

```typescript
try {
  const result = await deployer.deployFromProxy(templateId, params);
} catch (error) {
  console.error('Deployment error:', error);

  // Check if it's a revert
  if (error.code === 'CALL_EXCEPTION') {
    console.log('Contract reverted:', error.reason);
  }

  // Check transaction if hash is available
  if (error.transaction) {
    console.log('Transaction:', error.transaction);
    console.log(
      'Check on explorer:',
      `https://sepolia.arbiscan.io/tx/${error.transaction.hash}`
    );
  }
}
```

### Issue: "Insufficient funds for gas"

**Cause:** Not enough ETH in wallet for transaction.

**Solution:**

```typescript
// Check balance before deploying
const balance = await publicClient.getBalance({
  address: account.address,
});

console.log('Balance:', formatEther(balance), 'ETH');

if (balance < parseEther('0.01')) {
  console.error('Need at least 0.01 ETH for deployment');
  // Get testnet ETH from faucet
}

// Estimate gas first
const gasEstimate = await deployer.estimateGas({
  method: 'proxy',
  templateId: TEMPLATE_IDS.ERC20,
  initParams: { ... },
});

console.log('Estimated gas:', gasEstimate.toString());
```

### Issue: "Template not found"

**Cause:** Template ID doesn't exist in registry.

**Solution:**

```typescript
// List all available templates
const templateManager = new TemplateManager();
const templates = await templateManager.listTemplates();

console.log('Available templates:');
templates.forEach((t) => {
  console.log(`- ${t.name} (ID: ${t.id})`);
});

// Use correct template ID
import { TEMPLATE_IDS } from '@stylus-deploy/sdk';
console.log('Available IDs:', TEMPLATE_IDS);
```

### Issue: Gas price too high / Transaction too expensive

**Cause:** Network congestion or suboptimal gas settings.

**Solution:**

```typescript
// Check current gas price
const gasPrice = await publicClient.getGasPrice();
console.log('Current gas price:', formatGwei(gasPrice), 'gwei');

// Wait for lower gas prices or adjust
const result = await deployer.deployFromProxy(templateId, params, {
  maxFeePerGas: parseGwei('0.1'), // Set max gas price
  maxPriorityFeePerGas: parseGwei('0.01'), // Set priority fee
});
```

## React Hooks Issues

### Issue: "Hooks can only be called inside the body of a function component"

**Cause:** Calling React hooks outside components or in the wrong order.

**Solution:**

```typescript
// ❌ Wrong - outside component
const { deploy } = useDeployFromProxy();

// ✅ Correct - inside component
function MyComponent() {
  const { deploy, isLoading } = useDeployFromProxy();

  return <button onClick={() => deploy(...)}>Deploy</button>;
}
```

### Issue: Hook returns `undefined` or stale data

**Cause:** Missing wagmi/react-query setup or stale cache.

**Solution:**

```tsx
// Ensure proper provider setup
import { WagmiConfig, createConfig } from 'wagmi';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const config = createConfig({ ... });
const queryClient = new QueryClient();

function App() {
  return (
    <WagmiConfig config={config}>
      <QueryClientProvider client={queryClient}>
        <YourApp />
      </QueryClientProvider>
    </WagmiConfig>
  );
}

// Invalidate cache after deployment
const { deploy } = useDeployFromProxy();
const queryClient = useQueryClient();

await deploy(...);
queryClient.invalidateQueries(['deployments']);
```

## Transaction Issues

### Issue: Transaction pending forever

**Cause:** Low gas price or network congestion.

**Solution:**

```typescript
// Add timeout to deployment
const deployWithTimeout = async () => {
  const timeoutPromise = new Promise((_, reject) =>
    setTimeout(() => reject(new Error('Deployment timeout')), 120000) // 2 min
  );

  const deployPromise = deployer.deployFromProxy(...);

  return Promise.race([deployPromise, timeoutPromise]);
};

try {
  const result = await deployWithTimeout();
} catch (error) {
  if (error.message === 'Deployment timeout') {
    console.log('Transaction is taking too long, but may still succeed');
    // Check transaction status separately
  }
}
```

### Issue: "Nonce too low" error

**Cause:** Nonce collision or stale nonce.

**Solution:**

```typescript
// Let viem handle nonce automatically (default)
// Or manually manage nonce:
const nonce = await publicClient.getTransactionCount({
  address: account.address,
});

// Force refresh wallet client
const freshWalletClient = createWalletClient({
  account,
  chain: arbitrumSepolia,
  transport: http(),
});
```

## Template Issues

### Issue: Custom template not working

**Cause:** Template not properly registered or WASM issues.

**Solution:**

```bash
# Verify template compilation
cd packages/contracts/stylus/templates/my-template
cargo build --target wasm32-unknown-unknown --release

# Check WASM size
ls -lh target/wasm32-unknown-unknown/release/*.wasm

# Should be < 24KB compressed
```

```typescript
// Verify template is registered
const template = await templateManager.getTemplateMetadata(templateId);
console.log('Template info:', template);

if (!template.active) {
  console.error('Template is not active');
}
```

## Performance Issues

### Issue: Slow template loading

**Cause:** IPFS gateway latency.

**Solution:**

```typescript
// Use multiple IPFS gateways
const IPFS_GATEWAYS = [
  'https://ipfs.io/ipfs/',
  'https://gateway.pinata.cloud/ipfs/',
  'https://cloudflare-ipfs.com/ipfs/',
];

// Implement fallback fetching
async function fetchWithFallback(cid: string) {
  for (const gateway of IPFS_GATEWAYS) {
    try {
      const response = await fetch(`${gateway}${cid}`);
      if (response.ok) return response;
    } catch (error) {
      console.log(`Gateway ${gateway} failed, trying next...`);
    }
  }
  throw new Error('All gateways failed');
}
```

### Issue: High memory usage in browser

**Cause:** Too many templates cached or memory leak.

**Solution:**

```typescript
// Clear template cache
localStorage.removeItem('stylus-template-cache');

// Limit cache size
const MAX_CACHE_SIZE = 10; // Only cache 10 templates

// Monitor memory
if (performance.memory) {
  console.log(
    'Memory used:',
    (performance.memory.usedJSHeapSize / 1048576).toFixed(2),
    'MB'
  );
}
```

## Getting More Help

If your issue isn't covered here:

1. Check the [API Documentation](../api/sdk.md)
2. Search [GitHub Issues](https://github.com/signor1/stylus-deploy-sdk/issues)
3. Ask in [Discord](#) (coming soon)
4. Review [Example Code](../examples/)

## Report a Bug

Found a bug? Please [create an issue](https://github.com/signor1/stylus-deploy-sdk/issues/new) with:

- SDK version
- Network (Sepolia/Mainnet)
- Error message/stack trace
- Minimal reproduction code
- Expected vs actual behavior
