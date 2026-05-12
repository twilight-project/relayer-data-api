-- Make the back-fill trigger treat empty-string request_id the same as NULL,
-- and backfill rows that were already inserted with request_id = ''.

CREATE OR REPLACE FUNCTION public.update_request_id_after_pending_order_fill()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS $BODY$
BEGIN

	CASE
    	WHEN ((New.request_id IS NULL OR New.request_id = '') AND New.order_status = 'FILLED') THEN
			UPDATE public.transaction_hash
				SET request_id=(Select request_id from public.transaction_hash where order_id = New.order_id and order_status = 'PENDING' limit 1)
					WHERE id = New.id and order_status = 'FILLED';

    	WHEN ((New.request_id IS NULL OR New.request_id = '') AND New.order_status = 'LIQUIDATE') THEN
			UPDATE public.transaction_hash
				SET request_id=(Select request_id from public.transaction_hash where order_id = New.order_id and order_status = 'FILLED' limit 1)
					WHERE id = New.id and order_status = 'LIQUIDATE';
        ELSE
            NULL;
    END CASE;

    RETURN New;
END;
$BODY$;

-- Normalize existing empty-string request_id values to NULL so subsequent
-- queries and filters behave consistently.
UPDATE public.transaction_hash
    SET request_id = NULL
    WHERE request_id = '';

-- Backfill historical FILLED rows whose request_id is now NULL using the
-- matching PENDING row's request_id.
UPDATE public.transaction_hash t
    SET request_id = p.request_id
    FROM public.transaction_hash p
    WHERE t.order_status = 'FILLED'
      AND t.request_id IS NULL
      AND p.order_id = t.order_id
      AND p.order_status = 'PENDING'
      AND p.request_id IS NOT NULL;

-- Backfill historical LIQUIDATE rows from the matching FILLED row.
UPDATE public.transaction_hash t
    SET request_id = f.request_id
    FROM public.transaction_hash f
    WHERE t.order_status = 'LIQUIDATE'
      AND t.request_id IS NULL
      AND f.order_id = t.order_id
      AND f.order_status = 'FILLED'
      AND f.request_id IS NOT NULL;
