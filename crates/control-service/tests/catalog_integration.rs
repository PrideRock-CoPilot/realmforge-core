// Integration tests for Phase 3 Catalogs acceptance.
//
// Validates TEST-CATALOG-001 against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{CatalogId, CatalogModuleType, CatalogScope, TenantId};
use control_service::CatalogService;

async fn ensure_tenant(store: &control_store::CoreStore, tenant: &TenantId) {
    sqlx::query(
        "INSERT INTO tenants (id, name, status) VALUES ($1, $2, 'active') \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant.as_str())
    .bind(format!("Tenant {}", tenant.as_str()))
    .execute(store.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn test_catalog_tenant_copy_records_provenance() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = TenantId::generate();
    ensure_tenant(&store, &tenant).await;

    let svc = CatalogService::new(store);
    let global_id = CatalogId::generate();
    let tenant_id = CatalogId::generate();
    let copied_by = common::test_actor();
    let tenant_scope = CatalogScope::Tenant(tenant.as_str().to_string());

    let global = svc
        .register_global_module(&global_id, "Login Module", CatalogModuleType::Module)
        .await
        .unwrap();

    let copied = svc
        .copy_module(
            &global_id,
            &tenant_id,
            "Tenant Login Module",
            Some(&global_id),
            &tenant_scope,
            &copied_by,
        )
        .await
        .unwrap();

    assert_eq!(global.id, global_id);
    assert_eq!(global.scope, CatalogScope::Global);
    assert_eq!(copied.id, tenant_id);
    assert_eq!(copied.name, "Tenant Login Module");
    assert_eq!(copied.scope, tenant_scope);
    assert_eq!(copied.parent_id, Some(global_id.clone()));
    assert_eq!(copied.module_type, CatalogModuleType::Module);

    let provenance = copied.provenance.expect("copied entry needs provenance");
    assert_eq!(provenance.source_scope, CatalogScope::Global);
    assert_eq!(provenance.source_id, global_id);
    assert_eq!(provenance.copied_by, copied_by);

    let fetched_provenance = svc.get_module_provenance(&tenant_id).await.unwrap();
    assert_eq!(fetched_provenance.source_id, provenance.source_id);
    assert_eq!(fetched_provenance.copied_by, provenance.copied_by);

    let tenant_modules = svc.list_modules(&tenant_scope).await.unwrap();
    assert!(tenant_modules.iter().any(|entry| entry.id == tenant_id));
}

#[tokio::test]
async fn test_catalog_tree_preserves_parent_child_links() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = TenantId::generate();
    ensure_tenant(&store, &tenant).await;

    let svc = CatalogService::new(store);
    let global_id = CatalogId::generate();
    let tenant_root_id = CatalogId::generate();
    let child_id = CatalogId::generate();
    let tenant_scope = CatalogScope::Tenant(tenant.as_str().to_string());

    svc.register_global_module(&global_id, "Root Module", CatalogModuleType::Module)
        .await
        .unwrap();

    svc.copy_module(
        &global_id,
        &tenant_root_id,
        "Tenant Root Module",
        None,
        &tenant_scope,
        &common::test_actor(),
    )
    .await
    .unwrap();

    svc.copy_module(
        &tenant_root_id,
        &child_id,
        "Tenant Child Module",
        Some(&tenant_root_id),
        &tenant_scope,
        &common::test_actor(),
    )
    .await
    .unwrap();

    let tree = svc.get_catalog_tree(&tenant_scope).await.unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].entry.id, tenant_root_id);
    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].entry.id, child_id);
}
