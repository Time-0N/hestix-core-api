use crate::middleware::security::keycloak::claims::KeycloakClaims;

pub fn has_roles(claims: &KeycloakClaims, role: &str) -> bool {
    claims
        .realm_access
        .roles
        .iter()
        .any(|r| r == role)
}