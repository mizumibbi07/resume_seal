# resume_seal

## Project Title
resume_seal

## Project Description
Resume fraud is a growing problem in hiring: candidates embellish or fabricate work history, degrees, or skills, and employers have no easy way to verify a CV without making dozens of phone calls. `resume_seal` is a Soroban smart contract that lets a trusted issuer — a former employer, university, or certification body — write a tamper-proof on-chain seal over a candidate's resume hash. A recruiter (or any third-party app) can then ask the contract whether that seal is currently valid, expired, or revoked, and which organization issued it. The result is a portable, time-bounded proof of resume authenticity that travels with the candidate across borders, employers, and decades of career change.

## Project Vision
`resume_seal` aims to be the trust layer for professional identity on Stellar: a minimal, composable primitive that any HR platform, recruiting marketplace, or background-check service can call to attest — or verify — the authenticity of a CV. The long-term goal is a global, user-owned registry of sealed credentials where every job, degree, or skill comes with a cryptographic receipt that can be checked in one query and revoked in one transaction when something turns out to be false.

## Key Features
- **`seal`** — an authorized employer or school binds a 32-byte resume hash to a candidate's address with a chosen `valid_until` timestamp.
- **`revoke`** — the original sealer can invalidate a seal at any time with a symbolic reason (e.g. `"falsified"`, `"withdrawn"`); the seal then reads as `revoked` regardless of expiry.
- **`renew`** — extends the validity window of a non-revoked seal without changing the sealer or the underlying hash.
- **`verify`** — returns one of four numeric statuses (`none`, `valid`, `expired`, `revoked`) based on the chain's ledger timestamp and stored flags; read-only, no signature required.
- **`is_valid`** — boolean convenience wrapper over `verify` for UIs that just need a green / red indicator.
- **`get_employer`** — returns the `Address` of the sealer so a recruiter can display "verified by <organization>" next to a valid seal.
- **Auth-gated writes** — every state-changing entry point calls `require_auth()` on the sealer, so no one but the original issuer can revoke or renew a seal.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** work dApp — see `contracts/resume_seal/src/lib.rs` for the full resume_seal business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CAFU33UCDQS4QRG2EWW56IROJLABWM4MQONQUDPSBLPTNERIKZVFXTBN`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/f2978d0bf159769ca43798ce94e9f5f295b25ffb3518b9879d8cbb684b0c9bf4`

## Future Scope
- **Multi-sealer attestations** — require N independent seals (e.g. school + 2 previous employers) before a resume reads as fully verified.
- **Off-chain resume pointer** — extend `resume_hash` to a content-addressed pointer (IPFS / Arweave CID) so the full CV is retrievable from the seal.
- **Frontend dApp** — a small React/Freighter UI for candidates to mint their first seal and for recruiters to verify a CV by pasting a hash.
- **Privacy-preserving verification** — optional ZK layer so a candidate can prove a seal exists without revealing which employer issued it.
- **Structured revocation reasons** — replace the single `Symbol reason` with a richer record (timestamp, referee address, optional note) stored as a nested `contracttype`.
- **Mainnet launch with issuer fees** — a small, clawback-friendly credit charged per seal to deter spam and fund a public-goods treasury for the registry.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `resume_seal` (work)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
