use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;

use super::parser::{ParsedMessage, ParsedSession};
use super::pricing::PricingRegistry;

#[derive(Debug, Deserialize)]
struct MessageData {
    #[serde(rename = "role", default)]
    role: String,
    #[serde(rename = "modelID", default)]
    model_id: String,
    #[serde(default)]
    tokens: Option<TokenData>,
}

#[derive(Debug, Deserialize)]
struct TokenData {
    #[serde(default)]
    input: i64,
    #[serde(default)]
    output: i64,
    #[serde(default)]
    cache: Option<CacheTokens>,
    #[serde(default)]
    reasoning: i64,
}

#[derive(Debug, Deserialize)]
struct CacheTokens {
    #[serde(default)]
    read: i64,
    #[serde(default)]
    write: i64,
}

fn ms_to_iso8601(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
        .with_timezone(&Utc)
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

pub fn parse_opencode_db(
    db_path: &Path,
    pricing: &PricingRegistry,
) -> Result<Vec<(ParsedSession, Vec<ParsedMessage>)>> {
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let mut stmt = conn.prepare(
        "SELECT
            s.id          AS session_id,
            s.time_created AS session_created_ms,
            s.time_updated AS session_updated_ms,
            COALESCE(NULLIF(p.name, ''), NULLIF(p.worktree, ''), s.id) AS project_name,
            m.id          AS message_id,
            m.time_created AS message_created_ms,
            m.data        AS message_data
         FROM session s
         LEFT JOIN project p ON p.id = s.project_id
         JOIN message m ON m.session_id = s.id
         ORDER BY s.id, m.time_created ASC",
    )?;

    struct Row {
        session_id: String,
        session_created_ms: i64,
        session_updated_ms: i64,
        project_name: String,
        message_id: String,
        message_created_ms: i64,
        message_data: String,
    }

    let rows: Vec<Row> = stmt
        .query_map([], |row| {
            Ok(Row {
                session_id: row.get(0)?,
                session_created_ms: row.get(1)?,
                session_updated_ms: row.get(2)?,
                project_name: row.get(3)?,
                message_id: row.get(4)?,
                message_created_ms: row.get(5)?,
                message_data: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut sessions: std::collections::HashMap<String, (ParsedSession, Vec<ParsedMessage>)> =
        std::collections::HashMap::new();

    for row in rows {
        let data: MessageData = match serde_json::from_str(&row.message_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if data.role.is_empty() {
            continue;
        }

        let session_entry = sessions
            .entry(row.session_id.clone())
            .or_insert_with(|| {
                let session = ParsedSession {
                    id: row.session_id.clone(),
                    project: row.project_name.clone(),
                    started_at: ms_to_iso8601(row.session_created_ms),
                    updated_at: ms_to_iso8601(row.session_updated_ms),
                    model: None,
                    version: None,
                    provider: "opencode".to_string(),
                };
                (session, Vec::new())
            });

        if data.role == "assistant" {
            let tokens = data.tokens.as_ref();
            let input_base = tokens.map(|t| t.input).unwrap_or(0);
            let output = tokens.map(|t| t.output).unwrap_or(0);
            let cache_read = tokens
                .and_then(|t| t.cache.as_ref())
                .map(|c| c.read)
                .unwrap_or(0);
            let cache_creation = tokens
                .and_then(|t| t.cache.as_ref())
                .map(|c| c.write)
                .unwrap_or(0);

            // Fold reasoning tokens into input: reasoning tokens are charged
            // at the same rate as input tokens in the Anthropic billing model.
            let reasoning = tokens.map(|t| t.reasoning).unwrap_or(0);
            let input = input_base + reasoning;

            let cost = pricing.compute_cost(
                &data.model_id,
                input,
                output,
                cache_read,
                cache_creation,
            );

            if input == 0 && output == 0 && cache_read == 0 && cache_creation == 0 {
                continue;
            }

            if session_entry.0.model.is_none() && !data.model_id.is_empty() {
                session_entry.0.model = Some(data.model_id.clone());
            }

            session_entry.1.push(ParsedMessage {
                uuid: row.message_id,
                session_id: row.session_id,
                msg_type: "assistant".to_string(),
                timestamp: ms_to_iso8601(row.message_created_ms),
                model: if data.model_id.is_empty() {
                    None
                } else {
                    Some(data.model_id)
                },
                input_tokens: input,
                output_tokens: output,
                cache_read,
                cache_creation,
                cost_usd: cost,
                project: row.project_name,
                provider: "opencode".to_string(),
            });
        } else {
            session_entry.1.push(ParsedMessage {
                uuid: row.message_id,
                session_id: row.session_id,
                msg_type: "user".to_string(),
                timestamp: ms_to_iso8601(row.message_created_ms),
                model: None,
                input_tokens: 0,
                output_tokens: 0,
                cache_read: 0,
                cache_creation: 0,
                cost_usd: 0.0,
                project: row.project_name,
                provider: "opencode".to_string(),
            });
        }
    }

    Ok(sessions.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_db(dir: &TempDir) -> (std::path::PathBuf, Connection) {
        let db_path = dir.path().join("opencode.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE project (id TEXT PRIMARY KEY, worktree TEXT, name TEXT);
             CREATE TABLE session (id TEXT PRIMARY KEY, project_id TEXT, title TEXT,
                                   directory TEXT, time_created INTEGER, time_updated INTEGER);
             CREATE TABLE message (id TEXT PRIMARY KEY, session_id TEXT,
                                   time_created INTEGER, data TEXT);",
        )
        .unwrap();
        (db_path, conn)
    }

    fn insert_session(conn: &Connection, id: &str, project_id: &str, created: i64, updated: i64) {
        conn.execute(
            "INSERT INTO session (id, project_id, time_created, time_updated) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, project_id, created, updated],
        )
        .unwrap();
    }

    fn insert_project(conn: &Connection, id: &str, name: &str, worktree: &str) {
        conn.execute(
            "INSERT INTO project (id, name, worktree) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, worktree],
        )
        .unwrap();
    }

    fn insert_message(conn: &Connection, id: &str, session_id: &str, created: i64, data: &str) {
        conn.execute(
            "INSERT INTO message (id, session_id, time_created, data) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, session_id, created, data],
        )
        .unwrap();
    }

    #[test]
    fn test_empty_db_returns_empty() {
        let dir = TempDir::new().unwrap();
        let (db_path, _conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();
        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_nonexistent_db_returns_empty() {
        let pricing = PricingRegistry::builtin();
        let result =
            parse_opencode_db(Path::new("/nonexistent/path/opencode.db"), &pricing).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_assistant_message_parsed() {
        let dir = TempDir::new().unwrap();
        let (db_path, conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();

        insert_project(&conn, "proj1", "MyProject", "/home/user/myproject");
        insert_session(&conn, "sess1", "proj1", 1_700_000_000_000, 1_700_000_001_000);
        insert_message(
            &conn,
            "msg1",
            "sess1",
            1_700_000_000_500,
            r#"{"role":"assistant","modelID":"claude-sonnet-4.6","tokens":{"input":1000,"output":500,"cache":{"read":200,"write":100},"reasoning":0}}"#,
        );

        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        assert_eq!(result.len(), 1);

        let (session, messages) = &result[0];
        assert_eq!(session.id, "sess1");
        assert_eq!(session.project, "MyProject");
        assert_eq!(session.provider, "opencode");
        assert_eq!(session.model, Some("claude-sonnet-4.6".to_string()));

        assert_eq!(messages.len(), 1);
        let msg = &messages[0];
        assert_eq!(msg.uuid, "msg1");
        assert_eq!(msg.msg_type, "assistant");
        assert_eq!(msg.input_tokens, 1000);
        assert_eq!(msg.output_tokens, 500);
        assert_eq!(msg.cache_read, 200);
        assert_eq!(msg.cache_creation, 100);
        assert_eq!(msg.provider, "opencode");
        assert!(msg.cost_usd > 0.0);
    }

    #[test]
    fn test_reasoning_tokens_folded_into_input() {
        let dir = TempDir::new().unwrap();
        let (db_path, conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();

        insert_session(&conn, "sess1", "proj1", 1_700_000_000_000, 1_700_000_001_000);
        insert_message(
            &conn,
            "msg1",
            "sess1",
            1_700_000_000_500,
            r#"{"role":"assistant","modelID":"claude-sonnet-4.6","tokens":{"input":500,"output":200,"reasoning":300}}"#,
        );

        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        let (_, messages) = &result[0];
        let msg = &messages[0];
        assert_eq!(msg.input_tokens, 800);
    }

    #[test]
    fn test_user_message_has_zero_tokens() {
        let dir = TempDir::new().unwrap();
        let (db_path, conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();

        insert_session(&conn, "sess1", "proj1", 1_700_000_000_000, 1_700_000_001_000);
        insert_message(
            &conn,
            "msg1",
            "sess1",
            1_700_000_000_100,
            r#"{"role":"user","modelID":""}"#,
        );

        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        let (_, messages) = &result[0];
        let msg = &messages[0];
        assert_eq!(msg.msg_type, "user");
        assert_eq!(msg.input_tokens, 0);
        assert_eq!(msg.output_tokens, 0);
        assert_eq!(msg.cost_usd, 0.0);
    }

    #[test]
    fn test_zero_token_assistant_messages_skipped() {
        let dir = TempDir::new().unwrap();
        let (db_path, conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();

        insert_session(&conn, "sess1", "proj1", 1_700_000_000_000, 1_700_000_001_000);
        insert_message(
            &conn,
            "msg1",
            "sess1",
            1_700_000_000_100,
            r#"{"role":"assistant","modelID":"claude-sonnet-4.6","tokens":{"input":0,"output":0}}"#,
        );

        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        let (_, messages) = &result[0];
        assert!(messages.is_empty());
    }

    #[test]
    fn test_timestamp_conversion() {
        let ts = ms_to_iso8601(1_700_000_000_000);
        assert!(ts.starts_with("2023-11-14T"));
        assert!(ts.ends_with('Z'));
    }

    #[test]
    fn test_project_name_from_worktree_fallback() {
        let dir = TempDir::new().unwrap();
        let (db_path, conn) = setup_db(&dir);
        let pricing = PricingRegistry::builtin();

        insert_project(&conn, "proj1", "", "/home/user/worktree-project");
        insert_session(&conn, "sess1", "proj1", 1_700_000_000_000, 1_700_000_001_000);
        insert_message(
            &conn,
            "msg1",
            "sess1",
            1_700_000_000_100,
            r#"{"role":"user","modelID":""}"#,
        );

        let result = parse_opencode_db(&db_path, &pricing).unwrap();
        let (session, _) = &result[0];
        assert_eq!(session.project, "/home/user/worktree-project");
    }
}
