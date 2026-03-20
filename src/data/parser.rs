use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub uuid: String,
    pub session_id: String,
    pub msg_type: String, // "user", "assistant", "tool_result"
    pub timestamp: String,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub cost_usd: f64,
    pub project: String,
}

#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub id: String,
    pub project: String,
    pub started_at: String,
    pub updated_at: String,
    pub model: Option<String>,
    pub version: Option<String>,
}

// Raw JSONL structures for deserialization
#[derive(Debug, Deserialize)]
struct RawEntry {
    #[serde(default)]
    uuid: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    timestamp: Option<String>,
    message: Option<RawMessage>,
    version: Option<String>,
    #[serde(rename = "parentUuid")]
    parent_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    model: Option<String>,
    role: Option<String>,
    usage: Option<RawUsage>,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    cache_read_input_tokens: Option<i64>,
    cache_creation_input_tokens: Option<i64>,
    #[serde(default)]
    cache_creation: Option<CacheCreation>,
}

#[derive(Debug, Deserialize)]
struct CacheCreation {
    ephemeral_1h_input_tokens: Option<i64>,
    ephemeral_5m_input_tokens: Option<i64>,
}

/// Pricing per million tokens
fn cost_per_mtok(model: &str) -> (f64, f64, f64, f64) {
    // (input, output, cache_read, cache_creation)
    let model_lower = model.to_lowercase();
    if model_lower.contains("opus") {
        (15.0, 75.0, 1.50, 18.75)
    } else if model_lower.contains("haiku") {
        (0.80, 4.0, 0.08, 1.0)
    } else {
        // Default to Sonnet pricing (also covers explicit "sonnet" match)
        (3.0, 15.0, 0.30, 3.75)
    }
}

fn compute_cost(model: &str, input: i64, output: i64, cache_read: i64, cache_creation: i64) -> f64 {
    let (inp_rate, out_rate, cr_rate, cc_rate) = cost_per_mtok(model);
    let scale = 1_000_000.0;
    (input as f64 * inp_rate / scale)
        + (output as f64 * out_rate / scale)
        + (cache_read as f64 * cr_rate / scale)
        + (cache_creation as f64 * cc_rate / scale)
}

pub fn parse_jsonl_line(line: &str, project: &str) -> Option<(Option<ParsedSession>, Option<ParsedMessage>)> {
    let entry: RawEntry = serde_json::from_str(line).ok()?;

    let entry_type = entry.entry_type.as_deref()?;
    let uuid = entry.uuid.clone()?;
    let session_id = entry.session_id.clone().unwrap_or_default();
    let timestamp = entry.timestamp.clone().unwrap_or_default();

    match entry_type {
        "user" => {
            // First user message defines the session
            if entry.parent_uuid.is_none() {
                let session = ParsedSession {
                    id: session_id.clone(),
                    project: project.to_string(),
                    started_at: timestamp.clone(),
                    updated_at: timestamp.clone(),
                    model: None,
                    version: entry.version,
                };
                let msg = ParsedMessage {
                    uuid,
                    session_id,
                    msg_type: "user".to_string(),
                    timestamp,
                    model: None,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read: 0,
                    cache_creation: 0,
                    cost_usd: 0.0,
                    project: project.to_string(),
                };
                Some((Some(session), Some(msg)))
            } else {
                let msg = ParsedMessage {
                    uuid,
                    session_id,
                    msg_type: "user".to_string(),
                    timestamp,
                    model: None,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read: 0,
                    cache_creation: 0,
                    cost_usd: 0.0,
                    project: project.to_string(),
                };
                Some((None, Some(msg)))
            }
        }
        "assistant" => {
            let message = entry.message?;
            let usage = message.usage?;
            let model = message.model.unwrap_or_default();

            let input = usage.input_tokens.unwrap_or(0);
            let output = usage.output_tokens.unwrap_or(0);
            let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
            let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);

            let cost = compute_cost(&model, input, output, cache_read, cache_creation);

            let msg = ParsedMessage {
                uuid,
                session_id,
                msg_type: "assistant".to_string(),
                timestamp,
                model: Some(model),
                input_tokens: input,
                output_tokens: output,
                cache_read,
                cache_creation,
                cost_usd: cost,
                project: project.to_string(),
            };
            Some((None, Some(msg)))
        }
        _ => None,
    }
}

/// Parse all JSONL lines from file content (from a given offset), returning parsed data.
/// This is a pure function with no DB I/O, suitable for parallel execution.
pub fn parse_file_content(
    content: &[u8],
    offset: u64,
    project: &str,
) -> Vec<(Option<ParsedSession>, Option<ParsedMessage>)> {
    if (offset as usize) >= content.len() {
        return Vec::new();
    }

    let new_content = &content[offset as usize..];
    let text = String::from_utf8_lossy(new_content);
    let mut results = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(parsed) = parse_jsonl_line(line, project) {
            results.push(parsed);
        }
    }

    results
}

/// Decode project directory name to human-readable project name.
/// e.g., "-Users-saurabh-Dev-echopad" → "echopad"
pub fn decode_project_name(dir_name: &str) -> String {
    // URL-decode then take the last path component
    let decoded = urlencoding::decode(dir_name).unwrap_or_else(|_| dir_name.into());
    let path = decoded.replace('-', "/");
    path.rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or(dir_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user_line(uuid: &str, session_id: &str) -> String {
        format!(
            r#"{{"uuid":"{}","sessionId":"{}","type":"user","timestamp":"2025-01-01T00:00:00Z","message":{{"role":"user"}}}}"#,
            uuid, session_id
        )
    }

    fn make_assistant_line(uuid: &str, session_id: &str) -> String {
        format!(
            r#"{{"uuid":"{}","sessionId":"{}","type":"assistant","timestamp":"2025-01-01T00:01:00Z","message":{{"role":"assistant","model":"claude-3-5-sonnet-20241022","usage":{{"input_tokens":100,"output_tokens":200}}}}}}"#,
            uuid, session_id
        )
    }

    #[test]
    fn test_parse_file_content_empty() {
        let results = parse_file_content(b"", 0, "test");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_file_content_offset_past_end() {
        let content = b"some content";
        let results = parse_file_content(content, 100, "test");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_file_content_single_line() {
        let line = make_user_line("uuid1", "session1");
        let results = parse_file_content(line.as_bytes(), 0, "myproject");
        assert_eq!(results.len(), 1);
        let (session, msg) = &results[0];
        assert!(session.is_some());
        assert!(msg.is_some());
        assert_eq!(msg.as_ref().unwrap().session_id, "session1");
        assert_eq!(msg.as_ref().unwrap().project, "myproject");
    }

    #[test]
    fn test_parse_file_content_multiple_lines() {
        let content = format!(
            "{}\n{}\n",
            make_user_line("u1", "s1"),
            make_assistant_line("u2", "s1")
        );
        let results = parse_file_content(content.as_bytes(), 0, "proj");
        assert_eq!(results.len(), 2);
        // First line: user message with session
        assert!(results[0].0.is_some());
        // Second line: assistant message without session
        assert!(results[1].0.is_none());
        assert!(results[1].1.is_some());
        let msg = results[1].1.as_ref().unwrap();
        assert_eq!(msg.input_tokens, 100);
        assert_eq!(msg.output_tokens, 200);
        assert!(msg.cost_usd > 0.0);
    }

    #[test]
    fn test_parse_file_content_with_offset() {
        let line1 = make_user_line("u1", "s1");
        let line2 = make_assistant_line("u2", "s1");
        let content = format!("{}\n{}\n", line1, line2);
        let offset = (line1.len() + 1) as u64; // skip first line + newline
        let results = parse_file_content(content.as_bytes(), offset, "proj");
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_some());
        assert_eq!(results[0].1.as_ref().unwrap().msg_type, "assistant");
    }

    #[test]
    fn test_parse_file_content_skips_blank_lines() {
        let content = format!("\n\n{}\n\n", make_user_line("u1", "s1"));
        let results = parse_file_content(content.as_bytes(), 0, "proj");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parse_file_content_parallel_matches_sequential() {
        use rayon::prelude::*;

        // Create multiple "files" (byte slices)
        let files: Vec<(String, &str)> = (0..10)
            .map(|i| {
                let content = format!(
                    "{}\n{}\n",
                    make_user_line(&format!("u{i}a"), &format!("s{i}")),
                    make_assistant_line(&format!("u{i}b"), &format!("s{i}"))
                );
                (content, "project")
            })
            .collect();

        // Sequential parse
        let sequential: Vec<_> = files
            .iter()
            .flat_map(|(content, project)| parse_file_content(content.as_bytes(), 0, project))
            .collect();

        // Parallel parse
        let parallel: Vec<_> = files
            .par_iter()
            .flat_map(|(content, project)| parse_file_content(content.as_bytes(), 0, project))
            .collect();

        // Same number of results
        assert_eq!(sequential.len(), parallel.len());
        assert_eq!(sequential.len(), 20); // 2 per file * 10 files

        // Both should produce sessions for user messages and messages for all
        let seq_sessions: usize = sequential.iter().filter(|(s, _)| s.is_some()).count();
        let par_sessions: usize = parallel.iter().filter(|(s, _)| s.is_some()).count();
        assert_eq!(seq_sessions, par_sessions);
        assert_eq!(seq_sessions, 10); // one session per file
    }

    #[test]
    fn test_decode_project_name() {
        assert_eq!(decode_project_name("-Users-saurabh-Dev-echopad"), "echopad");
    }

    #[test]
    fn test_compute_cost_sonnet() {
        let cost = compute_cost("claude-3-5-sonnet-20241022", 1_000_000, 0, 0, 0);
        assert!((cost - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_cost_opus() {
        let cost = compute_cost("claude-3-opus", 1_000_000, 0, 0, 0);
        assert!((cost - 15.0).abs() < 0.01);
    }
}
