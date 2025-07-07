pub fn login_user(username: &str, password: &str) -> Option<String> {
    if username == "admin" && password == "secret" {
        Some("mock-token-123".to_string())
    } else {
        None
    }
}