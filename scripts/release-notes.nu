#!/usr/bin/env nu

# Print one release section from CHANGELOG.md as GitHub release notes.
#
# Usage: nu .github/scripts/release-notes.nu <version> [<changelog>]
#
# The version heading must match exactly, so the permanent `## Unreleased`
# section at the top of the changelog is skipped. The body ends at the next
# `## ` heading. Exits non-zero when the version has no non-empty section.

def main [
    version: string # version heading to look up, e.g. "v0.15.3"
    changelog: string = "CHANGELOG.md" # changelog to read
] {
    let lines = (open --raw $changelog | lines)
    let starts = ($lines | enumerate | where {|line|
        $line.item == $"## ($version)" or ($line.item | str starts-with $"## ($version) ")
    })
    let notes = (if ($starts | is-empty) { [] } else {
        let start = ($starts | first | get index) + 1
        $lines
        | skip $start
        | take while {|line| not ($line | str starts-with '## ')}
        | str join (char nl)
        | str trim
    })
    if ($notes | is-empty) {
        print --stderr $"no release notes found for ($version)"
        exit 1
    }
    print $notes
}
