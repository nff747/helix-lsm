use helix_lsm::{LsmConfig, LsmEngine};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("⚡ Helix-LSM: Log-Structured Merge Tree Storage Engine");
    println!("--------------------------------------------------");

    let db_path = std::env::temp_dir().join("helix_lsm_demo");
    let config = LsmConfig {
        memtable_threshold_bytes: 32 * 1024,
        sstable_compaction_threshold: 4,
    };

    let mut engine = LsmEngine::open(&db_path, config)?;
    println!("Opened storage engine at {:?}", db_path);

    println!("\nWriting 5,000 keys...");
    let start = Instant::now();
    for i in 0..5000 {
        let key = format!("user:{:06}", i);
        let val = format!("{{\"name\":\"Dev_{}\",\"status\":\"active\"}}", i);
        engine.put(key.as_bytes(), val.as_bytes())?;
    }
    let elapsed = start.elapsed();
    println!("Write completed in {:.2?} ({:.0} writes/sec)", elapsed, 5000.0 / elapsed.as_secs_f64());

    println!("\nReading back keys...");
    let read_start = Instant::now();
    let mut hits = 0;
    for i in 0..5000 {
        let key = format!("user:{:06}", i);
        if let Some(_) = engine.get(key.as_bytes())? {
            hits += 1;
        }
    }
    let read_elapsed = read_start.elapsed();
    println!("Read {}/5000 keys in {:.2?} ({:.0} reads/sec)", hits, read_elapsed, 5000.0 / read_elapsed.as_secs_f64());

    println!("\nFlushing MemTable and compacting SSTables...");
    engine.flush()?;
    engine.compact()?;
    println!("Compaction finished successfully. Engine clean.");

    let _ = std::fs::remove_dir_all(&db_path);
    Ok(())
}
