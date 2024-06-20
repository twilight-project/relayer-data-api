-- Your SQL goes here

-- FUNCTION: public.create_price_triger_after_insert()

-- DROP FUNCTION IF EXISTS public.create_price_triger_after_insert();

CREATE OR REPLACE FUNCTION public.create_price_triger_after_insert_for_candle_data_generation()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS $BODY$
BEGIN

	CASE 
    	WHEN (NOW() <( New."timestamp" + INTERVAL '500 ms')) THEN
			PERFORM  update_candles_1min();
			PERFORM  update_candles_1hour();
			PERFORM update_candles_1day();
        ELSE
            NULL; 
    END CASE;

    RETURN New;
END;
$BODY$;

ALTER FUNCTION public.create_price_triger_after_insert_for_candle_data_generation()
    OWNER TO relayer;
	
	
-- Trigger: after_insert_price_trigger

-- DROP TRIGGER IF EXISTS after_insert_price_trigger ON public.btc_usd_price;

CREATE OR REPLACE TRIGGER after_insert_price_trigger_for_candle_data_generation
    AFTER INSERT
    ON public.btc_usd_price
    FOR EACH ROW
    EXECUTE FUNCTION public.create_price_triger_after_insert_for_candle_data_generation();
	
	
	
-- FUNCTION: public.update_request_id_after_pending_order_fill()

-- DROP FUNCTION IF EXISTS public.update_request_id_after_pending_order_fill();

CREATE OR REPLACE FUNCTION public.update_request_id_after_pending_order_fill()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS $BODY$
BEGIN

	CASE 
    	WHEN (New.request_id is NULL  and  New.order_status = 'FILLED') THEN
			UPDATE public.transaction_hash
				SET request_id=(Select request_id from public.transaction_hash where order_id = New.order_id and order_status = 'PENDING' limit 1)
					WHERE id = New.id and order_status = 'FILLED';

    	WHEN (New.request_id is NULL  and  New.order_status = 'LIQUIDATE') THEN
			UPDATE public.transaction_hash
				SET request_id=(Select request_id from public.transaction_hash where order_id = New.order_id and order_status = 'FILLED' limit 1)
					WHERE id = New.id and order_status = 'LIQUIDATE';
        ELSE
            NULL; 
    END CASE;

    RETURN New;
END;
$BODY$;

ALTER FUNCTION public.update_request_id_after_pending_order_fill()
    OWNER TO relayer;
		

-- Trigger: after_insert_update_request_id_after_pending_order_fill

-- DROP TRIGGER IF EXISTS after_insert_update_request_id_after_pending_order_fill ON public.transaction_hash;

CREATE OR REPLACE TRIGGER after_insert_update_request_id_after_pending_order_fill
    AFTER INSERT
    ON public.transaction_hash
    FOR EACH ROW
    EXECUTE FUNCTION public.update_request_id_after_pending_order_fill();