#!/usr/bin/env nu

# Sync the CLI usage block in README.md with the output of `typstyle -h`.
#
# The block is the code block below the marker comment in the "CLI Usage"
# section. The script refuses to touch a README that does not contain exactly
# one marker line, so unrelated code blocks are never rewritten.
#
# Usage, from anywhere in the repository:
#   ./scripts/sync-readme-cli-help.nu
#   ./scripts/sync-readme-cli-help.nu --check
#   ./scripts/sync-readme-cli-help.nu --readme path/to/README.md

const MARKER = "<!-- The CLI usage below is synced from `typstyle -h` by `just sync-readme-cli-help`. -->"

# Reports an error on stderr and exits with status 1.
def fail [message: string] {
    print --stderr $"error: ($message)"
    exit 1
}

# The repository root, resolved from this script rather than the working directory.
def repo_root [] {
    $env.FILE_PWD | path dirname | path expand
}

# Formats a path relative to the repository when possible.
def display [path: path] {
    try { $path | path expand | path relative-to (repo_root) } catch { $path | into string }
}

# Normalizes CLI help output for embedding in Markdown.
def normalize_help [help_text: string] {
    let lines = (
        $help_text
        | str trim
        | lines
        | each {|line| $line | str trim --right | str replace --all "typstyle.exe" "typstyle" }
    )
    if not ($lines | any {|line| $line | str contains "Usage: typstyle" }) {
        fail "the CLI help output has no usage line"
    }
    $lines
}

# Locates the code block below the marker, returning its fence line indices.
def find_block [lines: list<string>] {
    let markers = ($lines | enumerate | where {|entry| ($entry.item | str trim) == $MARKER })
    if ($markers | length) != 1 {
        fail $"expected exactly one marker line, found ($markers | length)"
    }

    let opening = (
        $lines
        | enumerate
        | skip (($markers | first | get index) + 1)
        | where {|entry| ($entry.item | str trim) | str starts-with '```' }
        | first
    )
    if ($opening | is-empty) {
        fail "expected a code block after the CLI usage marker"
    }

    let closing = (
        $lines
        | enumerate
        | skip ($opening.index + 1)
        | where {|entry| ($entry.item | str trim) == '```' }
        | first
    )
    if ($closing | is-empty) {
        fail "unterminated code block after the CLI usage marker"
    }
    { opening: $opening.index, closing: $closing.index }
}

def main [--check, --readme: path] {
    let help_lines = (
        normalize_help (
            ^cargo run --manifest-path ((repo_root) | path join Cargo.toml) -p typstyle -- -h
        )
    )
    let readme = (if ($readme | is-empty) { (repo_root) | path join README.md } else { $readme })

    let raw = (open --raw $readme)
    let lines = ($raw | lines)
    let block = (find_block $lines)

    let body = ($lines | slice ($block.opening + 1)..<($block.closing))
    if $body == $help_lines {
        print $"((display $readme)): the CLI usage block is up to date."
        return
    }
    if $check {
        print --stderr $"error: the CLI usage block in (display $readme) is out of date"
        print --stderr 'Run `just sync-readme-cli-help` to update it.'
        exit 1
    }

    let updated = (
        ($lines | slice ..($block.opening))
        | append $help_lines
        | append ($lines | slice ($block.closing)..)
        | str join "\n"
    )
    let trailing = (if ($raw | str ends-with "\n") { "\n" } else { "" })
    $"($updated)($trailing)" | save --force $readme
    print $"Updated the CLI usage block in (display $readme)."
}
