//! Identity assurance and presence level types.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when parsing an assurance or presence string fails.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssuranceError {
    /// The string did not match any known `IdentityAssurance` level.
    #[error("unknown identity assurance level: {0:?}")]
    UnknownAssurance(String),

    /// The string did not match any known `PresenceLevel`.
    #[error("unknown presence level: {0:?}")]
    UnknownPresence(String),
}

/// How the human authenticated, as opposed to where the identity came from.
///
/// `sources` in the issued token names the attestor; this names the mechanism,
/// and the two answer different questions. Until hire-5s4b.129 they were the
/// same single-element expression in every token hired issued, so a consumer
/// reading `auth_methods` to decide whether a hardware authenticator was
/// involved read an attestor name instead.
///
/// THE VOCABULARY IS MECHANISMS, NOT SOURCES, and that is the point of having
/// it. A consumer gating on `hardware_key_possession` does not need to know
/// that gpg, PIV and FIDO2 exist, or which of them is installed here -- whereas
/// a `{source}_{mechanism}` vocabulary makes every policy a list of sources,
/// which is the field next door. SPEC-HIRE's illustrative `tailscale_oidc` and
/// `fido2_up` were the source-shaped form and are replaced by this one.
///
/// DERIVED FROM EVIDENCE, NEVER DECLARED. `Claim::derive` reads these off the
/// evidence variants exactly as it reads the tier, so an attestor cannot state
/// an authentication that did not happen. An empty list is a real answer: the
/// kernel naming the account a process runs under is not an authentication
/// method, so a claim resting on it alone carries none (hire-ouo5.7).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    /// A key the human controls signed this request's challenge.
    ///
    /// Possession of a key, and nothing about who held it: an agent key with no
    /// passphrase signs with no human in the loop.
    KeyPossession,

    /// The same, by a key on a device it cannot be copied off.
    ///
    /// Reported by the custodian rather than proved to hire -- a smartcard
    /// serial in gpg's key listing, say. It is recorded because a relying party
    /// in a paranoid environment wants to know, and it deliberately does not
    /// raise the assurance tier: `Iaa3` means hardware-bound *and* IdP-verified,
    /// and a self-asserted PGP key is not IdP-verified however good the card is.
    HardwareKeyPossession,

    /// An identity provider verified the human, as reported by a local daemon.
    ///
    /// Hearsay with a clock attached to the wrong instant: the daemon is
    /// believed because it was installed here, and the login it reports may be
    /// months old. That is why it is a separate method from [`IdpToken`].
    ///
    /// [`IdpToken`]: Self::IdpToken
    IdpSession,

    /// An identity provider verified the human, in a token hire checked itself.
    ///
    /// Not reachable yet: no attestor can verify an issuer signature.
    IdpToken,

    /// A human touched an authenticator.
    ///
    /// Not reachable yet: no attestor can drive one.
    UserPresence,
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthMethod::KeyPossession => f.write_str("key_possession"),
            AuthMethod::HardwareKeyPossession => f.write_str("hardware_key_possession"),
            AuthMethod::IdpSession => f.write_str("idp_session"),
            AuthMethod::IdpToken => f.write_str("idp_token"),
            AuthMethod::UserPresence => f.write_str("user_presence"),
        }
    }
}

/// Identity assurance level, roughly aligned with NIST SP 800-63 AAL tiers.
///
/// Ordered `Iaa1 < Iaa2 < Iaa3`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentityAssurance {
    /// Self-asserted: SSH key, GPG key, DID, or local username.
    Iaa1,

    /// IdP-verified: Tailscale OIDC, GNOME Online Accounts.
    Iaa2,

    /// Hardware-bound and IdP-verified: FIDO2, PIV, Windows Hello.
    Iaa3,
}

impl fmt::Display for IdentityAssurance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentityAssurance::Iaa1 => f.write_str("iaa1"),
            IdentityAssurance::Iaa2 => f.write_str("iaa2"),
            IdentityAssurance::Iaa3 => f.write_str("iaa3"),
        }
    }
}

impl FromStr for IdentityAssurance {
    type Err = AssuranceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iaa1" => Ok(IdentityAssurance::Iaa1),
            "iaa2" => Ok(IdentityAssurance::Iaa2),
            "iaa3" => Ok(IdentityAssurance::Iaa3),
            other => Err(AssuranceError::UnknownAssurance(other.to_owned())),
        }
    }
}

/// Physical/logical presence assertion level.
///
/// Ordered `None < Session < Software < Hardware`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceLevel {
    /// No presence assertion.
    None,

    /// Screen unlocked at login (session-level).
    Session,

    /// TOTP or password re-entry (software-level).
    Software,

    /// FIDO2 UP, Windows Hello, TouchID, or PIV PIN -- timestamped,
    /// hardware-backed.
    Hardware,
}

impl fmt::Display for PresenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PresenceLevel::None => f.write_str("none"),
            PresenceLevel::Session => f.write_str("session"),
            PresenceLevel::Software => f.write_str("software"),
            PresenceLevel::Hardware => f.write_str("hardware"),
        }
    }
}

impl FromStr for PresenceLevel {
    type Err = AssuranceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(PresenceLevel::None),
            "session" => Ok(PresenceLevel::Session),
            "software" => Ok(PresenceLevel::Software),
            "hardware" => Ok(PresenceLevel::Hardware),
            other => Err(AssuranceError::UnknownPresence(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Three production sites compare these levels with `>` / `>=`: the
    // best-of-assurance selection loop, the presence gate, and claim
    // derivation. The ordering is derived from declaration order, so pin it.

    #[test]
    fn assurance_levels_are_ordered() {
        assert!(IdentityAssurance::Iaa1 < IdentityAssurance::Iaa2);
        assert!(IdentityAssurance::Iaa2 < IdentityAssurance::Iaa3);
    }

    #[test]
    fn presence_levels_are_ordered() {
        assert!(PresenceLevel::None < PresenceLevel::Session);
        assert!(PresenceLevel::Session < PresenceLevel::Software);
        assert!(PresenceLevel::Software < PresenceLevel::Hardware);
    }
}
