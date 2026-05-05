// ─────────────────────────────────────────────
// plan_store.rs — Plan persistence trait + in-memory impl
// ─────────────────────────────────────────────
// Provides the PlanStore interface used by IntakeService.
// Ships with an in-memory implementation for testing
// and local development. PostgreSQL implementation TODO.
// ─────────────────────────────────────────────

use async_trait::async_trait;
use authority_domain::plan::Plan;
use std::collections::HashMap;
use std::sync::Mutex;

/// Storage interface for Plan objects.
/// All methods are async to support future PostgreSQL impl.
#[async_trait]
pub trait PlanStore: Send + Sync {
    /// Save a plan (insert or update by id).
    async fn save_plan(&self, plan: &Plan) -> Result<(), Box<dyn std::error::Error>>;

    /// Retrieve a plan by its id.
    async fn get_plan(&self, id: &str) -> Result<Plan, Box<dyn std::error::Error>>;

    /// List all plans, optionally filtered by pipeline stage.
    async fn list_plans(
        &self,
        stage: Option<&str>,
    ) -> Result<Vec<Plan>, Box<dyn std::error::Error>>;

    /// Delete a plan by id.
    async fn delete_plan(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// In-memory implementation of PlanStore.
/// Thread-safe via Mutex. Suitable for testing and single-instance dev.
pub struct InMemoryPlanStore {
    plans: Mutex<HashMap<String, Plan>>,
}

impl InMemoryPlanStore {
    pub fn new() -> Self {
        Self {
            plans: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPlanStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlanStore for InMemoryPlanStore {
    async fn save_plan(&self, plan: &Plan) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self
            .plans
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        store.insert(plan.id.clone(), plan.clone());
        Ok(())
    }

    async fn get_plan(&self, id: &str) -> Result<Plan, Box<dyn std::error::Error>> {
        let store = self
            .plans
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        store
            .get(id)
            .cloned()
            .ok_or_else::<Box<dyn std::error::Error>, _>(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Plan '{}' not found", id),
                ))
            })
    }

    async fn list_plans(
        &self,
        stage: Option<&str>,
    ) -> Result<Vec<Plan>, Box<dyn std::error::Error>> {
        let store = self
            .plans
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let plans: Vec<Plan> = store.values().cloned().collect();
        match stage {
            Some(stage_filter) => Ok(plans
                .into_iter()
                .filter(|p| p.current_stage.as_str() == stage_filter)
                .collect()),
            None => Ok(plans),
        }
    }

    async fn delete_plan(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self
            .plans
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        store
            .remove(id)
            .ok_or_else::<Box<dyn std::error::Error>, _>(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Plan '{}' not found", id),
                ))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::plan::*;
    use authority_domain::PlanStatus;
    use chrono::Utc;

    fn make_test_plan(id: &str) -> Plan {
        Plan {
            id: id.to_string(),
            name: "Test Plan".to_string(),
            goal: "Test goal".to_string(),
            scope: "Test scope".to_string(),
            constraints: vec![],
            assumptions: vec![],
            architecture_summary: String::new(),
            core_areas: vec![],
            decisions: vec![],
            risks: vec![],
            phases: vec![],
            work_packets: vec![],
            status: PlanStatus::Draft,
            current_stage: PipelineStage::Intake,
            next_action: "Refine".to_string(),
            owner: "alice".to_string(),
            audit_log: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_get() {
        let store = InMemoryPlanStore::new();
        let plan = make_test_plan("plan_001");
        store.save_plan(&plan).await.unwrap();
        let retrieved = store.get_plan("plan_001").await.unwrap();
        assert_eq!(retrieved.id, "plan_001");
        assert_eq!(retrieved.name, "Test Plan");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let store = InMemoryPlanStore::new();
        let result = store.get_plan("plan_nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_plans() {
        let store = InMemoryPlanStore::new();
        store.save_plan(&make_test_plan("plan_001")).await.unwrap();
        store.save_plan(&make_test_plan("plan_002")).await.unwrap();
        let all = store.list_plans(None).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryPlanStore::new();
        store.save_plan(&make_test_plan("plan_001")).await.unwrap();
        store.delete_plan("plan_001").await.unwrap();
        let result = store.get_plan("plan_001").await;
        assert!(result.is_err());
    }
}
