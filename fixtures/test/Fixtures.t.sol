// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

import {SymTest} from "@halmos-cheatcodes/SymTest.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {BythUtils} from "../src/Byth.Utils.sol";

contract Fixture20 is ERC20 {
    constructor(address to) ERC20("Fixture", "FIX") {
        _mint(to, 10 ** 18);
    }

}

contract FixturesTest is SymTest, Test {

    address private _uut;

    function setUp() public {
      /**
       * @dev Here, we inform byth that we want to accept
       * the current bytecode being analyzed - this is determined
       * by the indexer and will be loaded dynamically.
       */
      //// TODO: Eventually, we should modify the bytecode directly and not the high-level Solidity.
      ////       It would be a lot faster this way - our current implementation is very naive.
      bytes memory byth_bytecode_target = abi.encode(uint256(keccak256("byth.bytecode.target")) - 1);
      _uut = BythUtils.deploy(byth_bytecode_target);
    }

    function check_0_Multicall2771_ThirdWeb() public {

        bool success;

        bytes memory isTrustedForwarder = abi.encodeWithSignature("isTrustedForwarder(address)", address(0));

        // First, determine whether the contract supports ERC-2771.
        (success, ) = _uut.call(isTrustedForwarder);

        // At this point, the likely supports ERC-2771 - although,
        // there could be a false positive due to 4byte collision.
        vm.assume(success);

        // Ensure we're not false positive'ing simple low-level contracts
        // which have a leaky dispatch table.
        (success, ) = _uut.call(abi.encodeWithSignature("byth_invalid_contract(address)", address(0)));

        vm.assume(!success);

        // Attempt to delegatecall the exact same operation.
        bytes[] memory payload = new bytes[](1);
        payload[0] = isTrustedForwarder;

        (success, ) = _uut.call(abi.encodeWithSignature("multicall(bytes[])", payload));

        // If this `assert` terminates successfully, the contract is likely susceptible
        // to the ThirdWeb vulnerability: https://blog.thirdweb.com/vulnerability-report/
        assert(success);

    }

    function check_1_DangerouslyApproved_ArbitraryBasic(address mark, address attacker, bytes4 selector) public {
        vm.assume(mark != address(0));
        vm.assume(attacker != address(0));
        vm.assume(mark != attacker);

        Fixture20 fixture20 = new Fixture20(mark);

        assert(fixture20.balanceOf(mark) == 10 ** 18);
        assert(fixture20.balanceOf(attacker) == 0);

        vm.prank(mark);
        fixture20.approve(address(_uut), type(uint256).max);

        bool success;

        (success,) = address(_uut).call(
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
