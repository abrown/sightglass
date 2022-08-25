#!/usr/bin/env bash

set -e

PROJECT_DIR=$(dirname "$0" | xargs dirname)
RESULTS_DIR=/tmp/sightglass-results
BUILD_EXE=$PROJECT_DIR/engines/wasmtime/build
PIN="taskset --cpu-list 0-15"
SIGHTGLASS="cargo +nightly run --bin sightglass-cli --"
COMMITS=$(git list-by-interval --interval 4w --until 48w --directory /tmp/wasmtime)
export RUST_LOG=debug

# If an engine is not available, build it.
if [[ ! -f $BUILD_EXE ]]; then
    pushd $(dirname $BUILD_EXE)
    rustc build.rs
    popd
fi

# Helpful logging function.
print_header() {
    >&2 echo
    >&2 echo ===== $@ =====
}

for COMMIT in $COMMITS; do
    DIR=$RESULTS_DIR/$COMMIT
    SO=$DIR/libengine.so
    LOG=$DIR/results.log

    if [[ ! -f $SO ]]; then
        print_header "Building Wasmtime at $COMMIT"
        (set -x; mkdir -p $DIR)
        (set -x; REVISION=$COMMIT $BUILD_EXE $DIR)
    fi

    if [[ ! -f $LOG ]]; then
        print_header "Benchmarking Wasmtime at $COMMIT"
        (set -x; $PIN $SIGHTGLASS benchmark benchmarks/*/benchmark.wasm --engine $SO --raw > $LOG)
    fi

    print_header "Benchmarking results at $COMMIT"
    (set -x; $SIGHTGLASS summarize -f $LOG)

    print_header "Uploading results: $DIR/results.log"
    (set -x; $SIGHTGLASS upload -f $DIR/results.log)
done
