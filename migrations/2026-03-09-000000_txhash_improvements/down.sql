-- Remove new columns from transaction_hash table
ALTER TABLE transaction_hash DROP COLUMN IF EXISTS reason;
ALTER TABLE transaction_hash DROP COLUMN IF EXISTS old_price;
ALTER TABLE transaction_hash DROP COLUMN IF EXISTS new_price;

-- Note: PostgreSQL does not support removing enum values directly.
-- The new order_status values (LimitPriceUpdated, StopLossUpdated, TakeProfitUpdated, RejectedByRiskEngine)
-- will remain in the enum type.
