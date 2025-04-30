-- Add migration script here

CREATE TABLE IF NOT EXISTS json_data (
    id SERIAL PRIMARY KEY,
    file_name TEXT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_json_data_file_name ON json_data(file_name);
CREATE INDEX IF NOT EXISTS idx_json_data_created_at ON json_data(created_at);
