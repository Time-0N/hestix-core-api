use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeycloakError {
    #[error("Keycloak request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Keycloak returned an unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid client credentials")]
    InvalidClientCredentials,

    #[error("Missing token")]
    MissingToken,

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Other keycloak error: {0}")]
    Other(String),

    #[error("Missing keycloak user id")]
    MissingUserId,
}