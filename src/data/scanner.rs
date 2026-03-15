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

        // Look for .jsonl files directly in the project directory
        for file_entry in std::fs::read_dir(&project_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();

            if file_path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let session_id = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                files.push(SessionFile {
                    path: file_path,
                    session_id,
                    project: project_name.clone(),
                });
            }
        }
    }

    Ok(files)
}
