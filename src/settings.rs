//! User preference and settings management.
//!
//! Provides a settings schema with sensible defaults, a small management API
//! for reading/updating settings, persistence across sessions, and change
//! tracking (audit history) for preference updates.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Theme preference selected by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

/// How often the user wants to receive notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationFrequency {
    Instant,
    Daily,
    Weekly,
    Never,
}

impl Default for NotificationFrequency {
    fn default() -> Self {
        NotificationFrequency::Daily
    }
}

/// Notification-related preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub frequency: NotificationFrequency,
    pub email_enabled: bool,
    pub push_enabled: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        NotificationSettings {
            frequency: NotificationFrequency::default(),
            email_enabled: true,
            push_enabled: false,
        }
    }
}

/// Full user settings schema. `Default` provides the settings given to new users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: Theme,
    pub notifications: NotificationSettings,
    /// Free-form personalization options (e.g. language, timezone).
    pub preferences: HashMap<String, String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            theme: Theme::default(),
            notifications: NotificationSettings::default(),
            preferences: HashMap::new(),
        }
    }
}

/// A single recorded settings change for auditing purposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsChange {
    pub user_id: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: u64,
}

/// Persistence backend for user settings.
pub trait SettingsStore: Send + Sync {
    fn load(&self, user_id: &str) -> Option<UserSettings>;
    fn save(&self, user_id: &str, settings: &UserSettings);
}

/// In-memory store used as a default persistence implementation.
#[derive(Debug, Default)]
pub struct InMemorySettingsStore {
    data: Mutex<HashMap<String, UserSettings>>,
}

impl SettingsStore for InMemorySettingsStore {
    fn load(&self, user_id: &str) -> Option<UserSettings> {
        self.data.lock().unwrap().get(user_id).cloned()
    }

    fn save(&self, user_id: &str, settings: &UserSettings) {
        self.data
            .lock()
            .unwrap()
            .insert(user_id.to_string(), settings.clone());
    }
}

/// Settings management API: reads, updates, persists, and tracks changes.
pub struct SettingsManager {
    store: Arc<dyn SettingsStore>,
    history: Mutex<Vec<SettingsChange>>,
}

impl SettingsManager {
    pub fn new(store: Arc<dyn SettingsStore>) -> Self {
        SettingsManager {
            store,
            history: Mutex::new(Vec::new()),
        }
    }

    /// Load settings for a user, falling back to defaults for new users.
    pub fn get(&self, user_id: &str) -> UserSettings {
        self.store.load(user_id).unwrap_or_default()
    }

    /// Apply an update, persist it, and record the change.
    pub fn update<F>(&self, user_id: &str, timestamp: u64, mutate: F) -> UserSettings
    where
        F: FnOnce(&mut UserSettings),
    {
        let old = self.get(user_id);
        let mut new = old.clone();
        mutate(&mut new);

        if new != old {
            self.record_changes(user_id, &old, &new, timestamp);
            self.store.save(user_id, &new);
        }
        new
    }

    /// Change history for a user, oldest first.
    pub fn history(&self, user_id: &str) -> Vec<SettingsChange> {
        self.history
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect()
    }

    fn record_changes(&self, user_id: &str, old: &UserSettings, new: &UserSettings, timestamp: u64) {
        let mut history = self.history.lock().unwrap();
        let mut push = |field: &str, old_value: String, new_value: String| {
            history.push(SettingsChange {
                user_id: user_id.to_string(),
                field: field.to_string(),
                old_value,
                new_value,
                timestamp,
            });
        };

        if old.theme != new.theme {
            push(
                "theme",
                format!("{:?}", old.theme),
                format!("{:?}", new.theme),
            );
        }
        if old.notifications.frequency != new.notifications.frequency {
            push(
                "notifications.frequency",
                format!("{:?}", old.notifications.frequency),
                format!("{:?}", new.notifications.frequency),
            );
        }
        if old.notifications.email_enabled != new.notifications.email_enabled {
            push(
                "notifications.email_enabled",
                old.notifications.email_enabled.to_string(),
                new.notifications.email_enabled.to_string(),
            );
        }
        if old.notifications.push_enabled != new.notifications.push_enabled {
            push(
                "notifications.push_enabled",
                old.notifications.push_enabled.to_string(),
                new.notifications.push_enabled.to_string(),
            );
        }
        if old.preferences != new.preferences {
            push(
                "preferences",
                format!("{:?}", old.preferences),
                format!("{:?}", new.preferences),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> SettingsManager {
        SettingsManager::new(Arc::new(InMemorySettingsStore::default()))
    }

    #[test]
    fn new_users_get_defaults() {
        let m = manager();
        assert_eq!(m.get("alice"), UserSettings::default());
    }

    #[test]
    fn updates_persist_and_are_tracked() {
        let m = manager();
        m.update("alice", 1, |s| s.theme = Theme::Dark);
        assert_eq!(m.get("alice").theme, Theme::Dark);

        let history = m.history("alice");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].field, "theme");
    }

    #[test]
    fn no_change_records_nothing() {
        let m = manager();
        m.update("alice", 1, |_| {});
        assert!(m.history("alice").is_empty());
    }
}
