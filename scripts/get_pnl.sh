#!/bin/bash

BEARER=$(node gen-token/gen.js)


MINUTE_CANDLE='{"jsonrpc": "2.0", "id": "1", "method": "unrealized_pnl", "params": { "PublicKey": "af" } }'
curl -XPOST -H 'content-type: application/json' -H "Authorization: Bearer ${BEARER}" http://localhost:8989 -d "${MINUTE_CANDLE}"
