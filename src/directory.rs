use anyhow::{Context, Result};
use chrono::Local;
use std::collections::HashSet;
use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

pub fn create_directories(dirs: &[&Path]) -> Result<()> {
    for dir in dirs {
        create_dir_all(dir).with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}

pub fn create_timestamp_dir(base: &str) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let path = Path::new(base).join(timestamp);
    Ok(path)
}

pub fn has_extension(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

pub fn handle_md5_copy(source_path: &Path, md5_dir: &Path, md5_hex: &str) -> Result<String> {
    let md5_target = md5_dir.join(md5_hex);
    let full_md5_path = fs::canonicalize(&md5_target)
        .unwrap_or_else(|_| md5_target.clone())
        .to_string_lossy()
        .to_string();

    if !md5_target.exists() {
        fs::copy(source_path, &md5_target)
            .with_context(|| format!(
                "Failed to copy {} to {}",
                source_path.display(),
                md5_target.display()
            ))?;
    }

    Ok(full_md5_path)
}

/// Создает файл с записью во временной директории, сохраняя структуру каталогов
pub fn create_timestamp_record(
    source_path: &Path,
    source_base: &Path,
    timestamp_dir: &Path,
    full_md5_path: &str,
) -> Result<()> {
    // Получаем относительный путь от базовой директории
    let relative_path = source_path.strip_prefix(source_base)
        .with_context(|| format!(
            "Failed to get relative path for {} from base {}",
            source_path.display(),
            source_base.display()
        ))?;

    // Создаем полный путь во временной директории
    let record_path = timestamp_dir.join(relative_path);
    
    // Создаем все необходимые поддиректории
    if let Some(parent) = record_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!(
                "Failed to create directory: {}",
                parent.display()
            ))?;
    }

    // Создаем файл с записью
    let mut output = File::create(&record_path)
        .with_context(|| format!(
            "Failed to create file: {}",
            record_path.display()
        ))?;
    
    write!(output, "{}", full_md5_path)
        .with_context(|| format!(
            "Failed to write to file: {}",
            record_path.display()
        ))?;
    
    Ok(())
}