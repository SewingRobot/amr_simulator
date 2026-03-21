CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_type VARCHAR(50) NOT NULL CHECK (asset_type IN ('robot_model', 'static_object', 'dynamic_object', 'sensor_definition', 'mission_template', 'traffic_rule')),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    properties JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_assets_type ON assets(asset_type);
CREATE INDEX idx_assets_name ON assets(name);
