//! AWS IAM Identity Center enrollment -- an organisational candidate that
//! cannot prove.
//!
//! `~/.aws/sso/cache` holds an opaque bearer token, not an assertion. There is
//! no `idToken` and no JWT anywhere in it, so nothing in the file can be
//! checked on this machine. Learning who the token belongs to means exchanging
//! it for role credentials and calling `sts:GetCallerIdentity` -- a network
//! round trip to an authority, for an answer that is AWS's word rather than a
//! signature.
//!
//! What the cache does say with no network call at all is `startUrl`, which
//! names the Identity Center instance this desktop is enrolled in. That is an
//! organisation, not a person, so this source enumerates and stops.
//!
//! It has no `prove()`. The default declines, and that is the correct answer
//! rather than a gap: an attestor that can verify nothing contributes no
//! assurance level, and the daemon issues nothing. What tier "a remote
//! authority vouched for this bearer token" would earn is undecided
//! (`persona-vjir`), and this source does not prejudge it, because it never
//! reaches the question.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{Attestor, AttestorError, Candidate, ProofCost, SelfAssertedDomain};

/// Attestor that reads the AWS SSO token cache for the enrollment only.
#[derive(Debug)]
pub struct AwsSsoAttestor;

impl AwsSsoAttestor {
    pub fn new() -> Self {
        Self
    }

    /// True when this desktop has an AWS SSO token cache directory.
    ///
    /// A directory that exists but holds only a client registration probes
    /// available and enumerates nothing, which is the honest pair: the
    /// machine has an AWS CLI, and it names no Identity Center instance.
    pub fn is_available() -> bool {
        cache_dir().is_some_and(|d| d.is_dir())
    }
}

impl Default for AwsSsoAttestor {
    fn default() -> Self {
        Self::new()
    }
}

/// `$HOME/.aws/sso/cache`, the path the AWS CLI and SDKs write.
///
/// No environment override: `AWS_CONFIG_FILE` relocates the config file, not
/// the SSO cache, and inventing one here would be a knob nobody asked for.
fn cache_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".aws/sso/cache"))
}

/// The host of an `https://` start URL, if it is one we would put in a path.
///
/// Untrusted input at a trust boundary: any process running as this user can
/// write the cache, and the host goes on to be a SPIFFE path segment. The
/// charset is checked here rather than trusted to whatever wrote the file, so
/// a start URL of `https://x/../../root` names nothing instead of naming
/// something else's identity.
fn start_url_host(start_url: &str) -> Option<&str> {
    let host = start_url.strip_prefix("https://")?.split('/').next()?;
    let plausible = host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        });
    plausible.then_some(host)
}

/// An AWS region name, if it looks like one. Display only, and checked for the
/// same reason as the host: it reaches a human's screen.
fn plausible_region(region: &str) -> bool {
    !region.is_empty()
        && region.len() <= 32
        && region
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Build the enrollment candidate from one parsed cache file.
///
/// Two fields are read and the rest of the document is dropped with the
/// `Value`. The same file holds `accessToken`, `refreshToken` and
/// `clientSecret`; none of them are retained, returned or logged.
///
/// The candidate keeps `Candidate::new`'s floor tier deliberately. The tier a
/// remote authority's word would earn is the open question in `persona-vjir`,
/// and declaring an attainable tier here would answer it in a doc comment.
fn candidate_from_cache(json: &serde_json::Value) -> Option<Candidate> {
    let host = json
        .get("startUrl")
        .and_then(|v| v.as_str())
        .and_then(start_url_host)?;
    let region = json
        .get("region")
        .and_then(|v| v.as_str())
        .filter(|r| plausible_region(r));

    let display_name = match region {
        Some(region) => format!("{host} ({region})"),
        None => host.to_owned(),
    };

    // ponytail: the enrollment is self-asserted by a file, so it sits under
    //   ssh.local with every other unanchored source |
    //   ceiling: the trust domain says nothing about AWS |
    //   upgrade path: persona-5s4b.93 re-homes these, and an aws-sso source
    //   that actually verified something would anchor to its own domain
    Some(
        Candidate::new(
            "aws-sso",
            SelfAssertedDomain::SshLocal,
            format!("org/{host}/via/aws-sso"),
            display_name,
        )
        .with_proof_cost(ProofCost::Silent),
    )
}

/// Every Identity Center instance named by a cache file in `dir`, deduplicated.
///
/// The AWS CLI writes one file per SSO session plus a client registration, so
/// several files routinely name the same instance and one names no instance at
/// all. Sorted, because `read_dir` order is arbitrary and enumeration should
/// not depend on it.
fn enrollments_in(dir: &Path) -> Vec<Candidate> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut seen = BTreeSet::new();
    let mut found: Vec<Candidate> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(candidate) = candidate_from_cache(&json) {
            if seen.insert(candidate.path.clone()) {
                found.push(candidate);
            }
        }
    }

    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}

/// Runs [`enrollments_in`] on a blocking thread.
///
/// The scan reads a directory and a handful of small files under `$HOME`,
/// which may sit on NFS or autofs where a single read blocks for seconds.
/// Called directly from an async method it stalls the reactor thread and every
/// other task scheduled there. Same reasoning as the OIDC cache scan.
async fn scan_off_reactor() -> Result<Vec<Candidate>, AttestorError> {
    tokio::task::spawn_blocking(|| match cache_dir() {
        Some(dir) => enrollments_in(&dir),
        None => vec![],
    })
    .await
    .map_err(|e| AttestorError::Unavailable(format!("aws sso cache scan failed: {e}")))
}

#[async_trait]
impl Attestor for AwsSsoAttestor {
    fn name(&self) -> &str {
        "aws-sso"
    }

    async fn enumerate(&self) -> Result<Vec<Candidate>, AttestorError> {
        scan_off_reactor().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The token file's shape, with the key names observed on a real machine
    /// and every value replaced. `accessToken` and the two secrets are present
    /// precisely so a test fails if one of them ever reaches a candidate.
    const TOKEN_FILE: &str = r#"{
        "startUrl": "https://d-9067f0a4e2.awsapps.com/start",
        "region": "us-west-2",
        "accessToken": "aoaAAAAAAAAAsecrettokenvalue",
        "refreshToken": "aorAAAAAAAAAsecretrefreshvalue",
        "clientId": "Zm9vYmFyCg",
        "clientSecret": "c2VjcmV0Cg",
        "expiresAt": "2026-09-17T00:00:00Z",
        "registrationExpiresAt": "2026-12-01T00:00:00Z"
    }"#;

    /// The other file the AWS CLI writes: a client registration, naming no
    /// Identity Center instance at all.
    const REGISTRATION_FILE: &str = r#"{
        "clientId": "Zm9vYmFyCg",
        "clientSecret": "c2VjcmV0Cg",
        "expiresAt": "2026-12-01T00:00:00Z",
        "scopes": ["sso:account:access"]
    }"#;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hire-awssso-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[test]
    fn the_token_file_names_the_identity_center_instance() {
        let dir = scratch("token");
        std::fs::write(dir.join("a1b2.json"), TOKEN_FILE).expect("write cache file");

        let found = enrollments_in(&dir);
        assert_eq!(found.len(), 1, "one file, one enrollment");
        // Both expected strings are read off the fixture by hand, not produced
        // by the code under test.
        assert_eq!(found[0].path, "org/d-9067f0a4e2.awsapps.com/via/aws-sso");
        assert_eq!(
            found[0].display_name,
            "d-9067f0a4e2.awsapps.com (us-west-2)"
        );
        assert_eq!(found[0].source, "aws-sso");
    }

    #[test]
    fn no_secret_from_the_cache_reaches_the_candidate() {
        let dir = scratch("secrets");
        std::fs::write(dir.join("a1b2.json"), TOKEN_FILE).expect("write cache file");

        let found = enrollments_in(&dir);
        let rendered = format!("{found:?}");
        for secret in [
            "aoaAAAAAAAAAsecrettokenvalue",
            "aorAAAAAAAAAsecretrefreshvalue",
            "c2VjcmV0Cg",
        ] {
            assert!(
                !rendered.contains(secret),
                "a cache secret reached the candidate: {rendered}"
            );
        }
    }

    #[test]
    fn a_client_registration_names_no_enrollment() {
        let dir = scratch("registration");
        std::fs::write(dir.join("c3d4.json"), REGISTRATION_FILE).expect("write cache file");

        assert!(
            enrollments_in(&dir).is_empty(),
            "a registration file names no Identity Center instance"
        );
    }

    #[test]
    fn two_sessions_on_one_instance_enumerate_once() {
        let dir = scratch("dedup");
        std::fs::write(dir.join("a1b2.json"), TOKEN_FILE).expect("write cache file");
        std::fs::write(dir.join("e5f6.json"), TOKEN_FILE).expect("write cache file");
        std::fs::write(dir.join("c3d4.json"), REGISTRATION_FILE).expect("write cache file");
        std::fs::write(dir.join("notes.txt"), TOKEN_FILE).expect("write decoy");

        let found = enrollments_in(&dir);
        assert_eq!(
            found.len(),
            1,
            "one instance, however many sessions: {found:?}"
        );
    }

    #[test]
    fn a_missing_cache_directory_is_not_an_error() {
        let absent =
            std::env::temp_dir().join(format!("hire-awssso-absent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&absent);
        assert!(enrollments_in(&absent).is_empty());
    }

    #[test]
    fn only_an_https_host_of_plausible_shape_becomes_a_path() {
        // Every expectation is written out rather than computed, so the table
        // says what the rule is instead of restating the implementation.
        let cases = [
            (
                "https://d-9067f0a4e2.awsapps.com/start",
                Some("d-9067f0a4e2.awsapps.com"),
            ),
            ("https://sso.example.com", Some("sso.example.com")),
            // Traversal after the host is not this function's business: only
            // the host is used, and it is the only thing returned.
            ("https://host/../other", Some("host")),
            ("http://d-9067f0a4e2.awsapps.com/start", None),
            ("https:///start", None),
            ("https://../../root/start", None),
            ("https://host name/start", None),
            ("https://host%2fspiffe/start", None),
            ("spiffe://ssh.local/user/root", None),
            ("", None),
        ];
        for (start_url, expected) in cases {
            assert_eq!(
                start_url_host(start_url),
                expected,
                "start url: {start_url}"
            );
        }
    }

    #[tokio::test]
    async fn the_source_enumerates_and_cannot_prove() {
        let candidate = candidate_from_cache(
            &serde_json::from_str(TOKEN_FILE).expect("the fixture is valid json"),
        )
        .expect("the fixture names an instance");

        let refusal = AwsSsoAttestor::new()
            .prove(&candidate, b"challenge")
            .await
            .expect_err("an opaque bearer token proves nothing");
        assert!(
            matches!(refusal, AttestorError::ChallengeFailed(_)),
            "the refusal must be a declined challenge, not a transient error: {refusal:?}"
        );
    }
}
