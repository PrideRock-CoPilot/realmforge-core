//! # rfsource-query
//!
//! Search and query APIs for agents and frontend.
//!
//! Provides high-level search operations that combine text search,
//! symbol lookup, and grant-scoped filtering.
//!
//! ## Crate Law
//!
//! - Query orchestration only — delegates to `rfsource-index` and `rfsource-catalog`
//! - No direct file IO — reads go through `rfsource-store`

use rfsource_catalog::ArtifactRegistry;
use rfsource_core::{SearchHit, SourceChunk, SymbolRecord};

/// Query configuration for artifact searches.
pub struct ArtifactQuery {
    pub text_search: Option<String>,
    pub symbol_search: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

/// Combined search results.
pub struct SearchResults {
    pub text_hits: Vec<SearchHit>,
    pub symbol_hits: Vec<String>,
    pub total_count: usize,
}

/// Search artifacts by text content within grant scope.
pub fn search_artifacts(
    query: &ArtifactQuery,
    chunks: &[SourceChunk],
    symbols: &[SymbolRecord],
    registry: &ArtifactRegistry,
    actor_grant: &str,
) -> SearchResults {
    let grant_check = |_: &str| -> bool { true };

    let text_hits = if let Some(ref text_q) = query.text_search {
        let mut hits = rfsource_index::search_text(text_q, chunks, symbols, &grant_check);
        // Apply grant filtering
        hits.retain(|h| registry.grant_allows(&h.artifact_id, actor_grant));
        hits
    } else {
        vec![]
    };

    let symbol_hits = if let Some(ref sym_q) = query.symbol_search {
        rfsource_index::search_symbol(sym_q, symbols, &grant_check)
            .into_iter()
            .filter(|s| registry.grant_allows(&s.artifact_id, actor_grant))
            .map(|s| s.symbol_name.clone())
            .collect()
    } else {
        vec![]
    };

    let total_count = text_hits.len() + symbol_hits.len();
    let text_hits = text_hits
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .collect();

    SearchResults {
        text_hits,
        symbol_hits,
        total_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_search() {
        let reg = ArtifactRegistry::new();
        let results = search_artifacts(
            &ArtifactQuery {
                text_search: None,
                symbol_search: None,
                limit: 10,
                offset: 0,
            },
            &[],
            &[],
            &reg,
            "SGL-RFSOURCE-READ",
        );
        assert_eq!(results.total_count, 0);
    }
}
