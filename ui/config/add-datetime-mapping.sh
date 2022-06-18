#!/usr/bin/env bash

set -e
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";
ELASTICSEARCH_URL="${ELASTICSEARCH_URL:-http://localhost:9200}"

# First, create the `engines` index...
(set -x; curl --fail-with-body \
         -X PUT $ELASTICSEARCH_URL/engines)

# ...then tell it how to parse the `datetime` field as an ISO8601 date.
(set -x; curl --fail-with-body \
         -H "Content-Type: application/json" \
         -X PUT -d @$SCRIPT_DIR/mapping-engine-datetime.json \
         $ELASTICSEARCH_URL/engines/_mapping)
