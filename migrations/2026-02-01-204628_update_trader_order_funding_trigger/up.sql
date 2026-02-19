-- Your SQL goes here
-- ============================================================
-- Update trigger function:
-- update_trader_order_from_funding
-- ============================================================

CREATE OR REPLACE FUNCTION public.update_trader_order_from_funding()
RETURNS trigger
LANGUAGE plpgsql
AS $BODY$
BEGIN
    UPDATE public.trader_order
    SET
        available_margin   = NEW.available_margin,
        maintenance_margin = NEW.maintenance_margin,
        liquidation_price  = NEW.liquidation_price
    WHERE uuid = NEW.uuid
      AND order_status = 'FILLED';

    RETURN NEW;
END;
$BODY$;

ALTER FUNCTION public.update_trader_order_from_funding()
OWNER TO relayer;
