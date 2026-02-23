-- Add db_type column to saved_connections to support multiple database types
ALTER TABLE saved_connections ADD COLUMN db_type VARCHAR(20) NOT NULL DEFAULT 'postgres';
