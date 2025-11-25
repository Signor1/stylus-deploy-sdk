// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console2} from 'forge-std/Test.sol';
import {TemplateRegistry} from '../src/TemplateRegistry.sol';

contract TemplateRegistryTest is Test {
  TemplateRegistry public registry;

  address owner = address(this);
  address alice = address(0xA11CE);
  address bob = address(0xB0B);
  address deployer = address(0xDE9107E4);

  // Test data
  string constant NAME = 'ERC20 Token';
  string constant DESCRIPTION = 'Standard ERC-20 token template';
  string constant VERSION = '1.0.0';
  bytes32 constant WASM_HASH = keccak256('test-wasm-bytecode');
  string constant IPFS_CID = 'QmTest123456789';
  string constant INIT_SCHEMA =
    '{"type":"object","properties":{"name":{"type":"string"}}}';

  event TemplateRegistered(
    uint256 indexed templateId,
    address indexed author,
    string name,
    TemplateRegistry.TemplateType templateType,
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

  function setUp() public {
    registry = new TemplateRegistry();
  }

  /*//////////////////////////////////////////////////////////////
                            REGISTRATION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_RegisterTemplate() public {
    string[] memory tags = new string[](2);
    tags[0] = 'token';
    tags[1] = 'erc20';

    vm.expectEmit(true, true, false, true);
    emit TemplateRegistered(
      1,
      owner,
      NAME,
      TemplateRegistry.TemplateType.TOKEN,
      IPFS_CID
    );

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    assertEq(templateId, 1);
    assertEq(registry.getTemplateCount(), 1);

    TemplateRegistry.Template memory template = registry.getTemplate(
      templateId
    );
    assertEq(template.name, NAME);
    assertEq(template.description, DESCRIPTION);
    assertEq(template.version, VERSION);
    assertEq(template.wasmHash, WASM_HASH);
    assertEq(template.ipfsCID, IPFS_CID);
    assertEq(template.author, owner);
    assertEq(template.deployCount, 0);
    assertTrue(template.active);
    assertEq(
      uint256(template.templateType),
      uint256(TemplateRegistry.TemplateType.TOKEN)
    );
  }

  function test_RegisterMultipleTemplates() public {
    string[] memory tags = new string[](1);
    tags[0] = 'test';

    // Register first template
    uint256 id1 = registry.registerTemplate(
      'Template 1',
      'Description 1',
      '1.0.0',
      keccak256('wasm1'),
      'CID1',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    // Register second template
    uint256 id2 = registry.registerTemplate(
      'Template 2',
      'Description 2',
      '1.0.0',
      keccak256('wasm2'),
      'CID2',
      TemplateRegistry.TemplateType.NFT,
      tags,
      INIT_SCHEMA
    );

    assertEq(id1, 1);
    assertEq(id2, 2);
    assertEq(registry.getTemplateCount(), 2);
  }

  function test_RevertWhen_EmptyName() public {
    string[] memory tags = new string[](0);

    vm.expectRevert(TemplateRegistry.EmptyName.selector);
    registry.registerTemplate(
      '',
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );
  }

  function test_RevertWhen_InvalidWasmHash() public {
    string[] memory tags = new string[](0);

    vm.expectRevert(TemplateRegistry.InvalidWasmHash.selector);
    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      bytes32(0),
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );
  }

  function test_RevertWhen_InvalidIPFSCID() public {
    string[] memory tags = new string[](0);

    vm.expectRevert(TemplateRegistry.InvalidIPFSCID.selector);
    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      '',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );
  }

  function test_RevertWhen_DuplicateWasmHash() public {
    string[] memory tags = new string[](0);

    // Register first template
    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    // Try to register duplicate
    vm.expectRevert(
      abi.encodeWithSelector(
        TemplateRegistry.TemplateAlreadyExists.selector,
        WASM_HASH
      )
    );
    registry.registerTemplate(
      'Different Name',
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );
  }

  /*//////////////////////////////////////////////////////////////
                            UPDATE TESTS
    //////////////////////////////////////////////////////////////*/

  function test_UpdateTemplate() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    string memory newName = 'Updated Name';
    string memory newVersion = '2.0.0';

    vm.expectEmit(true, false, false, true);
    emit TemplateUpdated(templateId, newName, newVersion);

    registry.updateTemplate(
      templateId,
      newName,
      'Updated Description',
      newVersion,
      INIT_SCHEMA
    );

    TemplateRegistry.Template memory template = registry.getTemplate(
      templateId
    );
    assertEq(template.name, newName);
    assertEq(template.version, newVersion);
  }

  function test_RevertWhen_UpdateByNonAuthor() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    vm.prank(alice);
    vm.expectRevert(TemplateRegistry.InvalidAuthor.selector);
    registry.updateTemplate(
      templateId,
      'New Name',
      'New Desc',
      '2.0.0',
      INIT_SCHEMA
    );
  }

  /*//////////////////////////////////////////////////////////////
                        ACTIVATION/DEACTIVATION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_DeactivateTemplate() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    assertTrue(registry.isTemplateActive(templateId));

    vm.expectEmit(true, false, false, false);
    emit TemplateDeactivated(templateId);

    registry.deactivateTemplate(templateId);

    assertFalse(registry.isTemplateActive(templateId));
  }

  function test_ReactivateTemplate() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.deactivateTemplate(templateId);
    assertFalse(registry.isTemplateActive(templateId));

    vm.expectEmit(true, false, false, false);
    emit TemplateActivated(templateId);

    registry.activateTemplate(templateId);

    assertTrue(registry.isTemplateActive(templateId));
  }

  function test_AuthorCanDeactivate() public {
    string[] memory tags = new string[](0);

    vm.prank(alice);
    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    vm.prank(alice);
    registry.deactivateTemplate(templateId);

    assertFalse(registry.isTemplateActive(templateId));
  }

  function test_OwnerCanDeactivate() public {
    string[] memory tags = new string[](0);

    vm.prank(alice);
    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    // Owner deactivates alice's template
    registry.deactivateTemplate(templateId);

    assertFalse(registry.isTemplateActive(templateId));
  }

  /*//////////////////////////////////////////////////////////////
                        DEPLOYMENT TRACKING TESTS
    //////////////////////////////////////////////////////////////*/

  function test_RecordDeployment() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    address deployedContract = address(0x123);

    vm.expectEmit(true, true, true, false);
    emit TemplateDeployed(templateId, deployer, deployedContract);

    vm.prank(deployer);
    registry.recordDeployment(templateId, deployer, deployedContract);

    TemplateRegistry.Template memory template = registry.getTemplate(
      templateId
    );
    assertEq(template.deployCount, 1);
  }

  function test_RecordMultipleDeployments() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    for (uint256 i = 0; i < 5; i++) {
      registry.recordDeployment(templateId, deployer, address(uint160(i)));
    }

    TemplateRegistry.Template memory template = registry.getTemplate(
      templateId
    );
    assertEq(template.deployCount, 5);
  }

  function test_RevertWhen_RecordDeploymentInactiveTemplate() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.deactivateTemplate(templateId);

    vm.expectRevert(
      abi.encodeWithSelector(
        TemplateRegistry.TemplateNotActive.selector,
        templateId
      )
    );
    registry.recordDeployment(templateId, deployer, address(0x123));
  }

  /*//////////////////////////////////////////////////////////////
                            QUERY TESTS
    //////////////////////////////////////////////////////////////*/

  function test_GetTemplatesByType() public {
    string[] memory tags = new string[](0);

    // Register TOKEN templates
    registry.registerTemplate(
      'Token 1',
      DESCRIPTION,
      VERSION,
      keccak256('token1'),
      'CID1',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.registerTemplate(
      'Token 2',
      DESCRIPTION,
      VERSION,
      keccak256('token2'),
      'CID2',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    // Register NFT template
    registry.registerTemplate(
      'NFT 1',
      DESCRIPTION,
      VERSION,
      keccak256('nft1'),
      'CID3',
      TemplateRegistry.TemplateType.NFT,
      tags,
      INIT_SCHEMA
    );

    uint256[] memory tokenTemplates = registry.getTemplatesByType(
      TemplateRegistry.TemplateType.TOKEN
    );
    uint256[] memory nftTemplates = registry.getTemplatesByType(
      TemplateRegistry.TemplateType.NFT
    );

    assertEq(tokenTemplates.length, 2);
    assertEq(nftTemplates.length, 1);
  }

  function test_GetTemplatesByAuthor() public {
    string[] memory tags = new string[](0);

    // Alice registers 2 templates
    vm.startPrank(alice);
    registry.registerTemplate(
      'Alice Template 1',
      DESCRIPTION,
      VERSION,
      keccak256('alice1'),
      'CID1',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.registerTemplate(
      'Alice Template 2',
      DESCRIPTION,
      VERSION,
      keccak256('alice2'),
      'CID2',
      TemplateRegistry.TemplateType.NFT,
      tags,
      INIT_SCHEMA
    );
    vm.stopPrank();

    // Bob registers 1 template
    vm.prank(bob);
    registry.registerTemplate(
      'Bob Template',
      DESCRIPTION,
      VERSION,
      keccak256('bob1'),
      'CID3',
      TemplateRegistry.TemplateType.DEFI,
      tags,
      INIT_SCHEMA
    );

    uint256[] memory aliceTemplates = registry.getTemplatesByAuthor(alice);
    uint256[] memory bobTemplates = registry.getTemplatesByAuthor(bob);

    assertEq(aliceTemplates.length, 2);
    assertEq(bobTemplates.length, 1);
  }

  function test_GetTemplateByHash() public {
    string[] memory tags = new string[](0);

    uint256 templateId = registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    uint256 foundId = registry.getTemplateByHash(WASM_HASH);
    assertEq(foundId, templateId);

    uint256 notFoundId = registry.getTemplateByHash(keccak256('nonexistent'));
    assertEq(notFoundId, 0);
  }

  function test_GetAllTemplateIds() public {
    string[] memory tags = new string[](0);

    registry.registerTemplate(
      'Template 1',
      DESCRIPTION,
      VERSION,
      keccak256('1'),
      'CID1',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.registerTemplate(
      'Template 2',
      DESCRIPTION,
      VERSION,
      keccak256('2'),
      'CID2',
      TemplateRegistry.TemplateType.NFT,
      tags,
      INIT_SCHEMA
    );

    uint256[] memory allIds = registry.getAllTemplateIds();
    assertEq(allIds.length, 2);
    assertEq(allIds[0], 1);
    assertEq(allIds[1], 2);
  }

  function test_GetTemplatesPaginated() public {
    string[] memory tags = new string[](0);

    // Register 5 templates
    for (uint256 i = 1; i <= 5; i++) {
      registry.registerTemplate(
        string.concat('Template ', vm.toString(i)),
        DESCRIPTION,
        VERSION,
        keccak256(abi.encodePacked(i)),
        string.concat('CID', vm.toString(i)),
        TemplateRegistry.TemplateType.TOKEN,
        tags,
        INIT_SCHEMA
      );
    }

    // Get first 3 templates
    (TemplateRegistry.Template[] memory templates, uint256 total) = registry
      .getTemplatesPaginated(0, 3);

    assertEq(total, 5);
    assertEq(templates.length, 3);
    assertEq(templates[0].name, 'Template 1');
    assertEq(templates[2].name, 'Template 3');

    // Get next 2 templates
    (templates, total) = registry.getTemplatesPaginated(3, 3);

    assertEq(total, 5);
    assertEq(templates.length, 2);
    assertEq(templates[0].name, 'Template 4');
    assertEq(templates[1].name, 'Template 5');
  }

  /*//////////////////////////////////////////////////////////////
                            PAUSABLE TESTS
    //////////////////////////////////////////////////////////////*/

  function test_PauseAndUnpause() public {
    registry.pause();

    string[] memory tags = new string[](0);

    vm.expectRevert();
    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    registry.unpause();

    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );
  }

  function test_RevertWhen_NonOwnerPauses() public {
    vm.prank(alice);
    vm.expectRevert();
    registry.pause();
  }

  /*//////////////////////////////////////////////////////////////
                            EDGE CASES
    //////////////////////////////////////////////////////////////*/

  function test_RevertWhen_GetNonexistentTemplate() public {
    vm.expectRevert(
      abi.encodeWithSelector(TemplateRegistry.TemplateNotFound.selector, 999)
    );
    registry.getTemplate(999);
  }

  function test_IsTemplateActive_NonexistentTemplate() public view {
    assertFalse(registry.isTemplateActive(0));
    assertFalse(registry.isTemplateActive(999));
  }

  function test_GetTemplatesPaginated_OutOfBounds() public {
    string[] memory tags = new string[](0);

    registry.registerTemplate(
      NAME,
      DESCRIPTION,
      VERSION,
      WASM_HASH,
      IPFS_CID,
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      INIT_SCHEMA
    );

    (TemplateRegistry.Template[] memory templates, uint256 total) = registry
      .getTemplatesPaginated(100, 10);

    assertEq(total, 1);
    assertEq(templates.length, 0);
  }
}
