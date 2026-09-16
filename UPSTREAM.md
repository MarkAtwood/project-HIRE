<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Donating HIRE to SPIFFE

**Not yet, and the order matters more than the paperwork.** HIRE should ask SPIFFE for the
smallest thing first, and the smallest thing is not a donation -- it is attendance. The
donation is worth preparing for and worth being patient about, because the one asset that
makes the case is adoption, and HIRE has none yet.

This document holds the plan. **It states no engineering status of its own:** what works
is the Status table in [README.md](README.md), what is queued is the bead tracker, and
this document points at both rather than repeating them.

---

## How SPIFFE is actually governed

SPIFFE graduated from the CNCF incubator on 2022-08-23, announced 2022-09-20, alongside
SPIRE. Graduated status matters here: the project sets its own technical direction, and
the CNCF TOC is not in the path for a sub-project decision.

The **SPIFFE Steering Committee (SSC)** is the governance head and is "exclusively
responsible for SPIFFE's standards and the Project's strategic goals", with final
authority over technical direction, governance and process, and contribution policy. It
has at least five members, no more than two from any one organization, and -- the rule
worth reading twice -- **at least 40% must be from organizations running a SPIFFE
implementation in production.** Terms are 24 months.

Below the SSC are three SIGs: `sig-community`, `sig-spec` and `sig-spire`. A proposal
"under the purview of a SIG" goes to the SIG lead first. Decisions run on lazy consensus;
new or changed functionality needs two maintainer approvals.

Specifications live in `spiffe/spiffe/standards/` -- `SPIFFE-ID.md`, `X509-SVID.md`,
`JWT-SVID.md`, `SPIFFE_Workload_API.md`, `SPIFFE_Federation.md` and others, now including
`WIT-SVID.md` and a Broker API. They carry four stability levels: **Proposed**,
**Experimental**, **Incubating**, **Stable**. Incubating requires SIG-Spec consensus that
"the design is sound and ready for real world scenarios" plus two maintainer approvals;
Stable comes after multiple interoperable implementations.

**There is no documented process for adding a repository to the `spiffe` GitHub
organization.** The governance document does not describe one. That is a finding, not an
obstacle: it means the route is a conversation with the SSC rather than a form, and it
means the first move is to ask how, not to arrive with a filing.

The precedent is **Tornjak**, donated by IBM in September 2021 and now at
`github.com/spiffe/tornjak`. It arrived with a company behind it and a clear relationship
to SPIRE. That is the shape of a successful approach.

---

## What HIRE would be asking for

Three separate asks, in ascending cost to the other side. They are independent, and the
temptation is to ask for the third first.

**1. Presence in the community.** Slack, the mailing lists, the weekly community call,
SIG-Spec attendance. Costs SPIFFE nothing and requires no decision from anyone. HIRE has
not done this.

**2. A repository in `github.com/spiffe`.** An SSC decision, plus a Linux Foundation
Project Contribution Agreement and transfer of any trademark to the LF before onboarding
completes. This is the donation proper.

**3. A SPIFFE standard for human identity.** SIG-Spec and the SSC, entering at Proposed
and climbing the stability ladder. This is the largest ask and the one that should come
last.

### Why the spec ask goes last, and may never be needed

**HIRE requires no change to SPIFFE to work.** That is not an argument, it is a test
result: `hire-grpc/tests/stock_client.rs` drives the third-party `spiffe` crate -- its own
protobuf copy, endpoint parsing, SVID types and JWT verifier -- through connect, fetch,
bundle fetch and cryptographic validation against `hired`, unmodified. The extensions
HIRE adds are all ignorable by construction:

- the `hire` claim block is an extra JWT claim, and a consumer that does not understand it
  sees an ordinary JWT-SVID;
- the `hire_` audience parameters are query parameters inside an audience string, which
  SPIFFE treats as opaque;
- the `hint` tag is a field the Workload API already defines.

So HIRE can be adopted, deployed and useful with the spec untouched. Proposing a standard
before there are users is asking a standards body to bless a design with one
implementation and no deployments, which is the reliable way to receive a polite no. After
adoption, the same proposal is a description of something that already works -- a much
easier document to say yes to.

---

## What we control

The CNCF hygiene items are done: [CONTRIBUTING.md](CONTRIBUTING.md),
[GOVERNANCE.md](GOVERNANCE.md), [MAINTAINERS.md](MAINTAINERS.md),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) and [SECURITY.md](SECURITY.md) are in the tree,
every commit carries a DCO sign-off under a single author identity, and both license
texts are in every commit from the root.

**Keep it that way.** A sign-off that does not match its author, or a commit landing
without one, is cheap to prevent and expensive to fix once anyone else has pulled.
[CONTRIBUTING.md](CONTRIBUTING.md) states the requirement.

**Name and trademark.** Donation means assigning the trademark to the Linux Foundation, so
the name has to be one we can give away. `hire`/`HiRE` is taken on crates.io and
`github.com/hire` is taken; `github.com/spiffe-hire` was free as of 2026-09-15. Do not
accumulate brand equity in a name that cannot be assigned.

---

## What the community weights

The SSC composition rule -- 40% from organizations running SPIFFE in production -- says
what this community weights. Not design quality. Deployment.

**Lead with the conformance result, not the conformance claim.**
`hire-grpc/tests/stock_client.rs` drives a third-party SPIFFE client library through
connect, fetch, hint, bundle fetch and cryptographic validation against `hired`,
unmodified. It is the strongest artifact this project has, it is checkable by anyone in
under a minute, and it is an argument no specification can make.

Four questions will be asked, and the answer to each lives somewhere that stays current:

| Question | Where the answer is |
|---|---|
| What actually works? | The Status table in [README.md](README.md) |
| What is queued, and why is it not done? | The tracker: `bd ready`, and the bead named in each phase below |
| Who maintains it? | [MAINTAINERS.md](MAINTAINERS.md) |
| How many users? | Nowhere -- nothing tracks this. Count it honestly at the time, and expect zero to be the answer for a while |

The two that will hurt are the last two. A one-person project is a succession risk and a
foundation treats it as one; and adoption is the thing this plan cannot manufacture by
being well written.

---


## Sequence

Each phase has an exit criterion. Do not start the next one early; the cost of asking too
soon is a "no" that is expensive to reverse.

**These phases name no engineering status.** They name the bead that gates them, and the
tracker says whether it is done.

**Phase 0 -- be a participant.** Join `spiffe.slack.com`, the `spiffe-dev` and
`spiffe-users` lists, and the weekly community call. Attend SIG-Spec. Answer other
people's questions. Ask, without proposing anything, how the project thinks about human
identity on the Workload API -- it may already have an opinion, and if it has a bad
experience with this idea it is far cheaper to learn that now.

Lead with the origin story rather than the design. It is the first section of
[MOTIVATIONS.md](MOTIVATIONS.md): an application that used Tailscale as its identity
service, and the question of whether a standard API existed for that. Several people in
that room arrived at SPIFFE by the same route from the workload side, and a story they
recognise buys more attention than a specification does. It also answers the "why not
just SPIRE" question below before anyone has to ask it, because it starts from a human
at a keyboard rather than from a cluster.
*Gated by nothing. Exit: known by name to SIG-Spec, and an answer to whether human
identity is welcome in scope.*

**Phase 1 -- make the demo undeniable.** A stock consumer of either credential flavour
should work, and the identity most enterprises actually care about should be one of the
sources.
*Gated by `persona-apyr` (X.509-SVID issuance) and `persona-qbnm.7` (the OIDC verifier).
Exit: `spiffe-helper` or `ghostunnel` runs against `hired` unmodified.*

**Phase 2 -- ask the small question.** Offer a SIG-Spec demo or a community-call talk. Ask
about an ecosystem listing on spiffe.io. This is a request for attention, not for
governance, and the answer tells you what the donation conversation would be like.
*Exit: a public demo given, and an explicit reaction to the human-identity idea.*

**Phase 3 -- offer the donation.** Propose to the SSC that HIRE join as a sub-project, with
the Tornjak precedent as the model. Bring the adoption story, the conformance evidence and
a named second maintainer. Expect the LF Contribution Agreement and trademark assignment.
*Exit: an SSC decision either way. A "not yet" with conditions is a good outcome and
should be written down here.*

**Phase 4 -- propose the standard, if it is still wanted.** By now HIRE either has users or
does not. With users, a `SPIFFE_Human_Identity.md` entering at **Proposed** describes
something real: the trust-domain shapes for human sources, the assurance and presence
vocabularies, per-consumer pseudonymous SPIFFE IDs, and the audience-extension namespace.
Without users, skip it: the extensions are ignorable and the implementation stands alone.
*Exit: Experimental or Incubating status, or a deliberate decision not to standardize.*

---


## What we would be giving up

Worth being explicit, because it is not reversible:

- **The trademark**, assigned to the Linux Foundation.
- **Unilateral technical direction.** The SSC has final authority over it.
- **Release cadence and process**, which become the project's rather than ours.
- **Freedom to change the wire format.** The `hire` claim block and the `hire_` audience
  namespace are today changeable at will. Under a graduated project's stability rules they
  would not be. The HKDF scheme label is already versioned for this reason
  (`hire-pseudonym-v1:`); the rest of the wire surface is not, and should be before it
  matters.

In exchange: a trademark that cannot be taken away by a competitor, an IP position that
survives its author, a review culture that is stricter than one person can be alone, and
the only distribution channel that reaches the people who already run SPIFFE.

---

## Tripwires

Signs the plan is not working, and what each one means:

- **Phase 0 produces indifference rather than argument.** If SIG-Spec has no opinion about
  human identity on the Workload API, the idea is not contentious -- it is uninteresting to
  them. Reconsider whether SPIFFE is the right home, or whether HIRE is better off as an
  independent implementation of a SPIFFE-compatible API.
- **"Why is this not just SPIRE with a different attestor?"** Have the answer ready: SPIRE
  attests processes against a server-side registry; HIRE aggregates local human-identity
  custodians with no server and no registration, and a human is not a workload because a
  human can be absent. The origin story makes the same point without argument. If neither
  lands, the positioning is wrong, not the audience.
- **Someone else ships it first.** The correct response is to help them, not to race. The
  goal is a standard local API for "who is this human", not our implementation of it.

---

## Sources

- [SPIFFE and SPIRE graduate from the CNCF incubator](https://www.cncf.io/announcements/2022/09/20/spiffe-and-spire-projects-graduate-from-cloud-native-computing-foundation-incubator/)
- [SPIFFE governance](https://github.com/spiffe/spiffe/blob/main/GOVERNANCE.md) -- SSC composition and authority
- [SPIFFE contributing guide](https://github.com/spiffe/spiffe/blob/main/CONTRIBUTING.md) -- DCO and SIG review
- [SPIFFE standards directory](https://github.com/spiffe/spiffe/tree/main/standards) and [STABILITY.md](https://raw.githubusercontent.com/spiffe/spiffe/main/standards/STABILITY.md)
- [SIG creation procedure](https://raw.githubusercontent.com/spiffe/spiffe/main/community/sig-creation-procedure.md)
- [Tornjak](https://github.com/spiffe/tornjak) and [IBM's donation announcement](https://research.ibm.com/blog/tornjak-project-cncf)
- [CNCF project lifecycle and process](https://contribute.cncf.io/projects/lifecycle/) -- Contribution Agreement and trademark transfer
- [SPIFFE: get involved](https://spiffe.io/docs/latest/spiffe-about/get-involved/)

*Governance facts checked 2026-09-16. Re-check before acting on any phase: the SSC turns
over on 24-month terms and the standards set has grown since this was written.*
