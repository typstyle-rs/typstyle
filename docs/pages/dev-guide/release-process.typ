#import "../book.typ": *
#import "../../deps.typ": *

#show: book-page.with(title: "Release Process")

= Release Process

#callout.note[
  This document provides a template for the release process. Replace `<latest-tag>` with the actual latest tag (found using `git describe --tags --abbrev=0`) and `<new-version>` with the target version number throughout this guide.
]

This document provides a comprehensive guide for the Typstyle project release process, including version updates, changelog maintenance, build testing, and publishing steps.

== Preparation Phase

=== Check Current Status

Before starting a release, verify the current state of the repository:

```bash
# Check git status, ensure working directory is clean
git status

# Check current version
grep version Cargo.toml

# View recent commit history
git log --oneline -10

# Find the latest release tag
git describe --tags --abbrev=0

# View changes since last release (replace <latest-tag> with actual tag)
git log --oneline <latest-tag>..HEAD --no-merges

# Example: If the latest tag is v0.13.11, use:
# git log --oneline v0.13.11..HEAD --no-merges
```

=== Determine New Version Number

Follow semantic versioning rules to determine the version number:

- *Major (x.0.0)*: Breaking changes
- *Minor (0.x.0)*: New features, backward compatible
- *Patch (0.0.x)*: Bug fixes, backward compatible

== Analyze Changes

=== Review Specific Code Changes

Examine the actual code changes to understand what has been modified:

```bash
# View specific changes for each commit (crates directory only)
git show <commit-hash> -- crates/

# Or view all unreleased changes (replace <latest-tag> with actual tag)
git diff <latest-tag>..HEAD -- crates/
```

=== Categorize Change Types

Classify the changes into appropriate categories:

- *Feature*: New functionality
- *Bug fix*: Error corrections
- *Enhancement*: Improvements
- *Performance*: Performance optimizations
- *Refactor*: Code refactoring
- *CLI*: Command-line related changes
- *Breaking*: Breaking changes

== Update Version Numbers

Run the release preparation script without an argument to increment the patch version automatically:

```bash
just prepare-release
```

For a minor or major release, provide the stable semantic version explicitly without the `v` prefix: `just prepare-release <new-version>`.

The script checks that the working tree is clean, updates the workspace and internal dependency versions, refreshes `Cargo.lock`, and runs `cargo check --workspace`. If `CHANGELOG.md` has an `Unreleased` section, the script turns it into the dated release section and preserves its notes. Otherwise, it creates a release section with a `TODO` placeholder. It does not commit, tag, push, or publish anything.

=== Update Workspace Version

The script updates the workspace version in `Cargo.toml`:

```toml
[workspace.package]
version = "<new-version>"  # New version number (e.g., "0.13.12")
```

=== Update Dependency Versions

It also keeps the exact internal dependency version synchronized:

```toml
[workspace.dependencies]
typstyle-core = { path = "crates/typstyle-core", version = "=<new-version>" }
```

== Update Changelog

=== Add New Version Entry

Keep upcoming release notes in an optional `Unreleased` section at the top of `CHANGELOG.md`:

````markdown
## Unreleased

- Feature(CLI): Specific feature description
- Bug fix: Specific fix description
- Enhancement: Specific improvement description
````

The release script changes this heading to `## v<new-version> - [YYYY-MM-DD]`. If the section is absent, it inserts a version heading with a `TODO` instead.

=== Changelog Writing Guidelines

When writing changelog entries, follow these guidelines:

- Use present tense to describe changes
- Include specific examples when applicable
- Mark breaking changes clearly
- Sort by importance (Feature > Enhancement > Bug fix)
- Add CLI-related tags for command-line changes
- Include related issue/PR links when applicable

=== Changelog Classification Standards

Use these standard categories for changelog entries:

- *Feature*: New features
- *Feature(CLI)*: Command-line new features
- *Bug fix*: Error corrections
- *Enhancement*: Feature improvements
- *Performance*: Performance optimizations
- *(breaking)*: Breaking change marker

== Commit Changes

=== Commit Version Updates

Commit the version and changelog updates:

```bash
# Add changed files
git add Cargo.toml Cargo.lock CHANGELOG.md

# Commit changes
git commit -m "chore: update version to v<new-version> and update changelog"
```

=== Create Tags

Create and push the version tag:

```bash
# Create version tag with signature
git tag -s v<new-version>

# Push to remote repository
git push origin master
git push origin v<new-version>
```

== CI/CD and Publishing

=== Monitor CI

After pushing:

- Check GitHub Actions or other CI systems
- Ensure all tests pass
- Ensure build succeeds

=== Automatic Publishing

CI usually automatically detects new tags and publishes to:

- crates.io (Rust crates)
- npm (if WASM package exists)
- GitHub Releases

=== Update Documentation

After a successful release:

- Check if project homepage is updated
- Update version references in related documentation
- Notify community about new release

== Checklist

Please confirm the following items before release:

- #box[☐] Check git status, ensure working directory is clean
- #box[☐] Analyze all changes since last release
- #box[☐] Determine appropriate new version number
- #box[☐] Update version number in `Cargo.toml`
- #box[☐] Update `CHANGELOG.md` with new version entry
- #box[☐] Build project to ensure no errors
- #box[☐] Run tests (optional but recommended)
- #box[☐] Commit version updates and changelog
- #box[☐] Create and push version tag
- #box[☐] Monitor CI/CD process
- #box[☐] Verify successful release
- #box[☐] Update related documentation
