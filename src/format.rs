use crate::constants::{MAGIC, SALT_LEN, STREAM_NONCE_LEN, VERSION};
use anyhow::{Result, bail};
use rand::{RngCore, rngs::OsRng};
use std::io::{Read, Write};

pub(crate) struct FileHeader {
    pub(crate) password_mode: u8,
    pub(crate) salt: [u8; SALT_LEN],
    pub(crate) nonce: [u8; STREAM_NONCE_LEN],
    pub(crate) raw_size: u64,
    pub(crate) compressed_size: u64,
}

impl FileHeader {
    pub(crate) fn new(raw_size: u64, password_mode: u8) -> Self {
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; STREAM_NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);
        Self {
            password_mode,
            salt,
            nonce,
            raw_size,
            compressed_size: 0,
        }
    }

    pub(crate) fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(&[VERSION])?;
        writer.write_all(&[self.password_mode])?;
        writer.write_all(&self.salt)?;
        writer.write_all(&self.nonce)?;
        writer.write_all(&self.raw_size.to_le_bytes())?;
        writer.write_all(&self.compressed_size.to_le_bytes())?;
        Ok(())
    }

    pub(crate) fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; MAGIC.len()];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC {
            bail!("invalid minipacked file");
        }

        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != VERSION {
            bail!("unsupported minipacked version: {}", version[0]);
        }

        let mut password_mode = [0u8; 1];
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; STREAM_NONCE_LEN];
        let mut raw_size = [0u8; 8];
        let mut compressed_size = [0u8; 8];
        reader.read_exact(&mut password_mode)?;
        reader.read_exact(&mut salt)?;
        reader.read_exact(&mut nonce)?;
        reader.read_exact(&mut raw_size)?;
        reader.read_exact(&mut compressed_size)?;

        Ok(Self {
            password_mode: password_mode[0],
            salt,
            nonce,
            raw_size: u64::from_le_bytes(raw_size),
            compressed_size: u64::from_le_bytes(compressed_size),
        })
    }
}
