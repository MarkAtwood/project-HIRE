//! Key material the operator dropped in a directory, and the only place hire
//! holds a secret of its own.
//!
//! Every other source is a client of something that holds the key — ssh-agent,
//! gpg-agent, tailscaled, the kernel — and that is still the preferred shape.
//! This exists because the preference was not reachable for one real case: a
//! `did:key` whose secret is a file cannot be loaded into an agent at all.
//! `ssh-add` refuses a PKCS#8 PEM ed25519 key with "invalid format", and
//! OpenSSH stores ed25519 private keys only in its own format, so "put it in
//! your agent" was advice nobody could follow. Measured on OpenSSH 9.x,
//! 2026-09-15.
//!
//! ONE PLACE, NOT ONE PER ATTESTOR. hire-lnaj asks for a general drop
//! directory rather than another compiled-in vendor path, and the reason to put
//! custody here rather than in `did_key.rs` is that the next attestor needing a
//! local key must not add a second directory, a second format and a second set
//! of permission rules. Attestors ask this module; this module owns the disk.
//!
//! WHAT IS HERE IS A SECRET, so the rules are not lazy ones:
//!
//! * the file must not be readable by group or other — a key anyone on the box
//!   can read is one anyone on the box can be;
//! * the format is PKCS#8 PEM and nothing else, so the algorithm is named by an
//!   OID in the file rather than guessed from a length;
//! * a file that does not parse is skipped with a log, never treated as a key
//!   of some other kind.
//!
//! ponytail: ed25519 only, and only the `.pem` extension | ceiling: an RSA or
//!   P-256 key in the drop directory is skipped, and a dropped JWT or X.509
//!   credential is ignored rather than verified | upgrade path: hire-lnaj's
//!   other two halves — an unsigned name file that yields a candidate and
//!   nothing more, and a signed credential that yields `Evidence::IdpVerified`
//!   once a verifying constructor for it exists. Both belong in this module,
//!   beside the directory scan that already found them.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use ed25519_dalek::pkcs8::DecodePrivateKey as _;
use ed25519_dalek::{Signer as _, SigningKey};

/// Where the operator drops identity material.
///
/// `$XDG_CONFIG_HOME/hire/identities`, falling back to `~/.config` as the XDG
/// base directory specification requires — so this is the same directory on a
/// machine that sets the variable and one that does not.
pub(crate) fn drop_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned());
            PathBuf::from(home).join(".config")
        });
    base.join("hire").join("identities")
}

/// An ed25519 private key the operator dropped, and where it came from.
///
/// Holds the key rather than a handle to it, because there is nothing to hold a
/// handle to: no agent has it. The `SigningKey` zeroizes on drop — that is
/// `ed25519-dalek`'s own `zeroize` default feature, not something to re-derive
/// here.
pub(crate) struct DroppedKey {
    key: SigningKey,
    path: PathBuf,
}

impl DroppedKey {
    /// The ed25519 public point, which is how an attestor recognises the key it
    /// is looking for.
    pub(crate) fn point(&self) -> [u8; 32] {
        self.key.verifying_key().to_bytes()
    }

    /// Sign `data`, returning the raw 64-byte signature.
    pub(crate) fn sign(&self, data: &[u8]) -> [u8; 64] {
        self.key.sign(data).to_bytes()
    }

    /// The file this key came from, for logging. Never a SPIFFE path component:
    /// an identity is named by its key, not by where the operator filed it.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Debug for DroppedKey {
    /// Deliberately hand-written. The derived form would print the
    /// `SigningKey`, and `ed25519-dalek`'s own `Debug` redacts the scalar, but
    /// relying on someone else's redaction for our secret is a dependency
    /// upgrade away from being wrong.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DroppedKey")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// Every usable ed25519 key in the drop directory.
///
/// Returns empty when the directory does not exist, which is the ordinary case
/// and not an error. Unreadable, oddly-permissioned and unparseable files are
/// skipped with a log rather than failing the scan: one bad file must not cost
/// the operator every other identity they dropped.
///
/// ponytail: rescans the directory on every call | ceiling: one `readdir` and
///   one read per key per proof | upgrade path: cache on mtime, once anything
///   calls this often enough to notice — `oidc.rs` has the same shape and
///   hire-5s4b.2 is the bead for both.
pub(crate) fn ed25519_keys() -> Vec<DroppedKey> {
    let dir = drop_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut keys = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        // Unknown extensions are skipped rather than refused: hire-lnaj's whole
        // point is that an operator or a script writes this directory, and a
        // README next to the keys must not break the daemon. New kinds arrive
        // as new extensions.
        if path.extension().and_then(|e| e.to_str()) != Some("pem") {
            continue;
        }
        match load(&path) {
            Ok(key) => keys.push(key),
            Err(reason) => {
                tracing::info!(
                    event = "dropped_key_skipped",
                    path = %path.display(),
                    reason,
                    "identity file not usable"
                );
            }
        }
    }
    keys
}

/// Read one PKCS#8 PEM ed25519 private key, or say why not.
///
/// The reason strings are logged, never returned to a consumer: they name the
/// operator's own files.
fn load(path: &Path) -> Result<DroppedKey, &'static str> {
    let metadata = std::fs::metadata(path).map_err(|_| "cannot stat")?;
    if !metadata.is_file() {
        return Err("not a regular file");
    }
    refuse_shared_permissions(&metadata)?;

    let pem = std::fs::read_to_string(path).map_err(|_| "cannot read")?;
    let der = pem_body(&pem).ok_or("not a PEM private key")?;
    // PKCS#8 names the algorithm by OID, so a P-256 or RSA key in this file is
    // refused by the parser rather than by a length check here.
    let keypair = ed25519_dalek::pkcs8::KeypairBytes::from_pkcs8_der(&der)
        .map_err(|_| "not a PKCS#8 ed25519 private key")?;

    Ok(DroppedKey {
        key: SigningKey::from_bytes(&keypair.secret_key),
        path: path.to_owned(),
    })
}

/// Refuse a key file that group or other can read.
///
/// A secret every account on the box can read is an identity every account on
/// the box can assume, and hired runs as one user among several. This is the
/// check `ssh` makes on a private key for the same reason, and it is the one
/// piece of this module that must not be lazy.
#[cfg(unix)]
fn refuse_shared_permissions(metadata: &std::fs::Metadata) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("readable by group or other; chmod 600 it");
    }
    Ok(())
}

/// On a platform with no Unix mode bits there is nothing to check, and saying
/// so here keeps the decision visible rather than leaving the call site
/// `#[cfg]`-ed and the reason unwritten. `hired` does not run on Windows
/// (hire-vvs2), so this arm is unreached today.
#[cfg(not(unix))]
fn refuse_shared_permissions(_metadata: &std::fs::Metadata) -> Result<(), &'static str> {
    Ok(())
}

/// The DER body of a PEM private key, or `None` if the file is not one.
///
/// Hand-rolled rather than pulling a PEM feature through `ed25519-dalek`: it is
/// base64 between two fixed lines, `base64` is already a dependency, and the
/// alternative adds a dependency edge to save four lines. The label must be
/// `PRIVATE KEY` — PKCS#8's label — so an `OPENSSH PRIVATE KEY` or an
/// `RSA PRIVATE KEY` (PKCS#1, a different structure) is refused here rather
/// than producing a confusing parse error later.
fn pem_body(pem: &str) -> Option<Vec<u8>> {
    let after = pem.split_once("-----BEGIN PRIVATE KEY-----")?.1;
    let body = after.split_once("-----END PRIVATE KEY-----")?.0;
    let base64: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(base64)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ed25519 private key in PKCS#8 PEM, from
    /// `openssl genpkey -algorithm ed25519`. The oracle is OpenSSL: the seed
    /// and the public point below were printed by `openssl pkey -text`, not by
    /// the parser under test.
    const PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MC4CAQAwBQYDK2VwBCIEIGV2Z2g6KyPmHDTcq69qBACmoiylPq73K6Il2aZZtftW\n\
-----END PRIVATE KEY-----\n";

    /// `openssl pkey -in key.pem -noout -text` reports this as the public key.
    const POINT: [u8; 32] = [
        0x79, 0xb5, 0x8e, 0xab, 0x52, 0x3f, 0x17, 0xab, 0x33, 0xaf, 0x7b, 0xd9, 0xf4, 0x22, 0xdb,
        0x98, 0xfd, 0xb7, 0x76, 0x83, 0xf3, 0x71, 0xc8, 0xb6, 0x44, 0x57, 0x8a, 0xc4, 0xf8, 0xa9,
        0x0b, 0x2b,
    ];

    #[test]
    fn a_pkcs8_pem_body_decodes() {
        let der = pem_body(PEM).expect("a PKCS#8 PEM body");
        // PKCS#8 ed25519 private keys are 48 bytes: the SEQUENCE wrapping a
        // version, the algorithm OID and the 32-byte seed.
        assert_eq!(der.len(), 48);
        assert!(ed25519_dalek::pkcs8::KeypairBytes::from_pkcs8_der(&der).is_ok());
    }

    #[test]
    fn another_pem_label_is_not_a_private_key() {
        // The two an operator is most likely to drop by mistake. Both are real
        // PEM and neither is PKCS#8, so refusing on the label gives a reason
        // instead of a parse error about bytes they never chose.
        for label in ["OPENSSH PRIVATE KEY", "RSA PRIVATE KEY", "CERTIFICATE"] {
            let wrong = PEM
                .replace("BEGIN PRIVATE KEY", &format!("BEGIN {label}"))
                .replace("END PRIVATE KEY", &format!("END {label}"));
            assert!(pem_body(&wrong).is_none(), "{label}");
        }
        assert!(pem_body("not pem at all").is_none());
        // Truncated: a BEGIN with no END is not half a key.
        assert!(pem_body(PEM.split("-----END").next().unwrap()).is_none());
    }

    #[test]
    fn the_parsed_key_is_the_one_openssl_named() {
        let der = pem_body(PEM).unwrap();
        let keypair = ed25519_dalek::pkcs8::KeypairBytes::from_pkcs8_der(&der).unwrap();
        let key = SigningKey::from_bytes(&keypair.secret_key);
        assert_eq!(
            key.verifying_key().to_bytes(),
            POINT,
            "the public point must be the one openssl printed for this key"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_world_readable_key_is_refused() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = std::env::temp_dir().join(format!("hire-keystore-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let path = dir.join("k.pem");
        std::fs::write(&path, PEM).expect("write key");

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(load(&path).is_ok(), "a 0600 key must load");

        for mode in [0o644, 0o640, 0o604, 0o666] {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(load(&path).is_err(), "mode {mode:o} must be refused");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
