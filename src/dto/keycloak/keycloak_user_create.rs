use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct KeycloakUserCreate {
    pub username: String,
    pub email: String,
    pub enabled: bool,
    pub credentials: Vec<KeycloakCredential>,
}

#[derive(Debug, Serialize)]
pub struct KeycloakCredential {
    pub r#type: String,
    pub value: String,
    pub temporary: bool,
}

impl KeycloakUserCreate {
    pub fn new(username: String, email: String, password: String) -> Self {
        KeycloakUserCreate {
            username,
            email,
            enabled: true,
            credentials: vec![KeycloakCredential {
                r#type: "password".into(),
                value: password,
                temporary: false,
            }]
        }
    }
}