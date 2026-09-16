<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Roadmap

What order the work goes in, and why that order.

**Nothing here is maintained by hand except the order and the reasoning.** The inventory at
the bottom is generated from the issue tracker by `make roadmap`, so it is current rather
than remembered -- generated status is regenerable, which is what separates it from the
copied status that goes stale. What already works is the Status table in
[README.md](README.md).

## What this is aiming at

The best answer a machine can give to "who is this human", through the SPIFFE Workload
API, with its provenance and assurance attached, on every desktop. Where the hardware can
also establish that a human is present, that qualifies the answer -- it is not the
question. [MOTIVATIONS.md](MOTIVATIONS.md) argues the case and says where the project came
from.

## The order

**1. Breadth of sources, starting with the work identity.** `persona-qbnm.7`

On a corporate desktop, "who am I" is most often the work Google or Microsoft account, and
that is the one source which currently cannot answer. Everything HIRE can prove today is
either self-asserted -- an SSH key, a GPG key, a `did:key` -- or the operating system's
word about a local account, with Tailscale the single exception. One IdP-backed source is
not breadth. This is the largest gap between what the project claims and what it does.

**2. The other credential shape.** `persona-apyr`

"Any SPIFFE-aware consumer works against `hired`" is true only of the JWT-speaking subset.
Envoy, ghostunnel and spiffe-helper are the consumers a SPIFFE audience names first, and
they want X.509-SVIDs. Until that exists the strongest artifact this project has -- a stock
client library working unmodified -- proves half of what it appears to prove.

**3. Platform reach.** `persona-vvs2`

A desktop feature absent on Windows addresses half the desktops. The transport abstraction
is the gate, and landing it also unblocks `persona-vvs2.13`, the WAM source, which on a
domain-joined machine is the highest-quality answer available anywhere.

**4. Enrollment.** `persona-08sz`

Several designs already defer to it, which is the sign that it is load-bearing rather than
nice: pinning gpg's exported public key so trust in the local binary becomes point-in-time
rather than continuous, recording what a source's trust rests on, the elevated-scope
consumer path, and persisting the pseudonym key so a consumer still recognises the user
after a restart.

**5. Presence, last and deliberately.** `persona-3tly`, `persona-qbnm.3`

Worth having in high-assurance environments where the hardware exists, and not what the
product is for. Note that the presence epic carries P1 beads: they are P1 because the
*model* is wrong -- it claims more than it can establish -- not because presence ships
next. Fixing a wrong model is worth doing whenever it is touched; scheduling the epic ahead
of items 1 through 4 is not.

## Running alongside

The donation track in [UPSTREAM.md](UPSTREAM.md) is not sequenced against this list. Its
Phase 0 costs nothing and can start any day; its Phase 1 is gated by items 1 and 2 above.

**Readiness.** `persona-2myu`

Not features, and not sequenced against the five above, because the five get done on
their own. Feature work is legible -- someone can see it, demo it and take credit for it
-- so it gets scheduled. Versioning a wire format before anyone depends on it, owning a
name that can be withheld from an implementation that fails a conformance suite, and
deciding what a per-consumer pseudonym means to an auditor are invisible right up to the
day they become impossible.

That is the test for the epic: not whether an item is important, but whether its cost
goes from an afternoon to a migration on a date somebody else picks. The first serious
adopter picks it. Everything in that epic is cheap today and has no cheap version later,
which is the argument for doing it while nothing is forcing it.

One item there is also an exception to item 5 above. Presence need not ship early, but
either the authenticator's assertion reaches the consumer (`persona-3tly.1`) or the
`hint` stops claiming a level above `session`. A presence vocabulary that a relying party
cannot check is one an implementation can claim falsely and nobody can catch, and of the
two ways to be wrong, saying less is the recoverable one.

## Why this order and not another

The large-vendor survey (`persona-c7a6`) decides it. Microsoft built the right idea and
bound it to one operating system, one identity provider and one SDK. AWS and Google solved
identity cloud-side and delegated the endpoint signal to a browser extension or a partner
agent. Nobody built a neutral local API.

So the differentiator is neutrality -- every OS, every source, one API, no vendor in the
middle -- which makes **breadth (1) and reach (3) the work that distinguishes this project**
and presence (5) a feature other people also have. An ordering that front-loaded presence
would spend the project's scarce effort on the part of the pitch nobody is missing.

## What is in the tracker

The section below is generated from the issue tracker, so it is current rather than
remembered. It is the same work the sections above sequence, listed epic by epic. Items
below P2 are counted rather than named: real work, and not what someone reads a roadmap to
find out.

<!-- BEGIN GENERATED -- regenerate with `make roadmap`, do not edit by hand -->

*Generated from the issue tracker on 2026-09-16: 98 open, 126 closed.*

### Readiness: what adoption arriving in a lump would need

`persona-2myu` -- 0 of 3 done

- **P1** `persona-2myu.1` -- Version the wire surface before anyone freezes it
- **P1** `persona-2myu.2` -- A conformance suite, and a mark that can be withheld
- **P1** `persona-2myu.3` -- Decide whether a pseudonym can be un-mapped, and by whom

### The presence model claims more than it can establish

`persona-3tly` -- 0 of 10 done

- **P1** `persona-3tly.1` -- Presence is hearsay: the authenticator's assertion never reaches the consumer
- **P1** `persona-3tly.2` -- PresenceLevel collapses user presence and user verification
- **P1** `persona-3tly.3` -- No surface exists where a human sees what they are approving
- **P1** `persona-3tly.5` -- Presence is modelled as a totally-ordered scalar but is at least three independent axes
- **P1** `persona-3tly.6` -- Presence ends two different ways and the model can only express one
- **P1** `persona-3tly.7` -- Continuous presence has no carrier; departure events do
- **P1** `persona-3tly.7.1` -- (a) Derive JWT-SVID TTL from the source, not a constant
- **P2** `persona-3tly.4` -- Record the limits of the presence model in the spec
- **P2** `persona-3tly.7.2` -- (b) ValidateJWTSVID consults live presence state
- **P2** `persona-3tly.7.3` -- (c) Presence subscription on the HIRE service

### Review: persona 2026-09-13

`persona-5s4b` -- 103 of 132 done

- **P1** `persona-5s4b.117` -- Carry the LSM peer context from SO_PEERSEC in ConsumerIdentity
- **P1** `persona-5s4b.8` -- No registration path for third-party attestors; availability probe is not part of the Attestor trait
- **P2** `persona-5s4b.1` -- FetchJWTSVID enumerates all attestors serially on every request
- **P2** `persona-5s4b.130` -- SpiffeId has almost no test coverage
- **P2** `persona-5s4b.14` -- Attestor::prove and freshness have no documented preconditions on the claim, challenge, or assertion binding
- **P2** `persona-5s4b.2` -- OIDC attestor rescans and reparses all token caches on every call
- **P2** `persona-5s4b.24` -- SvidSigner::sign_jwt_svid takes untyped serde_json::Value and hardcodes a 300s TTL
- **P2** `persona-5s4b.90` -- persona prove performs no cryptographic proof and its doc comment claims replay protection it does not provide
- **P2** `persona-5s4b.93` -- did:key and GPG identities are issued under the ssh.local trust domain
- *and 20 lower-priority items*

### Discovery and proof are separate operations

`persona-jl4j` -- 4 of 8 done

- **P2** `persona-jl4j.4` -- Candidate-listing RPC on a second gRPC service
- **P2** `persona-jl4j.5` -- Proof cache: freshness moves from challenge-binding to the clock
- *and 2 lower-priority items*

### Seven attestors enumerate but cannot prove

`persona-qbnm` -- 3 of 13 done

- **P1** `persona-qbnm.3` -- fido2: the first attestor that establishes presence
- **P1** `persona-qbnm.7.2` -- The issuer URL comes from a file anyone on the box can write
- **P1** `persona-qbnm.7.4` -- Decide what binds_to means for a cached token, before writing the constructor
- **P2** `persona-qbnm.4` -- piv: PKCS#11 slot enumeration and challenge signing
- **P2** `persona-qbnm.7` -- oidc: verify the cached token against the issuer's JWKS
- **P2** `persona-qbnm.7.1` -- Choose the HTTPS client, and bound what it may reach
- **P2** `persona-qbnm.7.3` -- Resolve the issuer's JWKS: discovery, cache, and kid-miss refresh
- **P2** `persona-qbnm.7.5` -- The verifying constructor, and what Validation must and must not check
- **P2** `persona-qbnm.7.6` -- Wire oidc prove() and re-anchor the identity to its own trust domain
- *and 1 lower-priority item*

### Specified sources with no implementation at all

`persona-vmpi` -- 0 of 5 done

- **P2** `persona-vmpi.1` -- secure-enclave: TouchID and FaceID on macOS
- **P2** `persona-vmpi.2` -- did:web: resolve the DID document over HTTPS
- **P2** `persona-vmpi.3` -- kerberos: TGT as an identity claim
- *and 2 lower-priority items*

### Windows support: the transport abstraction is the gate

`persona-vvs2` -- 2 of 13 done

- **P2** `persona-vvs2.3` -- Abstract the transport over UnixListener/UnixStream
- **P2** `persona-vvs2.4` -- Named pipe listener and stream
- **P2** `persona-vvs2.5` -- Windows consumer attestation: the selector is undecided
- *and 8 lower-priority items*

### Not part of an epic

**P1**

- `persona-08sz` -- Enrolling an attestor: where a source's trust basis is recorded
- `persona-apyr` -- X.509-SVID issuance: sign_x509_svid is a stub and hire-4qm does not exist
- `persona-ouo5` -- Unix platform attestor: the OS as an iaa1 identity source

**P2**

- `persona-lbdx` -- Removable media carrying a signed identity attestation
- `persona-lnaj` -- No general file-drop identity source; every file path is compiled in
- `persona-ouo5.2` -- hire_max_age is unconditionally satisfiable on Unix
- `persona-ouo5.3` -- No hire_min_assurance, so a consumer cannot gate on assurance at all
- `persona-ux32` -- Trust for attestors we cannot reach by installing them

*And 11 lower-priority items: polish, documentation drift and small cleanups. `bd ready` lists them.*

<!-- END GENERATED -->

## What is deliberately not built

- **A policy engine.** OPA exists. Write Rego, not an engine. See SPEC-HIRE.
- **Workload identity.** SPIFFE/SPIRE exists; `hired` federates with it.
- **Attestors for overlay networks that expose no human identity** -- NetBird, ZeroTier,
  Nebula, the WARP daemon. They would enumerate candidates that can never prove a person,
  which is the defect `persona-qbnm` exists to remove. Recorded in `persona-puyb`.
- **A competitor to WAM.** On Windows it is the best available answer; consume it.
