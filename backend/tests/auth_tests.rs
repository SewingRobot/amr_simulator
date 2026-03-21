//! Integration tests for the auth_service module.
//!
//! These tests verify JWT generation/validation and password hashing
//! without requiring a running database or server.

#[cfg(test)]
mod tests {
    use amr_backend::services::auth_service;

    // ── JWT round-trip ──────────────────────────────────────────────

    #[test]
    fn test_jwt_roundtrip() {
        let secret = "test-secret-key-for-testing";
        let token = auth_service::generate_jwt("user-123", "admin@test.com", "admin", secret, 3600)
            .expect("token generation should succeed");

        let claims = auth_service::validate_jwt(&token, secret)
            .expect("token validation should succeed");

        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "admin@test.com");
        assert_eq!(claims.role, "admin");
    }

    // ── JWT with different roles ────────────────────────────────────

    #[test]
    fn test_jwt_operator_role() {
        let secret = "test-secret";
        let token =
            auth_service::generate_jwt("user-456", "op@test.com", "operator", secret, 3600)
                .unwrap();
        let claims = auth_service::validate_jwt(&token, secret).unwrap();
        assert_eq!(claims.role, "operator");
    }

    #[test]
    fn test_jwt_viewer_role() {
        let secret = "test-secret";
        let token =
            auth_service::generate_jwt("user-789", "viewer@test.com", "viewer", secret, 3600)
                .unwrap();
        let claims = auth_service::validate_jwt(&token, secret).unwrap();
        assert_eq!(claims.role, "viewer");
    }

    // ── JWT expiration ──────────────────────────────────────────────

    #[test]
    fn test_jwt_expired() {
        let secret = "test-secret";
        // Generate token with 1-second expiration and wait for it to expire
        let token =
            auth_service::generate_jwt("user-123", "test@test.com", "admin", secret, 1).unwrap();

        // Wait for token to expire (1s TTL + buffer)
        std::thread::sleep(std::time::Duration::from_secs(2));
        let result = auth_service::validate_jwt(&token, secret);
        assert!(result.is_err(), "expired token should fail validation");
    }

    // ── JWT wrong secret ────────────────────────────────────────────

    #[test]
    fn test_jwt_invalid_secret() {
        let token =
            auth_service::generate_jwt("user-123", "test@test.com", "admin", "secret1", 3600)
                .unwrap();

        let result = auth_service::validate_jwt(&token, "secret2");
        assert!(
            result.is_err(),
            "token validated with wrong secret should fail"
        );
    }

    // ── JWT malformed token ─────────────────────────────────────────

    #[test]
    fn test_jwt_malformed_token() {
        let result = auth_service::validate_jwt("not.a.valid.jwt", "secret");
        assert!(result.is_err(), "malformed token should fail validation");
    }

    #[test]
    fn test_jwt_empty_token() {
        let result = auth_service::validate_jwt("", "secret");
        assert!(result.is_err(), "empty token should fail validation");
    }

    // ── Password hashing ────────────────────────────────────────────

    #[test]
    fn test_password_hash_verify() {
        let hash = auth_service::hash_password("mypassword").expect("hashing should succeed");

        // Correct password should verify
        assert!(
            auth_service::verify_password("mypassword", &hash).is_ok(),
            "correct password should verify"
        );

        // Wrong password should fail
        assert!(
            auth_service::verify_password("wrongpassword", &hash).is_err(),
            "wrong password should fail verification"
        );
    }

    #[test]
    fn test_password_hash_unique_salts() {
        let hash1 = auth_service::hash_password("samepassword").unwrap();
        let hash2 = auth_service::hash_password("samepassword").unwrap();

        // Even with the same input, salts differ so hashes differ
        assert_ne!(hash1, hash2, "two hashes of same password should differ (different salts)");

        // But both should still verify
        assert!(auth_service::verify_password("samepassword", &hash1).is_ok());
        assert!(auth_service::verify_password("samepassword", &hash2).is_ok());
    }

    #[test]
    fn test_password_hash_invalid_hash_string() {
        let result = auth_service::verify_password("password", "not-a-valid-hash");
        assert!(result.is_err(), "invalid hash string should error");
    }
}
