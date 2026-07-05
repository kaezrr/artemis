use std::time::Duration;
use std::time::Instant;

use super::TokenResponse;

#[derive(Debug)]
pub struct Token {
    pub access_token: String,
    pub expires_in: Instant,
}

impl From<TokenResponse> for Token {
    fn from(raw: TokenResponse) -> Self {
        Self {
            access_token: raw.access_token,
            expires_in: Instant::now() + Duration::from_secs(raw.expires_in),
        }
    }
}
