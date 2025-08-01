#!/usr/bin/env nu

# Generate complete benchmark comparison report
# Usage: nu generate-benchmark-report.nu <base-results-dir> <pr-results-dir>

def main [base_dir: string, pr_dir: string] {
    # Generate benchmark comparison using critcmp
    let bench_result = generate_bench_report

    # Generate binary size comparison
    let binary_size_report = generate_bloat_report $base_dir $pr_dir

    # Generate the complete report
    let report = $"### 📊 Benchmark Performance Report

```console
($bench_result)
```

($binary_size_report)

Generated by GitHub Actions on (date now | format date '%Y-%m-%d %H:%M:%S UTC')
" # final newline required

    # Save to file
    $report | save -f benchmark-results.md

    print "Generated benchmark comparison report"
    print $report
}

def generate_bench_report [] {
    # Generate benchmark comparison using critcmp
    ^critcmp --color never base pr | str trim
}

def generate_bloat_report [base_dir: string, pr_dir: string] {
    let base_bloat_file = $"($base_dir)/bloat/bloat-base.json"
    let pr_bloat_file = $"($pr_dir)/bloat/bloat-pr.json"

    # Check if bloat files exist
    if not ($base_bloat_file | path exists) or not ($pr_bloat_file | path exists) {
        return "⚠️ Binary size data not available for comparison."
    }

    # Parse JSON files
    let base_data = ($base_bloat_file | open)
    let pr_data = ($pr_bloat_file | open)

    # Extract binary sizes from bloat JSON
    let base_size = $base_data."file-size"
    let pr_size = $pr_data."file-size"
    let base_text_size = $base_data."text-section-size"
    let pr_text_size = $pr_data."text-section-size"

    let diff = ($pr_size - $base_size)
    let text_diff = ($pr_text_size - $base_text_size)

    # Convert to human readable format
    let base_size_human = ($base_size | into filesize)
    let pr_size_human = ($pr_size | into filesize)
    let base_text_human = ($base_text_size | into filesize)
    let pr_text_human = ($pr_text_size | into filesize)

    # Calculate change indicators for file size
    let change_info = if $diff == 0 {
        {emoji: "=", text: "no change"}
    } else if $diff > 0 {
        let diff_human = ($diff | into filesize)
        let diff_percent = (($diff * 100.0) / $base_size | math round -p 2)
        {emoji: "📈", text: $"+($diff_human) \(+($diff_percent)%\)"}
    } else {
        let diff_abs = ($diff * -1)
        let diff_human = ($diff_abs | into filesize)
        let diff_percent = (($diff_abs * 100.0) / $base_size | math round -p 2)
        {emoji: "📉", text: $"-($diff_human) \(-($diff_percent)%\)"}
    }

    # Calculate change indicators for text section
    let text_change_info = if $text_diff == 0 {
        {emoji: "=", text: "no change"}
    } else if $text_diff > 0 {
        let diff_human = ($text_diff | into filesize)
        let diff_percent = (($text_diff * 100.0) / $base_text_size | math round -p 2)
        {emoji: "📈", text: $"+($diff_human) \(+($diff_percent)%\)"}
    } else {
        let diff_abs = ($text_diff * -1)
        let diff_human = ($diff_abs | into filesize)
        let diff_percent = (($diff_abs * 100.0) / $base_text_size | math round -p 2)
        {emoji: "📉", text: $"-($diff_human) \(-($diff_percent)%\)"}
    }

    # Generate bloat details
    let bloat_details = generate_bloat_diff $base_bloat_file $pr_bloat_file

    # Generate the report
    $"
### 📏 Binary Size Comparison

| Metric | Base | PR | Change |
|--------|------|----|---------|
| **File Size** | ($base_size_human) | ($pr_size_human) | ($change_info.emoji) ($change_info.text) |
| **Text Section** | ($base_text_human) | ($pr_text_human) | ($text_change_info.emoji) ($text_change_info.text) |
($bloat_details)"
}

def generate_bloat_diff [base_file: string, pr_file: string] {
    # Parse JSON files
    let base_data = ($base_file | open)
    let pr_data = ($pr_file | open)

    # Extract top 20 crates with human-readable sizes and aligned formatting
    let base_crates = ($base_data.crates | first 20 | each { |crate|
        $"($crate.name | fill -a l -w 25) ($crate.size | into filesize)"
    })
    let pr_crates = ($pr_data.crates | first 20 | each { |crate|
        $"($crate.name | fill -a l -w 25) ($crate.size | into filesize)"
    })

    # Generate diff with full context
    let base_content = ($base_crates | str join "\n")
    let pr_content = ($pr_crates | str join "\n")

    let diff_output = if $base_content == $pr_content {
        $"No changes detected in crate sizes:

($base_content)"
    } else {
        # Save to temp files for diff
        $base_crates | save -f base_crates.tmp
        $pr_crates | save -f pr_crates.tmp

        let result = try {
            # Note: `diff` exits with 1 in case of difference
            ^diff -U5 base_crates.tmp pr_crates.tmp | lines | skip 2 | str join "\n"
        } catch {
            ""
        }

        # Clean up temp files
        rm -f base_crates.tmp pr_crates.tmp

        $result
    }

    # Return details - always show the diff section
    $"

<details>
<summary>📦 Detailed Crate Size Diff \(cargo-bloat\)</summary>

**Note:** Numbers above are a result of guesswork. They are not 100% correct and never will be.

```diff
($diff_output)
```

</details>"
}
