// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Test, console} from "forge-std/Test.sol";
import {SymTest} from "@halmos-cheatcodes/SymTest.sol";
import {BythTest} from "@byth/BythTest.sol";

import {Ownable} from "@openzeppelin-contracts/access/Ownable.sol";
import {ERC20} from "@openzeppelin-contracts/token/ERC20/ERC20.sol";

contract Fixture20 is ERC20 {
    constructor(address to) ERC20("Fixture", "FIX") {
        _mint(to, 10 ** 18);
    }
}

contract DetectorTest is Test, BythTest {

    /// @dev this is the location where deployed
    ///      bytecode detected by byth will be
    ///      injected
    address internal _hook;

    function setUp() public {
        /// @dev instantiate the hook - since we are
        ///      deploying a new contract, we don't
        ///      want to do this often
        _hook = _createBythHook();
    }

    /// @notice this sequence uses halmos to detect vulnerable
    ///         bytecode vulnerable to the third web exploit:
    ///         https://blog.thirdweb.com/vulnerability-report/
    function check_0_Multicall2771_ThirdWeb() public {

        bool success;

        bytes memory isTrustedForwarder = abi.encodeWithSignature("isTrustedForwarder(address)", address(0));

        // First, determine whether the contract supports ERC-2771.
        (success, ) = _hook.call(isTrustedForwarder);

        // At this point, the likely supports ERC-2771 - although,
        // there could be a false positive due to 4byte collision.
        vm.assume(success);

        // Ensure we're not false positive'ing simple low-level contracts
        // which have a leaky dispatch table.
        (success, ) = _hook.call(abi.encodeWithSignature("byth_invalid_contract(address)", address(0)));

        vm.assume(!success);

        // Attempt to delegatecall the exact same operation.
        bytes[] memory payload = new bytes[](1);
        payload[0] = isTrustedForwarder;

        (success, ) = _hook.call(abi.encodeWithSignature("multicall(bytes[])", payload));

        // If this `assert` terminates successfully, the contract is likely susceptible
        // to the ThirdWeb vulnerability: https://blog.thirdweb.com/vulnerability-report/
        assert(success);

    }

    /// @notice this detector finds contracts which allow
    ///         arbitrary external calls - combined with
    ///         approvals detection and the approver is
    ///         in for a bad time
    function check_1_DangerouslyApproved_ArbitraryBasic(address mark, address attacker, bytes4 selector) public {
        vm.assume(mark != address(0));
        vm.assume(attacker != address(0));
        vm.assume(mark != attacker);

        Fixture20 fixture20 = new Fixture20(mark);

        assert(fixture20.balanceOf(mark) == 10 ** 18);
        assert(fixture20.balanceOf(attacker) == 0);

        vm.prank(mark);
        fixture20.approve(_hook, type(uint256).max);

        bool success;

        (success,) = _hook.call(
            abi.encodeWithSelector(
                selector,
                address(fixture20),
                abi.encodeWithSignature(
                    "transferFrom(address,address,uint256)",
                    mark,
                    attacker,
                    10 ** 18
                )
            )
        );

        vm.assume(success);

        assert(fixture20.balanceOf(attacker) == 10 ** 18);

        // Uncomment at to reveal the selector?
        // assert(!success);
        // vm.assume(!success);
    }

}
