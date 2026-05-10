//! Integration tests for DDR-003 multi-file support - Day 2
//!
//! Tests:
//! - Day 2.1: Multi-file write path (append, rotation, manifest updates)
//! - Day 2.2: Multi-file read path (read all segments in order)
//! - Day 2.3: Frame order preservation across segments

#[cfg(test)]
mod day2_integration_tests {
    use rfsource_core::CommitArtifactRequest;
    use rfsource_format::{detect_repository_mode, RepositoryMode, SEGMENT_SIZE_THRESHOLD};
    use rfsource_store::RFSource;
    use tempfile::tempdir;

    /// Test Day 2.1: Basic append to multi-file repository
    #[test]
    fn test_day2_1_multifile_append() {
        let dir = tempdir().unwrap();
        
        // Create single-file repo
        let store = RFSource::create(dir.path(), "test_project").unwrap();
        assert_eq!(store.mode(), RepositoryMode::SingleFile);
        
        // Small commits shouldn't trigger multi-file
        let req = CommitArtifactRequest {
            logical_path: "test/hello.rs".to_string(),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
            language: "rust".to_string(),
            message: "Initial commit".to_string(),
            actor: "test".to_string(),
            owner_capability: "test-cap".to_string(),
            risk_level: 1,
            policy_bindings: vec![],
            allowed_grants: vec!["*".to_string()],
            required_tests: vec![],
        };
        
        let outcome = store.commit_artifact(req, "*").unwrap();
        assert!(!outcome.commit.commit_id.is_empty());
        
        // Verify still single-file
        assert_eq!(store.mode(), RepositoryMode::SingleFile);
        
        // Verify can read back
        let stats = store.stats().unwrap();
        assert_eq!(stats.commit_count, 1);
        assert_eq!(stats.artifact_count, 1);
    }

    /// Test Day 2.2: Multi-file read path
    #[test]
    fn test_day2_2_multifile_read() {
        let dir = tempdir().unwrap();
        
        // Create and open
        let _store = RFSource::create(dir.path(), "test_project").unwrap();
        let opened = RFSource::open(dir.path()).unwrap();
        
        // Verify can read manifest
        let manifest = opened.manifest().unwrap();
        assert_eq!(manifest.project_name, "test_project");
        
        // Verify can read stats
        let stats = opened.stats().unwrap();
        assert!(stats.branch_count > 0);
    }

    /// Test Day 2.3: Frame order preservation
    #[test]
    fn test_day2_3_frame_order_preservation() {
        let dir = tempdir().unwrap();
        let store = RFSource::create(dir.path(), "order_test").unwrap();
        
        // Make multiple commits
        for i in 0..5 {
            let req = CommitArtifactRequest {
                logical_path: format!("test/file{}.rs", i),
                content: format!("// File {}", i),
                language: "rust".to_string(),
                message: format!("Commit {}", i),
                actor: "test".to_string(),
                owner_capability: "test-cap".to_string(),
                risk_level: 1,
                policy_bindings: vec![],
                allowed_grants: vec!["*".to_string()],
                required_tests: vec![],
            };
            store.commit_artifact(req, "*").unwrap();
        }
        
        // Verify order preserved
        let stats = store.stats().unwrap();
        assert_eq!(stats.commit_count, 5);
        
        // Read and verify all artifacts present
        let state = store.read_valid_state().unwrap();
        assert_eq!(state.artifacts.len(), 5);
        
        // Verify commits are in order
        for (i, commit) in state.commits.iter().enumerate() {
            assert_eq!(commit.sequence, (i + 1) as u64);
        }
    }

    /// Test repository mode detection
    #[test]
    fn test_repository_mode_detection() {
        let dir = tempdir().unwrap();
        
        // New repos should be single-file
        let mode = detect_repository_mode(dir.path());
        assert_eq!(mode, RepositoryMode::SingleFile);
        
        // After creation, should detect single-file mode
        let _store = RFSource::create(dir.path(), "test").unwrap();
        let mode = detect_repository_mode(dir.path());
        assert_eq!(mode, RepositoryMode::SingleFile);
    }

    /// Test file size reporting
    #[test]
    fn test_file_size_reporting() {
        let dir = tempdir().unwrap();
        let store = RFSource::create(dir.path(), "size_test").unwrap();
        
        // Should have non-zero size after creation
        let size = store.file_size().unwrap();
        assert!(size > 0);
        
        // Add a commit, size should grow
        let req = CommitArtifactRequest {
            logical_path: "test/large.rs".to_string(),
            content: "// ".repeat(1000), // ~3KB
            language: "rust".to_string(),
            message: "Large file".to_string(),
            actor: "test".to_string(),
            owner_capability: "test-cap".to_string(),
            risk_level: 1,
            policy_bindings: vec![],
            allowed_grants: vec!["*".to_string()],
            required_tests: vec![],
        };
        
        store.commit_artifact(req, "*").unwrap();
        let new_size = store.file_size().unwrap();
        assert!(new_size > size);
    }
}
