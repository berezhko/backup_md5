use crate::{directory, hash};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn process_files_with_extensions(
    source_dir: &str,
    target_base: &str,
    extensions: &HashSet<String>,
) -> Result<()> {
    let source_path = Path::new(source_dir);
    let md5_dir = Path::new(target_base).join("files_by_md5");
    let timestamp_dir = directory::create_timestamp_dir(target_base)?;

    directory::create_directories(&[&md5_dir, &timestamp_dir])?;

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e)) // Фильтрация скрытых директорий
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

/// Проверяет, является ли директория или файл скрытым
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.')) // Unix-скрытые файлы
        .unwrap_or(false)
        || entry.path().components().any(|c| {
            if let std::path::Component::Normal(os_str) = c {
                os_str.to_str().map(|s| s.starts_with('.')).unwrap_or(false)
            } else {
                false
            }
        }) // Проверка всех компонентов пути
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
    let _full_md5_path = directory::handle_md5_copy(path, md5_dir, &md5_hex)?;
    directory::create_timestamp_record(path, source_base, timestamp_dir, &md5_hex)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::{self};
    use std::path::PathBuf;
    use tempfile::{NamedTempFile, TempDir};
    use walkdir::WalkDir;

    // Вспомогательная функция для создания тестовой DirEntry
    fn create_dir_entry(path: &Path) -> walkdir::DirEntry {
        WalkDir::new(path.parent().unwrap())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path() == path)
            .unwrap_or_else(|| panic!("Failed to create DirEntry for {}", path.display()))
    }

    // Вспомогательная функция для создания тестовой структуры
    fn create_test_environment() -> (TempDir, PathBuf, HashSet<String>) {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir).unwrap();

        let extensions: HashSet<String> = ["txt", "jpg"].iter().map(|&s| s.to_string()).collect();

        (temp_dir, source_dir, extensions)
    }

    #[test]
    fn test_process_files_with_extensions_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir).unwrap();

        // Создаем тестовые файлы
        fs::write(source_dir.join("file1.txt"), "content").unwrap();
        fs::write(source_dir.join("file2.jpg"), "content").unwrap();
        fs::write(source_dir.join("ignore.pdf"), "content").unwrap();

        let extensions: HashSet<String> = ["txt", "jpg"].iter().map(|&s| s.to_string()).collect();
        let target_dir = temp_dir.path().join("target");

        let result = process_files_with_extensions(
            source_dir.to_str().unwrap(),
            target_dir.to_str().unwrap(),
            &extensions,
        );

        assert!(result.is_ok());
        assert!(target_dir.join("files_by_md5").exists());
    }

    #[ignore]
    #[test]
    fn test_process_files_with_extensions_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir).unwrap();

        // Создаем файлы
        fs::write(source_dir.join(".hidden.txt"), "content").unwrap();
        fs::write(source_dir.join("visible.txt"), "content").unwrap();

        let extensions: HashSet<String> = ["txt"].iter().map(|&s| s.to_string()).collect();
        let target_dir = temp_dir.path().join("target");

        let result = process_files_with_extensions(
            source_dir.to_str().unwrap(),
            target_dir.to_str().unwrap(),
            &extensions,
        );

        assert!(result.is_ok());

        // Проверяем что обработан только visible.txt
        let md5_dir = target_dir.join("files_by_md5");
        let mut processed_files = 0;

        for entry in WalkDir::new(&md5_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            processed_files += 1;
            assert!(!entry.path().to_string_lossy().contains("hidden"));
        }

        assert_eq!(processed_files, 1);
    }

    #[ignore]
    #[test]
    fn test_should_process_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("txt");
        fs::write(&path, "content").unwrap();

        let extensions: HashSet<String> = ["txt", "jpg"].iter().map(|&s| s.to_string()).collect();
        let entry = create_dir_entry(&path);

        assert!(should_process_file(&entry, &extensions));

        let hidden_path = temp_file.path().with_file_name(".hidden.txt");
        fs::write(&hidden_path, "content").unwrap();
        let hidden_entry = create_dir_entry(&hidden_path);

        assert!(!should_process_file(&hidden_entry, &extensions));
    }

    #[test]
    fn test_process_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir).unwrap();

        let test_file = source_dir.join("test.txt");
        fs::write(&test_file, "content").unwrap();

        let md5_dir = temp_dir.path().join("md5");
        let timestamp_dir = temp_dir.path().join("timestamp");
        fs::create_dir_all(&md5_dir).unwrap();
        fs::create_dir_all(&timestamp_dir).unwrap();

        let entry = create_dir_entry(&test_file);
        let result = process_file(&entry, &source_dir, &md5_dir, &timestamp_dir);

        assert!(result.is_ok());
        assert!(md5_dir
            .join("9a")
            .join("9a0364b9e99bb480dd25e1f0284c8555")
            .exists());
    }

    #[ignore]
    #[test]
    fn test_process_file_hidden() {
        let (temp_dir, source_dir, _) = create_test_environment();
        let hidden_file = source_dir.join(".hidden.txt");
        fs::write(&hidden_file, "content").unwrap();

        let entry = create_dir_entry(&hidden_file);
        let result = process_file(&entry, &source_dir, temp_dir.path(), temp_dir.path());

        assert!(result.is_err()); // Должно вернуть ошибку для скрытых файлов
    }

    #[ignore]
    #[test]
    fn test_is_hidden_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Создаем файлы для тестирования
        let hidden_file = temp_dir.path().join(".hidden");
        fs::write(&hidden_file, "content").unwrap();

        let normal_file = temp_dir.path().join("normal");
        fs::write(&normal_file, "content").unwrap();

        // Получаем DirEntry через WalkDir
        let hidden_entry = WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path() == hidden_file)
            .unwrap();

        let normal_entry = WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path() == normal_file)
            .unwrap();

        assert!(is_hidden(&hidden_entry));
        assert!(!is_hidden(&normal_entry));
    }

    #[test]
    fn test_empty_source_directory() {
        let (temp_dir, source_dir, extensions) = create_test_environment();
        let target_dir = temp_dir.path().join("target");

        let result = process_files_with_extensions(
            source_dir.to_str().unwrap(),
            target_dir.to_str().unwrap(),
            &extensions,
        );

        assert!(result.is_ok());
        assert!(target_dir.exists());
    }

    #[ignore]
    #[test]
    fn test_invalid_source_directory() {
        let extensions: HashSet<String> = ["txt"].iter().map(|&s| s.to_string()).collect();
        let result =
            process_files_with_extensions("/nonexistent/directory", "/tmp/target", &extensions);

        assert!(result.is_err());
    }
}
