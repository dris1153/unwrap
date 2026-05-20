use serde::{Deserialize, Serialize};

/// Opaque string wrapper for a project identifier (SHA-256 of root path).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        ProjectId(s)
    }
}

/// Unity scripting backend variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScriptingBackend {
    Mono,
    Il2Cpp,
    #[default]
    Unknown,
}

impl std::fmt::Display for ScriptingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptingBackend::Mono => write!(f, "mono"),
            ScriptingBackend::Il2Cpp => write!(f, "il2cpp"),
            ScriptingBackend::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for ScriptingBackend {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mono" => Ok(ScriptingBackend::Mono),
            "il2cpp" => Ok(ScriptingBackend::Il2Cpp),
            _ => Ok(ScriptingBackend::Unknown),
        }
    }
}

/// Cached project metadata stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub root_path: String,
    pub format_id: String,
    pub engine_version: Option<String>,
    pub scripting_backend: ScriptingBackend,
    /// Unix timestamp (seconds).
    pub created_at: i64,
    /// Unix timestamp (seconds).
    pub last_opened: i64,
}

/// An entry in the recents list — lightweight reference to a cached project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub project: Project,
    pub pinned: bool,
}

/// Opaque handle returned to the frontend after `open_project`.
/// Stores the project id so subsequent commands can look it up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHandle {
    pub id: ProjectId,
    pub root_path: String,
    pub format_id: String,
}
