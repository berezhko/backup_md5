use crate::{directory, hash};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path};
use walkdir::DirEntry;

pub fn process_files_with_extensions(
    source_dir: &str,
    target_base: &str,
    extensions: &HashSet<String>,
) -> Result<()> {
    let source_path = Path::new(source_dir);
    let md5_dir = Path::new(target_base).join("files_by_md5");
    let timestamp_dir = directory::create_timestamp_dir(target_base)?;

    directory::create_directories(&[&md5_dir, &timestamp_dir])?;

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if should_process_file(&entry, extensions) {
            if let Err(e) = process_file(&entry, source_path, &md5_dir, &timestamp_dir) {
                eprintln!("Error processing {}: {}", entry.path().display(), e);
            }
        }
    }

    Ok(())
}

fn should_process_file(entry: &DirEntry, extensions: &HashSet<String>) -> bool {
    directory::has_extension(entry.path(), extensions)
}

fn process_file(
    entry: &DirEntry,
    source_base: &Path,
    md5_dir: &Path,
    timestamp_dir: &Path,
) -> Result<()> {
    let path = entry.path();
    let md5_hex = hash::calculate_md5(path)?;
    let full_md5_path = directory::handle_md5_copy(path, md5_dir, &md5_hex)?;
    directory::create_timestamp_record(path, source_base, timestamp_dir, &full_md5_path)?;
    Ok(())
}