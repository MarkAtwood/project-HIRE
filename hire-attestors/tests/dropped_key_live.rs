//! A key dropped in `~/.config/hire/identities` becomes a provable identity.
//!
//! This is hire-lnaj's first half: an operator writes a file and the daemon can
//! prove an identity it was never taught about. No agent, no hardware, no
//! vendor path compiled into Rust.
//!
//! One `#[test]` per file, for the reason `ssh_agent_live.rs` gives: the
//! keystore is aimed through the process-global `XDG_CONFIG_HOME`.

use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use hire_attestors::{Attestor, Claim, DidKeyAttestor, ProofCost};
use hire_core::{IdentityAssurance, PresenceLevel};

/// An ed25519 private key in PKCS#8 PEM, from `openssl genpkey -algorithm
/// ed25519`. The same fixture `keystore`'s unit tests use, repeated here
/// because an integration test is its own crate and cannot reach into that
/// module -- and generating one at test time would need openssl installed.
const PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MC4CAQAwBQYDK2VwBCIEIGV2Z2g6KyPmHDTcq69qBACmoiylPq73K6Il2aZZtftW\n\
-----END PRIVATE KEY-----\n";

/// The `did:key` that key's public half names, computed by the Python base58
/// implementation the decoder's unit tests use as their oracle -- not by this
/// crate's encoder, which is what the identity under test is derived from.
const EXPECTED_DID: &str = "did:key:z6MkneMwtKWf5gLyu4pcTeZbiwnb9nQQD9yETUxDNbiaggki";

/// Removes the scratch config directory even if an assertion panics, so a
/// failing run leaves no private key behind.
struct DropDir {
    root: PathBuf,
}

impl Drop for DropDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn a_dropped_key_is_a_silently_provable_identity() {
    let root = std::env::temp_dir().join(format!("hire-drop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("hire").join("identities");
    std::fs::create_dir_all(&dir).expect("scratch drop dir");
    let guard = DropDir { root: root.clone() };

    let key = dir.join("did.pem");
    std::fs::write(&key, PEM).expect("write key");
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).expect("chmod");

    // Safe in edition 2021, and the whole reason this file holds one test.
    // HIRE_DID_KEYS is cleared so nothing here depends on the developer's own
    // configuration, and so the candidate can only have come from the file.
    std::env::set_var("XDG_CONFIG_HOME", &guard.root);
    std::env::remove_var("HIRE_DID_KEYS");

    assert!(
        DidKeyAttestor::is_available(),
        "a dropped key alone must make the source available"
    );
    let attestor = DidKeyAttestor::new();

    let candidates = attestor.enumerate().await.expect("enumerate");
    assert_eq!(candidates.len(), 1, "one key, one identity");
    let candidate = &candidates[0];
    assert_eq!(
        candidate.display_name, EXPECTED_DID,
        "the identity must be the one the key's public half names"
    );
    // The guarantee, and it is a real one here: nothing in the signing path can
    // reach a human, so this identity is answerable on the unprompted path.
    assert_eq!(candidate.proof_cost, ProofCost::Silent);

    let challenge = [0x7bu8; 32];
    let evidence = attestor
        .prove(candidate, &challenge)
        .await
        .expect("a dropped key must prove possession");
    let claim = Claim::derive(candidate, &challenge, &evidence)
        .expect("verified possession must yield a claim");
    assert_eq!(claim.assurance(), IdentityAssurance::Iaa1);
    assert_eq!(claim.presence(), PresenceLevel::None);

    // A signature answering some other challenge is a replay, not evidence.
    assert!(
        Claim::derive(
            candidate,
            b"a challenge this signature never saw",
            &evidence
        )
        .is_none(),
        "evidence bound to one challenge must not satisfy another"
    );

    // A key every account on the box can read is an identity every account on
    // the box can assume, so it stops being an identity at all.
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o644)).expect("chmod");
    assert!(
        attestor.enumerate().await.expect("enumerate").is_empty(),
        "a world-readable key must not be offered as an identity"
    );
    assert!(
        attestor.prove(candidate, &challenge).await.is_err(),
        "nor proved, even for a candidate enumerated while it was still private"
    );
}
