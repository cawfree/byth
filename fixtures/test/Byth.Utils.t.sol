// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

import {Test} from "forge-std/Test.sol";

import {BythUtils} from "../src/Byth.Utils.sol";

contract BythUtilsTest is Test {

    /**
     * @notice Verifies we can successfully deploy
     * bytecode reliably.
     * @dev It should be possible to declare conventional
     * unit tests within the same file, though these will
     * be ignored by Byth.
     */
    function test_deployBytecode() public {
        address addr = BythUtils.deploy(hex"600f8060093d393df36000356020350160005260206000f3");
        assertTrue(addr.code.length > 0);

        (bool s, bytes memory d) = addr.call(abi.encode(29, 40));

        assertTrue(s);
        assertEq(abi.decode(d, (uint256)), 69);
    }
    
}
