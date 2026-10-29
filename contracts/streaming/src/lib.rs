#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidTimes = 2,
    InvalidRecipients = 3,
    StreamNotFound = 4,
    NotRecipient = 5,
    NothingToWithdraw = 6,
    AlreadyCancelled = 7,
    AmountExceedsWithdrawable = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recipient {
    pub address: Address,
    /// Percentage of the stream, 1..=100. All shares must sum to 100.
    pub shares: u32,
    pub withdrawn: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub payer: Address,
    pub recipients: Vec<Recipient>,
    pub token: Address,
    pub amount: i128,
    pub start_time: u64,
    pub end_time: u64,
    /// Set when the payer cancels; vesting freezes at this timestamp.
    pub cancelled_at: Option<u64>,
}

#[contracttype]
enum DataKey {
    Counter,
    Stream(u64),
}

// TTL constants for stream entries and contract instance.
// Soroban ledger entries have a time-to-live (TTL); when the TTL expires,
// the entry is archived and must be restored before use. By extending the TTL
// on each write/read, we keep active streams alive for long-running use cases
// such as payroll and vesting streams.
// - threshold: the TTL (in ledgers) below which we extend the entry.
//   A typical Soroban ledger is ~5 seconds; 100_000 ledgers ≈ ~5.7 days.
// - extend_to: the new TTL (in ledgers) to set when extending.
//   200_000 ledgers ≈ ~11.6 days, which covers typical stream durations
//   while keeping the TTL manageable. The extension is only applied when
//   the current TTL is at or below the threshold, so entries with a larger
//   TTL are left unchanged.
const STREAM_TTL_THRESHOLD: u32 = 100_000;
const STREAM_TTL_EXTEND_TO: u32 = 200_000;

/// floor(amount * (t - start) / (end - start)), clamped to [0, amount].
pub fn vested_at(amount: i128, start: u64, end: u64, t: u64) -> i128 {
    if t <= start {
        0
    } else if t >= end {
        amount
    } else {
        amount * ((t - start) as i128) / ((end - start) as i128)
    }
}

fn effective_time(s: &Stream, now: u64) -> u64 {
    match s.cancelled_at {
        Some(c) if c < now => c,
        _ => now,
    }
}

fn load(env: &Env, id: u64) -> Result<Stream, Error> {
    let stream = env.storage()
        .persistent()
        .get(&DataKey::Stream(id))
        .ok_or(Error::StreamNotFound)?;
    // Extend TTL to keep the entry alive for long-running streams
    env.storage().persistent().extend_ttl(
        &DataKey::Stream(id),
        STREAM_TTL_THRESHOLD,
        STREAM_TTL_EXTEND_TO,
    );
    Ok(stream)
}

fn save(env: &Env, id: u64, s: &Stream) {
    env.storage().persistent().set(&DataKey::Stream(id), s);
    env.storage().persistent().extend_ttl(
        &DataKey::Stream(id),
        STREAM_TTL_THRESHOLD,
        STREAM_TTL_EXTEND_TO,
    );
}

fn entitlement(s: &Stream, r: &Recipient, now: u64) -> i128 {
    let v = vested_at(s.amount, s.start_time, s.end_time, effective_time(s, now));
    v * (r.shares as i128) / 100
}

#[contract]
pub struct StreamingContract;

#[contractimpl]
impl StreamingContract {
    /// Creates a stream, escrows `amount` of `token` from `payer`, returns the stream id.
    pub fn create_stream(
        env: Env,
        payer: Address,
        recipients: Vec<(Address, u32)>,
        token: Address,
        amount: i128,
        start_time: u64,
        end_time: u64,
    ) -> Result<u64, Error> {
        payer.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if start_time >= end_time || end_time <= env.ledger().timestamp() {
            return Err(Error::InvalidTimes);
        }
        if recipients.is_empty() || recipients.len() > 10 {
            return Err(Error::InvalidRecipients);
        }
        let mut total: u32 = 0;
        let mut rs: Vec<Recipient> = Vec::new(&env);
        for (address, shares) in recipients.iter() {
            if shares == 0 || shares > 100 {
                return Err(Error::InvalidRecipients);
            }
            total += shares;
            rs.push_back(Recipient { address, shares, withdrawn: 0 });
        }
        if total != 100 {
            return Err(Error::InvalidRecipients);
        }

        token::Client::new(&env, &token).transfer(&payer, &env.current_contract_address(), &amount);

        let id: u64 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0) + 1;
        env.storage().instance().set(&DataKey::Counter, &id);
        env.storage().instance().extend_ttl(STREAM_TTL_THRESHOLD, STREAM_TTL_EXTEND_TO);
        let stream = Stream {
            payer: payer.clone(),
            recipients: rs,
            token,
            amount,
            start_time,
            end_time,
            cancelled_at: None,
        };
        save(&env, id, &stream);
        env.events().publish((symbol_short!("created"), id), (payer, amount));
        Ok(id)
    }

    pub fn get_stream(env: Env, id: u64) -> Result<Stream, Error> {
        load(&env, id)
    }

    /// Total vested for the whole stream at `at_time` (ignores cancellation).
    pub fn get_vested_amount(env: Env, id: u64, at_time: u64) -> Result<i128, Error> {
        let s = load(&env, id)?;
        Ok(vested_at(s.amount, s.start_time, s.end_time, at_time))
    }

    /// Amount `who` can withdraw right now.
    pub fn get_withdrawable(env: Env, id: u64, who: Address) -> Result<i128, Error> {
        let s = load(&env, id)?;
        for r in s.recipients.iter() {
            if r.address == who {
                let e = entitlement(&s, &r, env.ledger().timestamp());
                return Ok((e - r.withdrawn).max(0));
            }
        }
        Err(Error::NotRecipient)
    }

    pub fn withdraw(env: Env, id: u64, who: Address, amount: i128) -> Result<(), Error> {
        who.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let mut s = load(&env, id)?;
        let now = env.ledger().timestamp();
        let mut idx: Option<u32> = None;
        for (i, r) in s.recipients.iter().enumerate() {
            if r.address == who {
                idx = Some(i as u32);
            }
        }
        let i = idx.ok_or(Error::NotRecipient)?;
        let mut r = s.recipients.get(i).unwrap();
        let available = (entitlement(&s, &r, now) - r.withdrawn).max(0);
        if available == 0 {
            return Err(Error::NothingToWithdraw);
        }
        if amount > available {
            return Err(Error::AmountExceedsWithdrawable);
        }
        r.withdrawn += amount;
        s.recipients.set(i, r);
        save(&env, id, &s);
        env.storage().instance().extend_ttl(STREAM_TTL_THRESHOLD, STREAM_TTL_EXTEND_TO);
        token::Client::new(&env, &s.token).transfer(&env.current_contract_address(), &who, &amount);
        env.events().publish((symbol_short!("withdrew"), id), (who, amount));
        Ok(())
    }

    /// Payer-only, any time. Vesting freezes now; the unvested remainder is
    /// refunded to the payer. Recipients keep their vested share and may still withdraw it.
    pub fn cancel_stream(env: Env, id: u64) -> Result<i128, Error> {
        let mut s = load(&env, id)?;
        s.payer.require_auth();
        if s.cancelled_at.is_some() {
            return Err(Error::AlreadyCancelled);
        }
        let now = env.ledger().timestamp();
        let vested = vested_at(s.amount, s.start_time, s.end_time, now);
        // Rounding dust from per-recipient floor division stays escrowed with the recipients' side.
        let refund = s.amount - vested;
        s.cancelled_at = Some(now);
        save(&env, id, &s);
        env.storage().instance().extend_ttl(STREAM_TTL_THRESHOLD, STREAM_TTL_EXTEND_TO);
        if refund > 0 {
            token::Client::new(&env, &s.token).transfer(
                &env.current_contract_address(),
                &s.payer,
                &refund,
            );
        }
        env.events().publish((symbol_short!("cancelled"), id), refund);
        Ok(refund)
    }
}

#[cfg(test)]
mod test;
