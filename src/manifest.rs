//! VersionSet & Manifest: tracks active SSTable files across levels atomically.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionEdit {
    AddTable { level: usize, table_id: u64, file_size: u64 },
    DeleteTable { level: usize, table_id: u64 },
}

pub struct VersionSet {
    pub levels: Vec<Vec<u64>>,
}

impl VersionSet {
    pub fn new(num_levels: usize) -> Self {
        Self {
            levels: vec![Vec::new(); num_levels],
        }
    }

    pub fn apply_edit(&mut self, edit: VersionEdit) {
        match edit {
            VersionEdit::AddTable { level, table_id, .. } => {
                if level < self.levels.len() {
                    self.levels[level].push(table_id);
                }
            }
            VersionEdit::DeleteTable { level, table_id } => {
                if level < self.levels.len() {
                    self.levels[level].retain(|&id| id != table_id);
                }
            }
        }
    }
}
