use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value as JsonValue;
use tracing::{debug, info};

use crate::database::pool::DatabasePool;
use crate::jobs::error::{JobError, JobResult};
use crate::jobs::handlers::query::QueryJobPayload;
use crate::queries::project::find_project_dates;
use crate::queries::travel::find_travel_dates;
use crate::queries::types::{ProjectCriteria, TravelCriteria, WeddingCriteria};
use crate::queries::wedding::find_wedding_dates;

/// Type alias for query execution functions
///
/// This uses a type-erased future pattern to allow different query functions
/// with different return types to be stored in the same HashMap.
pub type QueryExecuteFn = Arc<
    dyn Fn(
            &DatabasePool,
            &QueryJobPayload,
        ) -> Pin<Box<dyn Future<Output = JobResult<JsonValue>> + Send>>
        + Send
        + Sync,
>;

/// A query template with metadata and execution function
#[derive(Clone)]
pub struct QueryTemplate {
    /// Name of the query (e.g., "wedding", "project", "travel")
    pub name: String,
    /// Human-readable description of the query
    pub description: String,
    /// Function to execute the query
    pub execute_fn: QueryExecuteFn,
}

impl std::fmt::Debug for QueryTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryTemplate")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

/// Registry for query templates
///
/// Maps query names to their execution functions. Supports dynamic registration
/// of new query types and provides a unified interface for executing queries.
#[derive(Clone)]
pub struct QueryTemplateRegistry {
    templates: HashMap<String, QueryTemplate>,
}

impl std::fmt::Debug for QueryTemplateRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryTemplateRegistry")
            .field("templates", &self.templates.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl QueryTemplateRegistry {
    /// Create a new registry with built-in templates
    ///
    /// Registers the following query templates:
    /// - "wedding": Finds auspicious wedding dates
    /// - "project": Placeholder for project start dates (to be implemented in 07-02)
    /// - "travel": Placeholder for travel dates (to be implemented in 07-02)
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };

        // Register wedding query template
        registry.register(
            "wedding",
            "Finds auspicious wedding dates based on Moon sign and Venus aspects",
            Arc::new(|pool: &DatabasePool, payload: &QueryJobPayload| {
                let start_date_str = payload.start_date.clone();
                let days = payload.days;
                let pool = pool.clone();
                Box::pin(async move {
                    debug!("Executing wedding query");

                    // Parse start_date
                    let start_date = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
                        .map_err(|e| JobError::Other(format!("Invalid start_date: {}", e)))?;

                    // Calculate end_date
                    let end_date = start_date + chrono::Duration::days(days - 1);

                    // Create wedding criteria with defaults
                    let criteria = WeddingCriteria::new(start_date, end_date);

                    // Execute the wedding query
                    let result = find_wedding_dates(&pool, &criteria)
                        .await
                        .map_err(|e| JobError::Other(format!("Query failed: {}", e)))?;

                    // Serialize the results to JSON
                    let json_result = serde_json::to_value(&result).map_err(JobError::from)?;

                    debug!("Wedding query completed with {} results", result.data.len());
                    Ok(json_result)
                })
            }),
        );

        // Register project query template
        registry.register(
            "project",
            "Finds favorable dates to start projects (Mercury direct, favorable Moon signs)",
            Arc::new(|pool: &DatabasePool, payload: &QueryJobPayload| {
                let start_date_str = payload.start_date.clone();
                let days = payload.days;
                let pool = pool.clone();
                Box::pin(async move {
                    debug!("Executing project query");

                    // Parse start_date
                    let start_date = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
                        .map_err(|e| JobError::Other(format!("Invalid start_date: {}", e)))?;

                    // Calculate end_date
                    let end_date = start_date + chrono::Duration::days(days - 1);

                    // Create project criteria with defaults
                    let criteria = ProjectCriteria::new(start_date, end_date);

                    // Execute the project query
                    let result = find_project_dates(&pool, &criteria)
                        .await
                        .map_err(|e| JobError::Other(format!("Query failed: {}", e)))?;

                    // Serialize the results to JSON
                    let json_result = serde_json::to_value(&result).map_err(JobError::from)?;

                    debug!("Project query completed with {} results", result.data.len());
                    Ok(json_result)
                })
            }),
        );

        // Register travel query template
        registry.register(
            "travel",
            "Finds favorable travel dates (Moon not VoC, Mercury direct)",
            Arc::new(|pool: &DatabasePool, payload: &QueryJobPayload| {
                let start_date_str = payload.start_date.clone();
                let days = payload.days;
                let pool = pool.clone();
                Box::pin(async move {
                    debug!("Executing travel query");

                    // Parse start_date
                    let start_date = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
                        .map_err(|e| JobError::Other(format!("Invalid start_date: {}", e)))?;

                    // Calculate end_date
                    let end_date = start_date + chrono::Duration::days(days - 1);

                    // Create travel criteria with defaults
                    let criteria = TravelCriteria::new(start_date, end_date);

                    // Execute the travel query
                    let result = find_travel_dates(&pool, &criteria)
                        .await
                        .map_err(|e| JobError::Other(format!("Query failed: {}", e)))?;

                    // Serialize the results to JSON
                    let json_result = serde_json::to_value(&result).map_err(JobError::from)?;

                    debug!("Travel query completed with {} results", result.data.len());
                    Ok(json_result)
                })
            }),
        );

        info!(
            templates = registry.templates.len(),
            "QueryTemplateRegistry initialized with built-in templates"
        );

        registry
    }

    /// Register a new query template
    ///
    /// # Arguments
    ///
    /// * `name` - The query name (used in API requests)
    /// * `description` - Human-readable description
    /// * `execute_fn` - The async function to execute the query
    pub fn register(&mut self, name: &str, description: &str, execute_fn: QueryExecuteFn) {
        let template = QueryTemplate {
            name: name.to_string(),
            description: description.to_string(),
            execute_fn,
        };

        self.templates.insert(name.to_string(), template);
        debug!("Registered query template: {}", name);
    }

    /// Get a query template by name
    ///
    /// Returns `Some(&QueryTemplate)` if found, `None` otherwise.
    pub fn get(&self, query_name: &str) -> Option<&QueryTemplate> {
        self.templates.get(query_name)
    }

    /// Execute a query by name
    ///
    /// Convenience method that looks up the template and executes it.
    /// Returns an error if the query name is not found.
    ///
    /// # Arguments
    ///
    /// * `query_name` - The name of the query to execute
    /// * `pool` - The database pool for query execution
    /// * `payload` - The job payload containing query parameters
    pub async fn execute(
        &self,
        query_name: &str,
        pool: &DatabasePool,
        payload: &QueryJobPayload,
    ) -> JobResult<JsonValue> {
        let template = self
            .get(query_name)
            .ok_or_else(|| JobError::Other(format!("Unknown query: {}", query_name)))?;

        debug!("Executing query template: {}", query_name);
        (template.execute_fn)(pool, payload).await
    }

    /// List all registered query names
    pub fn list_queries(&self) -> Vec<&str> {
        self.templates.keys().map(|k| k.as_str()).collect()
    }

    /// Check if a query is registered
    pub fn has_query(&self, query_name: &str) -> bool {
        self.templates.contains_key(query_name)
    }
}

impl Default for QueryTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation_has_all_templates() {
        let registry = QueryTemplateRegistry::new();

        // Check all 3 templates are registered
        assert!(registry.has_query("wedding"));
        assert!(registry.has_query("project"));
        assert!(registry.has_query("travel"));

        // Check list_queries returns all 3
        let queries = registry.list_queries();
        assert_eq!(queries.len(), 3);
    }

    #[test]
    fn test_get_returns_correct_template() {
        let registry = QueryTemplateRegistry::new();

        // Check wedding template
        let wedding = registry.get("wedding");
        assert!(wedding.is_some());
        let wedding = wedding.unwrap();
        assert_eq!(wedding.name, "wedding");
        assert!(!wedding.description.is_empty());

        // Check project template
        let project = registry.get("project");
        assert!(project.is_some());
        assert_eq!(project.unwrap().name, "project");

        // Check travel template
        let travel = registry.get("travel");
        assert!(travel.is_some());
        assert_eq!(travel.unwrap().name, "travel");
    }

    #[test]
    fn test_get_returns_none_for_unknown_query() {
        let registry = QueryTemplateRegistry::new();

        assert!(registry.get("unknown").is_none());
        assert!(registry.get("").is_none());
        assert!(registry.get("birthday").is_none());
    }

    #[test]
    fn test_has_query() {
        let registry = QueryTemplateRegistry::new();

        assert!(registry.has_query("wedding"));
        assert!(registry.has_query("project"));
        assert!(registry.has_query("travel"));
        assert!(!registry.has_query("unknown"));
        assert!(!registry.has_query(""));
    }

    #[test]
    fn test_list_queries() {
        let registry = QueryTemplateRegistry::new();
        let queries = registry.list_queries();

        assert_eq!(queries.len(), 3);
        assert!(queries.contains(&"wedding"));
        assert!(queries.contains(&"project"));
        assert!(queries.contains(&"travel"));
    }

    #[test]
    fn test_custom_registration() {
        let mut registry = QueryTemplateRegistry::new();

        // Register a custom query
        registry.register(
            "custom",
            "A custom query for testing",
            Arc::new(|_pool: &DatabasePool, _payload: &QueryJobPayload| {
                Box::pin(async move { Ok(serde_json::json!({"test": true})) })
            }),
        );

        // Verify custom query exists
        assert!(registry.has_query("custom"));
        let custom = registry.get("custom");
        assert!(custom.is_some());
        assert_eq!(custom.unwrap().description, "A custom query for testing");

        // Original queries still exist
        assert!(registry.has_query("wedding"));
    }

    #[test]
    fn test_registry_debug_format() {
        let registry = QueryTemplateRegistry::new();
        let debug_str = format!("{:?}", registry);

        // Should contain the struct name but not the execute_fn
        assert!(debug_str.contains("QueryTemplateRegistry"));
    }

    #[test]
    fn test_query_template_debug_format() {
        let registry = QueryTemplateRegistry::new();
        let wedding = registry.get("wedding").unwrap();
        let debug_str = format!("{:?}", wedding);

        // Should contain name and description but not execute_fn
        assert!(debug_str.contains("QueryTemplate"));
        assert!(debug_str.contains("wedding"));
        assert!(!debug_str.contains("execute_fn")); // Should be non_exhaustive
    }

    // Note: execute() tests would require a database pool
    // These are integration tests that should be in tests/ directory
}
