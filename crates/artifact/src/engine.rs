use crate::{
    buildinfo, sightglass_data_dir,
    util::{sha256, slug},
    BuildInfo, DockerBuildArgs, Dockerfile, GitLocation,
};
use anyhow::{anyhow, Result};
use log;
use std::{borrow::Cow, convert::TryFrom, env, fmt, fs, path::Path, path::PathBuf, str};

// Retrieve a built engine library for running benchmarks; the returned value is a path to the built
// engine's dylib. This function will attempt to build the library if it does not yet exist.
pub fn get_built_engine(engine: &str) -> Result<PathBuf> {
    if Path::new(engine).exists() {
        log::debug!("Using already-built engine path: {}", engine);
        return Ok(PathBuf::from(engine));
    }

    // Get the path to where the known engine dylib would be if it is built, or else propagate an
    // unknown engine error.
    let engine_path = get_known_engine_path(engine)?;

    // If no file exists at the engine path, then we have to build it.
    if !engine_path.exists() {
        build_engine(engine, &engine_path)?;
        assert!(engine_path.exists());
    }

    log::debug!("Using known engine at path: {}", engine_path.display());
    Ok(engine_path)
}

/// List all known engines in the cache folder.
pub fn list_engines<'a>() -> Result<Vec<(EngineName<'a>, PathBuf, Option<BuildInfo>)>> {
    let sightglass_data_dir = sightglass_data_dir()?;
    let mut engines = vec![];
    for entry in fs::read_dir(sightglass_data_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let engine_dir = entry.path();
            match EngineName::try_from(engine_dir.as_path()) {
                Ok(name) => {
                    let path = Path::join(&engine_dir, get_engine_filename());
                    let buildinfo =
                        BuildInfo::parse_file(Path::join(&path, buildinfo::DEFAULT_FILE_NAME)).ok();
                    engines.push((name, path, buildinfo));
                }
                Err(err) => log::warn!("Invalid engine found at {}: {}", engine_dir.display(), err),
            }
        }
    }
    Ok(engines)
}

/// Calculate the path to an engine library: e.g. `<user's app data
/// dir>/sightglass/wasmtime@ab1234ef/libengine.so`.
pub fn get_known_engine_path(slug: &str) -> Result<PathBuf> {
    let mut p = super::sightglass_data_dir()?;
    p.push(slug);
    p.push(get_engine_filename());
    Ok(p)
}

/// Calculate the library name for a sightglass library on the target operating system: e.g.
/// `engine.dll`, `libengine.so`.
pub fn get_engine_filename() -> String {
    format!(
        "{}engine{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    )
}

/// Calculate the path to the Dockerfile for building a known engine.
pub fn get_known_dockerfile_path(slug: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from("."); // TODO calculate the project directory
    path.push("engines");
    path.push(slug);
    path.push("Dockerfile");
    Ok(path)
}

/// Build an engine from either a Dockerfile or a known engine.
pub fn build_engine(engine: &str, engine_path: &Path) -> Result<()> {
    // If the known engine's directory is not yet created, create it.
    let engine_dir = engine_path.parent().unwrap();
    if !engine_dir.is_dir() {
        fs::create_dir_all(&engine_dir)?;
        log::debug!("Created sightglass directory: {}", engine_dir.display());
    }

    let (dockerfile, args) = if Path::new(engine).exists() {
        (Dockerfile::from(PathBuf::from(engine)), None)
    } else {
        use std::str::FromStr;
        let engine_ref = EngineRef::from_str(engine)?;
        let dockerfile =
            Dockerfile::from(get_known_dockerfile_path(&engine_ref.engine.to_string())?);

        // Set up any additional arguments for building the library.
        let mut args = DockerBuildArgs::new();
        if let Some(revision) = &engine_ref.git.revision {
            args.set("REVISION".to_string(), revision.clone())
        }
        if let Some(repository) = &engine_ref.git.repository {
            args.set("REPOSITORY".to_string(), repository.clone())
        }

        (dockerfile, Some(args))
    };

    log::debug!("Using Dockerfile at path: {}", dockerfile);
    let container_engine_path = format!("/{}", get_engine_filename());
    let build_info_path = &Path::join(&engine_dir, ".build-info");
    let files = [
        (container_engine_path, engine_path),
        ("/.build-info".to_string(), build_info_path),
    ];
    dockerfile.extract(&files, args)?;
    Ok(())
}

/// An engine-ref is a parseable description of a benchmarking engine. This crate understands how to
/// build certain well-known engines.
#[derive(Clone, Debug)]
pub struct EngineRef {
    engine: WellKnownEngine,
    git: GitLocation,
}
impl fmt::Display for EngineRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.engine)?;
        if let Some(rev) = &self.git.revision {
            write!(f, "@{}", rev)?;
        }
        if let Some(repo) = &self.git.repository {
            write!(f, "@{}", repo)?;
        }
        Ok(())
    }
}
impl str::FromStr for EngineRef {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('@').collect();
        let engine = if let Some(name) = parts.get(0) {
            name.parse()?
        } else {
            return Err(anyhow!(
                "failed to parse engine-ref; the engine-ref must not be empty"
            ));
        };
        let revision = parts.get(1).map(|s| s.to_string());
        let repository = parts.get(2).map(|s| s.to_string());
        if parts.len() > 3 {
            return Err(anyhow!("an engine-ref must not have more than 3 parts: [engine name]@[revision]@[repository]"));
        }
        Ok(Self {
            engine,
            git: GitLocation {
                repository,
                revision,
            },
        })
    }
}

/// Enumerates the engines known to sightglass.
#[derive(Clone, Debug)]
pub enum WellKnownEngine {
    Wasmtime,
}
impl fmt::Display for WellKnownEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Wasmtime => "wasmtime",
        };
        write!(f, "{}", s)
    }
}
impl str::FromStr for WellKnownEngine {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wasmtime" => Ok(Self::Wasmtime),
            _ => Err(anyhow!("unable to parse an unknown engine: {}", s)),
        }
    }
}

/// Describes the shortened name for a built engine (i.e., an alias).
#[derive(Clone, Debug, PartialEq)]
pub struct EngineName<'a>(Cow<'a, str>, Cow<'a, str>);
impl<'a> EngineName<'a> {
    fn new<C: Into<Cow<'a, str>>>(name: C, slug: C) -> Self {
        Self(name.into(), slug.into())
    }
}

/// Extract the [EngineName] from the last component of a directory path, e.g.:
/// ```
/// # use sightglass_artifact::EngineName;
/// # use std::path::PathBuf;
/// # use std::convert::TryFrom;
/// let en = EngineName::try_from(PathBuf::from("/home/user/cache/<engine>-<slug>").as_path()).unwrap();
/// assert_eq!(en.to_string(), "<engine>-<slug>");
/// ```
impl<'a, 'b> TryFrom<&'b Path> for EngineName<'a> {
    type Error = anyhow::Error;
    fn try_from(path: &'b Path) -> Result<Self, Self::Error> {
        path.file_name()
            .ok_or(anyhow!("directory path must have a final component"))?
            .to_string_lossy()
            .to_string()
            .parse()
    }
}

impl TryFrom<BuildInfo> for EngineName<'_> {
    type Error = anyhow::Error;
    fn try_from(b: BuildInfo) -> Result<Self, Self::Error> {
        let name = b
            .get("NAME")
            .ok_or(anyhow!("BUILDINFO must contain a NAME variable"))?;
        let hash = if let Some(commit) = b.get("COMMIT") {
            commit.to_owned()
        } else {
            sha256::string(&b.as_uri())
        };
        Ok(Self::new(name.to_string(), slug(&hash).to_string()))
    }
}

impl<'a> str::FromStr for EngineName<'a> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, slug) = s.split_once("-").ok_or(anyhow!(
            "expected engine name to be hyphenated, e.g.: <engine>-<slug>"
        ))?;
        Ok(Self::new(name.to_string(), slug.to_string()))
    }
}

impl fmt::Display for EngineName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn engine_filename() {
        assert_eq!("libengine.so", get_engine_filename());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn engine_filename() {
        assert_eq!("engine.dll", get_engine_filename());
    }

    #[test]
    fn engine_ref() {
        fn roundtrip(engine_ref: &str) -> Result<()> {
            use std::str::FromStr;
            let er = EngineRef::from_str(engine_ref)?;
            if engine_ref != er.to_string() {
                Err(anyhow!("{} != {}", engine_ref, er.to_string()))
            } else {
                Ok(())
            }
        }

        // Check valid engine-refs.
        assert!(roundtrip("wasmtime").is_ok());
        assert!(roundtrip("wasmtime@1234567").is_ok());
        assert!(roundtrip("wasmtime@1234567@https://github.com/user/wasmtime").is_ok());

        // Check invalid engine-refs.
        assert!(roundtrip("other_engine").is_err());
        assert!(roundtrip("wasmtime@1234567@https://github.com/user/wasmtime@...").is_err());
    }

    #[test]
    fn engine_name_roundtrip() {
        let en = EngineName::new("<engine>", "<slug>");
        assert_eq!(en, en.to_string().parse().unwrap());
    }
}
