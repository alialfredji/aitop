use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

/// Top-level stats for the dashboard.
#[derive(Debug, Clone, Default)]
pub struct DashboardStats {
    pub burn_rate_per_hour: f64,
    pub spend_today: f64,
    pub spend_this_week: f64,
    pub spend_all_time: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read: i64,
    pub total_messages: i64,
    pub total_sessions: i64,
}

/// Per-model breakdown.
#[derive(Debug, Clone)]
pub struct ModelStats {
    pub model: String,
    pub cost: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read: i64,
    pub cache_creation: i64,
    pub call_count: i64,
}

/// Session summary for the sessions list.
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub project: String,
    pub model: String,
    pub total_cost: f64,
    pub total_tokens: i64,
    pub msg_count: i64,
    pub started_at: String,
    pub updated_at: String,
}

/// Daily spend data point.
#[derive(Debug, Clone)]
pub struct DailySpend {
    pub date: String,
    pub cost: f64,
}

/// Hourly token flow data point (for sparkline).
#[derive(Debug, Clone)]
pub struct TokenFlowPoint {
    pub minute: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
}

/// Recent activity entry.
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub timestamp: String,
    pub project: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read: i64,
    pub cost_usd: f64,
}

pub struct Aggregator {
    conn: Connection,
}

impl Aggregator {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Ok(Aggregator { conn })
    }

    pub fn dashboard_stats(&self) -> Result<DashboardStats> {
        let mut stats = DashboardStats::default();

        // All-time totals
        self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0), COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read), 0),
                    COUNT(*)
             FROM messages",
            [],
            |row| {
                stats.spend_all_time = row.get(0)?;
                stats.total_input_tokens = row.get(1)?;
                stats.total_output_tokens = row.get(2)?;
                stats.total_cache_read = row.get(3)?;
                stats.total_messages = row.get(4)?;
                Ok(())
            },
        )?;

        // Total sessions
        stats.total_sessions = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions",
            [],
            |row| row.get(0),
        )?;

        // Today's spend
        stats.spend_today = self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM messages WHERE date(timestamp) = date('now')",
            [],
            |row| row.get(0),
        )?;

        // This week's spend (last 7 days)
        stats.spend_this_week = self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM messages WHERE timestamp > datetime('now', '-7 days')",
            [],
            |row| row.get(0),
        )?;

        // Burn rate: cost in last hour extrapolated to $/hr
        stats.burn_rate_per_hour = self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM messages WHERE timestamp > datetime('now', '-1 hour')",
            [],
            |row| row.get(0),
        )?;

        Ok(stats)
    }

    pub fn model_breakdown(&self) -> Result<Vec<ModelStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(model, 'unknown'), SUM(cost_usd), SUM(input_tokens),
                    SUM(output_tokens), SUM(cache_read), SUM(cache_creation), COUNT(*)
             FROM messages
             WHERE model IS NOT NULL AND model != ''
             GROUP BY model
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ModelStats {
                model: row.get(0)?,
                cost: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                cache_read: row.get(4)?,
                cache_creation: row.get(5)?,
                call_count: row.get(6)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn sessions_list(&self, limit: usize) -> Result<Vec<SessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.project, COALESCE(s.model, 'unknown'), s.started_at, s.updated_at,
                    COALESCE(SUM(m.cost_usd), 0),
                    COALESCE(SUM(m.input_tokens + m.output_tokens), 0),
                    COUNT(m.id)
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             GROUP BY s.id
             ORDER BY s.updated_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                project: row.get(1)?,
                model: row.get(2)?,
                started_at: row.get(3)?,
                updated_at: row.get(4)?,
                total_cost: row.get(5)?,
                total_tokens: row.get(6)?,
                msg_count: row.get(7)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn daily_spend(&self, days: i32) -> Result<Vec<DailySpend>> {
        let mut stmt = self.conn.prepare(
            "SELECT date(timestamp) as day, SUM(cost_usd)
             FROM messages
             WHERE timestamp > datetime('now', ?1)
             GROUP BY day
             ORDER BY day",
        )?;

        let range = format!("-{} days", days);
        let rows = stmt.query_map(params![range], |row| {
            Ok(DailySpend {
                date: row.get(0)?,
                cost: row.get(1)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn token_flow_last_hour(&self) -> Result<Vec<TokenFlowPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT strftime('%H:%M', timestamp) as minute,
                    SUM(input_tokens), SUM(output_tokens),
                    SUM(input_tokens + output_tokens)
             FROM messages
             WHERE timestamp > datetime('now', '-1 hour')
             GROUP BY minute
             ORDER BY minute",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TokenFlowPoint {
                minute: row.get(0)?,
                input_tokens: row.get(1)?,
                output_tokens: row.get(2)?,
                total_tokens: row.get(3)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.timestamp, s.project, COALESCE(m.model, 'unknown'),
                    m.input_tokens, m.output_tokens, m.cache_read, m.cost_usd
             FROM messages m
             JOIN sessions s ON m.session_id = s.id
             WHERE m.type = 'assistant' AND m.model IS NOT NULL
             ORDER BY m.timestamp DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ActivityEntry {
                timestamp: row.get(0)?,
                project: row.get(1)?,
                model: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                cache_read: row.get(5)?,
                cost_usd: row.get(6)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn cache_hit_ratio(&self) -> Result<f64> {
        let result: (i64, i64) = self.conn.query_row(
            "SELECT COALESCE(SUM(cache_read), 0), COALESCE(SUM(input_tokens + cache_read + cache_creation), 0)
             FROM messages",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if result.1 == 0 {
            Ok(0.0)
        } else {
            Ok(result.0 as f64 / result.1 as f64)
        }
    }
}
