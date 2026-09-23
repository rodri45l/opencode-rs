//! Integration registry.
//!
//! Ports the pure, decidable parts of `packages/core/src/integration.ts`: a
//! scoped registry where closing a scope reveals the previous registration (or
//! absence), methods register and override independently of the integration
//! name, and connections project the active credential followed by the declared
//! env names that are present. The `Credential`/`EventV2` wiring and the OAuth
//! attempt lifecycle are not reproduced here.

use std::collections::BTreeMap;

/// Error raised by the integration registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationError {
    /// Placeholder for API compatibility.
    NotImplemented,
}

impl std::fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrationError::NotImplemented => write!(f, "not implemented"),
        }
    }
}

impl std::error::Error for IntegrationError {}

/// A registered method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method {
    /// Optional method id.
    pub id: Option<String>,
    /// The method kind.
    pub kind: &'static str,
    /// The display label.
    pub label: String,
}

/// A projected connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Connection {
    /// An active credential connection.
    Credential {
        /// Credential id.
        id: String,
        /// Credential label.
        label: String,
    },
    /// A declared environment-variable connection.
    Env {
        /// Environment-variable name.
        name: String,
    },
}

/// A resolved integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    /// Integration id.
    pub id: String,
    /// Integration name.
    pub name: String,
    /// Registered methods.
    pub methods: Vec<Method>,
    /// Projected connections.
    pub connections: Vec<Connection>,
}

/// A scope identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeId(pub u64);

#[derive(Default)]
struct ScopeData {
    integrations: BTreeMap<String, String>,
    methods: BTreeMap<(String, String), Method>,
}

/// The scoped integration registry.
#[derive(Default)]
pub struct Registry {
    scopes: Vec<(ScopeId, ScopeData)>,
    next: u64,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Result<Self, IntegrationError> {
        Ok(Registry::default())
    }

    /// Open a new scope.
    pub fn open_scope(&mut self) -> Result<ScopeId, IntegrationError> {
        self.next += 1;
        let id = ScopeId(self.next);
        self.scopes.push((id, ScopeData::default()));
        Ok(id)
    }

    /// Close `scope`, discarding its registrations.
    pub fn close_scope(&mut self, scope: ScopeId) -> Result<(), IntegrationError> {
        self.scopes.retain(|(id, _)| *id != scope);
        Ok(())
    }

    /// Register or override an integration in `scope`.
    pub fn update(&mut self, scope: ScopeId, id: &str, name: &str) -> Result<(), IntegrationError> {
        if let Some((_, data)) = self.scopes.iter_mut().find(|(entry, _)| *entry == scope) {
            data.integrations.insert(id.to_string(), name.to_string());
        }
        Ok(())
    }

    /// Register or override a method in `scope`.
    pub fn update_method(
        &mut self,
        scope: ScopeId,
        integration_id: &str,
        method: Method,
    ) -> Result<(), IntegrationError> {
        if let Some((_, data)) = self.scopes.iter_mut().find(|(entry, _)| *entry == scope) {
            let key = method.id.clone().unwrap_or_default();
            data.methods
                .insert((integration_id.to_string(), key), method);
        }
        Ok(())
    }

    /// Resolve the effective registration for `id`.
    pub fn get(&self, id: &str) -> Result<Option<Info>, IntegrationError> {
        let mut name: Option<String> = None;
        let mut methods: BTreeMap<String, Method> = BTreeMap::new();
        for (_, data) in &self.scopes {
            if let Some(value) = data.integrations.get(id) {
                name = Some(value.clone());
            }
            for ((integration, key), method) in &data.methods {
                if integration == id {
                    methods.insert(key.clone(), method.clone());
                }
            }
        }
        if name.is_none() && methods.is_empty() {
            return Ok(None);
        }
        Ok(Some(Info {
            id: id.to_string(),
            name: name.unwrap_or_else(|| id.to_string()),
            methods: methods.into_values().collect(),
            connections: Vec::new(),
        }))
    }

    /// List the effective integrations, sorted by id.
    pub fn list(&self) -> Result<Vec<Info>, IntegrationError> {
        let mut ids: Vec<String> = Vec::new();
        for (_, data) in &self.scopes {
            for id in data.integrations.keys() {
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
            }
            for (integration, _) in data.methods.keys() {
                if !ids.contains(integration) {
                    ids.push(integration.clone());
                }
            }
        }
        ids.sort();
        let mut out = Vec::new();
        for id in ids {
            if let Some(info) = self.get(&id)? {
                out.push(info);
            }
        }
        Ok(out)
    }
}

/// Project the active credential followed by the present env names.
pub fn project_connections(
    active: Option<(&str, &str)>,
    env: &[(&str, bool)],
) -> Result<Vec<Connection>, IntegrationError> {
    let mut connections = Vec::new();
    if let Some((id, label)) = active {
        connections.push(Connection::Credential {
            id: id.to_string(),
            label: label.to_string(),
        });
    }
    for (name, present) in env {
        if *present {
            connections.push(Connection::Env {
                name: (*name).to_string(),
            });
        }
    }
    Ok(connections)
}
