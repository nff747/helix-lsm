use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use crc32fast::Hasher;

use crate::types::{InternalKey, ValueType};

pub const WAL_RECORD_HEADER_SIZE: usize = 4 + 8 + 1 + 2 + 4; // CRC(4) + Seq(8) + Type(1) + KLen(2) + VLen(4)

#[derive(Debug, Clone)]
pub struct WalRecord {
    pub key: InternalKey,
    pub value: Bytes,
}

pub struct WalWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    current_size: u64,
}

impl WalWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())?;
        let current_size = file.metadata()?.len();

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            writer: BufWriter::with_capacity(256 * 1024, file), // 256KB write buffer for high throughput
            current_size,
        })
    }

    /// Append record to the write-ahead log with CRC32 verification header
    pub fn append(&mut self, key: &InternalKey, value: &[u8]) -> io::Result<u64> {
        let mut hasher = Hasher::new();
        hasher.update(&key.seq_num.to_le_bytes());
        hasher.update(&[key.value_type as u8]);
        hasher.update(&(key.user_key.len() as u16).to_le_bytes());
        hasher.update(&(value.len() as u32).to_le_bytes());
        hasher.update(&key.user_key);
        hasher.update(value);
        let checksum = hasher.finalize();

        // Write Frame
        self.writer.write_u32::<LittleEndian>(checksum)?;
        self.writer.write_u64::<LittleEndian>(key.seq_num)?;
        self.writer.write_u8(key.value_type as u8)?;
        self.writer.write_u16::<LittleEndian>(key.user_key.len() as u16)?;
        self.writer.write_u32::<LittleEndian>(value.len() as u32)?;
        self.writer.write_all(&key.user_key)?;
        self.writer.write_all(value)?;

        let bytes_written = WAL_RECORD_HEADER_SIZE + key.user_key.len() + value.len();
        self.current_size += bytes_written as u64;

        Ok(self.current_size)
    }

    /// Group Commit / Explicit fsync barrier
    pub fn sync(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        Ok(())
    }

    pub fn size(&self) -> u64 {
        self.current_size
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct WalReader {
    file: File,
}

impl WalReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(Self { file })
    }

    /// Replay all valid records from WAL, halting cleanly on EOF or torn write
    pub fn recover(&mut self) -> io::Result<Vec<WalRecord>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut records = Vec::new();

        loop {
            let expected_checksum = match self.file.read_u32::<LittleEndian>() {
                Ok(crc) => crc,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let seq_num = match self.file.read_u64::<LittleEndian>() {
                Ok(s) => s,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let op_type_byte = match self.file.read_u8() {
                Ok(t) => t,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let key_len = match self.file.read_u16::<LittleEndian>() {
                Ok(l) => l as usize,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let val_len = match self.file.read_u32::<LittleEndian>() {
                Ok(l) => l as usize,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let mut key_buf = vec![0u8; key_len];
            if let Err(ref e) = self.file.read_exact(&mut key_buf) {
                if e.kind() == io::ErrorKind::UnexpectedEof { break; }
                return Err(io::Error::new(e.kind(), "Torn write reading key"));
            }

            let mut val_buf = vec![0u8; val_len];
            if let Err(ref e) = self.file.read_exact(&mut val_buf) {
                if e.kind() == io::ErrorKind::UnexpectedEof { break; }
                return Err(io::Error::new(e.kind(), "Torn write reading value"));
            }

            // Verify CRC32 checksum to guarantee crash consistency
            let mut hasher = Hasher::new();
            hasher.update(&seq_num.to_le_bytes());
            hasher.update(&[op_type_byte]);
            hasher.update(&(key_len as u16).to_le_bytes());
            hasher.update(&(val_len as u32).to_le_bytes());
            hasher.update(&key_buf);
            hasher.update(&val_buf);
            let actual_checksum = hasher.finalize();

            if actual_checksum != expected_checksum {
                // Detected torn write or bit-rot; truncate replay at last consistent boundary
                break;
            }

            let internal_key = InternalKey::new(
                Bytes::from(key_buf),
                seq_num,
                ValueType::from(op_type_byte),
            );

            records.push(WalRecord {
                key: internal_key,
                value: Bytes::from(val_buf),
            });
        }

        Ok(records)
    }
}
