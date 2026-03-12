-- Add new order status enum values
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'LimitPriceUpdated';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'StopLossUpdated';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'TakeProfitUpdated';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'RejectedByRiskEngine';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'CancelledLimitClose';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'OrderUpdated';

-- Add new columns to transaction_hash table
ALTER TABLE transaction_hash ADD COLUMN IF NOT EXISTS reason VARCHAR;
ALTER TABLE transaction_hash ADD COLUMN IF NOT EXISTS old_price DOUBLE PRECISION;
ALTER TABLE transaction_hash ADD COLUMN IF NOT EXISTS new_price DOUBLE PRECISION;
