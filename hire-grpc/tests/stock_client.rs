//! A third-party SPIFFE client talks to `hired` unmodified.
//!
//! This is the project's central claim -- "any SPIFFE-aware consumer works
//! against hired out of the box" -- and until now the only evidence for it was
//! that our own tests drive a tonic client generated from the same `.proto`
//! this daemon serves. That proves the file round-trips, not that a stock
//! client library works.
//!
//! The `spiffe` crate is an independent implementation of the Workload API
//! client: its own protobuf copy, its own endpoint parsing, its own SVID types.
//! If `hired` deviates from the wire contract, this is where it shows.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;

use hire_attestors::{Attestor, AttestorError, Candidate, Evidence};
use hire_core::{SvidSigner, TrustBundle, TrustBundleStore, TrustDomain};
use hire_grpc::service::WorkloadApiService;
use spiffe::WorkloadApiClient;

mod common;

const AUDIENCE: &str = "https://test.example.com";

#[derive(Debug)]
struct TestAttestor;

#[async_trait]
impl Attestor for TestAttestor {
    fn name(&self) -> &str {
        "stock-client-test"
    }

    async fn enumerate(&self) -> Result<Vec<Candidate>, AttestorError> {
        Ok(vec![common::candidate("stock-client-test")])
    }

    async fn prove(
        &self,
        candidate: &Candidate,
        challenge: &[u8],
    ) -> Result<Vec<Evidence>, AttestorError> {
        Ok(vec![common::possession(candidate, challenge)])
    }
}

fn tmp_socket_path() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static SEQ: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("/tmp/hire-stock-{nanos}-{seq}.sock")
}

#[tokio::test]
async fn a_third_party_spiffe_client_fetches_a_jwt_svid() {
    let socket_path = tmp_socket_path();
    let signer = Arc::new(SvidSigner::new().unwrap());
    let bundles = Arc::new(TrustBundleStore::new());
    bundles.upsert(TrustBundle::local(TrustDomain::SshLocal, &signer));
    let service = WorkloadApiService::new(
        signer,
        bundles,
        vec![Arc::new(TestAttestor) as Arc<dyn Attestor>],
    );

    let sock = socket_path.clone();
    let handle = tokio::spawn(async move {
        hire_grpc::server::serve(std::path::Path::new(&sock), service)
            .await
            .expect("server error");
    });
    sleep(Duration::from_millis(100)).await;

    // The endpoint form every SPIFFE client takes, and the one an application
    // would put in SPIFFE_ENDPOINT_SOCKET.
    let client = WorkloadApiClient::connect_to(format!("unix://{socket_path}"))
        .await
        .expect("a stock SPIFFE client must be able to connect to hired");

    let svid = client
        .fetch_jwt_svid([AUDIENCE], None)
        .await
        .expect("a stock SPIFFE client must be able to fetch a JWT-SVID");

    // Parsed by the client's own SVID type, so the token satisfies a
    // third-party reading of the JWT-SVID spec and not just ours.
    let id = svid.spiffe_id().to_string();
    assert!(
        id.starts_with("spiffe://ssh.local/pseudonym/"),
        "the client must see the pseudonym, not the root identity: {id}"
    );
    assert_eq!(svid.audience().len(), 1);

    // And it reads the hint, which is the field hire-jl4j made normative.
    assert_eq!(
        svid.hint().map(|h| h.to_string()),
        Some("source=stock-client-test&identity_assurance=iaa1&presence=none&age=0".to_owned()),
    );

    // The other half of the loop, and the one that matters most: the client
    // fetches the bundle hired publishes and validates the token against it
    // with its own verifier, its own JWKS parsing and its own audience and
    // expiry checks. No code of ours takes part in the verification.
    let bundles = client
        .fetch_jwt_bundles()
        .await
        .expect("a stock SPIFFE client must be able to fetch the trust bundle");
    let validated = spiffe::JwtSvid::parse_and_validate(svid.token(), &bundles, &[AUDIENCE])
        .expect("the published bundle must validate the issued token");
    assert_eq!(validated.spiffe_id().to_string(), id);

    // And the negative: the same token against an audience it was not issued
    // for. A consumer that cannot tell those apart has no audience binding.
    assert!(
        spiffe::JwtSvid::parse_and_validate(svid.token(), &bundles, &["https://elsewhere.example"])
            .is_err(),
        "a token must not validate for an audience it does not name"
    );

    handle.abort();
    let _ = std::fs::remove_file(&socket_path);
}
