use crate::{ActorId, CatalogId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The scope at which a catalog entry lives.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CatalogScope {
    Global,
    Tenant(String),
    App(String),
}

impl std::fmt::Display for CatalogScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Tenant(id) => write!(f, "tenant:{id}"),
            Self::App(id) => write!(f, "app:{id}"),
        }
    }
}

/// The module type classification for a catalog entry.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogModuleType {
    Module,
    Contract,
    Policy,
    Handler,
    Watch,
}

impl std::fmt::Display for CatalogModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module => write!(f, "module"),
            Self::Contract => write!(f, "contract"),
            Self::Policy => write!(f, "policy"),
            Self::Handler => write!(f, "handler"),
            Self::Watch => write!(f, "watch"),
        }
    }
}

/// Provenance metadata for a catalog copy operation (global → tenant → app).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogCopyProvenance {
    pub source_scope: CatalogScope,
    pub source_id: CatalogId,
    pub copied_at: DateTime<Utc>,
    pub copied_by: ActorId,
}

/// A single entry in the catalog hierarchy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: CatalogId,
    pub name: String,
    pub scope: CatalogScope,
    pub parent_id: Option<CatalogId>,
    pub module_type: CatalogModuleType,
    pub provenance: Option<CatalogCopyProvenance>,
    pub created_at: DateTime<Utc>,
}

/// A node in the catalog tree, with children populated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogNode {
    pub entry: CatalogEntry,
    pub children: Vec<CatalogNode>,
}

impl CatalogNode {
    /// Flatten the catalog tree into a list of entries (breadth-first).
    pub fn flatten(&self) -> Vec<&CatalogEntry> {
        let mut result = vec![&self.entry];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str, name: &str, scope: CatalogScope) -> CatalogEntry {
        CatalogEntry {
            id: CatalogId::new(id).unwrap(),
            name: name.to_string(),
            scope,
            parent_id: None,
            module_type: CatalogModuleType::Module,
            provenance: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn catalog_scope_display() {
        assert_eq!(CatalogScope::Global.to_string(), "global");
        assert_eq!(
            CatalogScope::Tenant("t1".to_string()).to_string(),
            "tenant:t1"
        );
        assert_eq!(CatalogScope::App("a1".to_string()).to_string(), "app:a1");
    }

    #[test]
    fn catalog_module_type_display() {
        assert_eq!(CatalogModuleType::Module.to_string(), "module");
        assert_eq!(CatalogModuleType::Contract.to_string(), "contract");
        assert_eq!(CatalogModuleType::Policy.to_string(), "policy");
        assert_eq!(CatalogModuleType::Handler.to_string(), "handler");
        assert_eq!(CatalogModuleType::Watch.to_string(), "watch");
    }

    #[test]
    fn catalog_node_flatten() {
        let root = CatalogNode {
            entry: sample_entry("root", "Root", CatalogScope::Global),
            children: vec![CatalogNode {
                entry: sample_entry("child1", "Child 1", CatalogScope::Tenant("t1".to_string())),
                children: vec![CatalogNode {
                    entry: sample_entry(
                        "grandchild",
                        "Grandchild",
                        CatalogScope::App("a1".to_string()),
                    ),
                    children: vec![],
                }],
            }],
        };
        let flat = root.flatten();
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].id.as_str(), "root");
        assert_eq!(flat[1].id.as_str(), "child1");
        assert_eq!(flat[2].id.as_str(), "grandchild");
    }

    #[test]
    fn catalog_entry_serde_roundtrip() {
        let entry = CatalogEntry {
            id: CatalogId::new("cat-1").unwrap(),
            name: "Test Module".to_string(),
            scope: CatalogScope::Global,
            parent_id: None,
            module_type: CatalogModuleType::Policy,
            provenance: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: CatalogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, entry.id);
        assert_eq!(restored.name, entry.name);
        assert_eq!(restored.scope, entry.scope);
        assert_eq!(restored.module_type, entry.module_type);
    }
}
