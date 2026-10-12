//! Leveled compaction policy (L0: 4 SSTables, L1: 10MB, L2: 100MB, 10x growth).

pub struct LeveledCompactionController {
    pub l0_max_tables: usize,
    pub base_level_bytes: u64,
    pub level_multiplier: u64,
}

impl LeveledCompactionController {
    pub fn new() -> Self {
        Self {
            l0_max_tables: 4,
            base_level_bytes: 10 * 1024 * 1024,
            level_multiplier: 10,
        }
    }

    pub fn max_bytes_for_level(&self, level: usize) -> u64 {
        if level <= 1 {
            self.base_level_bytes
        } else {
            self.base_level_bytes * self.level_multiplier.pow((level - 1) as u32)
        }
    }

    pub fn should_compact_l0(&self, l0_table_count: usize) -> bool {
        l0_table_count >= self.l0_max_tables
    }
}
