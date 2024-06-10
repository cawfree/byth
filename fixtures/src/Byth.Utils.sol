// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

/**
 * @title Byth.Utils.sol
 * @author cawfree
 * @notice Exports helper functions which aid
 * bridging between the Byth Indexer and the
 * symbolic test environment.
 */
library BythUtils {

    /**
     * @dev A generic event emitted to verify execution depth
     * along arbitrary source.
     * @param depth The depth of the call - how far the caller
     * got.
     */
    event BythDepth(uint256 depth);

    /**
     * @param bytecode The bytecode to deploy, i.e. hex"600f8060093d393df36000356020350160005260206000f3".
     * @return addr The address of the deployed bytecode.
     */
    function deploy(bytes memory bytecode) internal returns (address addr) {
        assembly {
            addr := create(0, add(bytecode, 0x20), mload(bytecode))
        }
    }

}
