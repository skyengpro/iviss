-- Create agent_locations table for live tracking
CREATE TABLE IF NOT EXISTS agent_locations (
    agent_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Index for performance if we eventually query by area
CREATE INDEX IF NOT EXISTS idx_agent_locations_updated_at ON agent_locations(updated_at);
