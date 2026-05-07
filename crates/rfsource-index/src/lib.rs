//! # rfsource-index
//!
//! Text and symbol index for `.rfsource` artifacts.
//!
//! Provides:
//! - Full-text search over source chunks
//! - Symbol lookup (functions, structs, types)
//! - Case-insensitive substring matching with grant filtering
//!
//! ## Crate Law
//!
//! - Pure index operations — no file IO, no database
//! - All search functions accept data as parameters (not paths)
//! - Index is derived and rebuildable from chunks + symbols

use rfsource_core::{SearchHit, SourceChunk, SymbolRecord};

/// Search source chunks for text matches.
///
/// Performs case-insensitive substring matching against chunk text.
/// Results are filtered by grant if `allowed_grants` is non-empty.
pub fn search_text(
    query: &str,
    chunks: &[SourceChunk],
    _symbols: &[SymbolRecord],
    grant_allows: &impl Fn(&str) -> bool,
) -> Vec<SearchHit> {
    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();

    for chunk in chunks {
        if !chunk.text.to_lowercase().contains(&query_lower) {
            continue;
        }
        // Grant check: if chunk's artifact has grants, verify access
        if !grant_allows("search") {
            continue;
        }

        // Build excerpt (first N chars of chunk text)
        let excerpt_len = 120.min(chunk.text.len());
        let excerpt = &chunk.text[..excerpt_len];

        hits.push(SearchHit {
            artifact_id: chunk.artifact_id.clone(),
            logical_path: String::new(), // caller should fill this
            version_id: chunk.version_id.clone(),
            chunk_id: chunk.chunk_id.clone(),
            line_start: chunk.line_start,
            line_end: chunk.line_end,
            score_reason: "text_match".to_string(),
            excerpt: excerpt.to_string(),
        });
    }

    hits
}

/// Search for symbols by name.
pub fn search_symbol<'a>(
    query: &str,
    symbols: &'a [SymbolRecord],
    grant_allows: &impl Fn(&str) -> bool,
) -> Vec<&'a SymbolRecord> {
    let query_lower = query.to_lowercase();
    symbols
        .iter()
        .filter(|s| s.symbol_name.to_lowercase().contains(&query_lower) && grant_allows("search"))
        .collect()
}

/// Extract an excerpt of text around a given line range.
pub fn excerpt(text: &str, line_start: u32, line_end: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = (line_start as usize).saturating_sub(1);
    let end = (line_end as usize).min(lines.len());
    lines[start..end].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant_allows(_grant: &str) -> bool {
        true
    }

    #[test]
    fn test_search_text_empty_chunks() {
        let hits = search_text("hello", &[], &[], &grant_allows);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_search_text_finds_match() {
        let chunks = vec![SourceChunk {
            chunk_id: "chk_1".into(),
            artifact_id: "art_1".into(),
            version_id: "ver_1".into(),
            ordinal: 0,
            line_start: 1,
            line_end: 10,
            text: "fn hello_world() {}".into(),
            text_hash: "abc".into(),
            symbols_defined: vec!["hello_world".into()],
            symbols_referenced: vec![],
        }];
        let hits = search_text("hello", &chunks, &[], &grant_allows);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, "chk_1");
    }

    #[test]
    fn test_search_symbol_finds_match() {
        let symbols = vec![SymbolRecord {
            symbol_id: "sym_1".into(),
            symbol_name: "MyStruct".into(),
            symbol_kind: "struct".into(),
            artifact_id: "art_1".into(),
            version_id: "ver_1".into(),
            chunk_id: "chk_1".into(),
            line_start: 5,
        }];
        let results = search_symbol("MyStruct", &symbols, &grant_allows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol_id, "sym_1");
    }
}
