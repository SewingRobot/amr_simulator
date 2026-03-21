CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'viewer' CHECK (role IN ('admin', 'operator', 'viewer')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed admin user (password: admin123)
INSERT INTO users (email, password_hash, role)
VALUES ('admin@amr.local', '$argon2id$v=19$m=19456,t=2,p=1$placeholder', 'admin')
ON CONFLICT (email) DO NOTHING;
