use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use md5;
use chrono::Local;
use serde::Deserialize;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
struct Config {
    extensions: Vec<String>,
}

fn main() -> Result<()> {
    // Парсинг аргументов командной строки
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <source_dir> <target_base> <config_file>", args[0]);
        eprintln!("Example: {} ./src ./target config.toml", args[0]);
        std::process::exit(1);
    }

    let source_dir = &args[1];
    let target_base = &args[2];
    let config_file = &args[3];

    // Чтение конфигурационного файла
    let config_content = fs::read_to_string(config_file)
        .with_context(|| format!("Failed to read config file: {}", config_file))?;
    
    let config: Config = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {}", config_file))?;

    // Преобразование в HashSet с расширениями в нижнем регистре
    let extensions: HashSet<String> = config.extensions
        .into_iter()
        .map(|ext| ext.to_lowercase())
        .collect();

    process_files_with_extensions(source_dir, target_base, extensions)
}

fn process_files_with_extensions(
    source_dir: &str,
    target_base: &str,
    extensions: HashSet<String>,
) -> Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let md5_dir = Path::new(target_base).join("files_by_md5");
    let timestamp_dir = Path::new(target_base).join(&timestamp);
    let source_base = Path::new(source_dir);

    create_directories(&[&md5_dir, &timestamp_dir])?;

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if should_process_file(&entry, &extensions) {
            if let Err(e) = process_file(&entry, source_base, &md5_dir, &timestamp_dir) {
                eprintln!("Error processing {}: {}", entry.path().display(), e);
            }
        }
    }

    Ok(())
}

/// Создает необходимые директории
fn create_directories(dirs: &[&Path]) -> Result<()> {
    for dir in dirs {
        create_dir_all(dir).with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}

/// Проверяет, нужно ли обрабатывать файл
fn should_process_file(entry: &DirEntry, extensions: &HashSet<String>) -> bool {
    let path = entry.path();
    
    if !path.is_file() {
        return false;
    }

    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

fn process_file(
    entry: &DirEntry,
    source_base: &Path,
    md5_dir: &Path,
    timestamp_dir: &Path,
) -> Result<()> {
    let path = entry.path();
    let md5_hex = calculate_md5(path)?;
    let full_md5_path = handle_md5_copy(path, md5_dir, &md5_hex)?;
    create_timestamp_record(path, source_base, timestamp_dir, &full_md5_path)?;
    Ok(())
}

/// Вычисляет MD5 файла
fn calculate_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

/// Обрабатывает копирование файла в MD5 директорию
fn handle_md5_copy(
    source_path: &Path,
    md5_dir: &Path,
    md5_hex: &str,
) -> Result<String> {
    let md5_target = md5_dir.join(md5_hex);
    
    // Получаем абсолютный путь
    let full_md5_path = fs::canonicalize(&md5_target)
        .unwrap_or_else(|_| md5_target.clone())
        .to_string_lossy()
        .to_string();

    // Копируем только если файл не существует
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
fn create_timestamp_record(
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
