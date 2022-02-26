use anyhow::Result;
use sightglass_artifact::list_engines;
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
        for (name, path, buildinfo) in list_engines()? {
            println!("{} -> {}", name, path.display());
            if let Some(buildinfo) = buildinfo {
                for line in buildinfo.as_file_string().lines() {
                    println!("  {}", line);
                }
            }
        }
        Ok(())
    }
}
