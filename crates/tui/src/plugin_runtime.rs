//! Plugin runtime registry.
//!
//! Port of packages/tui/src/plugin/runtime.tsx `createPluginRuntime` behaviour
//! (upstream 18ef3cc). Solid reactivity is re-derived as a plain registry.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A route registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<R> {
    pub name: String,
    pub render: R,
}

/// A registry of render routes, newest registration winning.
pub struct RouteRegistry<R> {
    map: Arc<Mutex<HashMap<String, Vec<R>>>>,
}

impl<R> Default for RouteRegistry<R> {
    fn default() -> Self {
        RouteRegistry {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<R: Clone + 'static> RouteRegistry<R> {
    /// Create an empty route registry.
    pub fn new() -> Self {
        RouteRegistry::default()
    }

    /// Register routes, returning a dispose function that restores prior ones.
    pub fn register(&self, routes: Vec<Route<R>>) -> impl Fn() {
        let names: Vec<String> = routes.iter().map(|route| route.name.clone()).collect();
        {
            let mut map = self.map.lock().unwrap();
            for route in routes {
                map.entry(route.name).or_default().push(route.render);
            }
        }
        let map = self.map.clone();
        move || {
            let mut map = map.lock().unwrap();
            for name in names.iter().rev() {
                if let Some(stack) = map.get_mut(name) {
                    stack.pop();
                }
            }
        }
    }

    /// The latest render registered for a route name.
    pub fn get(&self, name: &str) -> Option<R> {
        self.map
            .lock()
            .unwrap()
            .get(name)
            .and_then(|stack| stack.last().cloned())
    }
}

/// The command surface a plugin host exposes.
pub trait PluginCommands {
    fn activate(&self, id: &str) -> bool;
}

/// A plugin status entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub id: String,
    pub source: String,
    pub spec: String,
    pub target: String,
    pub enabled: bool,
    pub active: bool,
}

/// The plugin runtime facade.
pub struct PluginRuntime<R, C> {
    pub routes: RouteRegistry<R>,
    commands: Option<C>,
    status: Vec<StatusEntry>,
}

impl<R: Clone + 'static, C: PluginCommands> Default for PluginRuntime<R, C> {
    fn default() -> Self {
        PluginRuntime {
            routes: RouteRegistry::new(),
            commands: None,
            status: Vec::new(),
        }
    }
}

impl<R: Clone + 'static, C: PluginCommands> PluginRuntime<R, C> {
    /// Create an empty runtime.
    pub fn new() -> Self {
        PluginRuntime::default()
    }

    /// Publish the command surface and status entries.
    pub fn update(&mut self, commands: Option<C>, status: Vec<StatusEntry>) {
        self.commands = commands;
        self.status = status;
    }

    /// The command facade.
    pub fn commands(&self) -> CommandFacade<'_, C> {
        CommandFacade {
            commands: self.commands.as_ref(),
        }
    }

    /// The published status entries.
    pub fn status(&self) -> &[StatusEntry] {
        &self.status
    }

    /// Clear all published state.
    pub fn clear(&mut self) {
        self.commands = None;
        self.status.clear();
    }
}

/// A borrowed view over the plugin command surface.
pub struct CommandFacade<'a, C> {
    commands: Option<&'a C>,
}

impl<C: PluginCommands> CommandFacade<'_, C> {
    /// Activate a plugin command, returning false when none is published.
    pub fn activate(&self, id: &str) -> bool {
        self.commands
            .map(|commands| commands.activate(id))
            .unwrap_or(false)
    }
}

/// Create a plugin runtime.
pub fn create_plugin_runtime<R: Clone + 'static, C: PluginCommands>() -> PluginRuntime<R, C> {
    PluginRuntime::new()
}
