//! PTZ protocol registry for managing available protocol implementations

use std::collections::HashMap;
use std::sync::Arc;

use super::bridge::PerlControlFactory;
use super::capabilities::PtzCapabilities;
use super::traits::{PtzConnectionConfig, PtzControl, PtzControlFactory};

/// Information about a registered protocol
#[derive(Debug, Clone)]
pub struct ProtocolInfo {
    /// Protocol name
    pub name: String,
    /// Whether this is a native Rust implementation
    pub is_native: bool,
    /// Description of the protocol
    pub description: Option<String>,
}

/// Registry for PTZ protocol implementations
pub struct PtzRegistry {
    /// Native Rust protocol factories
    native_factories: HashMap<String, Arc<dyn PtzControlFactory>>,

    /// Perl fallback factory (used when no native implementation exists)
    perl_factory: Arc<PerlControlFactory>,

    /// Whether to allow Perl fallback
    allow_perl_fallback: bool,
}

impl PtzRegistry {
    /// Create a new registry with Perl fallback enabled
    pub fn new() -> Self {
        Self {
            native_factories: HashMap::new(),
            perl_factory: Arc::new(PerlControlFactory::default()),
            allow_perl_fallback: true,
        }
    }

    /// Create a registry with custom zmcontrol.pl path and socket directory
    pub fn with_zmcontrol_path(zmcontrol_path: String, socket_dir: Option<String>) -> Self {
        Self {
            native_factories: HashMap::new(),
            perl_factory: Arc::new(PerlControlFactory::new(Some(zmcontrol_path), socket_dir)),
            allow_perl_fallback: true,
        }
    }

    /// Create a registry with custom socket directory only
    pub fn with_socket_dir(socket_dir: String) -> Self {
        Self {
            native_factories: HashMap::new(),
            perl_factory: Arc::new(PerlControlFactory::new(None, Some(socket_dir))),
            allow_perl_fallback: true,
        }
    }

    /// Enable or disable Perl fallback
    pub fn set_perl_fallback(&mut self, enabled: bool) {
        self.allow_perl_fallback = enabled;
    }

    /// Register a native Rust protocol factory
    pub fn register_native(&mut self, factory: Arc<dyn PtzControlFactory>) {
        let name = factory.protocol_name().to_lowercase();
        self.native_factories.insert(name, factory);
    }

    /// Check if a native implementation exists for a protocol
    pub fn has_native(&self, protocol: &str) -> bool {
        self.native_factories.contains_key(&protocol.to_lowercase())
    }

    /// Get a factory for a protocol (prefers native, falls back to Perl)
    pub fn get_factory(&self, protocol: &str) -> Option<Arc<dyn PtzControlFactory>> {
        let protocol_lower = protocol.to_lowercase();

        // Try native first
        if let Some(factory) = self.native_factories.get(&protocol_lower) {
            return Some(Arc::clone(factory));
        }

        // Fall back to Perl if allowed
        if self.allow_perl_fallback {
            return Some(Arc::clone(&self.perl_factory) as Arc<dyn PtzControlFactory>);
        }

        None
    }

    /// Create a PTZ control instance for a protocol
    pub fn create_control(
        &self,
        protocol: &str,
        config: PtzConnectionConfig,
        capabilities: PtzCapabilities,
    ) -> Option<Box<dyn PtzControl>> {
        self.get_factory(protocol)
            .map(|factory| factory.create(config, capabilities))
    }

    /// List all available protocols
    pub fn list_protocols(&self) -> Vec<ProtocolInfo> {
        let mut protocols: Vec<ProtocolInfo> = self
            .native_factories
            .iter()
            .map(|(name, factory)| ProtocolInfo {
                name: name.clone(),
                is_native: factory.is_native(),
                description: Some("Native Rust implementation".to_string()),
            })
            .collect();

        // Add a marker for Perl fallback
        if self.allow_perl_fallback {
            protocols.push(ProtocolInfo {
                name: "*".to_string(),
                is_native: false,
                description: Some("Perl fallback for all other protocols".to_string()),
            });
        }

        protocols.sort_by(|a, b| a.name.cmp(&b.name));
        protocols
    }

    /// Get the list of native protocol names
    pub fn native_protocols(&self) -> Vec<String> {
        self.native_factories.keys().cloned().collect()
    }
}

impl Default for PtzRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = PtzRegistry::new();
        assert!(registry.allow_perl_fallback);
        assert!(registry.native_factories.is_empty());
    }

    #[test]
    fn test_registry_perl_fallback() {
        let registry = PtzRegistry::new();

        // Should return Perl factory for unknown protocol
        let factory = registry.get_factory("unknown_protocol");
        assert!(factory.is_some());
        assert!(!factory.unwrap().is_native());
    }

    #[test]
    fn test_registry_no_perl_fallback() {
        let mut registry = PtzRegistry::new();
        registry.set_perl_fallback(false);

        // Should return None for unknown protocol
        let factory = registry.get_factory("unknown_protocol");
        assert!(factory.is_none());
    }

    #[test]
    fn test_list_protocols() {
        let registry = PtzRegistry::new();
        let protocols = registry.list_protocols();

        // Should have at least the Perl fallback marker
        assert!(!protocols.is_empty());
        assert!(protocols.iter().any(|p| p.name == "*"));
    }
}
