#!/usr/bin/env bash

# Usage: ./import.sh
#
# Store the dashboard configuration in `dashboard.json` into Kibana.

set -e
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";
KIBANA_URL="${KIBANA_URL:-http://localhost:5601}"
DASHBOARD_ID="${DASHBOARD_ID:-12665450-ee60-11ec-8182-eddbfa6f2f6c}"
DASHBOARD_JSON="${DASHBOARD_JSON:-$SCRIPT_DIR/dashboard.json}"

(set -x; curl --fail-with-body \
         -H "kbn-xsrf: reporting" -H "Content-Type: application/json" \
         -X POST -d @$DASHBOARD_JSON \
         $KIBANA_URL/api/kibana/dashboards/import)
