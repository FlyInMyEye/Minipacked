use crate::constants::{ARGON_M_COST, ARGON_P_COST, ARGON_T_COST, AUTH_TAG_LEN, CHUNK_SIZE, STREAM_NONCE_LEN};
use crate::progress::spinner;
use anyhow::{Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305,
    aead::stream::{DecryptorBE32, EncryptorBE32},
};
use std::io::{Read, Write};

pub(crate) fn derive_cipher(password: &str, salt: &[u8], message: &str) -> Result<XChaCha20Poly1305> {
    let spin = spinner(message);
    let cipher = cipher_from_password(password, salt);
    spin.finish_and_clear();
    cipher
}

fn cipher_from_password(password: &str, salt: &[u8]) -> Result<XChaCha20Poly1305> {
    let params = Params::new(ARGON_M_COST, ARGON_T_COST, ARGON_P_COST, Some(32))
        .map_err(|e| anyhow!(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(XChaCha20Poly1305::new((&key).into()))
}

pub(crate) struct StreamEncryptWriter<W> {
    inner: W,
    encryptor: EncryptorBE32<XChaCha20Poly1305>,
    buffer: Vec<u8>,
    plaintext_len: u64,
}

impl<W: Write> StreamEncryptWriter<W> {
    pub(crate) fn new(inner: W, cipher: XChaCha20Poly1305, nonce: [u8; STREAM_NONCE_LEN]) -> Result<Self> {
        Ok(Self {
            inner,
            encryptor: EncryptorBE32::from_aead(cipher, &nonce.into()),
            buffer: Vec::with_capacity(CHUNK_SIZE * 2),
            plaintext_len: 0,
        })
    }

    pub(crate) fn finish(mut self) -> Result<(W, u64)> {
        let encrypted = self
            .encryptor
            .encrypt_last(&self.buffer[..])
            .map_err(|_| anyhow!("encryption failed"))?;
        self.inner.write_all(&encrypted)?;
        self.plaintext_len = self.plaintext_len.saturating_add(self.buffer.len() as u64);
        self.inner.flush()?;
        Ok((self.inner, self.plaintext_len))
    }

    fn flush_chunks(&mut self) -> Result<()> {
        while self.buffer.len() > CHUNK_SIZE {
            let remaining = self.buffer.split_off(CHUNK_SIZE);
            let chunk = std::mem::replace(&mut self.buffer, remaining);
            let encrypted = self
                .encryptor
                .encrypt_next(&chunk[..])
                .map_err(|_| anyhow!("encryption failed"))?;
            self.inner.write_all(&encrypted)?;
            self.plaintext_len = self.plaintext_len.saturating_add(chunk.len() as u64);
        }
        Ok(())
    }
}

impl<W: Write> Write for StreamEncryptWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.flush_chunks().map_err(std::io::Error::other)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub(crate) struct StreamDecryptReader<R> {
    inner: R,
    decryptor: Option<DecryptorBE32<XChaCha20Poly1305>>,
    remaining_plaintext: u64,
    buffer: Vec<u8>,
    offset: usize,
    done: bool,
}

impl<R: Read> StreamDecryptReader<R> {
    pub(crate) fn new(inner: R, cipher: XChaCha20Poly1305, nonce: [u8; STREAM_NONCE_LEN], compressed_size: u64) -> Result<Self> {
        Ok(Self {
            inner,
            decryptor: Some(DecryptorBE32::from_aead(cipher, &nonce.into())),
            remaining_plaintext: compressed_size,
            buffer: Vec::new(),
            offset: 0,
            done: false,
        })
    }

    fn refill(&mut self) -> Result<()> {
        if self.done {
            self.buffer.clear();
            self.offset = 0;
            return Ok(());
        }

        let final_chunk = self.remaining_plaintext <= CHUNK_SIZE as u64;
        let plaintext_len = if final_chunk {
            self.remaining_plaintext as usize
        } else {
            CHUNK_SIZE
        };
        let mut encrypted = vec![0u8; plaintext_len + AUTH_TAG_LEN];
        self.inner.read_exact(&mut encrypted)?;

        let decrypted = if final_chunk {
            self.done = true;
            self.remaining_plaintext = 0;
            self.decryptor
                .take()
                .ok_or_else(|| anyhow!("wrong password or corrupted file"))?
                .decrypt_last(&encrypted[..])
                .map_err(|_| anyhow!("wrong password or corrupted file"))?
        } else {
            self.remaining_plaintext -= plaintext_len as u64;
            self.decryptor
                .as_mut()
                .ok_or_else(|| anyhow!("wrong password or corrupted file"))?
                .decrypt_next(&encrypted[..])
                .map_err(|_| anyhow!("wrong password or corrupted file"))?
        };

        self.buffer = decrypted;
        self.offset = 0;
        Ok(())
    }
}

impl<R: Read> Read for StreamDecryptReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.offset == self.buffer.len() {
            self.refill().map_err(std::io::Error::other)?;
            if self.offset == self.buffer.len() {
                return Ok(0);
            }
        }

        let available = &self.buffer[self.offset..];
        let read = available.len().min(buf.len());
        buf[..read].copy_from_slice(&available[..read]);
        self.offset += read;
        Ok(read)
    }
}
