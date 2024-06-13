<p align="center">
  <img src="public/byth.jpg" alt="Byth is the symbolic indexer for Ethereum."/>
</p>

[**Byth**](https://github.com/cawfree/byth) is an EVM indexer which uses [`halmos`](https://github.com/a16z/halmos) to formally verify bytecode using detectors written in [__Solidity__](https://github.com/ethereum/solidity). It helps you to **discover vulnerable contracts** using **customizable detectors**.

> [!IMPORTANT]
> Byth is experimental! Use at your own risk.

## Configuration

To compile and run this project, you'll need to install the [__Rust Toolchain__](https://www.rust-lang.org/tools/install):

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You also need [__Foundry__](https://getfoundry.sh/) installed:

```shell
curl -L https://foundry.paradigm.xyz | bash
```

Finally, install [__Halmos__](https://github.com/a16z/halmos):

```shell
pip3 install halmos
```

## Discovering Vulnerabilities

You can plug in your own custom detector logic to Byth easily. Just install the [__Byth Bindings__](./bindings/) to your [__Foundry__](https://getfoundry.sh/) tests:

```shell
forge install cawfree/byth --no-commit
```

Then add the remapping:

```shell
@byth/=byth/src/
```

Finally, inherit from [`BythTest.sol`](./bindings/src/BythTest.sol):

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Test} from "forge-std/Test.sol";
import {SymTest} from "@halmos-cheatcodes/SymTest.sol";
import {BythTest} from "@byth/BythTest.sol";

contract MyCustomDetector is Test, SymTest, BythTest {

    address internal _hook /* contract_under_test */;

    /// @notice Setup phase. Here Byth will inject deployment bytecode
    ///         via the `_createBythHook()` call, so you don't want to
    ///         do this frequently.
    function setUp() public {
        _hook = _createBythHook() /* initalize_contract_under_test*/;
    }

    /// @notice Then write your symbolic detectors! The convention is if your
    ///         test passes, it is considered vulnerable. Failures or timeouts
    ///         are considered as immune to the detector vulnerability.
    function check_stealTheMoney() public {
        (bool weStoleTheMoney,) = _hook.call(abi.encodeSignature("stealTheMoney()"));
        assert(weStoleTheMoney);
    }

}
```

Then you can start discovering vulnerabilities!

```shell
cargo run observe \
  --rpc-url $ETH_RPC_URL \
  --project ../custom_detector_project \ # your_project_name
  --debug \
  --block-number $ETH_BLOCK_NUMBER
```


> [!TIP]
> For a real-world example, check out [**this detector**](https://github.com/cawfree/byth/blob/d4362913905985ab5f09e00b9b01cf498049f664/detectors_default/test/Detectors.t.sol#L31C5-L63C6) for the [ThirdWeb](https://blog.thirdweb.com/vulnerability-report/) vulnerability!
> 
> You can also use `--project detectors_default` to access the built-in detectors.
