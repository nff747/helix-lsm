//! Write-Ahead Log (WAL) with record checksums, binary framing, and crash recovery.

use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

pub const WAL_MAGIC: u32 = 0x57414C31; // 'WAL1'

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Put = 1,
    Delete = 2,
}

impl RecordType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(RecordType::Put),
            2 => Some(RecordType::Delete),
            _ => None,
        }
    }
}

pub struct WalWriter {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl WalWriter {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_buf)?;

        let mut writer = BufWriter::new(file);
        // If file is newly created, write magic header
        if writer.get_ref().metadata()?.len() == 0 {
            writer.write_all(&WAL_MAGIC.to_le_bytes())?;
            writer.flush()?;
        }

        Ok(Self {
            path: path_buf,
            writer,
        })
    }

    pub fn append(&mut self, record_type: RecordType, key: &[u8], value: Option<&[u8]>) -> io::Result<()> {
        let val_bytes = value.unwrap_or(&[]);
        let key_len = key.len() as u32;
        let val_len = val_bytes.len() as u32;

        // Checksum covers type + key + val
        let mut hasher = SimpleHasher::new();
        hasher.update(&[record_type as u8]);
        hasher.update(&key_len.to_le_bytes());
        hasher.update(key);
        hasher.update(&val_len.to_le_bytes());
        hasher.update(val_bytes);
        let checksum = hasher.finish();

        self.writer.write_all(&checksum.to_le_bytes())?;
        self.writer.write_all(&[record_type as u8])?;
        self.writer.write_all(&key_len.to_le_bytes())?;
        self.writer.write_all(key)?;
        self.writer.write_all(&val_len.to_le_bytes())?;
        self.writer.write_all(val_bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct WalReader;

impl WalReader {
    pub fn recover<P: AsRef<Path>>(path: P) -> io::Result<Vec<(RecordType, Vec<u8>, Option<Vec<u8>>)>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(path)?;
        let mut reader = BufReader::new(&mut file);
        let mut magic = [0u8; 4];
        if reader.read_exact(&mut magic).is_err() {
            return Ok(Vec::new()); // Empty log
        }
        if u32::from_le_bytes(magic) != WAL_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Corrupted WAL magic header"));
        }

        let mut entries = Vec::new();

        loop {
            let mut checksum_buf = [0u8; 4];
            if reader.read_exact(&mut checksum_buf).is_err() {
                break; // EOF
            }
            let expected_checksum = u32::from_le_bytes(checksum_buf);

            let mut type_buf = [0u8; 1];
            reader.read_exact(&mut type_buf)?;
            let record_type = RecordType::from_u8(type_buf[0])
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid record type"))?;

            let mut key_len_buf = [0u8; 4];
            reader.read_exact(&mut key_len_buf)?;
            let key_len = u32::from_le_bytes(key_len_buf) as usize;

            let mut key = vec![0u8; key_len];
            reader.read_exact(&mut key)?;

            let mut val_len_buf = [0u8; 4];
            reader.read_exact(&mut val_len_buf)?;
            let val_len = u32::from_le_bytes(val_len_buf) as usize;

            let mut val = vec![0u8; val_len];
            reader.read_exact(&mut val)?;

            // Verify checksum
            let mut hasher = SimpleHasher::new();
            hasher.update(&type_buf);
            hasher.update(&key_len_buf);
            hasher.update(&key);
            hasher.update(&val_len_buf);
            hasher.update(&val);
            if hasher.finish() != expected_checksum {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "WAL record checksum mismatch"));
            }

            let value_opt = if record_type == RecordType::Put {
                Some(val)
            } else {
                None
            };

            entries.push((record_type, key, value_opt));
        }

        Ok(entries)
    }
}

/// Lightweight CRC-like 32-bit checksum
struct SimpleHasher {
    state: u32,
}

impl SimpleHasher {
    fn new() -> Self {
        Self { state: 0x811c9dc5 }
    }

    fn update(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state ^= b as u32;
            self.state = self.state.wrapping_mul(0x01000193);
        }
    }

    fn finish(&self) -> u32 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wal_append_and_recover() {
        let temp_dir = std::env::temp_dir().join("helix_wal_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let wal_path = temp_dir.join("00001.wal");
        {
            let mut wal = WalWriter::open(&wal_path).unwrap();
            wal.append(RecordType::Put, b"k1", Some(b"v1")).unwrap();
            wal.append(RecordType::Put, b"k2", Some(b"v2")).unwrap();
            wal.append(RecordType::Delete, b"k1", None).unwrap();
            wal.sync().unwrap();
        }

        let recovered = WalReader::recover(&wal_path).unwrap();
        assert_eq!(recovered.len(), 3);
        assert_eq!(recovered[0], (RecordType::Put, b"k1".to_vec(), Some(b"v1".to_vec())));
        assert_eq!(recovered[1], (RecordType::Put, b"k2".to_vec(), Some(b"v2".to_vec())));
        assert_eq!(recovered[2], (RecordType::Delete, b"k1".to_vec(), None));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
