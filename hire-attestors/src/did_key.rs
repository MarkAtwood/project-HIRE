//! DID attestor — discovers did:key identifiers from environment configuration
//! and proves one through whichever local agent holds the matching secret.
//!
//! No network I/O. Reads HIRE_DID_KEYS (whitespace-separated did:key URIs).
//!
//! TWO PLACES THE SECRET CAN LIVE, and the preference between them is the
//! design. `did:key` encodes the verification method in the identifier itself,
//! so the identifier says which key must answer without saying where that key
//! lives.
//!
//! First choice is an agent, because hire would rather not hold a secret at
//! all: every other source here is a client of something that does, and a key
//! in the ssh agent is one hire can use without ever seeing it. A `did:key`
//! whose point matches an agent key is proved through the agent.
//!
//! Second is [`crate::keystore`], the drop directory. That exists because the
//! first choice is unreachable for the person `did:key` is actually for: a DID
//! holder's key is a file, `ssh-add` refuses a PKCS#8 PEM ed25519 key outright,
//! and OpenSSH stores ed25519 private keys only in its own format — so "put it
//! in your agent" was advice nobody could follow. The keystore owns the disk,
//! the permission rules and the format, so this attestor does not.
//!
//! The two are not equivalent and the candidate says which it got: a dropped
//! key is [`ProofCost::Silent`], because nothing can prompt; an agent key is
//! [`ProofCost::Interactive`], because `ssh-add -c` prompts and the agent does
//! not report that constraint.
//!
//! HIRE_DID_KEYS STILL NAMES PUBLIC IDENTIFIERS. A DID configured there and
//! held nowhere local enumerates and does not prove, which is the model
//! working — some other attestor may yet be able to prove it.
//!
//! One physical key can therefore answer as two identities — `key/<fingerprint>`
//! from ssh-agent and `did/<hash>` from here. That is what a naming layer is;
//! the two SPIFFE IDs are distinct subjects and a consumer authorises whichever
//! it was given.
//!
// ponytail: did:web HTTP resolution | upgrade path: reqwest + did-web resolver once networking is desired
// ponytail: ed25519 did:key only (multicodec 0xed01, the z6Mk form) | ceiling: a
//   did:key naming a P-256 or secp256k1 key enumerates and never proves |
//   upgrade path: a sibling verifying constructor per curve in claim.rs, beside
//   the ssh one, never a second verifier here

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::claim::ChallengeSignature;
use crate::{keystore, ssh};
use crate::{
    AttainableAssurance, Attestor, AttestorError, Candidate, Evidence, ProofCost,
    SelfAssertedDomain,
};

/// Bitcoin's base58 alphabet, which multibase `z` names.
const BASE58BTC: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// The multicodec prefix for an ed25519 public key: `0xed 0x01`, varint-encoded.
const ED25519_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// The decoded length of a `did:key` naming an ed25519 key: 2 prefix + 32 key.
const ED25519_DID_LEN: usize = 34;

/// Attestor that returns did:key identifiers configured via HIRE_DID_KEYS.
#[derive(Debug)]
pub struct DidKeyAttestor;

impl DidKeyAttestor {
    pub fn new() -> Self {
        Self
    }

    /// Returns true if any did:key identifier is configured or dropped.
    pub fn is_available() -> bool {
        std::env::var("HIRE_DID_KEYS")
            .map(|v| v.split_whitespace().any(|s| s.starts_with("did:key:")))
            .unwrap_or(false)
            || !keystore::ed25519_keys().is_empty()
    }
}

impl Default for DidKeyAttestor {
    fn default() -> Self {
        Self::new()
    }
}

/// The ed25519 public key a `did:key` encodes, or `None` if it encodes anything
/// else.
///
/// `did:key:z<base58btc(multicodec || key)>`. The decode is a fixed-width
/// bignum over exactly [`ED25519_DID_LEN`] bytes, which is the whole length
/// check: a longer identifier carries out of the buffer and a shorter one
/// leaves leading zeros where the multicodec prefix has to be, so both fail
/// without a separate rule. Nothing here accepts a `did:key` for another curve
/// — the prefix comparison is the one place that decides, and a P-256 key
/// (`0x12 0x00`) is refused there rather than verified against the wrong
/// algorithm later.
pub(crate) fn ed25519_point(did: &str) -> Option<[u8; 32]> {
    let suffix = did.strip_prefix("did:key:z")?;

    let mut decoded = [0u8; ED25519_DID_LEN];
    for c in suffix.bytes() {
        let mut carry = BASE58BTC.iter().position(|&a| a == c)? as u32;
        for byte in decoded.iter_mut().rev() {
            carry += 58 * u32::from(*byte);
            *byte = carry as u8;
            carry >>= 8;
        }
        if carry != 0 {
            return None;
        }
    }

    let (prefix, key) = decoded.split_at(ED25519_MULTICODEC.len());
    (prefix == ED25519_MULTICODEC).then(|| key.try_into().expect("32 of 34 bytes"))
}

/// The `did:key` identifier an ed25519 public key names.
///
/// The inverse of [`ed25519_point`], and it exists because a dropped key has to
/// be able to say what identity it is: the operator wrote a key file, not an
/// identifier. Written here rather than in `keystore` so that both directions
/// of the encoding sit in one file and cannot drift.
pub(crate) fn did_key_of(point: &[u8; 32]) -> String {
    let mut payload = Vec::with_capacity(ED25519_DID_LEN);
    payload.extend_from_slice(&ED25519_MULTICODEC);
    payload.extend_from_slice(point);

    // Base-58 digits, least significant first: the standard big-endian-bytes to
    // base58 conversion. No leading-zero handling is needed for the values this
    // produces -- the multicodec prefix starts 0xed -- and writing the general
    // form anyway would be a branch no input reaches and no test could cover.
    let mut digits: Vec<u8> = Vec::new();
    for byte in payload {
        let mut carry = u32::from(byte);
        for digit in digits.iter_mut() {
            let acc = u32::from(*digit) * 256 + carry;
            *digit = (acc % 58) as u8;
            carry = acc / 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    let mut did = String::from("did:key:z");
    did.extend(
        digits
            .iter()
            .rev()
            .map(|&d| char::from(BASE58BTC[usize::from(d)])),
    );
    did
}

/// The SPIFFE path a DID maps to.
///
/// One function, because `enumerate` writes it and
/// [`ChallengeSignature::verify_did_key_ed25519`] checks it — the same reason
/// `ssh::spiffe_path` is one function. Two spellings that drift apart silently
/// stop binding anything.
pub(crate) fn spiffe_path(did: &str) -> String {
    format!("did/{}", did_id(did))
}

/// Returns SHA-256(did) as a 64-char hex string.
///
/// The whole digest: this is the authorization subject, and a truncation to 8
/// bytes puts a birthday collision at 2^32.
fn did_id(did: &str) -> String {
    let hash = Sha256::digest(did.as_bytes());
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parses a whitespace-separated list of DIDs, returning only valid did:key URIs.
fn parse_did_keys_from(raw: &str) -> Vec<String> {
    raw.split_whitespace()
        .filter(|s| s.starts_with("did:key:"))
        .map(str::to_owned)
        .collect()
}

fn parse_did_keys() -> Vec<String> {
    let raw = std::env::var("HIRE_DID_KEYS").unwrap_or_default();
    parse_did_keys_from(&raw)
}

#[async_trait]
impl Attestor for DidKeyAttestor {
    fn name(&self) -> &str {
        "did:key"
    }

    async fn enumerate(&self) -> Result<Vec<Candidate>, AttestorError> {
        // ponytail: did:key candidates sit under ssh.local | ceiling: the
        //   PersonalDid trust domain is never used | upgrade path: switch the
        //   domain once the DID is actually resolved rather than string-matched
        // A dropped key names itself, so it is a candidate whether or not
        // anyone configured it -- that is the point of the drop directory.
        let dropped: Vec<String> = keystore::ed25519_keys()
            .iter()
            .map(|key| did_key_of(&key.point()))
            .collect();

        let mut candidates: Vec<Candidate> = Vec::new();
        for did in dropped.iter().cloned().chain(parse_did_keys()) {
            // A DID both dropped and configured is one identity. The dropped
            // spelling comes first, so the survivor is the one that can prove
            // silently.
            if candidates.iter().any(|c| c.display_name == did) {
                continue;
            }
            // Silent is a guarantee, and for a key held in this process it is a
            // real one: nothing in the signing path can reach a human. An agent
            // key cannot give it, because `ssh-add -c` prompts and the agent
            // does not report that constraint -- the reason ssh.rs cannot either.
            let proof_cost = if dropped.contains(&did) {
                ProofCost::Silent
            } else {
                ProofCost::Interactive
            };
            candidates.push(
                Candidate::new(
                    "did:key",
                    SelfAssertedDomain::SshLocal,
                    spiffe_path(&did),
                    did,
                )
                .with_attainable(AttainableAssurance::Iaa1)
                .with_proof_cost(proof_cost),
            );
        }
        Ok(candidates)
    }

    async fn prove(
        &self,
        candidate: &Candidate,
        challenge: &[u8],
    ) -> Result<Vec<Evidence>, AttestorError> {
        // Re-enumerate rather than trusting the path, as ssh.rs re-lists the
        // agent: it asks the truthful question, which is whether this DID is
        // still one the operator holds or claims right now.
        let did = self
            .enumerate()
            .await?
            .into_iter()
            .find(|c| c.path == candidate.path)
            .map(|c| c.display_name)
            .ok_or_else(|| {
                AttestorError::ChallengeFailed(format!("nothing local names {}", candidate.path))
            })?;

        let point = ed25519_point(&did).ok_or_else(|| {
            // Refused before any agent is asked, so a confirm-flagged key never
            // raises a dialog for a proof that could not be verified anyway.
            // The verifying constructor checks this again -- that check is the
            // trust boundary, this one is the message an operator can act on.
            tracing::info!(
                event = "did_key_unsupported",
                did = %did,
                "hire proves ed25519 did:key identifiers only"
            );
            AttestorError::ChallengeFailed(format!("{did} does not name an ed25519 key"))
        })?;

        // A dropped key first: it cannot prompt, and asking the agent about a
        // key hire already holds would risk a prompt for nothing.
        let dropped = keystore::ed25519_keys();
        let signature = match dropped.iter().find(|key| key.point() == point) {
            Some(key) => {
                tracing::debug!(
                    event = "did_key_signed_locally",
                    did = %did,
                    path = %key.path().display(),
                );
                key.sign(challenge)
            }
            None => {
                let mut stream = ssh::connect().await?;
                let key_blob = ssh::list_identities(&mut stream)
                    .await?
                    .into_iter()
                    .map(|(blob, _comment)| blob)
                    .find(|blob| ssh::ed25519_point_of(blob) == Some(point))
                    .ok_or_else(|| {
                        AttestorError::ChallengeFailed(format!(
                            "no dropped key and no local agent holds the key {did} names"
                        ))
                    })?;
                let framed = ssh::sign(&mut stream, &key_blob, challenge, &candidate.path).await?;
                // The agent's framing is unwrapped here, where the transport is
                // known. What makes the signature evidence is that it verifies
                // under the key the identifier names, and that is true of 64
                // bytes from either custodian.
                ssh::raw_ed25519_signature(&framed).ok_or_else(|| {
                    AttestorError::ChallengeFailed("malformed ssh-agent signature".into())
                })?
            }
        };

        Ok(vec![Evidence::Possession(
            ChallengeSignature::verify_did_key_ed25519(candidate, challenge, &did, &signature)?,
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── did:key decoding ──────────────────────────────────────────────────
    //
    // Oracle: an independent base58 implementation in Python, run in both
    // directions over the same bytes, not the decoder below. The first vector
    // is the ed25519 example the did:key specification uses, and the X25519
    // vector round-trips to the `z6LS` prefix the specification assigns that
    // curve -- which is what confirms the multicodec bytes rather than the
    // arithmetic alone.

    /// The did:key spec's ed25519 example.
    const ED25519_DID: &str = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    const ED25519_KEY: [u8; 32] = [
        0x2e, 0x6f, 0xcc, 0xe3, 0x67, 0x01, 0xdc, 0x79, 0x14, 0x88, 0xe0, 0xd0, 0xb1, 0x74, 0x5c,
        0xc1, 0xe3, 0x3a, 0x4c, 0x1c, 0x9f, 0xcc, 0x41, 0xc6, 0x3b, 0xd3, 0x43, 0xdb, 0xbe, 0x09,
        0x70, 0xe6,
    ];

    /// The same 32 bytes under the X25519 multicodec (`0xec 0x01`): the right
    /// length, the wrong curve. An agreement key must never answer a signing
    /// challenge, and length alone cannot tell the two apart.
    const X25519_DID: &str = "did:key:z6LSeoSo7cnMZoT2JxZ8xk8qUPNkjmHgB3G51ZbXtTa5pnnh";

    /// A 35-byte payload, which is what a compressed key on a 256-bit curve
    /// needs. Synthetic -- what it exercises is the carry running off the end
    /// of the buffer, which is the whole length check.
    const OVERSIZE_DID: &str = "did:key:z2oAtJSedW8DmsVVAfKadgfm8AyLcNuNaMnCUAzMvTEUsD8pd";

    /// An 18-byte payload. Short input leaves leading zeros where the
    /// multicodec prefix has to be, so it fails on the prefix with no separate
    /// minimum-length rule.
    const SHORT_DID: &str = "did:key:zAq9wshcffbQriXyLzqN7tSZvg";

    #[test]
    fn ed25519_point_decodes_the_spec_vector() {
        assert_eq!(ed25519_point(ED25519_DID), Some(ED25519_KEY));
    }

    #[test]
    fn did_key_of_encodes_the_spec_vector() {
        // The same external vector the decoder is tested against, run the other
        // way. Checking the encoder only by round-tripping it through the
        // decoder would let a matched pair of faults agree with each other.
        assert_eq!(did_key_of(&ED25519_KEY), ED25519_DID);
    }

    #[test]
    fn did_key_of_and_ed25519_point_are_inverses() {
        // Round trip as a second check, not the only one. The all-zero point is
        // the interesting input: it is the shortest base58 number the multicodec
        // prefix can produce, so it is where a carry bug would show.
        for point in [ED25519_KEY, [0u8; 32], [0xffu8; 32]] {
            let did = did_key_of(&point);
            assert!(did.starts_with("did:key:z6Mk"), "{did}");
            assert_eq!(ed25519_point(&did), Some(point), "{did}");
        }
    }

    #[test]
    fn ed25519_point_refuses_another_curve() {
        assert_eq!(
            ed25519_point(X25519_DID),
            None,
            "X25519 is not a signing key"
        );
        assert_eq!(ed25519_point(OVERSIZE_DID), None, "35-byte payload");
        assert_eq!(ed25519_point(SHORT_DID), None, "18-byte payload");
    }

    #[test]
    fn ed25519_point_refuses_anything_that_is_not_a_did_key() {
        assert_eq!(ed25519_point("did:web:example.com"), None);
        // No multibase prefix: `did:key` without the `z` is not base58btc.
        assert_eq!(
            ed25519_point(&ED25519_DID.replace("did:key:z", "did:key:")),
            None
        );
        // `0` and `l` are absent from the base58 alphabet precisely because
        // they are misread, so an identifier containing one is not a decode
        // with a defect, it is not this encoding at all.
        assert_eq!(ed25519_point(&ED25519_DID.replace('o', "0")), None);
    }

    #[test]
    fn spiffe_path_is_the_hash_of_the_whole_did() {
        // The path binds the candidate to one DID; two DIDs sharing a prefix
        // must not share a path.
        assert_ne!(spiffe_path(ED25519_DID), spiffe_path(X25519_DID));
        assert_eq!(
            spiffe_path(ED25519_DID),
            format!("did/{}", did_id(ED25519_DID))
        );
    }

    #[test]
    fn parse_single_did_key() {
        let dids = parse_did_keys_from("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        assert_eq!(dids.len(), 1);
        assert!(dids[0].starts_with("did:key:"));
    }

    #[test]
    fn parse_multiple_did_keys() {
        let dids = parse_did_keys_from(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK did:key:z6MkiTBz1234",
        );
        assert_eq!(dids.len(), 2);
        assert!(dids.iter().all(|d| d.starts_with("did:key:")));
    }

    #[test]
    fn parse_empty_returns_empty() {
        assert!(parse_did_keys_from("").is_empty());
    }

    #[test]
    fn parse_ignores_non_did_key_entries() {
        let dids = parse_did_keys_from("did:web:example.com did:key:z6MkhaX did:ethr:0x123");
        assert_eq!(dids.len(), 1);
        assert_eq!(dids[0], "did:key:z6MkhaX");
    }

    #[test]
    fn did_id_is_stable() {
        let a = did_id("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        let b = did_id("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // 32 bytes = 64 hex chars
                                 // Oracle: printf '%s' "<did>" | sha256sum, cross-checked against
                                 // openssl dgst -sha256.
        assert_eq!(
            a,
            "8551f404ecfe6403c2fe960ab267cd8c74a9a0701628ce24b1753946f2ebb16e"
        );
    }

    #[test]
    fn did_id_differs_for_different_dids() {
        let a = did_id("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        let b = did_id("did:key:z6MkiTBz1234");
        assert_ne!(a, b);
    }
}
