#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

/// `verify` return value: no seal exists for this candidate/hash.
pub const STATUS_NONE: u32 = 0;
/// `verify` return value: a non-revoked seal whose `valid_until` has not passed.
pub const STATUS_VALID: u32 = 1;
/// `verify` return value: a non-revoked seal whose `valid_until` has passed.
pub const STATUS_EXPIRED: u32 = 2;
/// `verify` return value: a seal that was explicitly revoked by its sealer.
pub const STATUS_REVOKED: u32 = 3;

/// Default reason string written to a fresh, non-revoked seal.
const DEFAULT_REASON: &str = "active";

/// On-chain record of a single resume seal, stored under the
/// composite key `(candidate, resume_hash)`.
#[contracttype]
#[derive(Clone)]
pub struct SealData {
    /// Address of the employer / school that issued the seal.
    pub employer: Address,
    /// Ledger timestamp (seconds) at which the seal stops being valid.
    pub valid_until: u64,
    /// `true` if the original sealer has called `revoke` on this seal.
    pub revoked: bool,
    /// Short symbolic explanation; meaningful only when `revoked` is `true`.
    pub reason: Symbol,
}

/// `resume_seal` — a lightweight on-chain registry where a trusted issuer
/// (a former employer, university, or certification body) writes a
/// cryptographic seal over a candidate's resume hash. Recruiters can
/// later query the contract to check whether a seal is valid, expired,
/// or revoked, and to identify the original sealer.
#[contract]
pub struct ResumeSeal;

#[contractimpl]
impl ResumeSeal {
    /// Seal a candidate's resume hash.
    ///
    /// The `employer` (school, former employer, certification body, ...)
    /// authorizes the transaction; `candidate` is the person whose CV is
    /// being attested to. `resume_hash` is a 32-byte fingerprint of the
    /// resume document (e.g. SHA-256 of the PDF). `valid_until` is a
    /// ledger timestamp (seconds) past which the seal will read as
    /// `expired`. The seal is non-revocable from this call's perspective:
    /// it is active and un-revoked when written.
    ///
    /// Panics if a seal already exists for `(candidate, resume_hash)` or
    /// if `valid_until` is not strictly in the future.
    pub fn seal(
        env: Env,
        employer: Address,
        candidate: Address,
        resume_hash: BytesN<32>,
        valid_until: u64,
    ) {
        employer.require_auth();

        let key = (candidate, resume_hash);
        if env.storage().persistent().has(&key) {
            panic!("Seal already exists for this candidate and resume hash");
        }

        let now = env.ledger().timestamp();
        if valid_until <= now {
            panic!("valid_until must be strictly in the future");
        }

        let seal = SealData {
            employer,
            valid_until,
            revoked: false,
            reason: Symbol::new(&env, DEFAULT_REASON),
        };

        env.storage().persistent().set(&key, &seal);
    }

    /// Revoke a previously issued seal.
    ///
    /// Only the original sealer can revoke their own seal. `reason` is a
    /// short symbolic explanation (e.g. `"falsified"`, `"withdrawn"`,
    /// `"data_correction"`) that verifiers can inspect by reading the
    /// stored `SealData` directly. Once revoked, a seal will read as
    /// `STATUS_REVOKED` from `verify` regardless of its expiry.
    ///
    /// Panics if no seal exists or if `employer` is not the original sealer.
    pub fn revoke(
        env: Env,
        employer: Address,
        candidate: Address,
        resume_hash: BytesN<32>,
        reason: Symbol,
    ) {
        employer.require_auth();

        let key = (candidate, resume_hash);
        let mut seal: SealData = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Seal not found");

        if seal.employer != employer {
            panic!("Only the original sealer can revoke this seal");
        }

        seal.revoked = true;
        seal.reason = reason;
        env.storage().persistent().set(&key, &seal);
    }

    /// Inspect the current status of a seal.
    ///
    /// Returns one of:
    /// * `STATUS_NONE`     (0) — no seal exists for this key
    /// * `STATUS_VALID`    (1) — non-revoked and not yet expired
    /// * `STATUS_EXPIRED`  (2) — non-revoked but past `valid_until`
    /// * `STATUS_REVOKED`  (3) — explicitly revoked by the sealer
    ///
    /// This is a read-only call: no signature is required.
    pub fn verify(env: Env, candidate: Address, resume_hash: BytesN<32>) -> u32 {
        let key = (candidate, resume_hash);
        let seal: SealData = match env.storage().persistent().get(&key) {
            Some(s) => s,
            None => return STATUS_NONE,
        };

        if seal.revoked {
            return STATUS_REVOKED;
        }

        let now = env.ledger().timestamp();
        if now > seal.valid_until {
            return STATUS_EXPIRED;
        }

        STATUS_VALID
    }

    /// Return the `Address` of the employer that issued the seal.
    ///
    /// Panics if no seal exists for `(candidate, resume_hash)`. Read-only;
    /// no signature required. Useful for recruiters who want to display
    /// "verified by <employer>" alongside a valid seal.
    pub fn get_employer(env: Env, candidate: Address, resume_hash: BytesN<32>) -> Address {
        let key = (candidate, resume_hash);
        let seal: SealData = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Seal not found");
        seal.employer
    }

    /// Renew a non-revoked seal with a new expiry timestamp.
    ///
    /// Only the original sealer can renew. A revoked seal cannot be
    /// renewed — a fresh `seal` call would be required in that case.
    /// The new `valid_until` must be strictly in the future.
    ///
    /// Panics if no seal exists, the caller is not the original sealer,
    /// the seal is revoked, or `new_valid_until` is not in the future.
    pub fn renew(
        env: Env,
        employer: Address,
        candidate: Address,
        resume_hash: BytesN<32>,
        new_valid_until: u64,
    ) {
        employer.require_auth();

        let key = (candidate, resume_hash);
        let mut seal: SealData = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Seal not found");

        if seal.employer != employer {
            panic!("Only the original sealer can renew this seal");
        }

        if seal.revoked {
            panic!("Cannot renew a revoked seal");
        }

        let now = env.ledger().timestamp();
        if new_valid_until <= now {
            panic!("new_valid_until must be strictly in the future");
        }

        seal.valid_until = new_valid_until;
        env.storage().persistent().set(&key, &seal);
    }

    /// Quick boolean check: `true` iff `verify` returns `STATUS_VALID`.
    ///
    /// Read-only; no signature required. Handy for UIs that only need a
    /// green / red indicator.
    pub fn is_valid(env: Env, candidate: Address, resume_hash: BytesN<32>) -> bool {
        Self::verify(env, candidate, resume_hash) == STATUS_VALID
    }
}
