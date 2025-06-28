-- Orderbook view
CREATE OR REPLACE VIEW orderbook AS
WITH ranked AS (
    SELECT  t.*,
            row_number() OVER (
                PARTITION BY t.uuid
                ORDER BY     t.id DESC
            ) AS rn
    FROM trader_order t
),

sc_latest AS (
    SELECT  uuid,
            MAX(id) AS max_sc_id
    FROM    sorted_set_command
    WHERE   command IN (
              'ADD_CLOSE_LIMIT_PRICE'::sorted_set_command_type,
              'UPDATE_CLOSE_LIMIT_PRICE'::sorted_set_command_type
            )
    GROUP BY uuid
)

-- /* === branch A: newest OPEN-LIMIT order per uuid ================= */
SELECT  r.id,
        r.uuid,
        r.account_id,
        r.position_type,
        r.order_status,
        r.order_type,
        CAST(r.entryprice AS numeric)      AS entryprice,
        r.execution_price,
        r.positionsize,
        r.leverage,
        r.initial_margin,
        r.available_margin,
        r."timestamp",
        r.bankruptcy_price,
        r.bankruptcy_value,
        r.maintenance_margin,
        r.liquidation_price,
        r.unrealized_pnl,
        r.settlement_price,
        r.entry_nonce,
        r.exit_nonce,
        r.entry_sequence
        r.fee_filled,
        r.fee_settled
FROM   ranked r
WHERE  r.rn = 1
  AND  r.order_type = 'LIMIT'
  AND  r.order_status NOT IN ('FILLED','CANCELLED','LIQUIDATE','SETTLED')

UNION ALL

-- /* === branch B: newest FILLED order + latest ADD/UPDATE cmd ====== */
SELECT  r.id,
        r.uuid,
        r.account_id,
        CASE
            WHEN r.position_type = 'LONG'  THEN 'SHORT'
            WHEN r.position_type = 'SHORT' THEN 'LONG'
            ELSE r.position_type
        END                                     AS position_type,
        r.order_status,
        r.order_type,
        CAST(sc.amount AS numeric)        AS entryprice,
        r.execution_price,
        r.positionsize,
        r.leverage,
        r.initial_margin,
        r.available_margin,
        r."timestamp",
        r.bankruptcy_price,
        r.bankruptcy_value,
        r.maintenance_margin,
        r.liquidation_price,
        r.unrealized_pnl,
        r.settlement_price,
        r.entry_nonce,
        r.exit_nonce,
        r.entry_sequence,
        r.fee_filled,
        r.fee_settled
FROM       ranked            r
JOIN       sc_latest         ls ON ls.uuid = r.uuid
JOIN       sorted_set_command sc ON sc.id   = ls.max_sc_id
WHERE      r.rn = 1
  AND      r.order_status = 'FILLED';

-- /* ================================================================ */
-- /* 2️⃣  Performance indexes                                         */
-- /* ================================================================ */

-- /* Latest row look-ups for both open & filled branches */
CREATE INDEX IF NOT EXISTS trader_order_uuid_id_desc
    ON trader_order (uuid, id DESC);

-- /* Selective index for open-LIMIT orders */
CREATE INDEX IF NOT EXISTS trader_order_open_limit_idx
    ON trader_order (uuid, id DESC)
    WHERE order_type = 'LIMIT'
      AND order_status NOT IN ('FILLED','CANCELLED','LIQUIDATE','SETTLED');

-- /* Join filter on any close-limit command (ADD or UPDATE) */
CREATE INDEX IF NOT EXISTS sorted_set_command_cmd_uuid
    ON sorted_set_command (command, uuid);

-- /* Optional: index that matches sc_latest’s GROUP BY + MAX pattern */
CREATE INDEX IF NOT EXISTS sorted_set_command_uuid_id_desc_close_limit
    ON sorted_set_command (uuid, id DESC)
    WHERE command IN (
          'ADD_CLOSE_LIMIT_PRICE'::sorted_set_command_type,
          'UPDATE_CLOSE_LIMIT_PRICE'::sorted_set_command_type
        );
