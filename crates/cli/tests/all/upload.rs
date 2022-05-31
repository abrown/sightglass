//! Test `sightglass-cli upload`.

use super::util::{sightglass_cli, test_engine};
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn upload_dryrun() {
    let assert = sightglass_cli()
        .arg("upload")
        .arg("--dry-run")
        .arg("--input-file")
        .arg("tests/results.json")
        .env("RUST_LOG", "debug")
        .assert();

    // Gather up the logged output from stderr.
    let stderr = std::str::from_utf8(&assert.get_output().stderr).unwrap();
    eprintln!("=== stderr ===\n{}\n===========", stderr);

    // Gather the fingerprints of the system under test.
    let engine = sightglass_fingerprint::Engine::fingerprint(test_engine()).unwrap();
    let benchmark =
        sightglass_fingerprint::Benchmark::fingerprint("../../benchmarks-next/noop/benchmark.wasm")
            .unwrap();
    let machine = sightglass_fingerprint::Machine::fingerprint().unwrap();

    // Check that we upload measurement records for each of the measurements in the file.
    let num_uploaded_measurements = stderr
        .matches("Creating record in 'measurements' with ID None")
        .count();
    assert_eq!(num_uploaded_measurements, 600);

    // Also, heck that we create records for the engine/machine/benchmark.
    use predicate::str::*;
    assert
        .stderr(
            contains(format!(
                r#"Creating record in 'engines' with ID Some("{}")"#,
                engine.id
            ))
            .and(contains(format!(
                r#"Creating record in 'machines' with ID Some("{}")"#,
                machine.id
            )))
            .and(contains(format!(
                r#"Creating record in 'benchmarks' with ID Some("{}")"#,
                benchmark.id
            ))),
        )
        .success();
}
