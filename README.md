<p align="center">
  <img src="public/byth.jpg" alt="Byth is the symbolic indexer for Ethereum."/>
</p>

[**Byth**](https://github.com/cawfree/byth) is an EVM indexer which uses [`halmos`](https://github.com/a16z/halmos) to formally verify bytecode using detectors written in [__Solidity__](https://github.com/ethereum/solidity). It helps you to **discover vulnerable contracts** using **customizable detectors**.

Check out [**this detector**](https://github.com/cawfree/byth/blob/c5c01fdc0444a7070052881ef9e93b1061602288/fixtures/test/Fixtures.t.sol#L36C5-L65C6) for the [ThirdWeb](https://blog.thirdweb.com/vulnerability-report/) vulnerability!

> [!IMPORTANT]
> Byth is experimental! Use at your own risk.

## Launch

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

**To start indexing vulnerabilities, use the following:**

```shell
cargo run observe \
 --rpc-url $ETH_RPC_URL \
 --project fixtures \
 --debug \
 --block-number $ETH_BLOCK_NUMBER
```
