#!/bin/bash

BEARER=$(node gen-token/gen.js)


MINUTE_CANDLE='{"jsonrpc": "2.0", "id": "1", "method": "candle_data", "params": { "interval": "ONE_MINUTE", "since": "2023-07-26 10:35:23Z", "limit": 1000, "offset": 0 } }'
curl -XPOST -H 'content-type: application/json' -H "Authorization: Bearer ${BEARER}" http://localhost:8989 -d "${MINUTE_CANDLE}"
