//! # Audit Service
//!
//! Centralises all audit log operations so handlers, background tasks, and
//! future integrations share a single, consistent API.
//!
//! ## Available Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `record` | Persist a single audit event (non-blocking via `tokio::spawn`) |
//! | `record_blocking` | Persist synchronously (awaitable) |
//! | `list` | Paginated, filtered list |
//! | `export` | Full list for CSV / PDF export (max 10 000 rows) |
//! | `get_by_user` | All events for a specific user |
//! | `get_by_resource` | Full change timeline for a resource |
//! | `get_recent` | Latest N events (activity feed / dashboard) |
//! | `count_by_action_since` | Event count by action within a time window |

use crate::dto::audit::{AuditAction, AuditLogEntry, AuditLogQuery};
use crate::errors::AppError;
use crate::queries::audit_log_queries::{
    self, count_by_action_since, export_audit_logs, get_logs_by_resource, get_logs_by_user,
    get_recent_audit_logs, list_audit_logs, InsertAuditLogParams,
};
use crate::services::pdf::generate_audit_logs_pdf;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AuditService;

impl AuditService {
    // ─── Write operations ────────────────────────────────────────────────────

    /// Persist an audit event in the background (fire-and-forget).
    /// Errors are logged but do **not** propagate to the caller.
    pub fn record(pool: PgPool, params: InsertAuditLogParams) {
        tokio::spawn(async move {
            if let Err(e) = audit_log_queries::insert_audit_log(&pool, params).await {
                tracing::error!("Failed to record audit log: {:?}", e);
            }
        });
    }

    /// Persist an audit event synchronously.
    /// Use this when you need to guarantee the log is written before continuing.
    pub async fn record_blocking(
        pool: &PgPool,
        params: InsertAuditLogParams,
    ) -> Result<(), AppError> {
        audit_log_queries::insert_audit_log(pool, params).await
    }

    // ─── Read operations ─────────────────────────────────────────────────────

    /// Paginated, optionally-filtered list of audit log entries.
    pub async fn list(
        pool: &PgPool,
        query: &AuditLogQuery,
    ) -> Result<(Vec<AuditLogEntry>, i64), AppError> {
        list_audit_logs(pool, query).await
    }

    /// Full list of matching audit log entries (no pagination, capped at 10 000)
    /// intended for CSV / PDF export.
    pub async fn export(
        pool: &PgPool,
        query: &AuditLogQuery,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        export_audit_logs(pool, query).await
    }

    /// All audit events belonging to a specific user, most recent first.
    pub async fn get_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        get_logs_by_user(pool, user_id, limit).await
    }

    /// Full chronological change timeline for a resource (e.g. a vehicle,
    /// a pending submission).
    pub async fn get_by_resource(
        pool: &PgPool,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        get_logs_by_resource(pool, resource_type, resource_id).await
    }

    /// Most recent N audit events — suitable for a live activity feed or
    /// a dashboard widget.
    pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<AuditLogEntry>, AppError> {
        get_recent_audit_logs(pool, limit).await
    }

    /// Count how many events of `action` occurred in the past `hours` hours.
    /// Useful for rate-limit checks, anomaly detection, and dashboard KPIs.
    pub async fn count_by_action_since(
        pool: &PgPool,
        action: AuditAction,
        hours: i64,
    ) -> Result<i64, AppError> {
        count_by_action_since(pool, action, hours).await
    }

    // ─── Export helpers ──────────────────────────────────────────────────────

    /// Build a CSV string from a list of audit log entries.
    /// Headers: ID, Timestamp, User ID, User Name, Action, Resource Type,
    ///          Resource ID, IP Address, Before Snapshot, After Snapshot
    pub fn build_csv(entries: &[AuditLogEntry]) -> String {
        let mut csv = String::from(
            "ID,Timestamp,User ID,User Name,Action,Resource Type,Resource ID,IP Address,Before Snapshot,After Snapshot\n",
        );
        for entry in entries {
            let before = entry
                .before_snapshot
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default()
                .replace('"', "\"\"");
            let after = entry
                .after_snapshot
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default()
                .replace('"', "\"\"");

            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},\"{}\",\"{}\"\n",
                entry.id,
                entry.created_at,
                entry.user_id.map(|u| u.to_string()).unwrap_or_default(),
                entry.user_name.as_deref().unwrap_or(""),
                entry.action.as_str(),
                entry.resource_type.as_deref().unwrap_or(""),
                entry.resource_id.map(|u| u.to_string()).unwrap_or_default(),
                entry.ip_address.as_deref().unwrap_or(""),
                before,
                after,
            ));
        }
        csv
    }

    /// Generate a PDF document from a list of audit log entries.
    pub async fn build_pdf(entries: Vec<AuditLogEntry>) -> Result<Vec<u8>, AppError> {
        generate_audit_logs_pdf(entries).await
    }
}
