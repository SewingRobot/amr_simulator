CREATE TABLE IF NOT EXISTS roadmap_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name VARCHAR(255),
    node_type VARCHAR(50) NOT NULL DEFAULT 'waypoint',
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL DEFAULT 0,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_roadmap_nodes_map ON roadmap_nodes(map_id);

CREATE TABLE IF NOT EXISTS roadmap_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    source_node_id UUID NOT NULL REFERENCES roadmap_nodes(id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES roadmap_nodes(id) ON DELETE CASCADE,
    distance DOUBLE PRECISION NOT NULL,
    max_speed DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    direction VARCHAR(10) NOT NULL DEFAULT 'bi',
    cost_factor DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_roadmap_edges_map ON roadmap_edges(map_id);
CREATE INDEX IF NOT EXISTS idx_roadmap_edges_source ON roadmap_edges(source_node_id);
CREATE INDEX IF NOT EXISTS idx_roadmap_edges_target ON roadmap_edges(target_node_id);
