# Streamora

> **Money that moves with time.**

Streamora is open-source streaming-payments infrastructure built on Stellar using Soroban smart contracts. It enables time-based payments where payers create payment streams and recipients withdraw accumulated amounts as time progresses.

## Vision

Streaming payments are fundamental to modern finance—payroll, vendor contracts, and subscriptions all benefit from continuous accrual rather than lump-sum transactions. Streamora brings this capability to Stellar and the broader blockchain ecosystem as a transparent, auditable, open-source platform.

## Features

- ✅ **Linear Time-Based Vesting**: Smart contracts calculate withdrawable amounts based on elapsed time.
- ✅ **Stellar Native**: Built on Soroban smart contracts using Rust.
- ✅ **TypeScript SDK**: Clean, typed SDK for contract interaction.
- ✅ **React Application**: Dashboard for creating and managing streams.
- ✅ **Open Source**: MIT licensed. Community-driven development.
- ✅ **Multi-recipient streams**: 1–10 recipients per stream, with percentage shares totaling 100%.
- ✅ **Cancellation and refunds**: Payers may cancel, reclaim the unvested portion, and leave vested amounts available to recipients.
- ✅ **Freighter wallet flow**: Connect a wallet, sign transactions, and track their status.
- 🔄 **Planned**: Batch operations, recurring payment patterns, and additional production hardening.

## Quick Start

### Prerequisites

- **Node.js 18+** and npm
- **Recent stable Rust** for contract tests (optional for web development)
- **Freighter on Stellar testnet** for wallet-backed features (optional for public pages)

### Local Development

```bash
git clone https://github.com/Strea-mora/Streamora.git
cd Streamora
npm ci
# Vite runs from apps/web; place local configuration in that workspace.
cp .env.example apps/web/.env.local
npm run dev
```

Open the URL shown by Vite (typically http://localhost:5173). Edit `apps/web/.env.local` for your testnet settings. A root-level `.env.local` alone will **not** be loaded by the web app. The example contract/token IDs are testnet values, not production addresses.

### Running Tests

From the repository root:

```bash
npm test
npm run lint
npm run build
```

To run Rust contract tests separately:

```bash
(cd contracts/streaming && cargo test)
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for workspace-specific commands.

## Project Structure

```text
Streamora/
├── .github/
│   └── workflows/ci.yml
├── apps/
│   └── web/
│       ├── index.html
│       ├── package.json
│       ├── src/
│       │   ├── __tests__/
│       │   ├── components/
│       │   ├── contexts/
│       │   ├── hooks/
│       │   ├── lib/
│       │   ├── pages/
│       │   │   ├── app/
│       │   │   └── docs/
│       │   ├── styles/
│       │   └── App.tsx
│       ├── vite.config.ts
│       └── vitest.config.ts
├── contracts/
│   ├── Cargo.toml
│   └── streaming/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── test.rs
├── docs/
│   ├── architecture.md
│   ├── deployment.md
│   ├── frontend.md
│   ├── protocol-spec.md
│   └── sdk.md
├── scripts/
│   ├── copy-dist.mjs
│   └── deploy-testnet.sh
├── sdk/
│   ├── package.json
│   ├── src/
│   │   ├── client.ts
│   │   ├── errors.ts
│   │   ├── index.ts
│   │   ├── types.ts
│   │   ├── units.ts
│   │   └── utils.ts
│   └── tests/
│       └── sdk.test.ts
├── .env.example
├── CONTRIBUTING.md
├── DEPLOY.md
├── README.md
├── SECURITY.md
└── package.json
```

The tree summarizes tracked directories and key entry points from Git; some implementation and configuration files are omitted for readability.

## Contract MVP Specification

### Creating a Stream

```text
Stream {
  payer: Address,
  recipients: Vec<Recipient>, // each: { address, shares, withdrawn }
  token: Address,
  amount: i128,
  start_time: u64,
  end_time: u64,
  cancelled_at: Option<u64>,
}
// Stream IDs are u64 keys stored by the contract.
```

### Vesting Formula

At time `t`:
- If `t ≤ start_time`: vested = 0
- If `t ≥ end_time`: vested = amount
- Otherwise: vested = `floor(amount * (t - start_time) / (end_time - start_time))`

For each recipient: entitlement = floor(vested × shares / 100); withdrawable = entitlement − that recipient's withdrawn amount. Cancellation freezes vesting at the cancellation timestamp.

### Key Functions

- `create_stream()`: Escrow tokens for 1–10 recipients
- `withdraw()`: Recipient claims an available amount
- `get_stream()`: View stream details
- `get_vested_amount()`: Calculate linear vesting (ignores cancellation)
- `get_withdrawable()`: Calculate currently claimable amount for a recipient
- `cancel_stream()`: Payer freezes vesting and receives the unvested refund

## Routes

### Public Pages

- `/` – Landing page with hero and features
- `/how-it-works` – Stream mechanics explained
- `/features` – Feature list and roadmap
- `/use-cases` – Real-world examples
- `/developers` – Developer entry point
- `/docs` – Documentation hub
- `/about` – Project mission
- `/community` – Links and contribution info

### Application Routes (Authenticated)

- `/app` – Dashboard
- `/app/streams` – Stream list
- `/app/streams/create` – Create new stream
- `/app/streams/:id` – Stream details
- `/app/streams/:id/withdraw` – Withdrawal interface
- `/app/activity` – Transaction history
- `/app/settings` – Network and preferences

## Documentation

- **[Architecture](./docs/architecture.md)** – System design and contract architecture
- **[Protocol Spec](./docs/protocol-spec.md)** – Detailed stream specification
- **[Security](./SECURITY.md)** – Security assumptions and threat model
- **[Frontend](./docs/frontend.md)** – UI/UX patterns and component library
- **[SDK](./docs/sdk.md)** – TypeScript SDK API reference
- **[Deployment](./docs/deployment.md)** – Testnet and mainnet setup

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Security

See [SECURITY.md](./SECURITY.md) for security disclosure and known limitations.

## License

MIT. See LICENSE file.

## Status

**Alpha (testnet-oriented, unaudited).** Contract, SDK, and UI interfaces are evolving; do not assume production readiness.

---

Built with ❤️ on Stellar. [Contribute](./CONTRIBUTING.md) | [Report Issues](https://github.com/Strea-mora/Streamora/issues)
