#!/usr/bin/env bash

set -e
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";
ELASTICSEARCH_URL="${ELASTICSEARCH_URL:-http://localhost:9200}"

(set -x; curl --fail-with-body \
         -H "Content-Type: application/json" \
         -X PUT -d @ui/config/mapping-engine-datetime.json \
         $ELASTICSEARCH_URL/engines/_mapping)
