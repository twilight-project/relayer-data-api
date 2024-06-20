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
    	WHEN (NOW() <( New."timestamp" + INTERVAL '250 ms')) THEN
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
	
	
	
		
	