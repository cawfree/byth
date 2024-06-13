// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Test} from "forge-std/Test.sol";
import {BythTest} from "@src/BythTest.sol";

contract CounterTest is Test, BythTest {

    /// @notice connects a test to byth
    /// @dev this hook is overrided in a super hacky way to
    ///      inject contract bytecode since `halmos` doesn't
    ///      support foundry environment variables yet
    function testCreateHook() external {
        vm.expectRevert("Byth has failed to override createHook. Please raise an issue at https://github.com/cawfree/byth");
            _createBythHook();
    }

    function testDeployRawBytecode() public returns (bool success, bytes memory data) {
        address addr = _deployRawBytecode(hex"600f8060093d393df36000356020350160005260206000f3");
        assert(addr != address(0));
        (success, data) = addr.call(abi.encode(29, 40));
        assert(success);
        assert(abi.decode(data, (uint256)) == 69);
    }

}
