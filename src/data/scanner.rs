use anyhow::Result;
use std::path::{Path, PathBuf};

use super::parser::decode_project_name;

#[derive(Debug, Clone)]
pub struct SessionFile {
    pub path: PathBuf,
    pub session_id: String,
    pub project: String,
}

/// Scan the projects directory for all JSONL session files.
/// Recurses into subdirectories to find subagent session files
/// (e.g., `<project>/<session>/subagents/*.jsonl`).
pub fn scan_projects(projects_dir: &Path) -> Result<Vec<SessionFile>> {
    let mut files = Vec::new();

    if !projects_dir.exists() {
        return Ok(files);
    }

    for project_entry in std::fs::read_dir(projects_dir)? {
        let project_entry = project_entry?;
        let project_path = project_entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let dir_name = project_entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let project_name = decode_project_name(&dir_name);

        // Recursively find all .jsonl files under the project directory
        collect_jsonl_files(&project_path, &project_name, &mut files);
    }

    Ok(files)
}

fn collect_jsonl_files(dir: &Path, project_name: &str, files: &mut Vec<SessionFile>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, project_name, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            files.push(SessionFile {
                path,
                session_id,
                project: project_name.to_string(),
            });
        }
    }
}
