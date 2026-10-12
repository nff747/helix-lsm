use helix_lsm::manifest::{VersionEdit, VersionSet};

#[test]
fn test_version_set_edits() {
    let mut vs = VersionSet::new(3);
    vs.apply_edit(VersionEdit::AddTable { level: 0, table_id: 1, file_size: 100 });
    vs.apply_edit(VersionEdit::AddTable { level: 0, table_id: 2, file_size: 100 });
    assert_eq!(vs.levels[0], vec![1, 2]);

    vs.apply_edit(VersionEdit::DeleteTable { level: 0, table_id: 1 });
    assert_eq!(vs.levels[0], vec![2]);
}
