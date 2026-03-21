CREATE TABLE IF NOT EXISTS robots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    model_id VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'offline' CHECK (status IN ('idle', 'busy', 'error', 'offline', 'charging')),
    config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
