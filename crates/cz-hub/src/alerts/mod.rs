//! # Alerting Engine v2
//!
//! Rule-based alerting with incident lifecycle management and notification dispatch.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;

/// Incident status lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Open,
    Acknowledged,
    Resolved,
}

/// A single incident (triggered by an alert rule).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: String,
    pub status: IncidentStatus,
    pub message: String,
    pub timeline: Vec<TimelineEntry>,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub acknowledged_by: Option<String>,
}

/// A timeline entry attached to an incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: String,
    pub action: String,
    pub detail: String,
    pub actor: Option<String>,
}

/// Alert rule types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// Value exceeds threshold for N seconds
    Threshold,
    /// Rate of change exceeds percentage
    RateOfChange,
    /// Deviation from rolling average
    Anomaly,
    /// Pattern match on stream events
    Pattern,
}

/// Enhanced alert rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleV2 {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub stream: Option<String>,
    pub field: String,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: String,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
    pub runbook_url: Option<String>,
}

/// Notification channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String, // "webhook", "slack", "pagerduty"
    pub config: std::collections::HashMap<String, String>,
    pub enabled: bool,
}

/// The alert engine state.
pub struct AlertEngine {
    pub rules: RwLock<Vec<AlertRuleV2>>,
    pub incidents: RwLock<Vec<Incident>>,
    pub channels: RwLock<Vec<NotificationChannel>>,
    pub incident_history: RwLock<VecDeque<Incident>>,
    history_capacity: usize,
}

impl AlertEngine {
    pub fn new(history_capacity: usize) -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            incidents: RwLock::new(Vec::new()),
            channels: RwLock::new(Vec::new()),
            incident_history: RwLock::new(VecDeque::with_capacity(history_capacity)),
            history_capacity,
        }
    }

    /// Create a new incident from an alert rule trigger.
    pub async fn create_incident(&self, rule: &AlertRuleV2, message: String) -> Incident {
        let now = chrono::Utc::now().to_rfc3339();
        let incident = Incident {
            id: format!("inc-{}", uuid::Uuid::new_v4().as_simple()),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            severity: rule.severity.clone(),
            status: IncidentStatus::Open,
            message,
            timeline: vec![TimelineEntry {
                timestamp: now.clone(),
                action: "opened".into(),
                detail: format!("Alert rule '{}' triggered", rule.name),
                actor: Some("system".into()),
            }],
            created_at: now.clone(),
            updated_at: now,
            resolved_at: None,
            acknowledged_by: None,
        };

        let mut incidents = self.incidents.write().await;
        incidents.push(incident.clone());

        // Dispatch notifications
        self.dispatch_notification(&incident, &rule.notification_channels)
            .await;

        incident
    }

    /// Acknowledge an incident.
    pub async fn acknowledge_incident(
        &self,
        incident_id: &str,
        actor: &str,
    ) -> Result<Incident, String> {
        let mut incidents = self.incidents.write().await;
        let incident = incidents
            .iter_mut()
            .find(|i| i.id == incident_id)
            .ok_or_else(|| format!("Incident '{}' not found", incident_id))?;

        incident.status = IncidentStatus::Acknowledged;
        incident.acknowledged_by = Some(actor.to_string());
        let now = chrono::Utc::now().to_rfc3339();
        incident.updated_at = now.clone();
        incident.timeline.push(TimelineEntry {
            timestamp: now,
            action: "acknowledged".into(),
            detail: format!("Acknowledged by {}", actor),
            actor: Some(actor.to_string()),
        });

        Ok(incident.clone())
    }

    /// Resolve an incident.
    pub async fn resolve_incident(
        &self,
        incident_id: &str,
        actor: &str,
    ) -> Result<Incident, String> {
        let mut incidents = self.incidents.write().await;
        let idx = incidents
            .iter()
            .position(|i| i.id == incident_id)
            .ok_or_else(|| format!("Incident '{}' not found", incident_id))?;

        let incident = &mut incidents[idx];
        incident.status = IncidentStatus::Resolved;
        let now = chrono::Utc::now().to_rfc3339();
        incident.resolved_at = Some(now.clone());
        incident.updated_at = now.clone();
        incident.timeline.push(TimelineEntry {
            timestamp: now,
            action: "resolved".into(),
            detail: format!("Resolved by {}", actor),
            actor: Some(actor.to_string()),
        });

        let resolved = incident.clone();

        // Move to history
        let removed = incidents.remove(idx);
        let mut history = self.incident_history.write().await;
        if history.len() >= self.history_capacity {
            history.pop_front();
        }
        history.push_back(removed);

        Ok(resolved)
    }

    /// List active incidents.
    pub async fn list_active(&self) -> Vec<Incident> {
        self.incidents.read().await.clone()
    }

    async fn dispatch_notification(&self, incident: &Incident, channel_ids: &[String]) {
        let channels = self.channels.read().await;
        for ch_id in channel_ids {
            if let Some(ch) = channels.iter().find(|c| &c.id == ch_id && c.enabled) {
                match ch.channel_type.as_str() {
                    "webhook" => {
                        if let Some(url) = ch.config.get("url") {
                            tracing::info!(
                                "Dispatching webhook to {} for incident {}",
                                url,
                                incident.id
                            );
                            // TODO: actual HTTP POST
                        }
                    }
                    "slack" => {
                        tracing::info!(
                            "Dispatching Slack notification for incident {}",
                            incident.id
                        );
                        // TODO: Slack webhook POST
                    }
                    "pagerduty" => {
                        tracing::info!("Dispatching PagerDuty event for incident {}", incident.id);
                        // TODO: PagerDuty Events API v2
                    }
                    _ => {
                        tracing::warn!("Unknown notification channel type: {}", ch.channel_type);
                    }
                }
            }
        }
    }
}
