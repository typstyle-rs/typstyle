#!/usr/bin/env nu

# Run fresh base and PR benchmarks serially on one GitHub Actions runner.
# Invoke from the root checkout:
#   nu .github/scripts/benchmark-pair.nu revisions
#   nu .github/scripts/benchmark-pair.nu run
#
# `revisions` reads GITHUB_EVENT_NAME and GITHUB_EVENT_PATH. Manual dispatch
# also uses GITHUB_REPOSITORY, PR_NUMBER, and GH_TOKEN to fetch the PR with gh.
# It writes checkout outputs to GITHUB_OUTPUT and saves revisions.json plus
# pr-info/PR_NUMBER under BENCHMARK_RESULTS for execution and comment posting.
#
# `run` expects those revisions and a PR checkout at
# GITHUB_WORKSPACE/benchmark-workspace. Fetch the base commit from its
# repository using checkout's configured Git credentials, then build both
# revisions in this one checkout. GITHUB_SERVER_URL supplies the Git host.
# Switch revisions after the base build so Git updates changed files and
# Cargo reuses unchanged artifacts in the checkout's normal target/ directory.
# Keep that directory intact when switching revisions.
# Start and finish on the PR revision for rust-cache's dependency key and cleanup.
# The workflow selects Rust and supplies absolute BENCHMARK_RESULTS and
# CRITERION_HOME paths. Measurements are fresh and stay outside the build cache.
# CARGO_PROFILE_RELEASE_STRIP=false matches cargo-bloat's override so benchmark
# and CLI builds can reuse dependencies compiled with the same strip setting.
# git, cargo, rustc, lscpu, and cargo-bloat must be
# available. Run the report script afterward to compare the saved baselines.

# Resolve exact commit SHAs and their repositories for automatic or manual runs.
# Write base_sha, base_repository, head_sha, and head_repository to GITHUB_OUTPUT.
# Save revisions.json and pr-info/PR_NUMBER beneath BENCHMARK_RESULTS.
def 'main revisions' [] {
    let event = open $env.GITHUB_EVENT_PATH
    let pr = if $env.GITHUB_EVENT_NAME == workflow_dispatch {
        ^gh api $"repos/($env.GITHUB_REPOSITORY)/pulls/($env.PR_NUMBER)" | from json
    } else { $event.pull_request }
    let revisions = {
        base_sha: $pr.base.sha, base_repository: $pr.base.repo.full_name,
        head_sha: $pr.head.sha, head_repository: $pr.head.repo.full_name
    }
    mkdir ($env.BENCHMARK_RESULTS | path join pr-info)
    $pr.number | into string | save --raw -f ($env.BENCHMARK_RESULTS | path join pr-info/PR_NUMBER)
    $revisions | save -f ($env.BENCHMARK_RESULTS | path join revisions.json)
    $revisions | transpose key value | each {|item| $"($item.key)=($item.value)\n" }
        | str join | save --append $env.GITHUB_OUTPUT
}

# Buffer a command's stdout/stderr, save both verbatim, print them, and return
# its exit code. A process failure remains data so the other arm can still run.
def capture [
    command: closure # The external command to run.
    output: path # Stdout file; stderr is saved alongside it with a .stderr suffix.
] {
    let result = do $command | complete
    $result.stdout | save --raw -f $output
    $result.stderr | save --raw -f $"($output).stderr"
    print -n $result.stdout
    print -e -n $result.stderr
    $result.exit_code
}

# Run base bench/bloat, then PR bench/bloat with the workflow-selected compiler.
# Save compiler/CPU information and per-arm logs. Attempt both arms before
# failing with the collected command statuses and Criterion errors.
# Skip an arm if its revision cannot be checked out, but attempt the other arm.
def 'main run' [] {
    let revisions = open ($env.BENCHMARK_RESULTS | path join revisions.json)
    ^rustc -vV | save --raw -f ($env.BENCHMARK_RESULTS | path join environment.txt)
    ^lscpu | save --raw --append ($env.BENCHMARK_RESULTS | path join environment.txt)
    print (open --raw ($env.BENCHMARK_RESULTS | path join environment.txt))
    mkdir $env.CRITERION_HOME
    cd ($env.GITHUB_WORKSPACE | path join benchmark-workspace)
    mut failures = []
    for arm in [{name: base, sha: $revisions.base_sha} {name: pr, sha: $revisions.head_sha}] {
        let results = $env.BENCHMARK_RESULTS | path join $arm.name
        mkdir ($results | path join bloat)
        print $"Benchmarking ($arm.name) at ($arm.sha)"
        if $arm.name == base {
            let fetch = $results | path join fetch.txt
            let repository = $"($env.GITHUB_SERVER_URL)/($revisions.base_repository).git"
            let status = capture { ^git fetch --no-tags --depth 1 $repository $arm.sha } $fetch
            if $status != 0 {
                $failures = $failures | append $"($arm.name) at ($arm.sha): git fetch exited ($status)
Logs: ($fetch), ($fetch).stderr"
                continue
            }
        }
        let checkout = $results | path join checkout.txt
        let status = capture { ^git checkout --detach $arm.sha } $checkout
        if $status != 0 {
            $failures = $failures | append $"($arm.name) at ($arm.sha): git checkout exited ($status)
Logs: ($checkout), ($checkout).stderr"
            continue
        }
        let log = $results | path join output.txt
        let status = capture {
            ^cargo bench --locked -p typstyle-core --benches -- --save-baseline $arm.name
        } $log
        # Criterion can report failed cases on stdout while returning success.
        let errors = open --raw $log | lines | where {|line| $line | str contains 'Criterion.rs ERROR:' }
        if $status != 0 or not ($errors | is-empty) {
            $failures = $failures | append $"($arm.name) at ($arm.sha): cargo bench exited ($status)
($errors | str join (char newline))
Logs: ($log), ($log).stderr"
        }
        let bloat = $results | path join bloat $"bloat-($arm.name).json"
        print $"Measuring ($arm.name) binary size at ($arm.sha)"
        let status = capture {
            # cargo-bloat builds the release executable itself.
            ^cargo bloat --locked --release --crates --bin typstyle --message-format json
        } $bloat
        if $status != 0 {
            $failures = $failures | append $"($arm.name) at ($arm.sha): cargo bloat exited ($status)
Logs: ($bloat), ($bloat).stderr"
        }
    }
    if not ($failures | is-empty) {
        error make {msg: $"Benchmark failures:\n($failures | str join (char newline))"}
    }
}

# Resolve PR revisions, then run both benchmark arms; see the subcommand help.
def main [] { help main }
