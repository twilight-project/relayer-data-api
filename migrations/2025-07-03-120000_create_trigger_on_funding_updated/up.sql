-- Create a function to update trader_order table
CREATE OR REPLACE FUNCTION update_trader_order_from_funding()
RETURNS TRIGGER AS $$
BEGIN
    -- Update available_margin only when the order in trader_order is FILLED
    UPDATE public.trader_order
    SET available_margin = NEW.available_margin
    WHERE uuid = NEW.uuid
      AND order_status = 'FILLED';
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to call the function on insert or update
CREATE TRIGGER update_trader_order_trigger
AFTER INSERT OR UPDATE ON public.trader_order_funding_updated
FOR EACH ROW
EXECUTE FUNCTION update_trader_order_from_funding(); 