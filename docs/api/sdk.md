# SDK API Reference

Complete API reference for the Stylus Deploy SDK core classes.

## UniversalDeployer

Main class for deploying Stylus contracts.

### Constructor

```typescript
new UniversalDeployer(
  publicClient: PublicClient,
  walletClient: WalletClient,
  config: StylusDeployConfig
)
```

**Parameters:**

- `publicClient` - Viem public client for reading blockchain data
- `walletClient` - Viem wallet client for signing transactions
- `config` - Deployment configuration

**Example:**

```typescript
import { createPublicClient, createWalletClient, http } from 'viem';
import { arbitrumSepolia } from 'viem/chains';

const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http(),
});

const walletClient = createWalletClient({
  account,
  chain: arbitrumSepolia,
  transport: http(),
});

const deployer = new UniversalDeployer(publicClient, walletClient, {
  network: 'arbitrum-sepolia',
});
```

### Methods

#### deployFromProxy()

Deploy a contract using the proxy pattern (fast & cheap).

```typescript
async deployFromProxy(
  templateId: string,
  initParams: Record<string, any>,
  options?: Partial<DeploymentConfig>
): Promise<DeploymentResult>
```

**Parameters:**

- `templateId` - Template ID from registry (e.g., `TEMPLATE_IDS.ERC20`)
- `initParams` - Initialization parameters for the contract
- `options` - Optional deployment configuration

**Returns:** `Promise<DeploymentResult>`

**Example:**

```typescript
const result = await deployer.deployFromProxy(
  TEMPLATE_IDS.ERC20,
  {
    name: 'My Token',
    symbol: 'MTK',
    decimals: 18,
    totalSupply: '1000000',
  },
  {
    gasLimit: 200000n,
    salt: '0x1234...',
  }
);
```

**Gas Cost:** ~150,000 gas (~$1-2)

---

#### deployFromTemplate()

Deploy a contract from a template (flexible).

```typescript
async deployFromTemplate(
  template: Template | string,
  initParams: Record<string, any>,
  options?: Partial<DeploymentConfig>
): Promise<DeploymentResult>
```

**Parameters:**

- `template` - Template object or template ID
- `initParams` - Initialization parameters
- `options` - Optional configuration

**Returns:** `Promise<DeploymentResult>`

**Example:**

```typescript
const result = await deployer.deployFromTemplate(
  'custom-template-id',
  {
    owner: '0x...',
    config: { ... },
  }
);
```

**Gas Cost:** ~500,000 gas (~$5-15)

---

#### deployFromBytecode()

Deploy custom WASM bytecode directly.

```typescript
async deployFromBytecode(
  wasmBytecode: Uint8Array,
  initData?: string,
  options?: Partial<DeploymentConfig>
): Promise<DeploymentResult>
```

**Parameters:**

- `wasmBytecode` - Compiled WASM bytecode
- `initData` - ABI-encoded initialization data (optional)
- `options` - Optional configuration

**Returns:** `Promise<DeploymentResult>`

**Example:**

```typescript
const wasmBytes = await fetch('/my-contract.wasm')
  .then((r) => r.arrayBuffer())
  .then((b) => new Uint8Array(b));

const result = await deployer.deployFromBytecode(wasmBytes, encodedInitData);
```

---

#### predictAddress()

Predict the deployment address before deploying.

```typescript
async predictAddress(
  bytecode: Uint8Array,
  salt: string
): Promise<Address>
```

**Parameters:**

- `bytecode` - WASM bytecode
- `salt` - Salt for CREATE2

**Returns:** `Promise<Address>` - Predicted contract address

**Example:**

```typescript
const salt = generateSalt();
const predictedAddress = await deployer.predictAddress(wasmBytes, salt);
console.log('Will deploy to:', predictedAddress);

// Deploy with same salt
const result = await deployer.deployFromBytecode(wasmBytes, initData, { salt });
console.log('Deployed to:', result.address); // Same as predicted
```

---

#### estimateGas()

Estimate gas cost for deployment.

```typescript
async estimateGas(
  config: DeploymentConfig
): Promise<bigint>
```

**Parameters:**

- `config` - Deployment configuration

**Returns:** `Promise<bigint>` - Estimated gas amount

**Example:**

```typescript
const gasEstimate = await deployer.estimateGas({
  method: 'proxy',
  templateId: TEMPLATE_IDS.ERC20,
  initParams: { ... },
});

console.log('Estimated gas:', gasEstimate.toString());
console.log('Estimated cost:', formatEther(gasEstimate * gasPrice), 'ETH');
```

---

#### getDeploymentInfo()

Get information about a deployed contract.

```typescript
async getDeploymentInfo(
  address: Address
): Promise<DeploymentInfo | null>
```

**Parameters:**

- `address` - Deployed contract address

**Returns:** `Promise<DeploymentInfo | null>`

**Example:**

```typescript
const info = await deployer.getDeploymentInfo('0x1234...');

if (info) {
  console.log('Deployed by:', info.creator);
  console.log('Template:', info.templateId);
  console.log('Method:', info.method);
  console.log('Timestamp:', new Date(info.timestamp * 1000));
}
```

---

#### getUserDeployments()

Get all deployments by a user.

```typescript
async getUserDeployments(
  userAddress: Address
): Promise<Address[]>
```

**Parameters:**

- `userAddress` - User's address

**Returns:** `Promise<Address[]>` - Array of deployed contract addresses

**Example:**

```typescript
const myDeployments = await deployer.getUserDeployments(account.address);
console.log(`You have deployed ${myDeployments.length} contracts`);

// Get details for each
for (const addr of myDeployments) {
  const info = await deployer.getDeploymentInfo(addr);
  console.log(`- ${addr}: ${info?.method} deployment`);
}
```

---

## TemplateManager

Manage and discover contract templates.

### Constructor

```typescript
new TemplateManager(config?: TemplateManagerConfig)
```

### Methods

#### loadTemplate()

Load a template by ID.

```typescript
async loadTemplate(templateId: string): Promise<Template>
```

**Example:**

```typescript
const manager = new TemplateManager();
const template = await manager.loadTemplate(TEMPLATE_IDS.ERC20);

console.log('Template:', template.name);
console.log('Version:', template.version);
console.log('Author:', template.author);
```

---

#### loadFromIPFS()

Load template WASM from IPFS.

```typescript
async loadFromIPFS(cid: string): Promise<Uint8Array>
```

**Example:**

```typescript
const wasmBytes = await manager.loadFromIPFS('QmXxx...');
console.log('Loaded', wasmBytes.length, 'bytes');
```

---

#### listTemplates()

List all available templates.

```typescript
async listTemplates(
  category?: TemplateCategory
): Promise<Template[]>
```

**Example:**

```typescript
// All templates
const allTemplates = await manager.listTemplates();

// Filter by category
const tokenTemplates = await manager.listTemplates('token');
const nftTemplates = await manager.listTemplates('nft');

console.log('Token templates:', tokenTemplates.length);
console.log('NFT templates:', nftTemplates.length);
```

---

#### searchTemplates()

Search templates by name or description.

```typescript
async searchTemplates(query: string): Promise<Template[]>
```

**Example:**

```typescript
const results = await manager.searchTemplates('governance');
console.log(`Found ${results.length} templates matching "governance"`);
```

---

## Types

### DeploymentResult

```typescript
interface DeploymentResult {
  success: boolean;
  address: Address;
  transactionHash: Hash;
  method: DeploymentMethod;
  gasUsed: bigint;
  wasmSize?: number;
  blockNumber: bigint;
  timestamp: number;
  receipt: TransactionReceipt;
  error?: string;
}
```

### DeploymentConfig

```typescript
interface DeploymentConfig {
  method: DeploymentMethod;
  templateId?: string;
  initParams?: Record<string, any>;
  salt?: string;
  gasLimit?: bigint;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
}
```

### Template

```typescript
interface Template {
  id: string;
  name: string;
  version: string;
  category: TemplateCategory;
  description: string;
  author: Address;
  ipfsCID?: string;
  onChainAddress?: Address;
  deploymentCount: number;
  supportsProxy: boolean;
  supportsTemplate: boolean;
  recommendedMethod: DeploymentMethod;
}
```

## Utility Functions

### isValidAddress()

```typescript
function isValidAddress(address: string): address is Address;
```

Check if a string is a valid Ethereum address.

---

### generateSalt()

```typescript
function generateSalt(): string;
```

Generate a random salt for CREATE2 deployment.

---

### formatGas()

```typescript
function formatGas(gas: bigint): string;
```

Format gas amount with thousand separators.

---

### formatAddress()

```typescript
function formatAddress(address: Address, chars?: number): string;
```

Format address for display (0x1234...5678).

---

## Constants

### TEMPLATE_IDS

```typescript
const TEMPLATE_IDS = {
  ERC20: '1',
  ERC721: '2',
  ERC1155: '3',
  MULTISIG: '4',
  DAO: '5',
} as const;
```

### NETWORKS

```typescript
const NETWORKS: Record<string, NetworkConfig> = {
  'arbitrum-one': { ... },
  'arbitrum-sepolia': { ... },
};
```

### DEFAULT_GAS_LIMITS

```typescript
const DEFAULT_GAS_LIMITS = {
  PROXY_DEPLOY: 150_000n,
  TEMPLATE_DEPLOY: 500_000n,
  WASM_ACTIVATION: 50_000_000n,
  INITIALIZATION: 100_000n,
} as const;
```

## Error Handling

All async methods may throw errors. Always use try-catch:

```typescript
try {
  const result = await deployer.deployFromProxy(...);
} catch (error) {
  if (error.code === 'INSUFFICIENT_FUNDS') {
    console.error('Not enough ETH for deployment');
  } else if (error.code === 'NETWORK_ERROR') {
    console.error('Network connection failed');
  } else {
    console.error('Deployment failed:', error.message);
  }
}
```

## Next Steps

- [React Hooks API](./react-hooks.md)
- [Type Definitions](./types.md)
- [Usage Examples](../examples/)
