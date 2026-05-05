use authority_domain::{
    CatalogCopyProvenance, CatalogEntry, CatalogId, CatalogModuleType, CatalogScope,
};
use serde_json::Value;
use sqlx::PgPool;
use tracing::instrument;

use crate::StoreError;

/// Insert a new catalog entry into the database.
#[instrument(skip(pool))]
pub async fn insert_catalog_entry(
    pool: &PgPool,
    entry: &CatalogEntry,
) -> Result<(), StoreError> {
    let (scope_str, scope_tenant_id, scope_app_id) = match &entry.scope {
        CatalogScope::Global => ("global", None::<String>, None::<String>),
        CatalogScope::Tenant(t) => ("tenant", Some(t.clone()), None),
        CatalogScope::App(a) => ("app", None, Some(a.clone())),
    };

    let provenance_json: Option<Value> = entry
        .provenance
        .as_ref()
        .map(|p| serde_json::to_value(p).expect("provenance serialization"));

    sqlx::query(
        r#"
        INSERT INTO catalog_entries (id, name, scope, scope_tenant_id, scope_app_id, parent_id, module_type, provenance_json)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(entry.id.as_str())
    .bind(&entry.name)
    .bind(scope_str)
    .bind(scope_tenant_id)
    .bind(scope_app_id)
    .bind(entry.parent_id.as_ref().map(|id| id.as_str()))
    .bind(entry.module_type.to_string())
    .bind(provenance_json)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get a single catalog entry by ID.
#[instrument(skip(pool))]
pub async fn get_catalog_entry(
    pool: &PgPool,
    id: &CatalogId,
) -> Result<Option<CatalogEntry>, StoreError> {
    let row = sqlx::query_as::<_, CatalogEntryRow>(
        r#"
        SELECT id, name, scope, scope_tenant_id, scope_app_id, parent_id, module_type, provenance_json, created_at
        FROM catalog_entries
        WHERE id = $1
        "#,
    )
    .bind(id.as_str())
    .fetch_optional(pool)
    .await?;

    row.map(|r| r.into_entry()).transpose()
}

/// List catalog entries matching a given scope.
#[instrument(skip(pool))]
pub async fn list_catalog_entries(
    pool: &PgPool,
    scope: &CatalogScope,
) -> Result<Vec<CatalogEntry>, StoreError> {
    let (scope_str, scope_tenant_id) = match scope {
        CatalogScope::Global => ("global", None::<String>),
        CatalogScope::Tenant(t) => ("tenant", Some(t.clone())),
        CatalogScope::App(a) => ("app", Some(a.clone())),
    };

    let rows = sqlx::query_as::<_, CatalogEntryRow>(
        r#"
        SELECT id, name, scope, scope_tenant_id, scope_app_id, parent_id, module_type, provenance_json, created_at
        FROM catalog_entries
        WHERE scope = $1 AND ($2::TEXT IS NULL OR scope_tenant_id = $2 OR scope_app_id = $2)
        ORDER BY name
        "#,
    )
    .bind(scope_str)
    .bind(scope_tenant_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| r.into_entry())
        .collect::<Result<Vec<_>, _>>()
}

/// Copy a catalog entry with provenance tracking.
#[instrument(skip(pool))]
pub async fn copy_catalog_entry(
    pool: &PgPool,
    source_id: &CatalogId,
    new_id: &CatalogId,
    new_name: &str,
    new_parent_id: Option<&CatalogId>,
    new_scope: &CatalogScope,
    copied_by: &authority_domain::ActorId,
) -> Result<CatalogEntry, StoreError> {
    // Fetch source entry
    let source = get_catalog_entry(pool, source_id)
        .await?
        .ok_or_else(|| StoreError::invalid_data(format!("catalog entry {source_id} not found")))?;

    let provenance = CatalogCopyProvenance {
        source_scope: source.scope.clone(),
        source_id: source_id.clone(),
        copied_at: chrono::Utc::now(),
        copied_by: copied_by.clone(),
    };

    let new_entry = CatalogEntry {
        id: new_id.clone(),
        name: new_name.to_string(),
        scope: new_scope.clone(),
        parent_id: new_parent_id.cloned(),
        module_type: source.module_type,
        provenance: Some(provenance),
        created_at: chrono::Utc::now(),
    };

    insert_catalog_entry(pool, &new_entry).await?;
    Ok(new_entry)
}

// ── Database row helper types ──

#[derive(Debug, sqlx::FromRow)]
struct CatalogEntryRow {
    id: String,
    name: String,
    scope: String,
    scope_tenant_id: Option<String>,
    scope_app_id: Option<String>,
    parent_id: Option<String>,
    module_type: String,
    provenance_json: Option<Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl CatalogEntryRow {
    fn into_entry(self) -> Result<CatalogEntry, StoreError> {
        let scope = match self.scope.as_str() {
            "global" => CatalogScope::Global,
            "tenant" => CatalogScope::Tenant(
                self.scope_tenant_id
                    .ok_or_else(|| StoreError::invalid_data("missing scope_tenant_id"))?,
            ),
            "app" => CatalogScope::App(
                self.scope_app_id
                    .ok_or_else(|| StoreError::invalid_data("missing scope_app_id"))?,
            ),
            other => return Err(StoreError::invalid_data(format!("invalid scope: {other}"))),
        };

        let module_type = match self.module_type.as_str() {
            "module" => CatalogModuleType::Module,
            "contract" => CatalogModuleType::Contract,
            "policy" => CatalogModuleType::Policy,
            "handler" => CatalogModuleType::Handler,
            "watch" => CatalogModuleType::Watch,
            other => return Err(StoreError::invalid_data(format!("invalid module_type: {other}"))),
        };

        let provenance = self
            .provenance_json
            .map(|v| serde_json::from_value(v).map_err(|e| StoreError::invalid_data(e.to_string())))
            .transpose()?;

        Ok(CatalogEntry {
            id: CatalogId::new(self.id).map_err(|e| StoreError::invalid_data(e.to_string()))?,
            name: self.name,
            scope,
            parent_id: self
                .parent_id
                .map(|id| CatalogId::new(id).map_err(|e| StoreError::invalid_data(e.to_string())))
                .transpose()?,
            module_type,
            provenance,
            created_at: self.created_at,
        })
    }
}
