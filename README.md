# CWAR Token Staking

Staking pool based on [Scalable Reward Distribution On The Ethereum Blockchain](https://raw.githubusercontent.com/DeltaMichael/cwar-staking-program/master/scalable-reward-distribution-paper.pdf)

[Audit report by SmartState](https://raw.githubusercontent.com/DeltaMichael/cwar-staking-program/master/smartstate-audit-report.pdf)

# Build Solana Program (compiled for BPF)
Run the following from the program/ subdirectory:

```bash
$ cd program
$ cargo build-bpf
$ cargo test-bpf
```

# Build and use program interface for development

```
$ cd program/interface
$ npm pack
```

Copy tarball to your project

```
$ npm install /path/to/tarball
```

# Run integration tests

Requires that you have `solana-test-validator` installed

```
$ ./test.sh
```

# Directory structure

## program

Solana staking program in Rust

## program/interface

TS interface to interact with staking program
