//! Emits the build-time metadata shown in the binary's `--version` output.

use anyhow::Result;

fn main() -> Result<()> {
    let build = vergen::BuildBuilder::default()
        .build_timestamp(true)
        .build()?;
    let cargo = vergen::CargoBuilder::all_cargo()?;
    let rustc = vergen::RustcBuilder::default()
        .commit_hash(true)
        .semver(true)
        .host_triple(true)
        .channel(true)
        .llvm_version(true)
        .build()?;
    #[cfg(feature = "git-info")]
    let gitcl = vergen_gitcl::GitclBuilder::default()
        .sha(false)
        .describe(true, true, None)
        .build()?;

    // Emit the instructions
    let mut emitter = vergen::Emitter::default();
    emitter
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?;
    #[cfg(feature = "git-info")]
    emitter.add_instructions(&gitcl)?;

    emitter.emit()?;

    Ok(())
}
