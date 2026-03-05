

use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub jti: String,
    pub device_id: Uuid,
    pub shift_expires_at: usize,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl From<&JwtClaims> for AuthenticatedUser {
    fn from(claims: &JwtClaims) -> Self {
        Self {
            user_id: claims.sub,
        }
    }
}
