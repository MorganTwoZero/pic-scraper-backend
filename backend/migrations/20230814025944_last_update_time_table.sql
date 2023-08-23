-- Add migration script here
CREATE TABLE last_update_time (
  id SERIAL PRIMARY KEY,
  last_update_time TIMESTAMPTZ
);
INSERT INTO last_update_time (id, last_update_time)
  VALUES (0, NOW())
  ON CONFLICT (id) DO NOTHING;
