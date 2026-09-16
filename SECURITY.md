<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Security Policy

## Reporting a vulnerability

Email **mark@reviewcommit.com**, or open a [private security
advisory](https://github.com/MarkAtwood/project-HIRE/security/advisories/new)
on the repository. Do not open a public issue for a vulnerability.

**What to expect, stated honestly rather than as a service level:** one person
reads that mailbox, and there is no on-call rotation behind it. Assume days, not
hours, and say in the subject line if the issue is being actively exploited.
There is no bug bounty.

## What is in scope

`hired` is a user-session daemon that answers "who is this human" to local
applications and hands out signed credentials. The things worth reporting:

- a consumer obtaining an SVID for an identity it should not reach, or any path
  around the per-connection consumer attestation;
- a claim asserting an assurance tier, an authentication method or a presence
  level that the evidence does not establish -- the discovery/proof split exists
  to make that impossible, so a way around it is a real finding;
- recovering another consumer's pseudonym, or linking two consumers' pseudonyms
  back to one root identity;
- a signature, token or trust bundle accepted when it should be refused;
- key material reaching disk, a log, or a process that should not see it.

## What is not

- Anything requiring an attacker who is already root on the machine, or who can
  replace a binary the daemon trusts. Trust follows the install: whoever can
  replace `/usr/bin/gpg` or the ssh agent can replace `hired` itself, and a
  check below that line defends against an attacker who has already won.
- A source that refuses to prove. Declining is the designed behaviour; no
  evidence means no claim.
- Anything in a `*(not implemented)*` path. The documentation marks these
  deliberately -- see the status table in [README.md](README.md).

## Supported versions

There are no releases yet. `main` is the only supported branch, and fixes land
there.
