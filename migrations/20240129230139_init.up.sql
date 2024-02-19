-- Add up migration script here
CREATE TABLE IF NOT EXISTS batches (
  id SERIAL PRIMARY KEY,
  sku VARCHAR(255)
);