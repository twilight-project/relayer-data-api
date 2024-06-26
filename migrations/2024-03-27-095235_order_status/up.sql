-- Your SQL goes here


ALTER TYPE order_status ADD VALUE 'DuplicateOrder';
ALTER TYPE order_status ADD VALUE 'UtxoError';
ALTER TYPE order_status ADD VALUE 'Error';
ALTER TYPE order_status ADD VALUE 'NoResponseFromChain';
ALTER TYPE order_status ADD VALUE 'BincodeError';
ALTER TYPE order_status ADD VALUE 'HexCodeError';
ALTER TYPE order_status ADD VALUE 'SerializationError';
ALTER TYPE order_status ADD VALUE 'RequestSubmitted';
ALTER TYPE order_status ADD VALUE 'OrderNotFound';
ALTER TYPE order_status ADD VALUE 'RejectedFromChain';
CREATE INDEX transaction_hash_account_id ON transaction_hash(account_id);
CREATE INDEX transaction_hash_request_id ON transaction_hash(request_id);