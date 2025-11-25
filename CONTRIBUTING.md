# Contributing to Stylus Deploy SDK

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Node.js 20+ and pnpm 8+
- Rust 1.81+ with cargo
- Foundry (forge, cast, anvil)
- Git

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/signor1/stylus-deploy-sdk.git
cd stylus-deploy-sdk

# Install dependencies
pnpm install

# Build all packages
pnpm build

# Run tests
pnpm test
```

## Project Structure

This is a monorepo using:

- **pnpm workspaces** for package management
- **Turborepo** for build orchestration
- **Changesets** for version management

```
stylus-deploy-sdk/
├── packages/
│   ├── contracts/     # Smart contracts
│   ├── sdk/          # TypeScript SDK
│   ├── cli/          # CLI tools
│   └── demo-app/     # Demo application
├── docs/             # Documentation
└── scripts/          # Utility scripts
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

Follow the coding standards:

- Use TypeScript for SDK code
- Use Solidity 0.8.24+ for Solidity contracts
- Use Rust 1.81+ for Stylus contracts
- Follow existing code style
- Add tests for new features
- Update documentation

### 3. Run Tests

```bash
# Run all tests
pnpm test

# Run specific package tests
pnpm --filter @stylus-deploy/sdk test

# Run linting
pnpm lint

# Format code
pnpm format
```

### 4. Create a Changeset

If your changes affect published packages, create a changeset:

```bash
pnpm changeset
```

Follow the prompts to describe your changes.

### 5. Commit Your Changes

```bash
git add .
git commit -m "feat: add new feature"
# or
git commit -m "fix: resolve bug"
```

Commit message format:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test changes
- `chore:` Maintenance tasks
- `refactor:` Code refactoring

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Package-Specific Guidelines

### Contracts Package

**Stylus Contracts (Rust):**

```bash
cd packages/contracts/stylus/templates/token
cargo build --target wasm32-unknown-unknown
cargo test
```

**Solidity Contracts:**

```bash
cd packages/contracts/solidity
forge build
forge test
forge test -vvv  # Verbose output
```

### SDK Package

```bash
cd packages/sdk
pnpm build
pnpm test
pnpm typecheck
```

### CLI Package

```bash
cd packages/cli
pnpm build
pnpm link --global  # Test CLI locally
stylus-deploy --help
```

## Testing

### Unit Tests

```bash
pnpm test
```

### Integration Tests

```bash
pnpm test:integration
```

### E2E Tests

```bash
pnpm test:e2e
```

### Coverage

```bash
pnpm test:coverage
```

## Code Quality

### Linting

```bash
# Run linter
pnpm lint

# Fix linting issues
pnpm lint:fix
```

### Type Checking

```bash
pnpm typecheck
```

### Formatting

```bash
# Check formatting
pnpm format:check

# Fix formatting
pnpm format
```

## Documentation

- Update README files when adding features
- Add JSDoc comments to public APIs
- Update docs/ directory for significant changes
- Include examples for new features

## Pull Request Process

1. **Ensure all tests pass**
2. **Update documentation**
3. **Create changeset if needed**
4. **Request review** from maintainers
5. **Address review comments**
6. **Squash and merge** once approved

### PR Title Format

- `feat: description` - New feature
- `fix: description` - Bug fix
- `docs: description` - Documentation
- `test: description` - Tests
- `chore: description` - Maintenance

## Versioning

We use [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Changesets handle version bumps automatically.

## Release Process

Releases are handled by maintainers:

```bash
# Version packages
pnpm version-packages

# Build and publish
pnpm release
```

## Community Guidelines

- Be respectful and inclusive
- Help others learn
- Provide constructive feedback
- Follow the code of conduct

## Questions?

- Open an issue for bugs or feature requests
- Join our Discord for discussions (coming soon)
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Stylus Deploy SDK!
