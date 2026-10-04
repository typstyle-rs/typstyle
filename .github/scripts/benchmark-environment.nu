# Keep validation independent of report generation so incompatible measurements
# fail before critcmp can turn them into a misleading regression report.
export def validate-environments [base_dir: string, pr_dir: string] {
    let base_path = ($base_dir | path join benchmark-environment.json)
    let pr_path = ($pr_dir | path join benchmark-environment.json)
    if not ($base_path | path exists) or not ($pr_path | path exists) {
        error make {msg: "Benchmark environment metadata is missing. Rerun both benchmark jobs."}
    }

    let base = open $base_path
    let pr = open $pr_path
    if ($base.fingerprint | is-empty) or ($pr.fingerprint | is-empty) {
        error make {msg: "Benchmark environment fingerprints are missing. Rerun both benchmark jobs."}
    }
    if $base.fingerprint != $pr.fingerprint {
        error make {msg: "Benchmark compiler or runner environments differ. Rerun both jobs with the same compiler and runner hardware before comparing results."}
    }
    $base
}
