#!/usr/bin/env bash

# Usage: ./export.sh
#
# Retrieve all known dashboards, storing their configuration in `dashboard.json`. To add dashboards
# to the list of exported dashboards, modify `$DASHBOARD_QUERY`.

set -e
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";
KIBANA_URL="${KIBANA_URL:-http://localhost:5601}"
OVERVIEW_DASHBOARD_ID="${OVERVIEW_DASHBOARD_ID:-12665450-ee60-11ec-8182-eddbfa6f2f6c}"
BENCHMARK_DASHBOARD_ID="${BENCHMARK_DASHBOARD_ID:-4bbd6150-ee8b-11ec-a8c9-09849b416961}"
ENGINE_DASHBOARD_ID="${ENGINE_DASHBOARD_ID:-bb7a0250-ee81-11ec-a8c9-09849b416961}"
DASHBOARD_QUERY="dashboard=$OVERVIEW_DASHBOARD_ID&dashboard=$BENCHMARK_DASHBOARD_ID&dashboard=$ENGINE_DASHBOARD_ID"
DASHBOARD_JSON="${DASHBOARD_JSON:-$SCRIPT_DIR/dashboard.json}"

(set -x; curl --fail-with-body \
         -H "kbn-xsrf: reporting" -H "Content-Type: application/json" \
         $KIBANA_URL/api/kibana/dashboards/export?$DASHBOARD_QUERY \
         > $DASHBOARD_JSON)
