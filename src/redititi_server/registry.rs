//! Session and pane registry
//!
//! Manages the registry of active sessions and panes with random name generation.

use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: Instant,
    pub panes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PaneInfo {
    pub id: String,
    pub session_id: String,
    pub terminal_connected: bool,
}

pub struct Registry {
    sessions: HashMap<String, SessionInfo>,
    panes: HashMap<(String, String), PaneInfo>, // (session_id, pane_id) -> PaneInfo
}

impl Registry {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            panes: HashMap::new(),
        }
    }

    /// Generate a unique random session name
    pub fn generate_session_name(&self) -> String {
        loop {
            let name = Self::generate_memorable_name();
            if !self.sessions.contains_key(&name) {
                return name;
            }
        }
    }

    /// Generate a unique random pane name within a session
    pub fn generate_pane_name(&self, session_id: &str) -> String {
        loop {
            let name = Self::generate_memorable_name();
            if !self.panes.contains_key(&(session_id.to_string(), name.clone())) {
                return name;
            }
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, name: Option<String>) -> Result<String, String> {
        let session_id = name.unwrap_or_else(|| self.generate_session_name());

        if self.sessions.contains_key(&session_id) {
            return Err(format!("Session '{}' already exists", session_id));
        }

        self.sessions.insert(
            session_id.clone(),
            SessionInfo {
                id: session_id.clone(),
                created_at: Instant::now(),
                panes: Vec::new(),
            },
        );

        Ok(session_id)
    }

    /// Create a new pane in a session
    pub fn create_pane(
        &mut self,
        session_id: &str,
        name: Option<String>,
    ) -> Result<String, String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        let pane_id = name.unwrap_or_else(|| self.generate_pane_name(session_id));

        let key = (session_id.to_string(), pane_id.clone());
        if self.panes.contains_key(&key) {
            return Err(format!("Pane '{}' already exists in session '{}'", pane_id, session_id));
        }

        self.panes.insert(
            key,
            PaneInfo {
                id: pane_id.clone(),
                session_id: session_id.to_string(),
                terminal_connected: false,
            },
        );

        // Add pane to session's pane list
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.panes.push(pane_id.clone());
        }

        Ok(pane_id)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// List all panes in a session
    pub fn list_panes(&self, session_id: &str) -> Option<Vec<String>> {
        self.sessions.get(session_id).map(|s| s.panes.clone())
    }

    /// Get session info
    pub fn get_session(&self, session_id: &str) -> Option<&SessionInfo> {
        self.sessions.get(session_id)
    }

    /// Get pane info
    pub fn get_pane(&self, session_id: &str, pane_id: &str) -> Option<&PaneInfo> {
        self.panes.get(&(session_id.to_string(), pane_id.to_string()))
    }

    /// Remove a pane
    pub fn remove_pane(&mut self, session_id: &str, pane_id: &str) -> Result<(), String> {
        let key = (session_id.to_string(), pane_id.to_string());

        if !self.panes.contains_key(&key) {
            return Err(format!("Pane '{}' not found in session '{}'", pane_id, session_id));
        }

        self.panes.remove(&key);

        // Remove pane from session's pane list
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.panes.retain(|p| p != pane_id);
        }

        Ok(())
    }

    /// Remove a session and all its panes
    pub fn remove_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        // Remove all panes
        let panes = self.sessions.get(session_id).unwrap().panes.clone();
        for pane_id in panes {
            self.panes.remove(&(session_id.to_string(), pane_id));
        }

        // Remove session
        self.sessions.remove(session_id);

        Ok(())
    }

    /// Generate a memorable name (adjective-color+digit, max 10 chars)
    fn generate_memorable_name() -> String {
        const ADJECTIVES: &[&str] = &[
            "libre", "syno", "quick", "bold", "bright", "smart", "clear", "fresh", "prime",
            "swift", "noble", "grand", "vital", "keen",
        ];

        const COLORS: &[&str] = &[
            "red", "blue", "green", "gold", "silver", "blond", "azure", "coral", "amber",
            "pearl", "ruby", "jade", "onyx",
        ];

        let mut rng = rand::thread_rng();

        let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
        let color = COLORS[rng.gen_range(0..COLORS.len())];
        let digit = rng.gen_range(1..=9);

        format!("{}-{}{}", adj, color, digit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_memorable_name() {
        for _ in 0..100 {
            let name = Registry::generate_memorable_name();
            // Format: {adjective}-{color}{digit}
            // Max length: "bright-silver9" = 14 chars
            assert!(name.len() <= 15, "Name too long: {}", name);
            assert!(name.contains('-'), "Name should contain hyphen: {}", name);
            assert!(name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        }
    }

    #[test]
    fn test_create_session() {
        let mut registry = Registry::new();

        let session_id = registry.create_session(Some("test-session".to_string())).unwrap();
        assert_eq!(session_id, "test-session");

        // Should fail to create duplicate
        assert!(registry.create_session(Some("test-session".to_string())).is_err());
    }

    #[test]
    fn test_create_pane() {
        let mut registry = Registry::new();

        let session_id = registry.create_session(Some("test-session".to_string())).unwrap();
        let pane_id = registry.create_pane(&session_id, Some("test-pane".to_string())).unwrap();

        assert_eq!(pane_id, "test-pane");

        let panes = registry.list_panes(&session_id).unwrap();
        assert_eq!(panes, vec!["test-pane"]);
    }

    #[test]
    fn test_auto_generated_names() {
        let mut registry = Registry::new();

        let session_id = registry.create_session(None).unwrap();
        assert!(session_id.len() <= 15, "Session name too long: {}", session_id);

        let pane_id = registry.create_pane(&session_id, None).unwrap();
        assert!(pane_id.len() <= 15, "Pane name too long: {}", pane_id);
    }

    #[test]
    fn test_remove_pane() {
        let mut registry = Registry::new();

        let session_id = registry.create_session(Some("test".to_string())).unwrap();
        let pane_id = registry.create_pane(&session_id, Some("pane1".to_string())).unwrap();

        registry.remove_pane(&session_id, &pane_id).unwrap();

        assert!(registry.list_panes(&session_id).unwrap().is_empty());
    }

    #[test]
    fn test_remove_session() {
        let mut registry = Registry::new();

        let session_id = registry.create_session(Some("test".to_string())).unwrap();
        registry.create_pane(&session_id, Some("pane1".to_string())).unwrap();
        registry.create_pane(&session_id, Some("pane2".to_string())).unwrap();

        registry.remove_session(&session_id).unwrap();

        assert!(registry.list_sessions().is_empty());
        assert!(registry.get_pane(&session_id, "pane1").is_none());
    }
}
