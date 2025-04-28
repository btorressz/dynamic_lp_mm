# dynamic_lp_mm

# 🎯 Dynamic LP Market Maker Vault

## Overview

The **Dynamic LP Market Maker Vault** is a Solana smart contract (program) built with **Anchor** that enables **automated liquidity rebalancing across price bands** on decentralized exchanges (DEXs).

The vault allows users to deposit token pairs, mint share tokens representing ownership, and automatically rebalances liquidity within specified price bands (e.g., every 5 basis points).  
It helps liquidity providers maximize trading fees, maintain optimal market depth, and minimize impermanent loss.

This project was developed in Solana Playground IDE

---

## ✨ Key Features

- **Dynamic Liquidity Rebalancing**  
  Liquidity is rebalanced when the price crosses predefined bands (e.g., 5bps). Supports multiple rebalance modes: manual, automated, or volatility-based triggers.

- **Emergency Pause and Withdraw Mode**  
  Admins can pause all operations or enable emergency-only withdrawals if DEX hacks or extreme events occur.

- **Fee Collection and Auto-Compounding**  
  Vault collects fees from trading activities and can automatically reinvest them into liquidity positions.

- **Vault Treasury Management**  
  Accrued protocol fees can be swept to a designated treasury wallet.

- **Whitelist Control (Optional)**  
  Restrict deposits and withdrawals to whitelisted addresses if desired.

- **Multi-Admin System**  
  Supports assigning multiple admins beyond the initial authority for enhanced operational flexibility.

- **Deposit Limits and Protection**  
  Set minimum deposit amounts and maximum vault TVL to prevent spam and manage risk exposure.

---

## 🔧 Core Program Concepts

| Concept                   | Description |
|:---------------------------|:------------|
| **Vault**                  | Main account managing all parameters, token vaults, and treasury settings. |
| **Share Token (LP Token)**  | Represents ownership in the vault; minted to users when depositing. |
| **Base and Quote Vaults**   | Token accounts holding the pooled liquidity assets. |
| **Bands**                  | Price intervals (basis points) that trigger rebalancing events. |
| **Treasury Address**        | Receives protocol fees from vault operations. |
| **Admins**                 | Additional authorized operators who can manage and rebalance the vault. |

---

## 🛠 Functions

### Initialization

- `initialize(fee_bps, band_size_bp)`
  - Sets up the vault, mints, and token vaults.
  - Configures fees, treasury, and price band settings.

### Deposit / Withdraw

- `deposit(base_amount, quote_amount)`
  - User deposits base and quote tokens.
  - Vault mints LP share tokens proportional to deposit size.

- `withdraw(share_amount)`
  - User burns LP shares to redeem underlying tokens.
  - Withdrawal fees (optional) may be applied.

### Vault Management

- `setPause(paused)`
  - Pauses or unpauses vault operations (deposits, rebalances).

- `updateFee(fee_bps)`
  - Updates the trading fee percentage.

- `setWithdrawFee(fee_bps)`
  - Updates the withdrawal fee percentage.

- `addAdmin(admin_pubkey)` / `removeAdmin(admin_pubkey)`
  - Manage vault administrators.

- `addDepositWhitelist(user_pubkey)` / `removeDepositWhitelist(user_pubkey)`
  - Manage deposit whitelist access.

- `addWithdrawWhitelist(user_pubkey)` / `removeWithdrawWhitelist(user_pubkey)`
  - Manage withdraw whitelist access.

### Liquidity Operations

- `rebalance(current_price)`
  - Rebalances liquidity between bands based on latest price feed.
  - Clears accrued trading fees.

- `sweepFees()`
  - Transfers accrued fees from the vault token accounts to the treasury account.

---
