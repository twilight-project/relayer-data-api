ALTER TABLE sorted_set_command
  ADD COLUMN created_time TIMESTAMPTZ NOT NULL DEFAULT now();
