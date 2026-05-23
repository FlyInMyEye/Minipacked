use crate::progress::ProgressReader;
use anyhow::{Result, anyhow, bail};
use std::fs::{self, File};
use std::io::{Read, Write, copy, empty};
use std::path::{Component, Path, PathBuf};
use tar::{Archive, Builder, EntryType, Header};
use walkdir::WalkDir;

pub(crate) struct SourceInfo {
    pub(crate) root_name: PathBuf,
    pub(crate) total_bytes: u64,
}

pub(crate) fn suggested_archive_name(input: &Path) -> Result<String> {
    let base = if input.is_file() {
        input.file_stem()
    } else {
        input.file_name()
    }
    .and_then(|name| name.to_str())
    .ok_or_else(|| anyhow!("input path must have a valid UTF-8 file name"))?;

    Ok(format!("{base}.minipacked"))
}

pub(crate) fn inspect_source(input: &Path) -> Result<SourceInfo> {
    let root_name = input
        .file_name()
        .ok_or_else(|| anyhow!("input path must have a file name"))?
        .to_owned();
    let mut total_bytes = 0u64;

    if input.is_file() {
        total_bytes = fs::metadata(input)?.len();
    } else if input.is_dir() {
        for entry in WalkDir::new(input).follow_links(false) {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_bytes = total_bytes.saturating_add(metadata.len());
            } else if !metadata.is_dir() {
                bail!("unsupported path in input: {}", entry.path().display());
            }
        }
    } else {
        bail!("only regular files and directories are supported");
    }

    Ok(SourceInfo {
        root_name: root_name.into(),
        total_bytes,
    })
}

pub(crate) fn append_source<W: Write>(
    builder: &mut Builder<W>,
    input: &Path,
    root_name: &Path,
    progress: &indicatif::ProgressBar,
) -> Result<()> {
    if input.is_file() {
        append_file(builder, input, root_name, progress)?;
        return Ok(());
    }

    for entry in WalkDir::new(input).follow_links(false).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(input)?;
        let archive_path = if relative.as_os_str().is_empty() {
            PathBuf::from(root_name)
        } else {
            PathBuf::from(root_name).join(relative)
        };
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            append_directory(builder, path, &archive_path)?;
        } else if metadata.is_file() {
            append_file(builder, path, &archive_path, progress)?;
        } else {
            bail!("unsupported path in input: {}", path.display());
        }
    }

    Ok(())
}

pub(crate) fn unpack_archive<R: Read>(reader: R, progress: &indicatif::ProgressBar) -> Result<PathBuf> {
    let mut archive = Archive::new(reader);
    let mut first_top_level: Option<PathBuf> = None;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        let output_path = validate_output_path(&entry_path)?;

        if let Some(component) = entry_path.components().next() {
            let top_level = PathBuf::from(component.as_os_str());
            if first_top_level.is_none() {
                first_top_level = Some(top_level);
            }
        }

        if output_path.exists() {
            bail!("refusing to overwrite existing path: {}", output_path.display());
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&output_path)?;
            continue;
        }

        if !entry.header().entry_type().is_file() {
            bail!("unsupported archive entry: {}", entry_path.display());
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&output_path)?;
        let mut reader = ProgressReader::new(&mut entry, progress.clone());
        copy(&mut reader, &mut file)?;
    }

    first_top_level.ok_or_else(|| anyhow!("archive was empty"))
}

fn append_directory<W: Write>(builder: &mut Builder<W>, source_path: &Path, archive_path: &Path) -> Result<()> {
    let metadata = fs::metadata(source_path)?;
    let mut header = Header::new_gnu();
    header.set_metadata(&metadata);
    header.set_entry_type(EntryType::Directory);
    header.set_size(0);
    header.set_cksum();
    builder.append_data(&mut header, archive_path, empty())?;
    Ok(())
}

fn append_file<W: Write>(
    builder: &mut Builder<W>,
    source_path: &Path,
    archive_path: &Path,
    progress: &indicatif::ProgressBar,
) -> Result<()> {
    let mut file = File::open(source_path)?;
    let metadata = file.metadata()?;
    let mut header = Header::new_gnu();
    header.set_metadata(&metadata);
    header.set_cksum();
    let mut reader = ProgressReader::new(&mut file, progress.clone());
    builder.append_data(&mut header, archive_path, &mut reader)?;
    Ok(())
}

fn validate_output_path(path: &Path) -> Result<PathBuf> {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => output.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("archive contains an invalid output path: {}", path.display())
            }
        }
    }

    if output.as_os_str().is_empty() {
        bail!("archive contains an empty output path");
    }

    Ok(output)
}
