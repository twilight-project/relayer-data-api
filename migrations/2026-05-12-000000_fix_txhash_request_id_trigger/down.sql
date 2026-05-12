-- Revert the trigger function to only check IS NULL.

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
