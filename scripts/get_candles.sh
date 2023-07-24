#!/bin/bash

BEARER=$(node gen-token/gen.js)


MINUTE_CANDLE='{"jsonrpc": "2.0", "id": "1", "method": "candle_data", "params": { "interval": "FIFTEEN_MINUTE", "since": "2023-07-24T00:00:00.0Z" } }'
curl -XPOST -H 'content-type: application/json' -H "Authorization: Bearer ${BEARER}" http://localhost:8989 -d "${MINUTE_CANDLE}" -Dresp
