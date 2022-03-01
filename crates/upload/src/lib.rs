//!
mod database;

use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sightglass_data::{Measurement, Phase};
use sightglass_fingerprint::{Benchmark, Engine, Machine};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Upload `measurements` to the `server`. This will replace several fields of the raw [Measurement]
/// with fingerprinted data from the current server; this adds useful metadata to the results.
pub fn upload(server: &str, dryrun: bool, measurements: &Vec<Measurement>) -> Result<()> {
    let database = Database::new(server.to_string(), dryrun);

    // De-duplicate all of the engines and benchmarks used in this result set.
    let mut found_engines = HashSet::new();
    let mut found_benchmarks = HashSet::new();
    for m in measurements {
        found_engines.insert(m.engine.clone());
        found_benchmarks.insert(m.wasm.clone());
    }

    // Insert each fingerprinted version of an engine.
    let mut engines = HashMap::new();
    for engine_path in found_engines.into_iter() {
        let engine = Engine::fingerprint(engine_path.as_ref())?;
        let engine_id = database.create("engines", &engine, Some(&engine.name))?;
        log::debug!("Mapping engine: {} -> {}", &engine_path, &engine_id);
        engines.insert(engine_path, engine_id);
    }

    // Insert each fingerprinted version of a benchmark;
    let mut benchmarks = HashMap::new();
    for benchmark_path in found_benchmarks.into_iter() {
        let benchmark = Benchmark::fingerprint(benchmark_path.as_ref())?;
        let benchmark_id = database.create("benchmarks", &benchmark, Some(&benchmark.name))?;
        log::debug!(
            "Mapping benchmark: {} -> {}",
            &benchmark_path,
            &benchmark_id
        );
        benchmarks.insert(benchmark_path, benchmark_id);
    }

    // Fingerprint the current machine.
    let machine = Machine::fingerprint()?;
    let machine = database.create("machines", &machine, Some(&machine.name))?;

    // Upload the measurements.
    for m in measurements {
        let engine = engines.get(m.engine.as_ref()).unwrap().as_ref();
        let benchmark = benchmarks.get(m.wasm.as_ref()).unwrap().as_ref();
        let upload = UploadMeasurement::convert(&machine, engine, benchmark, m);
        database.create("measurements", &upload, None)?;
    }

    Ok(())
}

/// A conversion of a [Measurement], with fields replaced by fingerprinting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadMeasurement<'a> {
    /// The ID of the machine on which this measurement was taken; this relies on `upload` to insert
    /// the data for this ID.
    pub machine: Cow<'a, str>,

    /// The ID of the engine in which this measurement was taken; this relies on `upload` to insert
    /// the data for this ID.
    pub engine: Cow<'a, str>,

    /// The ID of the benchmark with which this measurement was taken; this relies on `upload` to
    /// insert the data for this ID.
    pub benchmark: Cow<'a, str>,

    /// The id of the process within which this measurement was taken.
    pub process: u32,

    /// This measurement was the `n`th measurement of this phase taken within a
    /// process.
    pub iteration: u32,

    /// The phase in a Wasm program's lifecycle that was measured: compilation,
    /// instantiation, or execution.
    pub phase: Phase,

    /// The event that was measured: micro seconds of wall time, CPU cycles
    /// executed, instructions retired, cache misses, etc.
    pub event: Cow<'a, str>,

    /// The event counts.
    ///
    /// The meaning and units depend on what the `event` is: it might be a count
    /// of microseconds if the event is wall time, or it might be a count of
    /// instructions if the event is instructions retired.
    pub count: u64,
}

impl<'a> UploadMeasurement<'a> {
    pub fn convert(
        machine: &'a str,
        engine: &'a str,
        benchmark: &'a str,
        measurement: &'a Measurement,
    ) -> Self {
        Self {
            machine: Cow::Borrowed(machine),
            engine: Cow::Borrowed(engine),
            benchmark: Cow::Borrowed(benchmark),
            process: measurement.process,
            iteration: measurement.process,
            phase: measurement.phase,
            event: Cow::Borrowed(measurement.event.as_ref()),
            count: measurement.count,
        }
    }
}
