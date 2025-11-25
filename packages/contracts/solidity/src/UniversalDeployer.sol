// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import '@openzeppelin/contracts/access/Ownable.sol';
import '@openzeppelin/contracts/utils/Pausable.sol';
import '@openzeppelin/contracts/utils/ReentrancyGuard.sol';
import '@openzeppelin/contracts/utils/Create2.sol';
import './TemplateRegistry.sol';

/**
 * @title UniversalDeployer
 * @notice Factory contract for deploying Stylus WASM contracts
 * @dev Supports proxy deployment (EIP-1167) and direct WASM deployment
 */
contract UniversalDeployer is Ownable, Pausable, ReentrancyGuard {
  /// @notice Template registry contract
  TemplateRegistry public immutable registry;

  /// @notice Deployment method types
  enum DeploymentMethod {
    PROXY, // EIP-1167 minimal proxy (cheap)
    DIRECT, // Direct WASM deployment (flexible)
    TEMPLATE // Deploy from registry template
  }

  /// @notice Deployment information structure
  struct DeploymentInfo {
    address deployer; // Address that deployed the contract
    address contractAddress; // Deployed contract address
    uint256 templateId; // Template ID (0 if not from template)
    DeploymentMethod method; // Deployment method used
    uint256 timestamp; // Deployment timestamp
    bytes32 salt; // Salt used for CREATE2
    bool isActive; // Whether contract is still active
  }

  /// @notice Counter for total deployments
  uint256 private _deploymentCount;

  /// @notice Mapping from deployed contract address to deployment info
  mapping(address => DeploymentInfo) private _deployments;

  /// @notice Mapping from deployer address to their deployed contracts
  mapping(address => address[]) private _deploymentsByUser;

  /// @notice Mapping from template ID to deployed contracts
  mapping(uint256 => address[]) private _deploymentsByTemplate;

  /// @notice All deployed contract addresses
  address[] private _allDeployments;

  /// @notice Minimum gas required for deployment
  uint256 public constant MIN_DEPLOYMENT_GAS = 100_000;

  /// @notice Events
  event ContractDeployed(
    address indexed deployer,
    address indexed contractAddress,
    uint256 indexed templateId,
    DeploymentMethod method,
    bytes32 salt
  );

  event DeploymentDeactivated(
    address indexed contractAddress,
    address indexed deployer
  );

  event TemplateImplementationSet(
    uint256 indexed templateId,
    address indexed implementation
  );

  /// @notice Errors
  error InvalidRegistry();
  error InsufficientGas();
  error DeploymentFailed();
  error InvalidTemplate(uint256 templateId);
  error InvalidBytecode();
  error InvalidInitData();
  error ContractAlreadyDeployed(address contractAddress);
  error DeploymentNotFound(address contractAddress);
  error UnauthorizedDeactivation();

  /// @notice Mapping from template ID to implementation address (for proxy deployment)
  mapping(uint256 => address) private _templateImplementations;

  constructor(address _registry) Ownable(msg.sender) {
    if (_registry == address(0)) revert InvalidRegistry();
    registry = TemplateRegistry(_registry);
  }

  /**
   * @notice Deploy a contract using minimal proxy pattern (EIP-1167)
   * @param templateId Template ID from registry
   * @param salt Salt for CREATE2 deterministic address
   * @param initData Initialization data to call after deployment
   * @return contractAddress Address of deployed proxy contract
   */
  function deployProxy(
    uint256 templateId,
    bytes32 salt,
    bytes calldata initData
  ) external payable whenNotPaused nonReentrant returns (address) {
    if (gasleft() < MIN_DEPLOYMENT_GAS) revert InsufficientGas();

    // Verify template exists and is active
    if (!registry.isTemplateActive(templateId)) {
      revert InvalidTemplate(templateId);
    }

    // Get implementation address for this template
    address implementation = _templateImplementations[templateId];
    if (implementation == address(0)) {
      revert InvalidTemplate(templateId);
    }

    // Deploy minimal proxy using CREATE2
    address contractAddress = _deployMinimalProxy(
      implementation,
      salt,
      msg.sender
    );

    // Initialize the proxy if init data provided
    if (initData.length > 0) {
      (bool success, ) = contractAddress.call{value: msg.value}(initData);
      if (!success) revert DeploymentFailed();
    }

    // Record deployment
    _recordDeployment(
      contractAddress,
      templateId,
      DeploymentMethod.PROXY,
      salt
    );

    // Update registry deployment count
    registry.recordDeployment(templateId, msg.sender, contractAddress);

    emit ContractDeployed(
      msg.sender,
      contractAddress,
      templateId,
      DeploymentMethod.PROXY,
      salt
    );

    return contractAddress;
  }

  /**
   * @notice Deploy a contract directly from WASM bytecode
   * @param wasmBytecode WASM contract bytecode
   * @param salt Salt for CREATE2 deterministic address
   * @param initData Initialization data
   * @return contractAddress Address of deployed contract
   */
  function deployDirect(
    bytes calldata wasmBytecode,
    bytes32 salt,
    bytes calldata initData
  ) external payable whenNotPaused nonReentrant returns (address) {
    if (gasleft() < MIN_DEPLOYMENT_GAS) revert InsufficientGas();
    if (wasmBytecode.length == 0) revert InvalidBytecode();

    // Deploy using CREATE2 for deterministic address
    address contractAddress = Create2.deploy(msg.value, salt, wasmBytecode);

    if (contractAddress == address(0)) revert DeploymentFailed();

    // Initialize if init data provided
    if (initData.length > 0) {
      (bool success, ) = contractAddress.call(initData);
      if (!success) revert DeploymentFailed();
    }

    // Record deployment (templateId = 0 for direct deployments)
    _recordDeployment(contractAddress, 0, DeploymentMethod.DIRECT, salt);

    emit ContractDeployed(
      msg.sender,
      contractAddress,
      0,
      DeploymentMethod.DIRECT,
      salt
    );

    return contractAddress;
  }

  /**
   * @notice Deploy a contract from a registry template
   * @param templateId Template ID from registry
   * @param salt Salt for CREATE2
   * @param initData Initialization data
   * @return contractAddress Address of deployed contract
   */
  function deployFromTemplate(
    uint256 templateId,
    bytes32 salt,
    bytes calldata initData
  ) external payable whenNotPaused returns (address) {
    if (gasleft() < MIN_DEPLOYMENT_GAS) revert InsufficientGas();

    // Verify template exists and is active
    if (!registry.isTemplateActive(templateId)) {
      revert InvalidTemplate(templateId);
    }

    // For now, we assume WASM bytecode would be fetched from IPFS
    // In a real implementation, this would involve IPFS/Arweave integration
    // This is a placeholder that will be implemented in the SDK layer

    // Deploy using proxy method as fallback
    return this.deployProxy{value: msg.value}(templateId, salt, initData);
  }

  /**
   * @notice Predict the address of a contract before deployment
   * @param salt Salt for CREATE2
   * @param bytecodeHash Hash of the bytecode
   * @return predicted Address that will be deployed
   */
  function predictAddress(
    bytes32 salt,
    bytes32 bytecodeHash
  ) external view returns (address) {
    return Create2.computeAddress(salt, bytecodeHash);
  }

  /**
   * @notice Set implementation address for a template (owner only)
   * @param templateId Template ID
   * @param implementation Implementation contract address
   */
  function setTemplateImplementation(
    uint256 templateId,
    address implementation
  ) external onlyOwner {
    if (!registry.isTemplateActive(templateId)) {
      revert InvalidTemplate(templateId);
    }
    if (implementation == address(0)) revert InvalidBytecode();

    _templateImplementations[templateId] = implementation;

    emit TemplateImplementationSet(templateId, implementation);
  }

  /**
   * @notice Get implementation address for a template
   * @param templateId Template ID
   * @return implementation Implementation address
   */
  function getTemplateImplementation(
    uint256 templateId
  ) external view returns (address) {
    return _templateImplementations[templateId];
  }

  /**
   * @notice Deactivate a deployment (only deployer)
   * @param contractAddress Address of deployed contract
   */
  function deactivateDeployment(address contractAddress) external {
    DeploymentInfo storage deployment = _deployments[contractAddress];

    if (deployment.contractAddress == address(0)) {
      revert DeploymentNotFound(contractAddress);
    }

    if (deployment.deployer != msg.sender && msg.sender != owner()) {
      revert UnauthorizedDeactivation();
    }

    deployment.isActive = false;

    emit DeploymentDeactivated(contractAddress, msg.sender);
  }

  /**
   * @notice Get deployment information
   * @param contractAddress Deployed contract address
   * @return Deployment info
   */
  function getDeployment(
    address contractAddress
  ) external view returns (DeploymentInfo memory) {
    DeploymentInfo memory deployment = _deployments[contractAddress];
    if (deployment.contractAddress == address(0)) {
      revert DeploymentNotFound(contractAddress);
    }
    return deployment;
  }

  /**
   * @notice Get all deployments by a user
   * @param user User address
   * @return Array of deployed contract addresses
   */
  function getDeploymentsByUser(
    address user
  ) external view returns (address[] memory) {
    return _deploymentsByUser[user];
  }

  /**
   * @notice Get all deployments of a specific template
   * @param templateId Template ID
   * @return Array of deployed contract addresses
   */
  function getDeploymentsByTemplate(
    uint256 templateId
  ) external view returns (address[] memory) {
    return _deploymentsByTemplate[templateId];
  }

  /**
   * @notice Get total number of deployments
   * @return Total count
   */
  function getDeploymentCount() external view returns (uint256) {
    return _deploymentCount;
  }

  /**
   * @notice Get all deployments (paginated)
   * @param offset Starting index
   * @param limit Number of items to return
   * @return deployments Array of deployment addresses
   * @return total Total number of deployments
   */
  function getDeploymentsPaginated(
    uint256 offset,
    uint256 limit
  ) external view returns (address[] memory deployments, uint256 total) {
    total = _allDeployments.length;

    if (offset >= total) {
      return (new address[](0), total);
    }

    uint256 end = offset + limit;
    if (end > total) {
      end = total;
    }

    uint256 length = end - offset;
    deployments = new address[](length);

    for (uint256 i = 0; i < length; i++) {
      deployments[i] = _allDeployments[offset + i];
    }

    return (deployments, total);
  }

  /**
   * @notice Check if an address is a deployment from this factory
   * @param contractAddress Address to check
   * @return bool True if deployed from this factory
   */
  function isDeployment(address contractAddress) external view returns (bool) {
    return _deployments[contractAddress].contractAddress != address(0);
  }

  /**
   * @notice Pause the contract (owner only)
   */
  function pause() external onlyOwner {
    _pause();
  }

  /**
   * @notice Unpause the contract (owner only)
   */
  function unpause() external onlyOwner {
    _unpause();
  }

  /**
   * @notice Internal function to deploy minimal proxy
   * @param implementation Implementation address to proxy to
   * @param salt Salt for CREATE2
   * @param deployer Address of the deployer
   * @return proxy Address of deployed proxy
   */
  function _deployMinimalProxy(
    address implementation,
    bytes32 salt,
    address deployer
  ) internal returns (address proxy) {
    // EIP-1167 minimal proxy bytecode
    bytes memory bytecode = abi.encodePacked(
      hex'3d602d80600a3d3981f3363d3d373d3d3d363d73',
      implementation,
      hex'5af43d82803e903d91602b57fd5bf3'
    );

    // Add deployer to salt for uniqueness per user
    bytes32 finalSalt = keccak256(abi.encodePacked(salt, deployer));

    proxy = Create2.deploy(0, finalSalt, bytecode);

    if (proxy == address(0)) revert DeploymentFailed();

    return proxy;
  }

  /**
   * @notice Internal function to record deployment
   * @param contractAddress Deployed contract address
   * @param templateId Template ID (0 if none)
   * @param method Deployment method
   * @param salt Salt used
   */
  function _recordDeployment(
    address contractAddress,
    uint256 templateId,
    DeploymentMethod method,
    bytes32 salt
  ) internal {
    if (_deployments[contractAddress].contractAddress != address(0)) {
      revert ContractAlreadyDeployed(contractAddress);
    }

    _deployments[contractAddress] = DeploymentInfo({
      deployer: msg.sender,
      contractAddress: contractAddress,
      templateId: templateId,
      method: method,
      timestamp: block.timestamp,
      salt: salt,
      isActive: true
    });

    _deploymentsByUser[msg.sender].push(contractAddress);

    if (templateId > 0) {
      _deploymentsByTemplate[templateId].push(contractAddress);
    }

    _allDeployments.push(contractAddress);
    _deploymentCount++;
  }

  /**
   * @notice Withdraw accumulated ETH (owner only)
   */
  function withdraw() external onlyOwner {
    uint256 balance = address(this).balance;
    (bool success, ) = owner().call{value: balance}('');
    if (!success) revert DeploymentFailed();
  }

  /**
   * @notice Receive function to accept ETH
   */
  receive() external payable {}
}
