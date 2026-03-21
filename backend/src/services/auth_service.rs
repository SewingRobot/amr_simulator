use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // user ID
    pub email: String,
    pub role: String,
    pub exp: usize,      // expiration timestamp
    pub iat: usize,      // issued at
}

/// Hash a password using Argon2id.
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored hash.
pub fn verify_password(password: &str, hash: &str) -> Result<()> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow::anyhow!("Password verification failed"))
}

/// Generate a JWT token for the given user.
pub fn generate_jwt(
    user_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expiration_secs: u64,
) -> Result<String> {
    let now = chrono::Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: now + expiration_secs as usize,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode JWT")?;

    Ok(token)
}

/// Validate a JWT token and return the claims.
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.leeway = 0; // Strict expiration check
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("Failed to decode JWT")?;

    Ok(token_data.claims)
}
