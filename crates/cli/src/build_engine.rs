use anyhow::Result;
use sightglass_artifact::{build_engine, get_known_engine_path};
use structopt::StructOpt;

/// Build a Wasm engine from either a BUILD-INFO string or a Dockerfile and print the path to the
/// generated library.
#[derive(Debug, StructOpt)]
#[structopt(name = "build-engine")]
pub struct BuildEngineCommand {
    /// Force this tool to rebuild the benchmark.
    #[structopt(long, short)]
    force_rebuild: bool,

    /// Either a BUILD-INFO string (e.g. `wasmtime` or `wasmtime?COMMIT=92350bf2` or
    /// `wasmtime?COMMIT=92350bf2&RUSTC=1.60`) or a path to a Dockerfile. See TODO for more
    /// information on build-info strings.
    #[structopt(index = 1, required = true, value_name = "ENGINE-REF OR DOCKERFILE")]
    location: String,
}

impl BuildEngineCommand {
    pub fn execute(&self) -> Result<()> {
        let engine_path = get_known_engine_path(&self.location)?;
        if !engine_path.exists() || self.force_rebuild {
            build_engine(&self.location, &engine_path)?;
        }
        println!("{}", engine_path.display());
        Ok(())
    }
}
