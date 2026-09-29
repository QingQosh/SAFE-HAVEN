# Testnet Faucet

The faucet is implemented by the SAFE-HAVEN Soroban contract. It distributes administrator-funded Stellar Asset Contract (SAC) tokens and supports three named testnet assets: `Usdc`, `Eth`, and `Btc`.

## Deploy and Configure

Build and deploy the contract with a funded testnet administrator identity:

```bash
make build
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/safe_haven.wasm \
  --source faucet-admin \
  --network testnet
```

Initialize the deployed contract using the required initialization arguments from the contract interface. Obtain or deploy the testnet SAC contracts for USDC, ETH, and BTC through the chosen testnet issuer, then configure their contract addresses. The administrator chooses the cap in each token's base units; for seven-decimal SACs, `100 USDC` is `1000000000`.

```bash
stellar contract invoke --id "$CONTRACT_ID" --source faucet-admin --network testnet -- \
  configure_faucet_asset --admin "$(stellar keys address faucet-admin)" \
  --asset Usdc --token "$USDC_SAC" --max_amount 1000000000

stellar contract invoke --id "$CONTRACT_ID" --source faucet-admin --network testnet -- \
  configure_faucet_asset --admin "$(stellar keys address faucet-admin)" \
  --asset Eth --token "$ETH_SAC" --max_amount 1000000000

stellar contract invoke --id "$CONTRACT_ID" --source faucet-admin --network testnet -- \
  configure_faucet_asset --admin "$(stellar keys address faucet-admin)" \
  --asset Btc --token "$BTC_SAC" --max_amount 1000000000
```

Fund the contract from the administrator's SAC balances. Funding is a token transfer to the contract; the faucet never mints tokens and issuer secrets must not be placed in frontend environment variables.

```bash
stellar contract invoke --id "$CONTRACT_ID" --source faucet-admin --network testnet -- \
  fund_faucet --admin "$(stellar keys address faucet-admin)" \
  --token "$USDC_SAC" --amount 10000000000
```

Repeat funding for ETH and BTC. Check the configured pool before publishing the faucet address:

```bash
stellar contract invoke --id "$CONTRACT_ID" --network testnet -- \
  get_faucet_status --asset Usdc
```

## Rules

- A wallet must authorize every claim with `request_faucet`.
- Each account can claim once per rolling 3600-second window, across all faucet assets.
- Each claim must be positive and no larger than the configured asset cap.
- Claims fail when the contract's SAC balance is insufficient or the contract is paused.
- The frontend exposes the faucet and live pool status from the `Faucet` tab.