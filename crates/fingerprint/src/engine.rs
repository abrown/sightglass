use crate::util::{sha256, stringify};
use serde::{Deserialize, Serialize};
use sightglass_artifact::{get_known_dockerfile_path, BuildInfo, Dockerfile};
use std::{fs, path::Path};

/// Describes a WebAssembly engine.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Engine {
    /// The canonical name for the engine; e.g.:
    /// - `wasmtime@<commit>` for an engine built using Sightglass with the given commit
    /// - `custom-<hash>` for a pre-built engine at a given path
    pub name: String,
    /// The path to the engine.
    pub path: String,
    /// Describes how to rebuild the engine using Sightglass, if this method was used.
    pub rebuild: Option<String>,
}

impl Engine {
    /// Extract the build information
    pub fn fingerprint<P: AsRef<Path>>(library_path: P) -> Self {
        let library_path = library_path
            .as_ref()
            .canonicalize()
            .expect("must have a canonical path to the engine");
        let build_info_path = Path::join(
            &library_path
                .parent()
                .expect("the engine to have a parent directory"),
            ".build-info",
        );

        if let Ok(build_info_contents) = fs::read_to_string(build_info_path) {
            let build_info = BuildInfo::parse_file_string(&build_info_contents)
                .expect("the .build-info file could not be parsed");
            let name = build_info
                .get("ENGINE")
                .expect(".build-info must have a valid ENGINE value")
                .to_owned();

            let dockerfile = Dockerfile::from(get_known_dockerfile_path(&name).unwrap());
            let build_info_defaults = dockerfile
                .default_buildinfo()
                .expect("the Dockerfile could not be parsed");

            let diffed_build_info = build_info.diff(build_info_defaults);
            Self {
                name,
                path: stringify(library_path),
                rebuild: Some(diffed_build_info.to_string()),
            }
        } else {
            log::warn!(
                "No .build-info for the engine at: {}",
                &library_path.display()
            );
            Self {
                name: format!("custom-{}", sha256::file(&library_path)),
                path: stringify(library_path),
                rebuild: None,
            }
        }
    }
}
