// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import '@openzeppelin/contracts/access/Ownable.sol';
import '@openzeppelin/contracts/utils/Pausable.sol';
import '@openzeppelin/contracts/utils/ReentrancyGuard.sol';

/**
 * @title TemplateRegistry
 * @notice Registry for storing and managing Stylus WASM contract templates
 * @dev Templates are stored on-chain as metadata, with actual WASM bytecode on IPFS
 */
contract TemplateRegistry is Ownable, Pausable, ReentrancyGuard {
  /// @notice Template category types
  enum TemplateType {
    TOKEN, // ERC-20 tokens
    NFT, // ERC-721/ERC-1155 NFTs
    MULTISIG, // Multi-signature wallets
    GOVERNANCE, // DAO and governance contracts
    DEFI, // DeFi protocols (AMM, lending, etc.)
    GAME, // Gaming contracts
    CUSTOM // Custom/uncategorized
  }

  /// @notice Template metadata structure
  struct Template {
    string name; // Human-readable name
    string description; // Template description
    string version; // Semantic version (e.g., "1.0.0")
    bytes32 wasmHash; // Keccak256 hash of WASM bytecode (for verification)
    string ipfsCID; // IPFS content identifier for WASM bytecode
    address author; // Template creator
    uint256 createdAt; // Creation timestamp
    uint256 deployCount; // Number of times deployed
    bool active; // Whether template is active/usable
    TemplateType templateType; // Category
    string[] tags; // Searchable tags
    string initSchema; // JSON schema for initialization parameters
  }

  /// @notice Counter for template IDs
  uint256 private _nextTemplateId;

  /// @notice Mapping from template ID to template data
  mapping(uint256 => Template) private _templates;

  /// @notice Mapping from template type to list of template IDs
  mapping(TemplateType => uint256[]) private _templatesByType;

  /// @notice Mapping from author address to their template IDs
  mapping(address => uint256[]) private _templatesByAuthor;

  /// @notice Mapping from WASM hash to template ID (prevent duplicates)
  mapping(bytes32 => uint256) private _hashToTemplateId;

  /// @notice List of all template IDs
  uint256[] private _allTemplateIds;

  /// @notice Events
  event TemplateRegistered(
    uint256 indexed templateId,
    address indexed author,
    string name,
    TemplateType templateType,
    string ipfsCID
  );

  event TemplateUpdated(
    uint256 indexed templateId,
    string name,
    string version
  );

  event TemplateActivated(uint256 indexed templateId);
  event TemplateDeactivated(uint256 indexed templateId);

  event TemplateDeployed(
    uint256 indexed templateId,
    address indexed deployer,
    address indexed deployedContract
  );

  /// @notice Errors
  error TemplateNotFound(uint256 templateId);
  error TemplateAlreadyExists(bytes32 wasmHash);
  error TemplateNotActive(uint256 templateId);
  error InvalidAuthor();
  error InvalidWasmHash();
  error InvalidIPFSCID();
  error EmptyName();

  constructor() Ownable(msg.sender) {
    _nextTemplateId = 1; // Start from 1 (0 is reserved for invalid)
  }

  /**
   * @notice Register a new template
   * @param name Template name
   * @param description Template description
   * @param version Template version
   * @param wasmHash Keccak256 hash of WASM bytecode
   * @param ipfsCID IPFS CID where WASM is stored
   * @param templateType Category of the template
   * @param tags Searchable tags
   * @param initSchema JSON schema for initialization
   * @return templateId The ID of the newly registered template
   */
  function registerTemplate(
    string calldata name,
    string calldata description,
    string calldata version,
    bytes32 wasmHash,
    string calldata ipfsCID,
    TemplateType templateType,
    string[] calldata tags,
    string calldata initSchema
  ) external whenNotPaused nonReentrant returns (uint256) {
    if (bytes(name).length == 0) revert EmptyName();
    if (wasmHash == bytes32(0)) revert InvalidWasmHash();
    if (bytes(ipfsCID).length == 0) revert InvalidIPFSCID();
    if (_hashToTemplateId[wasmHash] != 0) {
      revert TemplateAlreadyExists(wasmHash);
    }

    uint256 templateId = _nextTemplateId++;

    Template storage template = _templates[templateId];
    template.name = name;
    template.description = description;
    template.version = version;
    template.wasmHash = wasmHash;
    template.ipfsCID = ipfsCID;
    template.author = msg.sender;
    template.createdAt = block.timestamp;
    template.deployCount = 0;
    template.active = true;
    template.templateType = templateType;
    template.tags = tags;
    template.initSchema = initSchema;

    // Update mappings
    _hashToTemplateId[wasmHash] = templateId;
    _templatesByType[templateType].push(templateId);
    _templatesByAuthor[msg.sender].push(templateId);
    _allTemplateIds.push(templateId);

    emit TemplateRegistered(
      templateId,
      msg.sender,
      name,
      templateType,
      ipfsCID
    );

    return templateId;
  }

  /**
   * @notice Update template metadata (author only)
   * @param templateId Template ID to update
   * @param name New name
   * @param description New description
   * @param version New version
   * @param initSchema New initialization schema
   */
  function updateTemplate(
    uint256 templateId,
    string calldata name,
    string calldata description,
    string calldata version,
    string calldata initSchema
  ) external {
    if (templateId == 0 || templateId >= _nextTemplateId) {
      revert TemplateNotFound(templateId);
    }

    Template storage template = _templates[templateId];
    if (template.author != msg.sender) revert InvalidAuthor();

    if (bytes(name).length > 0) template.name = name;
    if (bytes(description).length > 0) template.description = description;
    if (bytes(version).length > 0) template.version = version;
    if (bytes(initSchema).length > 0) template.initSchema = initSchema;

    emit TemplateUpdated(templateId, template.name, template.version);
  }

  /**
   * @notice Activate a template (owner or author)
   * @param templateId Template ID to activate
   */
  function activateTemplate(uint256 templateId) external {
    if (templateId == 0 || templateId >= _nextTemplateId) {
      revert TemplateNotFound(templateId);
    }

    Template storage template = _templates[templateId];
    if (msg.sender != owner() && msg.sender != template.author) {
      revert InvalidAuthor();
    }

    template.active = true;
    emit TemplateActivated(templateId);
  }

  /**
   * @notice Deactivate a template (owner or author)
   * @param templateId Template ID to deactivate
   */
  function deactivateTemplate(uint256 templateId) external {
    if (templateId == 0 || templateId >= _nextTemplateId) {
      revert TemplateNotFound(templateId);
    }

    Template storage template = _templates[templateId];
    if (msg.sender != owner() && msg.sender != template.author) {
      revert InvalidAuthor();
    }

    template.active = false;
    emit TemplateDeactivated(templateId);
  }

  /**
   * @notice Increment deployment count (called by deployer contract)
   * @param templateId Template ID that was deployed
   * @param deployer Address that deployed the contract
   * @param deployedContract Address of the deployed contract
   */
  function recordDeployment(
    uint256 templateId,
    address deployer,
    address deployedContract
  ) external whenNotPaused {
    if (templateId == 0 || templateId >= _nextTemplateId) {
      revert TemplateNotFound(templateId);
    }

    Template storage template = _templates[templateId];
    if (!template.active) revert TemplateNotActive(templateId);

    template.deployCount++;

    emit TemplateDeployed(templateId, deployer, deployedContract);
  }

  /**
   * @notice Get template by ID
   * @param templateId Template ID
   * @return Template data
   */
  function getTemplate(
    uint256 templateId
  ) external view returns (Template memory) {
    if (templateId == 0 || templateId >= _nextTemplateId) {
      revert TemplateNotFound(templateId);
    }
    return _templates[templateId];
  }

  /**
   * @notice Get all templates of a specific type
   * @param templateType Category to filter by
   * @return Array of template IDs
   */
  function getTemplatesByType(
    TemplateType templateType
  ) external view returns (uint256[] memory) {
    return _templatesByType[templateType];
  }

  /**
   * @notice Get all templates by a specific author
   * @param author Author address
   * @return Array of template IDs
   */
  function getTemplatesByAuthor(
    address author
  ) external view returns (uint256[] memory) {
    return _templatesByAuthor[author];
  }

  /**
   * @notice Get all template IDs
   * @return Array of all template IDs
   */
  function getAllTemplateIds() external view returns (uint256[] memory) {
    return _allTemplateIds;
  }

  /**
   * @notice Get template ID by WASM hash
   * @param wasmHash Keccak256 hash of WASM bytecode
   * @return templateId (0 if not found)
   */
  function getTemplateByHash(bytes32 wasmHash) external view returns (uint256) {
    return _hashToTemplateId[wasmHash];
  }

  /**
   * @notice Get total number of registered templates
   * @return Total count
   */
  function getTemplateCount() external view returns (uint256) {
    return _nextTemplateId - 1;
  }

  /**
   * @notice Check if a template exists and is active
   * @param templateId Template ID to check
   * @return bool True if exists and active
   */
  function isTemplateActive(uint256 templateId) external view returns (bool) {
    if (templateId == 0 || templateId >= _nextTemplateId) return false;
    return _templates[templateId].active;
  }

  /**
   * @notice Get paginated list of templates
   * @param offset Starting index
   * @param limit Number of templates to return
   * @return templates Array of templates
   * @return total Total number of templates
   */
  function getTemplatesPaginated(
    uint256 offset,
    uint256 limit
  ) external view returns (Template[] memory templates, uint256 total) {
    total = _nextTemplateId - 1;
    if (offset >= total) {
      return (new Template[](0), total);
    }

    uint256 end = offset + limit;
    if (end > total) {
      end = total;
    }

    uint256 length = end - offset;
    templates = new Template[](length);

    for (uint256 i = 0; i < length; i++) {
      templates[i] = _templates[offset + i + 1];
    }

    return (templates, total);
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
}
