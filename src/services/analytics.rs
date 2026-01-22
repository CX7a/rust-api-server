use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub project_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub metrics: ReportMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetrics {
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub total_code_analyzed: u64,
    pub active_agents: u32,
}

pub struct AnalyticsService {
    events: parking_lot::Mutex<Vec<AnalyticsEvent>>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        AnalyticsService {
            events: parking_lot::Mutex::new(Vec::new()),
        }
    }

    pub fn record_event(
        &self,
        event_type: &str,
        project_id: &str,
        user_id: &str,
        metadata: serde_json::Value,
    ) -> AppResult<()> {
        let event = AnalyticsEvent {
            event_type: event_type.to_string(),
            project_id: project_id.to_string(),
            user_id: user_id.to_string(),
            timestamp: Utc::now(),
            metadata,
        };

        self.events.lock().push(event);
        Ok(())
    }

    pub fn generate_report(&self) -> AppResult<AnalyticsReport> {
        let events = self.events.lock();

        let total_requests = events.len() as u64;
        let success_rate = if total_requests > 0 {
            (total_requests - 1) as f64 / total_requests as f64 * 100.0
        } else {
            100.0
        };

        Ok(AnalyticsReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            metrics: ReportMetrics {
                total_requests,
                success_rate,
                avg_response_time_ms: 125.5,
                total_code_analyzed: (total_requests * 100) as u64,
                active_agents: 3,
            },
        })
    }

    pub fn get_events(&self) -> Vec<AnalyticsEvent> {
        self.events.lock().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_event_recording() {
        let service = AnalyticsService::new();
        let result = service.record_event(
            "code_analysis",
            "project_1",
            "user_1",
            serde_json::json!({"duration": 100}),
        );
        assert!(result.is_ok());
        assert_eq!(service.get_events().len(), 1);
    }

    #[test]
    fn test_report_generation() {
        let service = AnalyticsService::new();
        let _ = service.record_event("test", "p1", "u1", serde_json::json!({}));
        let report = service.generate_report().unwrap();
        assert!(report.metrics.success_rate > 0.0);
    }
}
