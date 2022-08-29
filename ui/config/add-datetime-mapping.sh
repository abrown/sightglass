#!/usr/bin/env bash

set -e
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";
ELASTICSEARCH_URL="${ELASTICSEARCH_URL:-http://localhost:9200}"
INDEX="$1"
if [[ -z "$INDEX" ]]; then
    echo "error: must pass the index as the first parameter, e.g.:"
    echo "  add-datetime-mapping.sh engines"
    exit 1
fi

# First, create the index...
(set -x; curl --fail-with-body \
         -X PUT $ELASTICSEARCH_URL/$INDEX)
echo

# ...then tell it how to parse the `datetime` field as an ISO8601 date.
(set -x; curl --fail-with-body \
         -H "Content-Type: application/json" \
         -X PUT -d @$SCRIPT_DIR/mapping-datetime.json \
         $ELASTICSEARCH_URL/$INDEX/_mapping)
echo
