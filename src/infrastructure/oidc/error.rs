use thiserror::Error;

#[derive(Debug, Error)]
pub enum OidcError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("jwt validation error: {0}")]
    Jwt(String),

    #[error("missing claim: {0}")]
    MissingClaim(&'static str),

    #[error("invalid claim {0}: {1}")]
    InvalidClaim(&'static str, String),

    #[error("provider metadata error: {0}")]
    Discovery(String),

    #[error("jwks error: {0}")]
    Jwks(String),

    #[error("token exchange error: {0}")]
    TokenExchange(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("jwks key not found")]
    KeyNotFound,            // <— add

    #[error("not implemented: {0}")]
    NotImplemented(String), // <— add

    #[error("internal oidc error: {0}")]
    Internal(String),
}
