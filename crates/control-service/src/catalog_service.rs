use authority_domain::{
    ActorId, CatalogCopyProvenance, CatalogEntry, CatalogId, CatalogModuleType, CatalogNode,
    CatalogScope,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Catalog service for managing global → tenant → app module hierarchy.
#[derive(Clone)]
pub struct CatalogService {
    store: CoreStore,
}

impl CatalogService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Register a new global module in the catalog.
    #[instrument(skip(self), fields(catalog_id = %id, name = %name))]
    pub async fn register_global_module(
        &self,
        id: &CatalogId,
        name: &str,
        module_type: CatalogModuleType,
    ) -> Result<CatalogEntry, ServiceError> {
        let entry = CatalogEntry {
            id: id.clone(),
            name: name.to_string(),
            scope: CatalogScope::Global,
            parent_id: None,
            module_type,
            provenance: None,
            created_at: Utc::now(),
        };
        self.store.insert_catalog_entry(&entry).await?;
        info!("global catalog module registered");
        Ok(entry)
    }

    /// Copy a module from a source scope to a target scope with provenance.
    #[instrument(skip(self), fields(source = %source_id, target_scope = %new_scope))]
    pub async fn copy_module(
        &self,
        source_id: &CatalogId,
        new_id: &CatalogId,
        new_name: &str,
        new_parent_id: Option<&CatalogId>,
        new_scope: &CatalogScope,
        copied_by: &ActorId,
    ) -> Result<CatalogEntry, ServiceError> {
        let entry = self
            .store
            .copy_catalog_entry(
                source_id,
                new_id,
                new_name,
                new_parent_id,
                new_scope,
                copied_by,
            )
            .await?;
        info!("catalog module copied with provenance");
        Ok(entry)
    }

    /// List all modules at a given scope.
    #[instrument(skip(self), fields(scope = %scope))]
    pub async fn list_modules(
        &self,
        scope: &CatalogScope,
    ) -> Result<Vec<CatalogEntry>, ServiceError> {
        let entries = self.store.list_catalog_entries(scope).await?;
        Ok(entries)
    }

    /// Get a single catalog entry by ID.
    #[instrument(skip(self), fields(id = %id))]
    pub async fn get_module(&self, id: &CatalogId) -> Result<CatalogEntry, ServiceError> {
        self.store
            .get_catalog_entry(id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("catalog entry {id} not found")))
    }

    /// Get the provenance chain for a catalog entry.
    #[instrument(skip(self), fields(id = %id))]
    pub async fn get_module_provenance(
        &self,
        id: &CatalogId,
    ) -> Result<CatalogCopyProvenance, ServiceError> {
        let entry = self.get_module(id).await?;
        entry.provenance.ok_or_else(|| {
            ServiceError::Validation(format!("catalog entry {id} has no provenance"))
        })
    }

    /// Build a catalog tree rooted at a given scope (returns all entries in tree form).
    #[instrument(skip(self))]
    pub async fn get_catalog_tree(
        &self,
        scope: &CatalogScope,
    ) -> Result<Vec<CatalogNode>, ServiceError> {
        let entries = self.store.list_catalog_entries(scope).await?;
        Ok(build_tree(&entries, None))
    }

    /// Traverse catalog hierarchy: returns entries at current scope and children.
    #[instrument(skip(self))]
    pub async fn traverse_hierarchy(
        &self,
        scope: &CatalogScope,
    ) -> Result<Vec<CatalogEntry>, ServiceError> {
        let entries = self.store.list_catalog_entries(scope).await?;
        Ok(entries)
    }
}

/// Build a tree from a flat list of entries, starting with a given parent.
fn build_tree(entries: &[CatalogEntry], parent_id: Option<&CatalogId>) -> Vec<CatalogNode> {
    entries
        .iter()
        .filter(|e| e.parent_id.as_ref() == parent_id)
        .map(|entry| {
            let children = build_tree(entries, Some(&entry.id));
            CatalogNode {
                entry: entry.clone(),
                children,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str, name: &str, parent_id: Option<&CatalogId>) -> CatalogEntry {
        CatalogEntry {
            id: CatalogId::new(id).unwrap(),
            name: name.to_string(),
            scope: CatalogScope::Global,
            parent_id: parent_id.cloned(),
            module_type: CatalogModuleType::Module,
            provenance: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn build_tree_flat_list_to_hierarchy() {
        let root = sample_entry("root", "Root", None);
        let child1 = sample_entry("c1", "Child 1", Some(&root.id));
        let child2 = sample_entry("c2", "Child 2", Some(&root.id));
        let grandchild = sample_entry("gc", "Grandchild", Some(&child1.id));

        let entries = vec![root, child1, child2, grandchild];
        let tree = build_tree(&entries, None);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].entry.name, "Root");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].entry.name, "Child 1");
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].entry.name, "Grandchild");
    }

    #[test]
    fn build_tree_empty_entries() {
        let tree = build_tree(&[], None);
        assert!(tree.is_empty());
    }
}
