use crate::archive::{append_source, inspect_source, suggested_archive_name, unpack_archive};
use crate::args::{forwarded_args, parse_pack_args};
use crate::constants::{DEFAULT_PASSWORD, PASSWORD_MODE_DEFAULT};
use crate::crypto::{StreamDecryptReader, StreamEncryptWriter, derive_cipher};
use crate::format::FileHeader;
use crate::progress::{ProgressWriter, byte_progress_bar, pack_progress_bar};
use crate::prompt::{prompt_line, prompt_password, prompt_password_pair};
use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub fn run_pack(args: &[String]) -> Result<()> {
    let pack_args = parse_pack_args(args)?;
    let input = pack_args.input;
    let suggested = suggested_archive_name(&input)?;
    let output_name = prompt_line(&format!("Enter name ({suggested}): "))?;
    let chosen_name = if output_name.trim().is_empty() {
        suggested
    } else if output_name.ends_with(".minipacked") {
        output_name
    } else {
        format!("{output_name}.minipacked")
    };

    let (password, password_mode) = prompt_password_pair()?;
    let output_path = PathBuf::from(&chosen_name);
    if output_path.exists() {
        bail!("output already exists: {}", output_path.display());
    }

    let source = inspect_source(&input)?;
    let mut header = FileHeader::new(source.total_bytes, password_mode);
    let cipher = derive_cipher(&password, &header.salt, "Deriving key")?;

    let file = File::create(&output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let mut writer = BufWriter::new(file);
    header.write_to(&mut writer)?;

    let progress = pack_progress_bar(source.total_bytes, "Packing");
    let progress_writer = ProgressWriter::new(writer, progress.clone(), "Packing");
    let encrypt_writer = StreamEncryptWriter::new(progress_writer, cipher, header.nonce)?;
    let encoder = zstd::Encoder::new(encrypt_writer, pack_args.compression.zstd_level())?;
    let mut builder = tar::Builder::new(encoder);
    append_source(&mut builder, &input, &source.root_name, &progress)?;
    let encoder = builder.into_inner()?;
    let encrypt_writer = encoder.finish()?;
    let (progress_writer, compressed_size) = encrypt_writer.finish()?;
    let mut writer = progress_writer.into_inner();
    progress.finish_and_clear();

    header.compressed_size = compressed_size;
    writer.seek(SeekFrom::Start(0))?;
    header.write_to(&mut writer)?;
    writer.flush()?;

    println!("Done: ./{}", output_path.display());
    Ok(())
}

pub fn run_unpack(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        bail!("usage: miniunpack <file.minipacked>");
    }

    let input = PathBuf::from(&args[1]);
    if !input.is_file() {
        bail!("input archive not found: {}", input.display());
    }

    let file = File::open(&input).with_context(|| format!("failed to read {}", input.display()))?;
    let mut reader = BufReader::new(file);
    let header = FileHeader::read_from(&mut reader)?;
    let password = if header.password_mode == PASSWORD_MODE_DEFAULT {
        DEFAULT_PASSWORD.to_string()
    } else {
        prompt_password("Enter password: ", false)?
    };

    let cipher = derive_cipher(&password, &header.salt, "Deriving key")?;
    let decrypt_reader = StreamDecryptReader::new(reader, cipher, header.nonce, header.compressed_size)?;
    let decoder = zstd::Decoder::new(decrypt_reader)?;
    let progress = byte_progress_bar(header.raw_size, "Unpacking");
    let restored = unpack_archive(decoder, &progress)?;
    progress.finish_and_clear();

    println!("Done: {}", restored.display());
    Ok(())
}

pub fn run_root(args: &[String]) -> Result<()> {
    match args.get(1).map(String::as_str) {
        Some("pack") => {
            let forwarded = forwarded_args(args, "minipack");
            run_pack(&forwarded)
        }
        Some("unpack") => {
            let forwarded = forwarded_args(args, "miniunpack");
            run_unpack(&forwarded)
        }
        _ => {
            eprintln!("Usage: minipacked pack [--fast|--compact] <file>");
            eprintln!("       minipacked pack [--fast|--compact] -r <directory>");
            eprintln!("       minipacked unpack <file.minipacked>");
            Ok(())
        }
    }
}
