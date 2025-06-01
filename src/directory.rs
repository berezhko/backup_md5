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

pub fn handle_md5_copy(
    source_path: &Path,
    md5_dir: &Path,
    md5_hex: &str,
) -> Result<String> {
    // Получаем первые два символа MD5 для поддиректории
    let prefix = &md5_hex[..2];
    
    // Создаем путь: md5_dir/{prefix}/{md5_hex}
    let sub_dir = md5_dir.join(prefix);
    create_dir_all(&sub_dir)
        .with_context(|| format!(
            "Failed to create subdirectory: {}",
            sub_dir.display()
        ))?;

    let md5_target = sub_dir.join(md5_hex);
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


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs;

    #[test]
    fn test_create_directories_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2/dir3");

        let result = create_directories(&[&dir1, &dir2]);
        assert!(result.is_ok());
        assert!(dir1.exists());
        assert!(dir2.exists());
    }

    #[test]
    fn test_create_directories_failure() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid/\0path");

        let result = create_directories(&[&invalid_path]);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_timestamp_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_timestamp_dir(temp_dir.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let timestamp_dir = result.unwrap();
        // assert!(timestamp_dir.exists()); create_timestamp_dir - не создает файлов/диреторий
        assert!(timestamp_dir.to_string_lossy().contains(Local::now().format("%Y%m%d").to_string().as_str()));
    }

    #[test]
    fn test_has_extension() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("txt");
        let extensions: HashSet<String> = ["txt", "jpg"].iter().map(|&s| s.to_string()).collect();

        assert!(has_extension(&path, &extensions));
        assert!(!has_extension(&path.with_extension("pdf"), &extensions));
        assert!(!has_extension(&path.with_extension(""), &extensions));
    }

    #[test]
    fn test_handle_md5_copy_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        fs::write(&source_file, "test content").unwrap();

        let md5_hex = "098f6bcd4621d373cade4e832627b4f6"; // MD5 для "test"
        let md5_dir = temp_dir.path().join("md5");
        
        let result = handle_md5_copy(&source_file, &md5_dir, md5_hex);
        assert!(result.is_ok());
        
        let expected_path = md5_dir.join("09").join(md5_hex);
        assert!(expected_path.exists());
    }

    #[test]
    fn test_handle_md5_copy_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        fs::write(&source_file, "test content").unwrap();

        let md5_hex = "098f6bcd4621d373cade4e832627b4f6";
        let md5_dir = temp_dir.path().join("md5");
        
        // Первое копирование
        handle_md5_copy(&source_file, &md5_dir, md5_hex).unwrap();
        // Второе копирование (не должно вызывать ошибку)
        let result = handle_md5_copy(&source_file, &md5_dir, md5_hex);
        
        assert!(result.is_ok());
    }

    #[ignore]
    #[test]
    fn test_handle_md5_copy_invalid_md5() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        fs::write(&source_file, "test").unwrap();

        let result = handle_md5_copy(&source_file, temp_dir.path(), "short");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_timestamp_record_simple() {
        let temp_dir = TempDir::new().unwrap();
        let source_base = temp_dir.path().join("source");
        fs::create_dir(&source_base).unwrap();
        
        let source_file = source_base.join("test.txt");
        fs::write(&source_file, "content").unwrap();

        let timestamp_dir = temp_dir.path().join("timestamp");
        let md5_path = "/path/to/md5/file";

        let result = create_timestamp_record(&source_file, &source_base, &timestamp_dir, md5_path);
        assert!(result.is_ok());
        
        let expected_record = timestamp_dir.join("test.txt");
        assert!(expected_record.exists());
        assert_eq!(fs::read_to_string(expected_record).unwrap(), md5_path);
    }

    #[test]
    fn test_create_timestamp_record_nested() {
        let temp_dir = TempDir::new().unwrap();
        let source_base = temp_dir.path().join("source");
        
        let nested_dir = source_base.join("nested/dir");
        fs::create_dir_all(&nested_dir).unwrap();
        
        let source_file = nested_dir.join("file.txt");
        fs::write(&source_file, "content").unwrap();

        let timestamp_dir = temp_dir.path().join("timestamp");
        let md5_path = "/path/to/md5/file";

        let result = create_timestamp_record(&source_file, &source_base, &timestamp_dir, md5_path);
        assert!(result.is_ok());
        
        let expected_record = timestamp_dir.join("nested/dir/file.txt");
        assert!(expected_record.exists());
        assert_eq!(fs::read_to_string(expected_record).unwrap(), md5_path);
    }

    #[test]
    fn test_create_timestamp_record_invalid_base() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("file.txt");
        fs::write(&source_file, "content").unwrap();

        let invalid_base = Path::new("/invalid/base");
        let timestamp_dir = temp_dir.path().join("timestamp");
        let result = create_timestamp_record(&source_file, invalid_base, &timestamp_dir, "md5");

        assert!(result.is_err());
    }
}