use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha256;
use std::path::PathBuf;
use std::{ffi::OsStr, fs::File, io, path::Path};

/// Describes a fingerprinted benchmark.
///
/// ```
/// # use sightglass_fingerprint::Benchmark;
/// let benchmark = Benchmark::fingerprint("../../benchmarks-next/noop/benchmark.wasm");
/// assert_eq!(benchmark.name, "noop");
/// assert_eq!(benchmark.path, "benchmarks-next/noop/benchmark.wasm");
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Benchmark {
    /// The benchmark name; this is calculated from either the benchark name (if it does not start
    /// with "benchmark") or from the parent directory. This accommodates both the current benchmark
    /// structure where each directory contains a "benchmark.wasm" file as well as non-Sightglass
    /// benchmarks, e.g., "spidermonkey.wasm".
    pub name: String,
    /// The path to the benchmark on the system.
    pub path: String,
    /// The SHA256 hash of the benchmark file.
    pub hash: String,
    /// The size of the file; this may be useful for comparing compile times between benchmarks of
    /// different sizes.
    pub size: u64,
}

impl Benchmark {
    pub fn fingerprint<P: AsRef<Path>>(path: P) -> Self {
        let path = path
            .as_ref()
            .canonicalize()
            .expect("must have a canonical path to the benchmark");

        // Calculate the hash for the benchmark file.
        let mut file = File::open(&path).expect("the benchmark to be a file that can be opened");
        let mut hasher = Sha256::new();
        let size =
            io::copy(&mut file, &mut hasher).expect("to be able to hash the benchmark bytes");
        let hash = hasher.finalize();

        Self {
            name: simplify_benchmark_name(&path),
            path: simplify_benchmark_path(&path),
            hash: hexify(hash.as_slice()),
            size,
        }
    }
}

/// Simplify the benchmark name if possible; e.g.:
/// - `.../<name>/benchmark.wasm` -> <name>
/// - `.../<name>.wasm -> <name>
fn simplify_benchmark_name<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    let stem = path
        .file_stem()
        .expect("the benchmark must have a file name");
    let name = if stem == "benchmark" {
        path.parent()
            .expect("the benchmark must have a parent directory")
            .file_name()
            .expect("the parent directory to have a name")
    } else {
        stem
    };
    stringify(name)
}

/// Simplify the benchmark path if possible; e.g.:
/// `/home/user/code/sightglass/benchmarks-next/<name>/benchmark.wasm` ->
/// `benchmarks-next/<name>/benchmark.wasm`. This function finds a path component natching
/// `benchmarks/` or `benchmarks-next/` and cuts the path there.
fn simplify_benchmark_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    if let Some(i) = path
        .iter()
        .position(|c| c == "benchmarks" || c == "benchmarks-next")
    {
        let shortened_path: PathBuf = path.iter().skip(i).collect();
        stringify(shortened_path.as_os_str())
    } else {
        stringify(path.as_os_str())
    }
}

/// Provide a common way to create `String`s from `OsStr` in this module.
fn stringify(s: &OsStr) -> String {
    s.to_string_lossy().to_string()
}

/// Create a hexadecimal string from a sequence of bytes.
fn hexify(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for byte in bytes {
        write!(&mut s, "{:x}", byte).expect("unable to write byte as hex");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortened_benchmark_names() {
        assert_eq!(
            simplify_benchmark_name("benchmarks-next/noop/benchmark.wasm"),
            "noop"
        );
        assert_eq!(simplify_benchmark_name("a/b/c.wasm"), "c");
    }

    #[test]
    fn shortened_benchmark_paths() {
        assert_eq!(
            simplify_benchmark_path("/home/user/sightglass/benchmarks/benchmark.wasm"),
            "benchmarks/benchmark.wasm"
        );
        assert_eq!(
            simplify_benchmark_path("code/benchmarks-next/noop.wasm"),
            "benchmarks-next/noop.wasm"
        );
        assert_eq!(
            simplify_benchmark_path("some/other/path.wasm"),
            "some/other/path.wasm"
        );
    }
}
