use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const SESSION_TOKEN_BYTES: usize = 32;
const CSRF_TOKEN_BYTES: usize = 32;

pub struct Session {
    pub user_id: String,
    pub csrf_token: String,
    last_active: Instant,
}

impl Session {
    fn is_expired(&self) -> bool {
        self.last_active.elapsed() > SESSION_TIMEOUT
    }

    fn touch(&mut self) {
        self.last_active = Instant::now();
    }
}

pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new session for the given user, returning the session token.
    /// Tokens are generated with a cryptographically secure RNG.
    pub fn create_session(&self, user_id: &str) -> String {
        let token = generate_token(SESSION_TOKEN_BYTES);
        let csrf_token = generate_token(CSRF_TOKEN_BYTES);
        let session = Session {
            user_id: user_id.to_string(),
            csrf_token,
            last_active: Instant::now(),
        };
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(token.clone(), session);
        token
    }

    /// Validate a session token, refreshing its activity timestamp.
    /// Expired sessions are removed and treated as invalid.
    pub fn validate_session(&self, token: &str) -> Option<String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(token) {
            if session.is_expired() {
                sessions.remove(token);
                return None;
            }
        }
        sessions.get_mut(token).map(|session| {
            session.touch();
            session.user_id.clone()
        })
    }

    /// Retrieve the CSRF token associated with a valid session.
    pub fn csrf_token(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .get(token)
            .filter(|session| !session.is_expired())
            .map(|session| session.csrf_token.clone())
    }

    /// Verify a CSRF token for a state-changing request.
    pub fn verify_csrf(&self, token: &str, csrf_token: &str) -> bool {
        match self.csrf_token(token) {
            Some(expected) => constant_time_eq(expected.as_bytes(), csrf_token.as_bytes()),
            None => false,
        }
    }

    /// Invalidate a session on logout so the token can no longer be used.
    pub fn destroy_session(&self, token: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(token);
    }

    /// Remove all expired sessions.
    pub fn purge_expired(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, session| !session.is_expired());
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a hardened session cookie value with HttpOnly and Secure flags.
pub fn session_cookie(token: &str) -> String {
    format!(
        "session={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        token,
        SESSION_TIMEOUT.as_secs()
    )
}

/// Build a cookie that clears the session cookie on logout.
pub fn clear_session_cookie() -> String {
    "session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0".to_string()
}

fn generate_token(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    OsRng.fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
