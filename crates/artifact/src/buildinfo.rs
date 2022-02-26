use anyhow::Result;
use std::fmt::Write;
use std::fs;
use std::{collections::BTreeMap, fmt, io, iter::FromIterator, path::Path};

/// The default file name used for [BuildInfo] files.
pub const DEFAULT_FILE_NAME: &'static str = ".build-info";

/// A collection of variables and values used to reproduce a build of some artifact.
#[derive(Debug)]
pub struct BuildInfo(BTreeMap<String, String>);
impl BuildInfo {
    /// Return the [BuildInfo] name; every artifact with associated [BuildInfo] is expected to have
    /// a name.
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// assert_eq!(Some("test"), BuildInfo::parse_uri("test").unwrap().name());
    /// assert_eq!(Some("test"), BuildInfo::parse_uri("test?a=b").unwrap().name());
    /// assert_eq!(None, BuildInfo::parse_uri("a=b").unwrap().name());
    /// ```
    pub fn name(&self) -> Option<&str> {
        self.get("NAME")
    }

    /// Extract one of the [BuildInfo] values.
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// assert_eq!(Some("test"), BuildInfo::parse_uri("test").unwrap().get("NAME"));
    /// assert_eq!(Some("b"), BuildInfo::parse_uri("test?a=b").unwrap().get("a"));
    /// assert_eq!(None, BuildInfo::parse_uri("a=b").unwrap().get("c"));
    /// ```
    pub fn get(&self, variable: &str) -> Option<&str> {
        self.0.get(variable).map(String::as_str)
    }

    /// Parse [BuildInfo] from a URI-like string; e.g., `<name>`,
    /// `<name>?<var1>=<val1>&<var2>=<val2>`
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// assert_eq!("test", BuildInfo::parse_uri("test").unwrap().as_uri());
    /// assert_eq!("a=b", BuildInfo::parse_uri("a=b").unwrap().as_uri());
    /// assert_eq!("test?a=b", BuildInfo::parse_uri("test?a=b").unwrap().as_uri());
    /// assert_eq!("test?a=b&c=d", BuildInfo::parse_uri("test?a=b&c=d").unwrap().as_uri());
    /// assert_eq!("test?a=b&c='d e'", BuildInfo::parse_uri("test?a=b&c='d e'").unwrap().as_uri());
    /// ```
    pub fn parse_uri(uri: &str) -> Result<Self> {
        if let Some((name, rest)) = uri.split_once("?") {
            let mut b = BuildInfo::from_iter(rest.split("&").map(|p| split_pair_str(p)));
            b.0.insert("NAME".to_string(), name.to_string());
            Ok(b)
        } else if uri.contains("=") {
            Ok(BuildInfo::from_iter(
                uri.split("&").map(|p| split_pair_str(p)),
            ))
        } else {
            Ok(BuildInfo::from_iter([("NAME", uri)]))
        }
    }

    /// Emit [BuildInfo] as a URI-like string; e.g., `<name>`,
    /// `<name>?<var1>=<val1>&<var2>=<val2>`. See [`Self::parse_uri()`].
    pub fn as_uri(&self) -> String {
        let mut uri = self.name().unwrap_or("").to_owned();

        // Interject "&" between each printed variable-value pair.
        let mut iter = self.0.iter().filter(|(var, _)| var != &"NAME");
        if let Some(pair) = iter.next() {
            if uri.len() > 0 {
                write!(&mut uri, "?").unwrap();
            }
            write_pair_str(&mut uri, pair).unwrap();
        }
        for pair in iter {
            write!(&mut uri, "&").unwrap();
            write_pair_str(&mut uri, pair).unwrap();
        }

        uri
    }

    /// Parse [BuildInfo] from newline-separated file contents; e.g.:
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// let b = BuildInfo::parse_file_string("A=1
    ///  NAME=C
    ///  D=E F G").unwrap();
    /// assert_eq!(b.as_uri(), "C?A=1&D='E F G'")
    /// ```
    pub fn parse_file_string(file_contents: &str) -> Result<Self> {
        use io::BufRead;
        Ok(io::BufReader::new(file_contents.as_bytes())
            .lines()
            .filter_map(Result::ok)
            .map(|l| l.trim().to_string())
            .filter(|l| !l.starts_with("#"))
            .map(|l| split_pair(&l))
            .collect())
    }

    /// Parse [BuildInfo] from a file. See [`Self::parse_file_string()`].
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::parse_file_string(&fs::read_to_string(path.as_ref())?)
    }

    /// Emit [BuildInfo] as newline-separated string; e.g.:
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// let b = BuildInfo::parse_uri("test?A=1&B='2 3'").unwrap();
    /// assert_eq!(b.as_file_string(), "A=1
    /// B='2 3'
    /// NAME=test
    /// ");
    /// ```
    pub fn as_file_string(&self) -> String {
        let mut s = String::new();
        for pair in self.0.iter() {
            write_pair_str(&mut s, pair).unwrap();
            write!(&mut s, "\n").unwrap();
        }
        s
    }

    /// Generate a new [BuildInfo] with only the differences from `defaults`; any variables not
    /// known by `defaults` are discarded.
    ///
    /// ```
    /// # use sightglass_artifact::BuildInfo;
    /// let b1 = BuildInfo::parse_uri("test?A=1&B=42").unwrap();
    /// let b2 = BuildInfo::parse_uri("test?A=1&B=2&C=3").unwrap();
    /// // Only the settings in `b1` that are different than `b2` are retained.
    /// assert_eq!(b1.diff(b2).as_uri(), "B=42");
    /// ```
    pub fn diff(&self, defaults: BuildInfo) -> BuildInfo {
        self.0
            .iter()
            .filter(|(var, val)| match defaults.0.get(*var) {
                Some(default_val) => *val != default_val,
                None => false,
            })
            .collect()
    }

    // pub fn parse<R: io::BufRead>(reader: R) -> Result<Self> {
    //     Ok(reader
    //         .lines()
    //         .filter_map(Result::ok)
    //         .map(|l| l.trim().to_string())
    //         .filter(|l| !l.starts_with("#"))
    //         .map(|l| split_pair(&l))
    //         .collect())
    // }

    // pub fn parse_dockerfile<R: io::BufRead>(reader: R) -> Result<Self> {
    //     Ok(reader
    //         .lines()
    //         .filter_map(Result::ok)
    //         .map(|l| l.trim().to_string())
    //         .filter(|l| l.starts_with("ARG"))
    //         .map(|l| split_pair(&l.trim_start_matches("ARG ")))
    //         .collect())
    // }
}

impl FromIterator<(String, String)> for BuildInfo {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (var, val) in iter {
            map.insert(var, val);
        }
        Self(map)
    }
}

impl<'a> FromIterator<(&'a String, &'a String)> for BuildInfo {
    fn from_iter<T: IntoIterator<Item = (&'a String, &'a String)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (var, val) in iter {
            map.insert(var.to_string(), val.to_string());
        }
        Self(map)
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for BuildInfo {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (var, val) in iter {
            map.insert(var.to_string(), val.to_string());
        }
        Self(map)
    }
}

impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Interject "&" between each printed variable-value pair.
        let mut iter = self.0.iter();
        if let Some(pair) = iter.next() {
            write_pair(f, pair)?;
        }
        for pair in iter {
            write!(f, "&")?;
            write_pair(f, pair)?;
        }

        Ok(())
    }
}

fn split_pair_str(line: &str) -> (&str, &str) {
    let (var, val) = line
        .trim()
        .split_once("=")
        .expect("expected equals-delimited lines");
    (var, val.trim_matches(|c| c == '"' || c == '\''))
}

pub(crate) fn split_pair(line: &str) -> (String, String) {
    let (var, val) = line
        .trim()
        .split_once("=")
        .expect("expected equals-delimited lines");
    (
        var.to_owned(),
        val.trim_matches(|c| c == '"' || c == '\'').to_owned(),
    )
}

// Write a variable-value pair, e.g., a='b c d'
fn write_pair(f: &mut std::fmt::Formatter<'_>, (var, val): (&String, &String)) -> std::fmt::Result {
    assert!(
        !var.contains(" "),
        "build info variables cannot contain spaces"
    );
    assert!(
        !var.contains("&"),
        "build info variables cannot contain ampersands"
    );
    assert!(
        !val.contains("&"),
        "build info values cannot contain ampersands"
    );
    assert!(
        !val.contains("'"),
        "build info values cannot contain single quotes"
    );
    if val.contains(" ") {
        write!(f, "{}='{}'", var, val)
    } else {
        write!(f, "{}={}", var, val)
    }
}

fn write_pair_str(f: &mut dyn Write, (var, val): (&String, &String)) -> std::fmt::Result {
    assert!(
        !var.contains(" "),
        "build info variables cannot contain spaces"
    );
    assert!(
        !var.contains("&"),
        "build info variables cannot contain ampersands"
    );
    assert!(
        !val.contains("&"),
        "build info values cannot contain ampersands"
    );
    assert!(
        !val.contains("'"),
        "build info values cannot contain single quotes"
    );
    if val.contains(" ") {
        write!(f, "{}='{}'", var, val)
    } else {
        write!(f, "{}={}", var, val)
    }
}
