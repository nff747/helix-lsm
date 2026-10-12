//! Column Family partitioning: isolating keyspaces with separate MemTables and SSTables.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnFamilyDescriptor {
    pub name: String,
    pub id: u32,
}

pub struct ColumnFamilyRegistry {
    families: Vec<ColumnFamilyDescriptor>,
}

impl ColumnFamilyRegistry {
    pub fn new() -> Self {
        let default_cf = ColumnFamilyDescriptor {
            name: "default".to_string(),
            id: 0,
        };
        Self {
            families: vec![default_cf],
        }
    }

    pub fn register(&mut self, name: &str) -> u32 {
        let id = self.families.len() as u32;
        self.families.push(ColumnFamilyDescriptor {
            name: name.to_string(),
            id,
        });
        id
    }

    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.families.iter().find(|cf| cf.name == name).map(|cf| cf.id)
    }
}
