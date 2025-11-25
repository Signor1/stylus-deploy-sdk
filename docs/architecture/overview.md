# Architecture Overview

This document provides a high-level overview of the Stylus Deploy SDK architecture.

## System Components

The Stylus Deploy SDK consists of three main components:

### 1. Template System

Pre-compiled WASM contracts stored in decentralized storage:

```
┌─────────────────────────────────────┐
│     Template Storage                │
├─────────────────────────────────────┤
│  • On-chain Registry (metadata)    │
│  • IPFS/Arweave (WASM bytecode)    │
│  • Local Cache (browser storage)   │
└─────────────────────────────────────┘
```

**Key Features:**

- Templates are deployed once and reused
- Stored on IPFS for decentralization
- Cached locally for performance
- Versioned for upgradability

### 2. Universal Deployer

Smart contract that handles deployment:

```solidity
UniversalDeployer {
    // Deploy from registry
    deployFromTemplate(templateId, initData)

    // Deploy custom WASM
    deployFromBytecode(wasmBytes, initData)

    // Query deployments
    getDeploymentInfo(address)
}
```

**Key Features:**

- CREATE2 for deterministic addresses
- Automatic Stylus activation
- Deployment tracking
- Gas optimization

### 3. TypeScript SDK

Frontend library for easy integration:

```typescript
@stylus-deploy/sdk
├── Core SDK (ethers/viem)
├── React Hooks (wagmi)
├── Template Manager
└── Utilities
```

**Key Features:**

- Type-safe API
- React hooks for wagmi
- Template discovery
- Error handling & retries

## Deployment Flow

### Proxy Deployment (Fast & Cheap)

```
User Frontend
    ↓
SDK prepares init data
    ↓
Factory.deployFromProxy(templateId, initData)
    ↓
EIP-1167 Minimal Proxy Created
    ↓
Proxy points to template
    ↓
Initialize with custom data
    ↓
✅ Contract ready (~$1-2)
```

**Use Case:** Standardized contracts (tokens, NFTs)
**Cost:** ~$1-2 per deployment
**Speed:** <30 seconds

### Template Deployment (Flexible)

```
User Frontend
    ↓
SDK fetches WASM from IPFS
    ↓
Deployer.deployFromBytecode(wasm, initData)
    ↓
Full contract deployed
    ↓
ArbWasm activation
    ↓
Initialize with data
    ↓
✅ Contract ready (~$5-15)
```

**Use Case:** Custom contracts with unique logic
**Cost:** ~$5-15 per deployment
**Speed:** ~1 minute

## Data Flow

### Template Registration

```
Developer
    ↓
1. Compile Rust → WASM
    ↓
2. Upload to IPFS
    ↓
3. Register in on-chain registry
    ↓
4. Template available for all users
```

### Template Discovery

```
User/SDK
    ↓
1. Query registry for templates
    ↓
2. Filter by category
    ↓
3. Load template metadata
    ↓
4. Cache locally
    ↓
5. Ready for deployment
```

## Security Model

### On-Chain Security

1. **Factory Contract**
   - Ownable pattern for admin functions
   - Pausable in emergencies
   - Template validation before registration

2. **Template Registry**
   - Immutable template addresses
   - WASM hash verification
   - Author tracking

3. **Deployed Contracts**
   - Initialization guards
   - Reentrancy protection
   - Access control

### Off-Chain Security

1. **IPFS Storage**
   - Content-addressed (tamper-proof)
   - Redundant storage
   - Gateway fallbacks

2. **SDK Security**
   - No private key handling in SDK
   - Transaction simulation before sending
   - Gas estimation safeguards

## Network Architecture

### Arbitrum Sepolia (Testnet)

```
RPC: https://sepolia-rollup.arbitrum.io/rpc
Explorer: https://sepolia.arbiscan.io
Factory: 0x... (deployed address)
Registry: 0x... (deployed address)
```

### Arbitrum One (Mainnet)

```
RPC: https://arb1.arbitrum.io/rpc
Explorer: https://arbiscan.io
Factory: 0x... (deployed address)
Registry: 0x... (deployed address)
```

## Gas Optimization

### Proxy Deployment

| Operation      | Gas Cost  | Optimization                   |
| -------------- | --------- | ------------------------------ |
| Clone creation | ~50k      | EIP-1167 minimal proxy         |
| Initialization | ~100k     | Efficient storage layout       |
| **Total**      | **~150k** | **96% savings vs full deploy** |

### Template Deployment

| Operation       | Gas Cost        | Optimization          |
| --------------- | --------------- | --------------------- |
| WASM deployment | ~300k           | Compressed bytecode   |
| Activation      | ~50M (one-time) | Stylus precompile     |
| Initialization  | ~100k           | Efficient storage     |
| **Total**       | **~500k**       | **Still 80% cheaper** |

## Scalability

### Horizontal Scaling

- Template registry is read-heavy (cacheable)
- Deployments are independent (parallel)
- IPFS distributes load across gateways
- No central server bottleneck

### Performance Characteristics

- Template list query: <100ms (cached)
- WASM fetch from IPFS: <500ms (first time)
- Deployment transaction: <30s (network dependent)
- Address prediction: instant (client-side)

## Extensibility

### Adding New Templates

1. Create Rust contract
2. Compile to WASM
3. Upload to IPFS
4. Register via CLI/SDK
5. Instantly available to all users

### Supporting New Networks

1. Deploy factory + registry
2. Add network config to SDK
3. Update constants
4. Deploy done

### Custom Features

- Plugin architecture (future)
- Custom deployment methods
- Extended template metadata
- Integration with other protocols

## Monitoring & Observability

### On-Chain Events

```solidity
event TemplateRegistered(...)
event ContractDeployed(...)
event TemplateUpdated(...)
```

### SDK Telemetry

- Deployment success/failure rates
- Gas cost tracking
- Template popularity
- Error monitoring

## Next Steps

- [Template System Details](./templates.md)
- [Factory Pattern Deep Dive](./factory.md)
- [Registry System](./registry.md)
