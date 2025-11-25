// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console2} from 'forge-std/Test.sol';
import {UniversalDeployer} from '../src/UniversalDeployer.sol';
import {TemplateRegistry} from '../src/TemplateRegistry.sol';

// Mock implementation contract for testing
contract MockImplementation {
  uint256 public value;
  bool public initialized;

  function initialize(uint256 _value) external {
    require(!initialized, 'Already initialized');
    value = _value;
    initialized = true;
  }

  function getValue() external view returns (uint256) {
    return value;
  }
}

// Mock WASM contract (simplified for testing)
contract MockWasmContract {
  uint256 public data;

  constructor() {
    data = 42;
  }

  function initialize(uint256 _data) external {
    data = _data;
  }
}

contract UniversalDeployerTest is Test {
  UniversalDeployer public deployer;
  TemplateRegistry public registry;
  MockImplementation public implementation;

  address owner = address(this);
  address alice = address(0xA11CE);
  address bob = address(0xB0B);

  uint256 constant TEMPLATE_ID = 1;
  bytes32 constant SALT = keccak256('test-salt');

  // Allow test contract to receive ETH
  receive() external payable {}

  event ContractDeployed(
    address indexed deployer,
    address indexed contractAddress,
    uint256 indexed templateId,
    UniversalDeployer.DeploymentMethod method,
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

  function setUp() public {
    // Deploy registry
    registry = new TemplateRegistry();

    // Deploy deployer
    deployer = new UniversalDeployer(address(registry));

    // Deploy mock implementation
    implementation = new MockImplementation();

    // Register a template
    string[] memory tags = new string[](1);
    tags[0] = 'test';

    registry.registerTemplate(
      'Test Template',
      'Test Description',
      '1.0.0',
      keccak256('test-wasm'),
      'QmTestCID',
      TemplateRegistry.TemplateType.TOKEN,
      tags,
      '{}'
    );

    // Set implementation for template
    deployer.setTemplateImplementation(TEMPLATE_ID, address(implementation));
  }

  /*//////////////////////////////////////////////////////////////
                          PROXY DEPLOYMENT TESTS
    //////////////////////////////////////////////////////////////*/

  function test_DeployProxy() public {
    bytes memory initData = abi.encodeWithSelector(
      MockImplementation.initialize.selector,
      uint256(100)
    );

    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, initData);

    assertTrue(deployed != address(0));
    assertTrue(deployer.isDeployment(deployed));

    // Verify initialization worked
    MockImplementation proxy = MockImplementation(deployed);
    assertEq(proxy.getValue(), 100);
    assertTrue(proxy.initialized());

    // Verify deployment info
    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );
    assertEq(info.deployer, address(this));
    assertEq(info.templateId, TEMPLATE_ID);
  }

  function test_DeployProxy_WithoutInitData() public {
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    assertTrue(deployed != address(0));
    assertTrue(deployer.isDeployment(deployed));
  }

  function test_DeployProxy_MultipleWithDifferentSalts() public {
    address deployed1 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt1'),
      ''
    );
    address deployed2 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt2'),
      ''
    );

    assertTrue(deployed1 != deployed2);
    assertEq(deployer.getDeploymentCount(), 2);
  }

  function test_RevertWhen_ProxyDeployInactiveTemplate() public {
    registry.deactivateTemplate(TEMPLATE_ID);

    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.InvalidTemplate.selector,
        TEMPLATE_ID
      )
    );
    deployer.deployProxy(TEMPLATE_ID, SALT, '');
  }

  function test_RevertWhen_ProxyDeployNonexistentTemplate() public {
    vm.expectRevert(
      abi.encodeWithSelector(UniversalDeployer.InvalidTemplate.selector, 999)
    );
    deployer.deployProxy(999, SALT, '');
  }

  function test_RevertWhen_ProxyDeployNoImplementation() public {
    // Register new template without setting implementation
    string[] memory tags = new string[](0);
    uint256 newTemplateId = registry.registerTemplate(
      'New Template',
      'Description',
      '1.0.0',
      keccak256('new-wasm'),
      'QmNewCID',
      TemplateRegistry.TemplateType.NFT,
      tags,
      '{}'
    );

    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.InvalidTemplate.selector,
        newTemplateId
      )
    );
    deployer.deployProxy(newTemplateId, SALT, '');
  }

  /*//////////////////////////////////////////////////////////////
                          DIRECT DEPLOYMENT TESTS
    //////////////////////////////////////////////////////////////*/

  function test_DeployDirect() public {
    // Get bytecode of MockWasmContract
    bytes memory bytecode = type(MockWasmContract).creationCode;

    address deployed = deployer.deployDirect(bytecode, SALT, '');

    assertTrue(deployed != address(0));
    assertTrue(deployer.isDeployment(deployed));

    // Verify it's a valid contract
    MockWasmContract wasm = MockWasmContract(deployed);
    assertEq(wasm.data(), 42);
  }

  function test_DeployDirect_WithInitData() public {
    bytes memory bytecode = type(MockWasmContract).creationCode;
    bytes memory initData = abi.encodeWithSelector(
      MockWasmContract.initialize.selector,
      uint256(999)
    );

    address deployed = deployer.deployDirect(bytecode, SALT, initData);

    MockWasmContract wasm = MockWasmContract(deployed);
    assertEq(wasm.data(), 999);
  }

  function test_RevertWhen_DirectDeployEmptyBytecode() public {
    vm.expectRevert(UniversalDeployer.InvalidBytecode.selector);
    deployer.deployDirect('', SALT, '');
  }

  /*//////////////////////////////////////////////////////////////
                        TEMPLATE DEPLOYMENT TESTS
    //////////////////////////////////////////////////////////////*/

  function test_DeployFromTemplate() public {
    bytes memory initData = abi.encodeWithSelector(
      MockImplementation.initialize.selector,
      uint256(200)
    );

    address deployed = deployer.deployFromTemplate(TEMPLATE_ID, SALT, initData);

    assertTrue(deployed != address(0));
    assertTrue(deployer.isDeployment(deployed));

    MockImplementation proxy = MockImplementation(deployed);
    assertEq(proxy.getValue(), 200);
  }

  function test_RevertWhen_TemplateDeployInactiveTemplate() public {
    registry.deactivateTemplate(TEMPLATE_ID);

    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.InvalidTemplate.selector,
        TEMPLATE_ID
      )
    );
    deployer.deployFromTemplate(TEMPLATE_ID, SALT, '');
  }

  /*//////////////////////////////////////////////////////////////
                        ADDRESS PREDICTION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_PredictAddress() public view {
    bytes memory bytecode = type(MockWasmContract).creationCode;
    bytes32 bytecodeHash = keccak256(bytecode);

    address predicted = deployer.predictAddress(SALT, bytecodeHash);
    assertTrue(predicted != address(0));
  }

  /*//////////////////////////////////////////////////////////////
                    TEMPLATE IMPLEMENTATION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_SetTemplateImplementation() public {
    address newImpl = address(new MockImplementation());

    vm.expectEmit(true, true, false, false);
    emit TemplateImplementationSet(TEMPLATE_ID, newImpl);

    deployer.setTemplateImplementation(TEMPLATE_ID, newImpl);

    assertEq(deployer.getTemplateImplementation(TEMPLATE_ID), newImpl);
  }

  function test_RevertWhen_SetImplementationNonOwner() public {
    vm.prank(alice);
    vm.expectRevert();
    deployer.setTemplateImplementation(TEMPLATE_ID, address(implementation));
  }

  function test_RevertWhen_SetImplementationZeroAddress() public {
    vm.expectRevert(UniversalDeployer.InvalidBytecode.selector);
    deployer.setTemplateImplementation(TEMPLATE_ID, address(0));
  }

  function test_RevertWhen_SetImplementationInactiveTemplate() public {
    registry.deactivateTemplate(TEMPLATE_ID);

    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.InvalidTemplate.selector,
        TEMPLATE_ID
      )
    );
    deployer.setTemplateImplementation(TEMPLATE_ID, address(implementation));
  }

  /*//////////////////////////////////////////////////////////////
                        DEPLOYMENT INFO TESTS
    //////////////////////////////////////////////////////////////*/

  function test_GetDeployment() public {
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );

    assertEq(info.deployer, address(this));
    assertEq(info.contractAddress, deployed);
    assertEq(info.templateId, TEMPLATE_ID);
    assertEq(
      uint256(info.method),
      uint256(UniversalDeployer.DeploymentMethod.PROXY)
    );
    assertTrue(info.isActive);
  }

  function test_RevertWhen_GetNonexistentDeployment() public {
    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.DeploymentNotFound.selector,
        address(0x123)
      )
    );
    deployer.getDeployment(address(0x123));
  }

  function test_GetDeploymentsByUser() public {
    vm.startPrank(alice);

    address deployed1 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt1'),
      ''
    );
    address deployed2 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt2'),
      ''
    );

    vm.stopPrank();

    address[] memory aliceDeployments = deployer.getDeploymentsByUser(alice);
    assertEq(aliceDeployments.length, 2);
    assertEq(aliceDeployments[0], deployed1);
    assertEq(aliceDeployments[1], deployed2);
  }

  function test_GetDeploymentsByTemplate() public {
    address deployed1 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt1'),
      ''
    );
    address deployed2 = deployer.deployProxy(
      TEMPLATE_ID,
      keccak256('salt2'),
      ''
    );

    address[] memory templateDeployments = deployer.getDeploymentsByTemplate(
      TEMPLATE_ID
    );
    assertEq(templateDeployments.length, 2);
    assertEq(templateDeployments[0], deployed1);
    assertEq(templateDeployments[1], deployed2);
  }

  function test_GetDeploymentCount() public {
    assertEq(deployer.getDeploymentCount(), 0);

    deployer.deployProxy(TEMPLATE_ID, keccak256('salt1'), '');
    assertEq(deployer.getDeploymentCount(), 1);

    deployer.deployProxy(TEMPLATE_ID, keccak256('salt2'), '');
    assertEq(deployer.getDeploymentCount(), 2);
  }

  function test_GetDeploymentsPaginated() public {
    // Deploy 5 contracts
    for (uint256 i = 0; i < 5; i++) {
      deployer.deployProxy(TEMPLATE_ID, keccak256(abi.encodePacked(i)), '');
    }

    // Get first 3
    (address[] memory deployments, uint256 total) = deployer
      .getDeploymentsPaginated(0, 3);

    assertEq(total, 5);
    assertEq(deployments.length, 3);

    // Get next 2
    (deployments, total) = deployer.getDeploymentsPaginated(3, 3);

    assertEq(total, 5);
    assertEq(deployments.length, 2);
  }

  function test_GetDeploymentsPaginated_OutOfBounds() public {
    deployer.deployProxy(TEMPLATE_ID, SALT, '');

    (address[] memory deployments, uint256 total) = deployer
      .getDeploymentsPaginated(100, 10);

    assertEq(total, 1);
    assertEq(deployments.length, 0);
  }

  function test_IsDeployment() public {
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    assertTrue(deployer.isDeployment(deployed));
    assertFalse(deployer.isDeployment(address(0x123)));
  }

  /*//////////////////////////////////////////////////////////////
                      DEPLOYMENT DEACTIVATION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_DeactivateDeployment() public {
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    vm.expectEmit(true, true, false, false);
    emit DeploymentDeactivated(deployed, address(this));

    deployer.deactivateDeployment(deployed);

    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );
    assertFalse(info.isActive);
  }

  function test_DeactivateDeployment_ByOwner() public {
    vm.prank(alice);
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    // Owner can deactivate any deployment
    deployer.deactivateDeployment(deployed);

    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );
    assertFalse(info.isActive);
  }

  function test_RevertWhen_DeactivateByNonDeployer() public {
    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, '');

    vm.prank(alice);
    vm.expectRevert(UniversalDeployer.UnauthorizedDeactivation.selector);
    deployer.deactivateDeployment(deployed);
  }

  function test_RevertWhen_DeactivateNonexistentDeployment() public {
    vm.expectRevert(
      abi.encodeWithSelector(
        UniversalDeployer.DeploymentNotFound.selector,
        address(0x123)
      )
    );
    deployer.deactivateDeployment(address(0x123));
  }

  /*//////////////////////////////////////////////////////////////
                          PAUSABLE TESTS
    //////////////////////////////////////////////////////////////*/

  function test_PauseAndUnpause() public {
    deployer.pause();

    vm.expectRevert();
    deployer.deployProxy(TEMPLATE_ID, SALT, '');

    deployer.unpause();

    deployer.deployProxy(TEMPLATE_ID, SALT, '');
  }

  function test_RevertWhen_NonOwnerPauses() public {
    vm.prank(alice);
    vm.expectRevert();
    deployer.pause();
  }

  /*//////////////////////////////////////////////////////////////
                          WITHDRAWAL TESTS
    //////////////////////////////////////////////////////////////*/

  function test_Withdraw() public {
    // Send some ETH to deployer
    vm.deal(address(deployer), 10 ether);

    uint256 ownerBalanceBefore = owner.balance;

    deployer.withdraw();

    assertEq(address(deployer).balance, 0);
    assertEq(owner.balance, ownerBalanceBefore + 10 ether);
  }

  function test_RevertWhen_NonOwnerWithdraws() public {
    vm.prank(alice);
    vm.expectRevert();
    deployer.withdraw();
  }

  /*//////////////////////////////////////////////////////////////
                          RECEIVE TESTS
    //////////////////////////////////////////////////////////////*/

  function test_ReceiveETH() public {
    vm.deal(alice, 1 ether);

    vm.prank(alice);
    (bool success, ) = address(deployer).call{value: 1 ether}('');

    assertTrue(success);
    assertEq(address(deployer).balance, 1 ether);
  }

  /*//////////////////////////////////////////////////////////////
                          INTEGRATION TESTS
    //////////////////////////////////////////////////////////////*/

  function test_Integration_ProxyDeploymentFlow() public {
    // Alice deploys a proxy
    vm.startPrank(alice);

    bytes memory initData = abi.encodeWithSelector(
      MockImplementation.initialize.selector,
      uint256(500)
    );

    address deployed = deployer.deployProxy(TEMPLATE_ID, SALT, initData);

    vm.stopPrank();

    // Verify deployment
    assertTrue(deployer.isDeployment(deployed));

    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );
    assertEq(info.deployer, alice);
    assertTrue(info.isActive);

    // Verify initialization
    MockImplementation proxy = MockImplementation(deployed);
    assertEq(proxy.getValue(), 500);

    // Verify queries
    address[] memory aliceDeployments = deployer.getDeploymentsByUser(alice);
    assertEq(aliceDeployments.length, 1);
    assertEq(aliceDeployments[0], deployed);

    address[] memory templateDeployments = deployer.getDeploymentsByTemplate(
      TEMPLATE_ID
    );
    assertEq(templateDeployments.length, 1);

    // Verify registry was updated
    TemplateRegistry.Template memory template = registry.getTemplate(
      TEMPLATE_ID
    );
    assertEq(template.deployCount, 1);
  }

  function test_Integration_DirectDeploymentFlow() public {
    bytes memory bytecode = type(MockWasmContract).creationCode;
    bytes memory initData = abi.encodeWithSelector(
      MockWasmContract.initialize.selector,
      uint256(777)
    );

    address deployed = deployer.deployDirect(bytecode, SALT, initData);

    // Verify deployment
    MockWasmContract wasm = MockWasmContract(deployed);
    assertEq(wasm.data(), 777);

    UniversalDeployer.DeploymentInfo memory info = deployer.getDeployment(
      deployed
    );
    assertEq(info.templateId, 0); // No template for direct deploy
    assertEq(
      uint256(info.method),
      uint256(UniversalDeployer.DeploymentMethod.DIRECT)
    );
  }

  function test_Integration_MultipleUsersMultipleDeployments() public {
    // Alice deploys 2 contracts
    vm.startPrank(alice);
    deployer.deployProxy(TEMPLATE_ID, keccak256('alice1'), '');
    deployer.deployProxy(TEMPLATE_ID, keccak256('alice2'), '');
    vm.stopPrank();

    // Bob deploys 1 contract
    vm.prank(bob);
    deployer.deployProxy(TEMPLATE_ID, keccak256('bob1'), '');

    // Verify counts
    assertEq(deployer.getDeploymentCount(), 3);
    assertEq(deployer.getDeploymentsByUser(alice).length, 2);
    assertEq(deployer.getDeploymentsByUser(bob).length, 1);
    assertEq(deployer.getDeploymentsByTemplate(TEMPLATE_ID).length, 3);

    // Verify template deployment count
    TemplateRegistry.Template memory template = registry.getTemplate(
      TEMPLATE_ID
    );
    assertEq(template.deployCount, 3);
  }
}
