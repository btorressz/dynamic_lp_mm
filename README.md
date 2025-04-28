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
