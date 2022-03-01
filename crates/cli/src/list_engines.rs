use anyhow::Result;
use sightglass_build::engine::list_engines;
use sightglass_fingerprint::Engine;
use structopt::StructOpt;

/// List the built Wasm engines known to the Sightglass cache.
#[derive(Debug, StructOpt)]
#[structopt(name = "list-engines")]
pub struct ListEnginesCommand {
    /// Force each entry on to a single line.
    #[structopt(long, short)]
    oneline: bool,
}

impl ListEnginesCommand {
    pub fn execute(&self) -> Result<()> {
        for (name, path) in list_engines()? {
            println!("{}", name);
            if !self.oneline {
                let fingerprint = match Engine::fingerprint(&path) {
                    Ok(finterprint) => finterprint,
                    Err(err) => {
                        println!("  Unable to fingerprint: {}", err);
                        continue;
                    }
                };
                if name.to_string() != fingerprint.name {
                    log::warn!(
                        "The cache directory name and the fingerprint name do not match: {} != {}",
                        name,
                        fingerprint.name
                    );
                }
                println!("  Path: {}", path.display());
                if let Some(rebuild) = fingerprint.rebuild {
                    println!("  Rebuild command: {}", rebuild);
                }
                if let Some(buildinfo) = fingerprint.buildinfo {
                    println!("  .build-info:");
                    for line in buildinfo.lines() {
                        println!("    {}", line);
                    }
                }
            }
        }
        Ok(())
    }
}
