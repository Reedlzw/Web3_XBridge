# Web3-Bridge-Solana

This project is build with Anchor, a framework for Solana smart contracts.
This repository contains the code for the Web3 Bridge Solana project.
This project is a bridge aggregator on Solana, allowing users to bridge tokens between different blockchains.

## Installation

To install the project dependencies, run the following command:

0. make sure you have anchor cli installed and solana tool suite installed
   incledeing rust, cargo, and rustup, and solana cli, and anchor cli
1. `yarn install`

## Build & Deploy

0. `anchor build`
   anchor build will build the program and place the binary in the target/deploy directory,
   if you want's to build the single program, you can run `anchor build --program-name <program-name>`.
   if you want's to build the devnet program, you first need to config the depenencies in Cargo.toml of the web3 bridge program `features = ["cpi", "mainnet"]`, change it to `features = ["cpi", "devnet"]`, then run `anchor build --program-name <program-name>`
1. `anchor deploy`
   anchor deploy will deploy the program to the solana cluster which config on the .env file.
   if you want't to deploy the sigle program, you can run `anchor deploy --program-name <program-name>`.
   if deploy faild casue of unknow resson, you can use `solana program show --buffers` to fecth no finish deploy program, and use `solana program close <program-id>` to close the program, then you can deploy again.

## Testing

Do not use the anchor test, it will cause the program deploy again on mainnet, use the script on client folder to test the program

0. make show the ts-package is installed, if not, run `yarn install` to install the ts-package
1. use the script on client folder to send tx on mainnet or simulate tx on mainnet

## Verify Anchor IDL

0. `anchor idl verify <program-id>`
   eg: anchor idl verify <program-id> eg: anchor idl upgrade --filepath ./target/idl/web3_bridge_v2.json "okxBd18urPbBi2vsExxUDArzQNcju2DugV9Mt46BxYE"

## Usefull Command

solana program close
solana program show --buffers
solana program show --programs