use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme preference for the application UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl Default for ThemePreference {
    fn default() -> Self {
        ThemePreference::System
    }
}

/// How often a user wants to receive notifications.
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
        NotificationFrequency::Instant
    }
}

/// Per-user settings. Defaults are provided for new users via `Default`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: ThemePreference,
    pub notification_frequency: NotificationFrequency,
    pub email_notifications: bool,
    pub push_notifications: bool,
    /// Free-form personalization options (e.g. language, timezone).
    #[serde(default)]
    pub preferences: HashMap<String, String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            theme: ThemePreference::default(),
            notification_frequency: NotificationFrequency::default(),
            email_notifications: true,
            push_notifications: true,
            preferences: HashMap::new(),
        }
    }
}

/// A single tracked change to a user's settings, for audit/history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsChange {
    pub user_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: String,
}

/// Request payload for updating settings. All fields optional so callers can
/// patch only what they need; unspecified fields keep their current value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemePreference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_frequency: Option<NotificationFrequency>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_notifications: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<HashMap<String, String>>,
}

impl UserSettings {
    /// Apply a partial update, returning the list of tracked changes.
    pub fn apply_update(&mut self, user_id: &str, update: UpdateSettingsRequest, changed_at: &str) -> Vec<SettingsChange> {
        let mut changes = Vec::new();

        if let Some(theme) = update.theme {
            if theme != self.theme {
                changes.push(SettingsChange {
                    user_id: user_id.to_string(),
                    field: "theme".to_string(),
                    old_value: Some(format!("{:?}", self.theme)),
                    new_value: Some(format!("{:?}", theme)),
                    changed_at: changed_at.to_string(),
                });
                self.theme = theme;
            }
        }

        if let Some(freq) = update.notification_frequency {
            if freq != self.notification_frequency {
                changes.push(SettingsChange {
                    user_id: user_id.to_string(),
                    field: "notification_frequency".to_string(),
                    old_value: Some(format!("{:?}", self.notification_frequency)),
                    new_value: Some(format!("{:?}", freq)),
                    changed_at: changed_at.to_string(),
                });
                self.notification_frequency = freq;
            }
        }

        if let Some(email) = update.email_notifications {
            if email != self.email_notifications {
                changes.push(SettingsChange {
                    user_id: user_id.to_string(),
                    field: "email_notifications".to_string(),
                    old_value: Some(self.email_notifications.to_string()),
                    new_value: Some(email.to_string()),
                    changed_at: changed_at.to_string(),
                });
                self.email_notifications = email;
            }
        }

        if let Some(push) = update.push_notifications {
            if push != self.push_notifications {
                changes.push(SettingsChange {
                    user_id: user_id.to_string(),
                    field: "push_notifications".to_string(),
                    old_value: Some(self.push_notifications.to_string()),
                    new_value: Some(push.to_string()),
                    changed_at: changed_at.to_string(),
                });
                self.push_notifications = push;
            }
        }

        if let Some(prefs) = update.preferences {
            if prefs != self.preferences {
                changes.push(SettingsChange {
                    user_id: user_id.to_string(),
                    field: "preferences".to_string(),
                    old_value: Some(format!("{:?}", self.preferences)),
                    new_value: Some(format!("{:?}", prefs)),
                    changed_at: changed_at.to_string(),
                });
                self.preferences = prefs;
            }
        }

        changes
    }
}
