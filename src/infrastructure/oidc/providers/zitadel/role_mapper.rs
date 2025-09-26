use serde_json::Value;
use crate::infrastructure::oidc::provider::RoleMapper;

pub struct ZitadelRoleMapper;

impl RoleMapper for ZitadelRoleMapper {
    fn extract_roles(&self, claims: &Value) -> Vec<String> {
        use std::collections::BTreeSet;

        let mut roles = BTreeSet::new();

        // A) Generic project roles for the current client/project
        if let Some(obj) = claims
            .get("urn:zitadel:iam:org:project:roles")
            .and_then(|v| v.as_object())
        {
            roles.extend(obj.keys().cloned());
        }

        // B) Any project-specific roles key: urn:zitadel:iam:org:project:{projectId}:roles
        if let Some(root) = claims.as_object() {
            for (k, v) in root {
                if k.starts_with("urn:zitadel:iam:org:project:")
                    && k.ends_with(":roles")
                    && k != "urn:zitadel:iam:org:project:roles"
                {
                    if let Some(obj) = v.as_object() {
                        roles.extend(obj.keys().cloned());
                    }
                }
            }
        }

        roles.into_iter().collect()
    }
}
