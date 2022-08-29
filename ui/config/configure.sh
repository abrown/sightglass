#!/usr/bin/env bash

# Usage: ./configure.sh
#
# This wrapper script calls all scripts necessary to set up ElasticSearch and Kibana.

set -e

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";

echo "==== Add datetime mappings to ElasticSearch ====="
$SCRIPT_DIR/add-datetime-mapping.sh engines
$SCRIPT_DIR/add-datetime-mapping.sh measurements
echo

echo "==== Import dashboards to Kibana ====="
$SCRIPT_DIR/import-dashboards.sh
