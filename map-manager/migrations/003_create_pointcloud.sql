CREATE TABLE IF NOT EXISTS pointcloud_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    raw_file_path TEXT,
    tiles_path TEXT,
    point_count BIGINT,
    format VARCHAR(50),
    lod_levels INTEGER,
    processing_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pointcloud_data_map ON pointcloud_data(map_id);
CREATE INDEX idx_pointcloud_data_status ON pointcloud_data(processing_status);
