# NFT Auction Project

## Overview
This project implements an NFT auction platform using Solana and Rust. It features extensive Rust and TypeScript tests to ensure the platform's functionality on the Solana blockchain.

## Prerequisites

### Solana
- Install Solana CLI v1.15.2:
  ```sh
  sh -c "$(curl -sSfL https://release.solana.com/v1.15.2/install)"

### Rust
- Install Rust v1.66.1:
  ```
  rustup install 1.66.1
  rustup default 1.66.1
  ```
## Setup Instructions
### Build and Test Rust Programs
1. Build the Project:
    ```
    cargo build-bpf
    cp ./programs/auctioneer/tests/token_metadata_program/mpl_token_metadata-keypair.json ./target/deploy/
    cp ./programs/auctioneer/tests/token_metadata_program/mpl_token_metadata.so ./target/deploy/
    ```
2. Run Tests:
    ```
    cargo test-bpf
    ```

# TypeScript Tests (Localnet)

## Amman

### Install Amman
To install Amman globally, run the following command:

### Run Amman
Start Amman in a separate terminal:

## Build
Install the required dependencies and build the project:
```
yarn install
anchor build
```


## Deploy
Deploy the Anchor programs:
```
anchor deploy
```

After deploying the programs, replace the old program IDs with the new ones in the `Anchor.toml` and `lib.rs` files (in all programs).

## Run Tests
To run the tests, use the following command:
```
anchor test --skip-local-validator
```

# Project Structure
## Directory Layout
- `programs/auctioneer`: Contains the core auction program written in Rust.
- `programs/auction_house`: Manages the auction logic, handling bids, bid validation, and auction finalization.
- `programs/nft_minter`: Facilitates the minting of NFTs for use within the auction platform, ensuring proper metadata and token standards.
- `tests`: Includes comprehensive tests for the auction program and related components.

## Key Files
- `Anchor.toml`: Configuration file for the Anchor framework.
- `lib.rs`: Main Rust library file for the auction program.

# Usage
## Auction Creation
To create a new auction, deploy the auction program to the Solana blockchain and invoke the appropriate methods to initialize and manage the auction lifecycle.
