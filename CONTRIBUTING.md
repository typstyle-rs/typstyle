# Contributing

Thank you for considering a contribution to **typstyle**, a code formatter for Typst. Bug fixes, tests, and documentation improvements are all welcome.

## Quick Start

1. **Reproduce first**: Use our [playground](https://typstyle-rs.github.io/typstyle/playground) to verify formatting behavior before opening an issue or a PR.
2. **Read the docs**: See our [documentation](https://typstyle-rs.github.io/typstyle/) for usage examples.
3. **Follow the guide**: See our [Developer Guide](https://typstyle-rs.github.io/typstyle/dev-guide.html) for setup instructions.

## Contribution Policy

We care about intent, understanding, and follow-through. These rules apply to every pull request.

### Own and understand your changes

Do not open a PR consisting of unreviewed or unedited LLM output.

LLM-assisted changes are fine, provided you reviewed the patch and can explain what it does, why it is needed, and why the approach is right.

### Quality bar

Keep PRs focused, complete, and free of AI slop: avoid over-defensive checks, redundant tests, and poorly organized file structure. Test changed behavior and update docs where needed.

### Every PR needs a motivation

Do not open a PR without a concrete reason. "Refactoring", "cleanup", and "improvements" are not motivations on their own: state the bug, limitation, or user-facing problem being addressed.

Link an existing issue when one exists. Behavior changes must show the expected behavior before and after.

### Design changes require prior agreement

Changes to formatting rules (beyond bug fixes), the public API, CLI/options, file layout, or architecture affect the whole project. Open an issue and get maintainer agreement on the design before writing code, then link that agreed issue in the PR.

### Stay active

Authors are expected to answer questions, discuss alternatives, and address review feedback. Inactive PRs may be closed.

### Enforcement

Maintainers may close a PR that violates these rules without a full review, keeping review time for contributors who understand their changes and engage in discussion.

## Pull Request Guidelines

### Bug fixes

- Link the bug report, or include a minimal reproduction if none exists.
- Add a regression test that fails before the fix and passes after.
- Stay focused: put unrelated refactors in a separate PR.
- Preserve formatter invariants: formatting twice must converge, and rendered output must not change.
- Add an entry under `Unreleased` in `CHANGELOG.md` for user-visible fixes.

### New features

- Start from an agreed issue with maintainer sign-off.
- Include before/after examples when changing formatting.
- Add snapshot, convergence, and rendering-correctness tests where applicable.
- Update relevant docs, `CHANGELOG.md`, and follow the [Formatter Options guide](https://typstyle-rs.github.io/typstyle/dev-guide/formatter-options.html) when changing options.
