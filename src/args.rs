use crate::constants::{ZSTD_LEVEL_COMPACT, ZSTD_LEVEL_DEFAULT, ZSTD_LEVEL_FAST};
use anyhow::{Result, anyhow, bail};
use std::path::PathBuf;

pub(crate) struct PackArgs {
    pub(crate) input: PathBuf,
    pub(crate) compression: CompressionMode,
}

pub(crate) enum CompressionMode {
    Fast,
    Default,
    Compact,
}

impl CompressionMode {
    pub(crate) fn zstd_level(&self) -> i32 {
        match self {
            Self::Fast => ZSTD_LEVEL_FAST,
            Self::Default => ZSTD_LEVEL_DEFAULT,
            Self::Compact => ZSTD_LEVEL_COMPACT,
        }
    }
}

pub(crate) fn forwarded_args(args: &[String], bin_name: &str) -> Vec<String> {
    let mut forwarded = Vec::with_capacity(args.len().saturating_sub(1));
    forwarded.push(bin_name.to_string());
    forwarded.extend(args.iter().skip(2).cloned());
    forwarded
}

pub(crate) fn parse_pack_args(args: &[String]) -> Result<PackArgs> {
    let mut compression = CompressionMode::Default;
    let mut recursive = false;
    let mut input: Option<PathBuf> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--fast" => {
                if !matches!(compression, CompressionMode::Default) {
                    bail!("choose only one compression mode");
                }
                compression = CompressionMode::Fast;
            }
            "--compact" => {
                if !matches!(compression, CompressionMode::Default) {
                    bail!("choose only one compression mode");
                }
                compression = CompressionMode::Compact;
            }
            "-r" => {
                if recursive {
                    bail!("-r was provided more than once");
                }
                recursive = true;
            }
            value if value.starts_with('-') => bail!("unknown option: {value}"),
            value => {
                if input.is_some() {
                    bail!("usage: minipack [--fast|--compact] <file> | minipack [--fast|--compact] -r <directory>");
                }
                input = Some(PathBuf::from(value));
            }
        }
    }

    let input = validate_input_path(input.ok_or_else(|| {
        anyhow!("usage: minipack [--fast|--compact] <file> | minipack [--fast|--compact] -r <directory>")
    })?)?;

    if recursive && !input.is_dir() {
        bail!("-r requires a directory input");
    }

    if !recursive && input.is_dir() {
        bail!("directory input requires -r");
    }

    Ok(PackArgs { input, compression })
}

fn validate_input_path(input: PathBuf) -> Result<PathBuf> {
    if !input.exists() {
        bail!("input not found: {}", input.display());
    }
    Ok(input)
}
