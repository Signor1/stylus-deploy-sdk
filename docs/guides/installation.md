# Installation Guide

Complete installation instructions for Stylus Deploy SDK.

## Prerequisites

Before installing, ensure you have:

- **Node.js** 20.0.0 or higher
- **Package Manager**: npm, yarn, or pnpm
- **TypeScript** 5.0+ (recommended)
- **A Wallet**: MetaMask, WalletConnect, etc.
- **Testnet ETH**: For Arbitrum Sepolia testing

## Package Installation

### Using npm

```bash
npm install @stylus-deploy/sdk ethers
```

### Using pnpm (recommended)

```bash
pnpm add @stylus-deploy/sdk ethers
```

### Using yarn

```bash
yarn add @stylus-deploy/sdk ethers
```

## Optional Dependencies

### For React/Next.js Projects

If using React hooks:

```bash
npm install @tanstack/react-query wagmi viem
```

### For TypeScript Projects

```bash
npm install --save-dev typescript @types/node @types/react
```

## Framework-Specific Setup

### Next.js 14+ (App Router)

1. **Install dependencies:**

```bash
npm install @stylus-deploy/sdk @tanstack/react-query wagmi viem
```

2. **Create providers component** (`app/providers.tsx`):

```tsx
'use client';

import { WagmiProvider, createConfig, http } from 'wagmi';
import { arbitrumSepolia } from 'wagmi/chains';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ConnectKitProvider, getDefaultConfig } from 'connectkit';

const config = createConfig(
  getDefaultConfig({
    chains: [arbitrumSepolia],
    transports: {
      [arbitrumSepolia.id]: http(),
    },
    walletConnectProjectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID!,
    appName: 'My App',
  })
);

const queryClient = new QueryClient();

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <ConnectKitProvider>{children}</ConnectKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
```

3. **Wrap your app** (`app/layout.tsx`):

```tsx
import { Providers } from './providers';

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
```

4. **Use in your components:**

```tsx
'use client';

import { useDeployFromProxy } from '@stylus-deploy/sdk/react';

export function DeployButton() {
  const { deploy, isLoading } = useDeployFromProxy();
  // ... rest of component
}
```

### Create React App

1. **Install:**

```bash
npm install @stylus-deploy/sdk wagmi viem @tanstack/react-query
```

2. **Setup providers** (`src/App.tsx`):

```tsx
import { WagmiProvider } from 'wagmi';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { config } from './wagmi';

const queryClient = new QueryClient();

function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        {/* Your app */}
      </QueryClientProvider>
    </WagmiProvider>
  );
}
```

### Vite + React

1. **Install:**

```bash
npm install @stylus-deploy/sdk wagmi viem @tanstack/react-query
```

2. **Configure Vite** (`vite.config.ts`):

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  define: {
    global: 'globalThis',
  },
  resolve: {
    alias: {
      process: 'process/browser',
      buffer: 'buffer',
      util: 'util',
    },
  },
});
```

3. **Install polyfills:**

```bash
npm install --save-dev process buffer util
```

### Vanilla JavaScript/TypeScript

For non-React projects:

```bash
npm install @stylus-deploy/sdk viem
```

Usage:

```typescript
import { UniversalDeployer } from '@stylus-deploy/sdk';
import { createPublicClient, createWalletClient, http } from 'viem';
```

## Environment Setup

### Create `.env` file

```env
# Arbitrum Sepolia (testnet)
NEXT_PUBLIC_ARBITRUM_SEPOLIA_RPC=https://sepolia-rollup.arbitrum.io/rpc

# Arbitrum One (mainnet)
NEXT_PUBLIC_ARBITRUM_ONE_RPC=https://arb1.arbitrum.io/rpc

# WalletConnect Project ID (get from https://cloud.walletconnect.com)
NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=your_project_id

# Private key (for testing - NEVER commit this!)
PRIVATE_KEY=0x...
```

**Security Note:** Never commit private keys! Use `.env.local` and add to `.gitignore`.

### TypeScript Configuration

Create or update `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "allowSyntheticDefaultImports": true,
    "jsx": "react-jsx"
  },
  "include": ["src"]
}
```

## Verify Installation

Create a test file to verify everything works:

```typescript
// test-install.ts
import { UniversalDeployer, TEMPLATE_IDS } from '@stylus-deploy/sdk';

console.log('✅ SDK imported successfully');
console.log('Available templates:', Object.keys(TEMPLATE_IDS));

// Test types
const checkTypes = () => {
  const deployer: UniversalDeployer = {} as any;
  const templateId: string = TEMPLATE_IDS.ERC20;
  console.log('✅ Types working correctly');
};

checkTypes();
```

Run:

```bash
npx ts-node test-install.ts
```

Expected output:

```
✅ SDK imported successfully
Available templates: [ 'ERC20', 'ERC721', 'ERC1155', 'MULTISIG', 'DAO' ]
✅ Types working correctly
```

## Get Testnet ETH

To test deployments, get free testnet ETH:

### Arbitrum Sepolia Faucets

1. **QuickNode Faucet**: https://faucet.quicknode.com/arbitrum/sepolia
2. **Alchemy Faucet**: https://www.alchemy.com/faucets/arbitrum-sepolia
3. **Triangle Faucet**: https://faucet.triangleplatform.com/arbitrum/sepolia

You'll need:

- ~0.01 ETH for testing
- A wallet address (MetaMask, etc.)

## Common Installation Issues

### Issue: `Cannot find module` errors

**Solution:**

```bash
# Clear cache
rm -rf node_modules package-lock.json
npm install

# Or with pnpm
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Issue: Build errors with Next.js

**Solution:**
Add to `next.config.js`:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      net: false,
      tls: false,
    };
    return config;
  },
};

module.exports = nextConfig;
```

### Issue: TypeScript errors

**Solution:**

```bash
npm install --save-dev @types/node
```

And add to `tsconfig.json`:

```json
{
  "compilerOptions": {
    "skipLibCheck": true
  }
}
```

### Issue: `global is not defined` in browser

**Solution:**
Add to your HTML or entry file:

```html
<script>
  window.global = window;
</script>
```

Or in your main JS/TS file:

```typescript
if (typeof global === 'undefined') {
  (window as any).global = window;
}
```

## Next Steps

After installation:

1. **Follow Quick Start**: [Quick Start Guide](./quickstart.md)
2. **Deploy Your First Token**: [Token Deployment Example](../examples/token-deployment.md)
3. **Explore API**: [SDK API Reference](../api/sdk.md)

## Getting Help

If you encounter issues:

- Check [Troubleshooting Guide](../troubleshooting/common-issues.md)
- Search [GitHub Issues](https://github.com/signor1/stylus-deploy-sdk/issues)
- Ask on [Discord](#) (coming soon)

---

**Ready to deploy?** Continue to [Quick Start Guide](./quickstart.md)
