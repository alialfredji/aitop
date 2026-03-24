use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Claude,
    Gemini,
    OpenClaw,
    OpenCode,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Claude => write!(f, "claude"),
            Provider::Gemini => write!(f, "gemini"),
            Provider::OpenClaw => write!(f, "openclaw"),
            Provider::OpenCode => write!(f, "opencode"),
        }
    }
}

impl Provider {
    /// Returns the default data directory for this provider.
    /// For OpenCode, returns the path to the SQLite database file.
    pub fn default_dir(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        match self {
            Provider::Claude => home.join(".claude").join("projects"),
            Provider::Gemini => home.join(".gemini").join("tmp"),
            Provider::OpenClaw => home.join(".openclaw").join("agents"),
            Provider::OpenCode => {
                let primary = home
                    .join(".local")
                    .join("share")
                    .join("opencode")
                    .join("opencode.db");
                let fallback = home.join(".opencode").join("opencode.db");
                if primary.exists() {
                    primary
                } else {
                    fallback
                }
            }
        }
    }

    /// All providers that aitop can scan.
    pub fn all() -> &'static [Provider] {
        &[
            Provider::Claude,
            Provider::Gemini,
            Provider::OpenClaw,
            Provider::OpenCode,
        ]
    }
}

/// Represents a file belonging to a specific provider.
pub struct ProviderFile {
    pub provider: Provider,
    pub path: PathBuf,
    pub session_id: String,
    pub project: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::Claude.to_string(), "claude");
        assert_eq!(Provider::Gemini.to_string(), "gemini");
        assert_eq!(Provider::OpenClaw.to_string(), "openclaw");
        assert_eq!(Provider::OpenCode.to_string(), "opencode");
    }

    #[test]
    fn test_provider_default_dirs() {
        let claude_dir = Provider::Claude.default_dir();
        assert!(claude_dir.to_string_lossy().ends_with(".claude/projects")
            || claude_dir.to_string_lossy().ends_with(".claude\\projects"));

        let gemini_dir = Provider::Gemini.default_dir();
        assert!(gemini_dir.to_string_lossy().ends_with(".gemini/tmp")
            || gemini_dir.to_string_lossy().ends_with(".gemini\\tmp"));

        let openclaw_dir = Provider::OpenClaw.default_dir();
        assert!(openclaw_dir.to_string_lossy().ends_with(".openclaw/agents")
            || openclaw_dir.to_string_lossy().ends_with(".openclaw\\agents"));

        let opencode_dir = Provider::OpenCode.default_dir();
        let s = opencode_dir.to_string_lossy();
        assert!(
            s.contains("opencode"),
            "OpenCode path should contain 'opencode', got: {s}"
        );
    }

    #[test]
    fn test_provider_all() {
        let all = Provider::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], Provider::Claude);
        assert_eq!(all[1], Provider::Gemini);
        assert_eq!(all[2], Provider::OpenClaw);
        assert_eq!(all[3], Provider::OpenCode);
    }
}
