#[macro_export]
macro_rules! require_role {
    ($claims:expr, $role:expr) => {
        if !$claims.0.realm_access.roles.iter().any(|r| r == &role) {
            return Err((
                axum::http::StatusCode::FORBIDDEN,
                format!("Missing required role: {}", $role),
            ));
        }
    };
}

#[macro_export]
macro_rules! require_any_role {
    ($claims:expr, [$( $role:expr ), +]) => {
        if ![$( $role ),+].iter().any(|role| {
            $claims.0.realm_access.roles.iter().any(|r| r == role)
        }) {
            return.Err((
                axum::http::StatusCode::FORBIDDEN,
                format!("Missing any of required roles: {:?}", [$( $role ),+]),
            ));
        }
    };
}