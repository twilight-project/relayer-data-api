-- This file should undo anything in `up.sql`


ALTER TYPE order_status DROP VALUE 'DuplicateOrder';
ALTER TYPE order_status DROP VALUE 'UtxoError';
ALTER TYPE order_status DROP VALUE 'Error';
ALTER TYPE order_status DROP VALUE 'NoResponseFromChain';
ALTER TYPE order_status DROP VALUE 'BincodeError';
ALTER TYPE order_status DROP VALUE 'HexCodeError';
ALTER TYPE order_status DROP VALUE 'SerializationError';
ALTER TYPE order_status DROP VALUE 'RequestSubmitted';
ALTER TYPE order_status DROP VALUE 'OrderNotFound';
ALTER TYPE order_status DROP VALUE 'RejectedFromChain';
ALTER TYPE order_status DROP VALUE 'FilledUpdated';
DROP INDEX transaction_hash_account_id;
DROP INDEX transaction_hash_request_id;