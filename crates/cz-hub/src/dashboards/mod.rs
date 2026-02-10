//! # Dashboards
//!
//! Customizable visualization layouts for monitoring streams and metrics.

use serde::{Deserialize, Serialize};

use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub layout: Vec<GridItem>,
    pub widgets: Vec<Widget>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridItem {
    pub i: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Widget {
    TimeSeries {
        id: String,
        title: String,
        query: String, // SQL-like
    },
    Value {
        id: String,
        title: String,
        stream: String,
        field: String,
        unit: Option<String>,
    },
    Table {
        id: String,
        title: String,
        query: String,
        columns: Vec<String>,
    },
    LogStream {
        id: String,
        title: String,
        stream: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDashboardRequest {
    pub layout: Vec<GridItem>,
    pub widgets: Vec<Widget>,
}

pub struct DashboardManager {
    dashboards: RwLock<Vec<Dashboard>>,
}

impl DashboardManager {
    pub fn new() -> Self {
        Self {
            dashboards: RwLock::new(Vec::new()),
        }
    }

    pub async fn list(&self) -> Vec<Dashboard> {
        self.dashboards.read().await.clone()
    }

    pub async fn create(&self, name: String, description: Option<String>) -> Dashboard {
        let now = chrono::Utc::now().to_rfc3339();
        let dashboard = Dashboard {
            id: format!("dash-{}", uuid::Uuid::new_v4().as_simple()),
            name,
            description,
            layout: Vec::new(),
            widgets: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        };
        self.dashboards.write().await.push(dashboard.clone());
        dashboard
    }

    pub async fn get(&self, id: &str) -> Option<Dashboard> {
        self.dashboards
            .read()
            .await
            .iter()
            .find(|d| d.id == id)
            .cloned()
    }

    pub async fn update(
        &self,
        id: &str,
        layout: Vec<GridItem>,
        widgets: Vec<Widget>,
    ) -> Result<Dashboard, String> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or("Dashboard not found")?;
        dashboard.layout = layout;
        dashboard.widgets = widgets;
        dashboard.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(dashboard.clone())
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let mut dashboards = self.dashboards.write().await;
        let idx = dashboards
            .iter()
            .position(|d| d.id == id)
            .ok_or("Dashboard not found")?;
        dashboards.remove(idx);
        Ok(())
    }
}
