-- Add migration script here
CREATE TABLE last_update_time (
  id SERIAL PRIMARY KEY,
  last_update_time TIMESTAMPTZ
);
