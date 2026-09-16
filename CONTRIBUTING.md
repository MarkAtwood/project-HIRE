<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Contributing

Issues and pull requests are welcome. There is no CLA. Read
[GOVERNANCE.md](GOVERNANCE.md) for who decides and
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for how to behave while doing it.

## Sign your work: the Developer Certificate of Origin

Every commit must carry a `Signed-off-by` line, which `git commit -s` adds:

```
Signed-off-by: Your Name <your.email@example.com>
```

That line is a certification under the [Developer Certificate of
Origin](https://developercertificate.org/) that you wrote the patch or otherwise
have the right to submit it under this project's license. It must match the
commit author. Commits without it will not be merged.

## Licensing of contributions

Code is Apache-2.0, documentation is CC-BY-4.0, and contributions are accepted
under those terms. All work in this repository from the first commit onward is
contributed under them, including the commits made before the license files
were added to the tree.

Do not change a license tag or an SPDX identifier in a pull request. Raise it as
an issue instead.

## Commit messages

- Conventional-commit prefix: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`,
  `chore:`, `deps:`, `build:`, `ci:`.
- Subject in the imperative, **50 characters or fewer**.
- Body wrapped at **72 columns**, separated from the subject by a blank line.
- **ASCII only.** No em dashes, no curly quotes. Use `--`.
- Say what changed and why. The expensive half of a commit message is the
  alternative you rejected and the reason -- that is the part nobody can
  reconstruct from the diff.

## Attribution

**AI agents do not take credit.** No `Co-Authored-By` trailer naming a model, no
"Generated with" footer, no attribution in source comments or documentation, and
nothing on GitHub under an agent's identity. The human contributor is the author
and the one who signs off. [AGENTS.md](AGENTS.md) has the full rule.

## Before you open a pull request

```sh
make ci     # fmt-check, clippy -D warnings, tests, cargo audit,
            # dependency policy, cross-target checks
```

All of it must pass. Three further rules that are not negotiable:

- **Never weaken a test to make it pass.** Fix the code, or say the code cannot
  be fixed in scope and leave the test failing with an explanation.
- **A test may not be its own oracle.** Encrypting with a function and
  decrypting with the same function proves nothing. Use published test vectors,
  a second implementation, or a byte-exact comparison against a reference.
- **Cryptography is reached through the RustCrypto traits, and `ring`,
  `aws-lc-rs` and `aws-lc-sys` are not used.** `deny.toml` enforces it and
  `make bans` runs the check. A crate that needs one of them needs a different
  crate; where a dependency offers a choice of backend, select the RustCrypto
  one explicitly in `Cargo.toml` rather than taking the default.

## Issue tracking

This project uses [beads](https://github.com/steveyegge/beads) rather than
GitHub issues for its own backlog; the tracker lives in `.beads/` in the tree.
Issues opened on GitHub are read and welcome -- they get filed into the tracker
by a maintainer.
