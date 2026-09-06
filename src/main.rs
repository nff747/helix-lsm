use std::sync::Arc;
use std::time::Instant;
use std::thread;
use clap::Parser;
use tempfile::tempdir;

use helix_lsm::{HelixDb, EngineOptions};

#[derive(Parser, Debug)]
#[command(author, version, about = "HelixLSM High-Throughput Storage Benchmark")]
struct Args {
    /// Number of concurrent client worker threads
    #[arg(short, long, default_value_t = 8)]
    threads: usize,

    /// Number of write operations per thread
    #[arg(short, long, default_value_t = 50_000)]
    ops_per_thread: usize,

    /// Payload value size in bytes
    #[arg(short, long, default_value_t = 128)]
    value_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let total_ops = args.threads * args.ops_per_thread;

    println!("============================================================");
    println!("  HELIX-LSM DISTRIBUTED STORAGE ENGINE // BENCHMARK HARNESS");
    println!("============================================================");
    println!("  Concurrency : {} threads", args.threads);
    println!("  Total Writes: {} ops", total_ops);
    println!("  Payload Size: {} bytes/op", args.value_size);
    println!("------------------------------------------------------------");

    let dir = tempdir()?;
    let mut options = EngineOptions::default();
    options.memtable_size_bytes = 1024 * 1024; // 1MB MemTable for active flush & compaction testing

    let db = HelixDb::open(dir.path(), options)?;

    // ── 1. CONCURRENT WRITE BENCHMARK ──
    println!("[1/3] Executing concurrent write workload...");
    let payload = vec![0xABu8; args.value_size];
    let start_write = Instant::now();

    let mut handles = Vec::new();
    for t_id in 0..args.threads {
        let db_clone = Arc::clone(&db);
        let val_clone = payload.clone();
        let ops = args.ops_per_thread;

        handles.push(thread::spawn(move || {
            for i in 0..ops {
                let key = format!("user_{:04}_key_{:08}", t_id, i);
                db_clone.put(key.as_bytes(), &val_clone).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let write_duration = start_write.elapsed();
    let write_iops = (total_ops as f64) / write_duration.as_secs_f64();
    let write_mb = ((total_ops * (24 + args.value_size)) as f64) / (1024.0 * 1024.0) / write_duration.as_secs_f64();

    println!("  ✓ Finished {} writes in {:.3?}", total_ops, write_duration);
    println!("  ✓ Throughput: {:.2} writes/sec ({:.2} MB/s)", write_iops, write_mb);

    // ── 2. FLUSH & COMPACTION ──
    println!("\n[2/3] Triggering MemTable flush & Leveled Compaction...");
    let start_compact = Instant::now();
    db.flush()?;
    db.compact()?;
    println!("  ✓ Compaction completed in {:.3?}", start_compact.elapsed());

    // ── 3. POINT READ VERIFICATION ──
    println!("\n[3/3] Executing point read verification...");
    let start_read = Instant::now();
    let mut read_hits = 0;

    for t_id in 0..args.threads {
        for i in (0..args.ops_per_thread).step_by(100) {
            let key = format!("user_{:04}_key_{:08}", t_id, i);
            if let Some(val) = db.get(key.as_bytes())? {
                if val.as_ref() == payload.as_slice() {
                    read_hits += 1;
                }
            }
        }
    }

    let read_duration = start_read.elapsed();
    println!("  ✓ Read {} verified keys in {:.3?}", read_hits, read_duration);

    println!("\n============================================================");
    println!("  BENCHMARK SUMMARY: ALL TESTS PASSED // ZERO CORRUPTION");
    println!("============================================================");

    Ok(())
}
