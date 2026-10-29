# Architecture

## System Overview

Streamora consists of three main layers:

### 1. Smart Contract Layer (Soroban)

The `streaming` contract implements the core payment streaming logic:

- **Stream Creation**: Payers initialize streams with recipient, token, amount, and time bounds
- **Vesting Calculation**: Linear vesting based on elapsed time
- **Withdrawal Mechanism**: Recipients claim accrued amounts
- **State Management**: Tracks stream status and withdrawal history

**Key Features:**
- Time-based linear vesting
- Per-stream withdrawal tracking
- Authorization checks (payer/recipient validation)
- Token transfer integration

### 2. SDK Layer (TypeScript)

The `@streamora/sdk` package provides:

- **Type Definitions**: Interfaces for Stream, StreamInfo, and configuration
- **Client API**: StreamoraClient class for contract interaction
- **Calculations**: Vesting amount, withdrawable amount, time-based logic
- **Utilities**: Address validation, formatting, unit conversion
- **No Private Key Storage**: SDK prepares transactions; caller controls signing

**Key Interfaces:**
```typescript
interface Stream {
  id: string;
  payer: string;
  recipient: string;
  token: string;
  amount: bigint;
  startTime: number;
  endTime: number;
  withdrawn: bigint;
  isCancelled: boolean;
}
```

### 3. Frontend Layer (React + Vite)

The web application provides:

- **Landing Pages**: Hero, features, use cases, documentation
- **Application Dashboard**: Stream management and withdrawal interface
- **Wallet Integration**: Freighter or compatible Stellar wallet connection
- **Transaction Workflow**: Create, withdraw, manage streams

**Architecture:**
- React Router for navigation
- TypeScript for type safety
- Tailwind CSS for styling
- React Query or equivalent for data fetching
- State management (React Context or lightweight store)

## Data Flow

### Stream Creation

1. User connects wallet (Freighter)
2. Frontend calls SDK `prepareCreateStream()`
3. SDK validates inputs and returns transaction data
4. Frontend presents summary to user
5. User signs transaction via wallet
6. Frontend submits signed transaction
7. Contract executes, creates stream, transfers token
8. Frontend displays confirmation

### Withdrawal

1. User selects stream from dashboard
2. Frontend calculates withdrawable amount using SDK
3. User specifies withdrawal amount
4. SDK validates withdrawal amount
5. User signs withdrawal transaction
6. Contract verifies eligibility and updates withdrawn counter
7. Token transferred to recipient
8. Frontend confirms success

## Deployment Architecture

### Testnet
- Deployed to Stellar Futurenet or Testnet
- Used for development and testing
- No real funds at risk

### Mainnet (Future)
- Requires security audit
- Mainnet contract deployment
- Production-grade monitoring

## Contract State

Each stream maintains:
- Stream metadata (parties, token, amount, times)
- Withdrawal history (withdrawn amount)
- Cancellation status
- Owner/authorization information

## Storage & TTL

Soroban contract data lives in ledger entries with a time-to-live (TTL). Entries whose TTL runs out are archived and must be restored before use. To prevent long-running streams (such as payroll and vesting) from expiring before recipients withdraw, the contract extends TTLs on each state change:

- **Stream entries** (`DataKey::Stream`): TTL is extended in `save()` and `load()` using constants `STREAM_TTL_THRESHOLD` (100,000 ledgers) and `STREAM_TTL_EXTEND_TO` (200,000 ledgers). The extension only applies when the current TTL is at or below the threshold.
- **Contract instance storage** (`DataKey::Counter`): TTL is extended in `create_stream()`, `withdraw()`, and `cancel_stream()` using `env.storage().instance().extend_ttl(threshold, extend_to)`.

The chosen threshold/extend-to values (100k/200k ledgers ≈ 5.7/11.6 days at ~5s/ledger) cover typical stream durations while keeping TTL management efficient. Each write/read operation implicitly extends the TTL, so as long as the contract remains active, entries stay alive.

### TTL Constants

| Constant | Value | Description |
|---|---|---|
| `STREAM_TTL_THRESHOLD` | 100,000 ledgers | Extend TTL if current TTL ≤ this value |
| `STREAM_TTL_EXTEND_TO` | 200,000 ledgers | New TTL value after extension |

## SDK vs. Smart Contract Responsibilities

**SDK Handles:**
- Input validation (address format, amount ranges, time logic)
- Vesting calculations
- Formatting and display logic
- Transaction preparation

**Contract Handles:**
- Authorization (who can call what)
- Token transfers
- State mutations
- Blockchain-specific validation

## Network Configuration

The SDK supports multiple Soroban networks:
- `testnet`: Stellar Testnet
- `public`: Stellar Public Mainnet
- `standalone`: Local Soroban instance

Configuration passed at client initialization.

## Testing Strategy

- **Contract Tests**: Rust tests in `contracts/streaming/src/test.rs`
- **SDK Tests**: Vitest suite in `sdk/tests/`
- **Integration Tests**: Frontend + SDK + Mock Contract
- **End-to-End**: On testnet with live contract

## Security Considerations

1. **No Private Keys in Frontend**: Wallet provider handles signing
2. **Amount Overflow Prevention**: Use bigint for token calculations
3. **Authorization Checks**: Only eligible parties can withdraw
4. **Time Boundary Validation**: Timestamps checked before vesting
5. **Reentrancy**: Not applicable to Soroban invocations
6. **Token Decimals**: Handled consistently in calculations

See [SECURITY.md](../SECURITY.md) for known limitations.
