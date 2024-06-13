// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract BythTest {

    function _createBythHook() internal returns (address) {
        revert("Byth has failed to override createHook. Please raise an issue at https://github.com/cawfree/byth");
    }

    function _deployRawBytecode(bytes memory bytecode) internal returns (address addr) {
        assembly {
            addr := create(0, add(bytecode, 0x20), mload(bytecode))
        }
    }

}
