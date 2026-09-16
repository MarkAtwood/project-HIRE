<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Roadmap

What order the work goes in, and why that order.

**No status lives here.** What works today is the Status table in [README.md](README.md);
what is queued is the tracker, which `bd ready` prints. This file says which of it matters
first and for what reason, and names the epic or bead so the tracker stays the source of
truth.

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

## Why this order and not another

The large-vendor survey (`persona-c7a6`) decides it. Microsoft built the right idea and
bound it to one operating system, one identity provider and one SDK. AWS and Google solved
identity cloud-side and delegated the endpoint signal to a browser extension or a partner
agent. Nobody built a neutral local API.

So the differentiator is neutrality -- every OS, every source, one API, no vendor in the
middle -- which makes **breadth (1) and reach (3) the work that distinguishes this project**
and presence (5) a feature other people also have. An ordering that front-loaded presence
would spend the project's scarce effort on the part of the pitch nobody is missing.

## What is deliberately not built

- **A policy engine.** OPA exists. Write Rego, not an engine. See SPEC-HIRE.
- **Workload identity.** SPIFFE/SPIRE exists; `hired` federates with it.
- **Attestors for overlay networks that expose no human identity** -- NetBird, ZeroTier,
  Nebula, the WARP daemon. They would enumerate candidates that can never prove a person,
  which is the defect `persona-qbnm` exists to remove. Recorded in `persona-puyb`.
- **A competitor to WAM.** On Windows it is the best available answer; consume it.
