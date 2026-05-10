// crates/rfsource-cli/src/commands/compact.rs
//! Compaction CLI command for RFSource
//!
//! Provides user-friendly interface for:
//! - Planning compaction with dry-run
//! - Executing compaction with progress reporting
//! - Rolling back failed/unwanted compactions
//! - Inspecting compaction history

use clap::{Args, Subcommand};
use rfsource_store::{MultiFileRepo, compaction::*};
use std::path::PathBuf;
use colored::Colorize;

#[derive(Args)]
pub struct CompactArgs {
    /// Repository directory
    #[arg(short, long, default_value = ".")]
    repo: PathBuf,
    
    #[command(subcommand)]
    command: CompactCommand,
}

#[derive(Subcommand)]
enum CompactCommand {
    /// Plan compaction (dry-run)
    Plan {
        /// Compaction strategy
        #[arg(short, long, default_value = "small")]
        strategy: String,
        
        /// Threshold size for small segments (MB)
        #[arg(short, long, default_value = "100")]
        threshold: u64,
    },
    
    /// Execute compaction
    Run {
        /// Compaction strategy
        #[arg(short, long, default_value = "small")]
        strategy: String,
        
        /// Threshold size for small segments (MB)
        #[arg(short, long, default_value = "100")]
        threshold: u64,
        
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    
    /// Rollback last compaction
    Rollback {
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    
    /// Show compaction history
    History,
    
    /// Verify compacted segments
    Verify,
}

pub fn execute(args: CompactArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        CompactCommand::Plan { strategy, threshold } => {
            plan_compaction(&args.repo, &strategy, threshold)
        }
        
        CompactCommand::Run { strategy, threshold, yes } => {
            run_compaction(&args.repo, &strategy, threshold, yes)
        }
        
        CompactCommand::Rollback { yes } => {
            rollback_compaction(&args.repo, yes)
        }
        
        CompactCommand::History => {
            show_history(&args.repo)
        }
        
        CompactCommand::Verify => {
            verify_compacted_segments(&args.repo)
        }
    }
}

fn plan_compaction(
    repo_dir: &PathBuf,
    strategy_name: &str,
    threshold: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Planning compaction...".bold());
    println!();
    
    // Load manifest
    let manifest_path = repo_dir.join(".rfsource.manifest");
    let manifest = rfsource_format::Manifest::load(&manifest_path)?;
    
    // Parse strategy
    let strategy = parse_strategy(strategy_name, threshold)?;
    
    // Create plan
    let plan = CompactionPlan::create(&manifest, strategy)?;
    
    // Display plan
    println!("{}", "Compaction Plan:".green().bold());
    println!("  Strategy: {:?}", plan.strategy);
    println!("  Segments to compact: {}", plan.segments.len());
    println!("  Segment indices: {:?}", plan.segments);
    println!("  Total frames: {}", format_number(plan.total_frames));
    println!("  Total size: {}", format_bytes(plan.total_size_bytes));
    println!("  Estimated duration: {}s", plan.estimated_duration_seconds());
    println!();
    
    // Show segment details
    println!("{}", "Segment Details:".bold());
    println!("  {:<8} {:<12} {:<12} {:<20}", "Index", "Frames", "Size", "Created");
    println!("  {}", "─".repeat(60));
    
    for &seg_idx in &plan.segments {
        let segment = manifest.segments.iter()
            .find(|s| s.index == seg_idx)
            .unwrap();
        
        println!("  {:<8} {:<12} {:<12} {:<20}",
            segment.index,
            format_number(segment.frame_count),
            format_bytes(segment.size_bytes),
            segment.created_at.format("%Y-%m-%d %H:%M:%S"),
        );
    }
    
    println!();
    println!("{}", "→ Run with: rfsource compact run".yellow());
    
    Ok(())
}

fn run_compaction(
    repo_dir: &PathBuf,
    strategy_name: &str,
    threshold: u64,
    skip_confirmation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Compacting repository...".bold());
    println!();
    
    // Open repo
    let mut repo = MultiFileRepo::open_or_init(repo_dir)?;
    
    // Parse strategy
    let strategy = parse_strategy(strategy_name, threshold)?;
    
    // Create plan
    let plan = CompactionPlan::create(&repo.manifest, strategy)?;
    
    // Show summary
    println!("Will compact {} segments ({} frames, {})",
        plan.segments.len(),
        format_number(plan.total_frames),
        format_bytes(plan.total_size_bytes)
    );
    println!();
    
    // Confirm (unless --yes)
    if !skip_confirmation {
        print!("Proceed with compaction? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }
    
    // Execute compaction
    println!("{}", "Executing compaction...".green().bold());
    
    let mut compactor = Compactor::new(repo_dir, &mut repo.manifest);
    
    // TODO: Add progress reporting
    let result = compactor.compact::<serde_json::Value>(&plan)?;
    
    // Show results
    println!();
    println!("{}", "Compaction complete!".green().bold());
    println!("  Segments compacted: {}", result.segments_compacted);
    println!("  Frames processed: {}", format_number(result.frames_compacted));
    println!("  Bytes read: {}", format_bytes(result.bytes_read));
    println!("  Bytes written: {}", format_bytes(result.bytes_written));
    println!("  Compression ratio: {:.2}%",
        (1.0 - result.bytes_written as f64 / result.bytes_read as f64) * 100.0
    );
    println!("  Duration: {:.2}s", result.duration.as_secs_f64());
    println!("  Throughput: {} MB/s",
        (result.bytes_read / 1024 / 1024) as f64 / result.duration.as_secs_f64()
    );
    println!();
    println!("  Compacted segment: {}", result.compacted_path.display());
    println!("  Checksum: {}", result.compacted_checksum);
    println!();
    println!("{}", "→ Original segments archived to .archived/".yellow());
    println!("{}", "→ Rollback with: rfsource compact rollback".yellow());
    
    Ok(())
}

fn rollback_compaction(
    repo_dir: &PathBuf,
    skip_confirmation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Rolling back compaction...".bold());
    println!();
    
    // Load manifest
    let manifest_path = repo_dir.join(".rfsource.manifest");
    let mut manifest = rfsource_format::Manifest::load(&manifest_path)?;
    
    let last_gen = manifest.compaction_generation - 1;
    
    // Find segments to restore
    let to_restore: Vec<_> = manifest.archived_segments.iter()
        .filter(|a| a.compacted_into == Some(last_gen))
        .collect();
    
    if to_restore.is_empty() {
        println!("{}", "No compaction to rollback.".yellow());
        return Ok(());
    }
    
    println!("Will restore {} archived segments", to_restore.len());
    for archived in &to_restore {
        println!("  - Segment {} from {}", archived.index, archived.path);
    }
    println!();
    
    // Confirm
    if !skip_confirmation {
        print!("Proceed with rollback? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }
    
    // Execute rollback
    let mut compactor = Compactor::new(repo_dir, &mut manifest);
    compactor.rollback_last_compaction()?;
    
    println!("{}", "Rollback complete!".green().bold());
    
    Ok(())
}

fn show_history(repo_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Compaction History:".bold());
    println!();
    
    // Load manifest
    let manifest_path = repo_dir.join(".rfsource.manifest");
    let manifest = rfsource_format::Manifest::load(&manifest_path)?;
    
    // Find compacted segments
    let compacted: Vec<_> = manifest.segments.iter()
        .filter(|s| s.is_compacted)
        .collect();
    
    if compacted.is_empty() {
        println!("{}", "No compactions performed yet.".yellow());
        return Ok(());
    }
    
    println!("{}", format!("Current generation: {}", manifest.compaction_generation).green());
    println!();
    
    println!("{:<10} {:<15} {:<15} {:<20} {:<15}",
        "Index", "Source Segments", "Frames", "Size", "Created");
    println!("{}", "─".repeat(80));
    
    for segment in compacted {
        let source_count = segment.compacted_from.as_ref()
            .map(|v| v.len())
            .unwrap_or(0);
        
        println!("{:<10} {:<15} {:<15} {:<20} {:<15}",
            segment.index,
            format!("{} segments", source_count),
            format_number(segment.frame_count),
            format_bytes(segment.size_bytes),
            segment.created_at.format("%Y-%m-%d %H:%M"),
        );
    }
    
    println!();
    
    // Show archived segments
    if !manifest.archived_segments.is_empty() {
        println!("{}", "Archived Segments:".bold());
        println!("{:<10} {:<20} {:<25}",
            "Index", "Compacted Into", "Archived At");
        println!("{}", "─".repeat(60));
        
        for archived in &manifest.archived_segments {
            println!("{:<10} {:<20} {:<25}",
                archived.index,
                archived.compacted_into.map(|g| format!("Gen {}", g))
                    .unwrap_or_else(|| "N/A".to_string()),
                archived.archived_at.format("%Y-%m-%d %H:%M:%S"),
            );
        }
    }
    
    Ok(())
}

fn verify_compacted_segments(repo_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Verifying compacted segments...".bold());
    println!();
    
    // Load manifest
    let manifest_path = repo_dir.join(".rfsource.manifest");
    let manifest = rfsource_format::Manifest::load(&manifest_path)?;
    
    // Find compacted segments
    let compacted: Vec<_> = manifest.segments.iter()
        .filter(|s| s.is_compacted)
        .collect();
    
    if compacted.is_empty() {
        println!("{}", "No compacted segments to verify.".yellow());
        return Ok(());
    }
    
    let mut all_ok = true;
    
    for segment in compacted {
        print!("  Segment {}: ", segment.index);
        
        // Verify checksum
        match &segment.checksum_blake3 {
            Some(checksum_hex) => {
                let expected = rfsource_format::SegmentChecksum::from_hex(checksum_hex)?;
                
                match rfsource_format::verify_file(&segment.path, &expected) {
                    Ok(_) => {
                        println!("{}", "✓ OK".green());
                    }
                    Err(e) => {
                        println!("{}", format!("✗ FAILED: {}", e).red());
                        all_ok = false;
                    }
                }
            }
            None => {
                println!("{}", "✗ No checksum".yellow());
                all_ok = false;
            }
        }
    }
    
    println!();
    
    if all_ok {
        println!("{}", "All compacted segments verified successfully!".green().bold());
    } else {
        println!("{}", "Some segments failed verification!".red().bold());
        return Err("Verification failed".into());
    }
    
    Ok(())
}

fn parse_strategy(name: &str, threshold: u64) -> Result<CompactionStrategy, String> {
    match name.to_lowercase().as_str() {
        "small" => Ok(CompactionStrategy::SmallSegments {
            threshold_bytes: threshold * 1024 * 1024,
        }),
        "oldest" => Ok(CompactionStrategy::OldestSegments {
            count: threshold as usize,
        }),
        "all" => Ok(CompactionStrategy::All),
        _ => Err(format!("Unknown strategy: {}", name)),
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
        count += 1;
    }
    
    result
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
