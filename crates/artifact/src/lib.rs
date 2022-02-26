mod buildinfo;
mod docker;
mod engine;
mod git;
mod util;
mod wasm;

use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

pub use buildinfo::BuildInfo;
pub use docker::Dockerfile;
pub use engine::{
    build_engine, get_built_engine, get_known_dockerfile_path, get_known_engine_path, list_engines,
    EngineName,
};
pub use git::{resolve_to_commit, GitLocation};
pub use wasm::WasmBenchmark;

pub const SIGHTGLASS_PROJECT_DIRECTORY: &'static str = env!("SIGHTGLASS_PROJECT_DIRECTORY");

/// Get the local directory where sightglass stores cached data.
pub fn sightglass_data_dir() -> Result<PathBuf> {
    let mut p = dirs::data_local_dir().ok_or_else(|| {
        anyhow::anyhow!(
            "missing an application data folder for storing sightglass engines; e.g. \
             /home/.../.local/share, C:\\Users\\...\\AppData\\Local"
        )
    })?;
    p.push("sightglass");
    Ok(p)
}

/// Clean up and remove any cached data.
pub fn clean() -> Result<()> {
    let sightglass_data_dir = sightglass_data_dir()?;
    if !sightglass_data_dir.is_dir() {
        return Ok(());
    }

    log::info!(
        "Recursively removing sightglass cached data in {}",
        sightglass_data_dir.display()
    );
    fs::remove_dir_all(&sightglass_data_dir).with_context(|| {
        format!(
            "failed to recursively remove {}",
            sightglass_data_dir.display(),
        )
    })?;

    Ok(())
}
