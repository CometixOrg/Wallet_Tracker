# Wallet Tracker

**The Wallet Tracker** is an open-source tool for monitoring wallet activity in Web3.  
Integrated into the **Cometix Security Web Extension**, it tracks wallet actions in real time  
on Ethereum and Solana blockchains without using artificial intelligence. The tool  
observes transactions, dApp connections, and balance changes, providing instant  
notifications about unusual activities to help users and developers maintain secure  
decentralized interactions.

## Key Features

- **Real-Time Transaction Monitoring** *(Open Source)*  
  Tracks wallet transactions in real time, capturing incoming and outgoing transfers,  
  with instant notifications.

- **Balance Change Detection** *(Open Source)*  
  Observes wallet balance fluctuations, notifying users of unexpected changes,  
  such as sudden withdrawals, for quick response.

- **Anomaly Alerts** *(Open Source)*  
  Identifies unusual wallet activities, like irregular transaction patterns, with prompt  
  warnings to mitigate risks.

- **Activity Log Compilation**  
  Maintains a detailed record of wallet transactions and connections, presenting  
  clear summaries to support secure operations.

- **Instant Notification System**  
  Sends real-time alerts about wallet events, ensuring timely awareness for  
  enhanced Web3 security.

---

## Architecture

```text
                  ┌────────────────────────────┐
                  │    Cometix Web Extension   │
                  └────────────┬───────────────┘
                               │
        ┌──────────────────────▼──────────────────────┐
        │        Wallet Tracker (Rust + WASM)         │
        │  - Transaction Listener                     │
        │  - Balance Delta Module                     |
        |  - Notification Module                      │
        │  - Anomaly Detector (State Machine)         |
        │  - AI Module                                |
        └──────────────────────┬──────────────────────┘
                               │
              ┌────────────────▼─────────────────┐
              │      Ethereum / Solana RPC       │
              └──────────────────────────────────┘
```

---

### Requirements

| Tool                  | Version      |
|-----------------------|--------------|
| `cometix-sdk`         | `1.0.16`     |
| `solana-instruction`  | `>=2.2.1`    |
| `cometix-security`    | `1.0.52`     |
| `cometix-utils`       | `1.1.12`     |
| `wasm-pack`           | `>=0.10.2`   |
| `wasm-pack`           | `>=0.10.2`   |
| `Rust`                | `>=1.70.0`   |
| `Node.js`             | `>=18.0.0`   |
| `Vite`                | `>=4.0.0`    |

---
## Local Development

Install Rust toolchain:

```bash
curl https://sh.rustup.rs -sSf | sh
cargo install wasm-pack
```

Install frontend deps:

```bash
cd frontend/
npm install
```

## Building the WASM Module

```bash
cd wallet-tracker/
wasm-pack build --target web --out-dir ../output/out
```

Flags:
- `--target web`: for browser-based WASM
- `--out-dir`: exports to frontend for integration
