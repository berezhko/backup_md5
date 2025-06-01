use anyhow::{Context, Result};
use md5;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn calculate_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_calculate_md5_empty_file() {
        // Создаем временный файл
        let file = NamedTempFile::new().unwrap();
        // Пустой файл должен иметь определенный MD5
        let expected_md5 = "d41d8cd98f00b204e9800998ecf8427e";
        
        let actual_md5 = calculate_md5(file.path()).unwrap();
        assert_eq!(actual_md5, expected_md5);
    }

    #[test]
    fn test_calculate_md5_simple_content() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        
        // Предварительно вычисленный MD5 для "hello world"
        let expected_md5 = "5eb63bbbe01eeed093cb22bb8f5acdc3";
        let actual_md5 = calculate_md5(file.path()).unwrap();
        
        assert_eq!(actual_md5, expected_md5);
    }

    #[test]
    fn test_calculate_md5_binary_content() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        
        // MD5 для байтов 0x00 0x01 0x02 0x03
        let expected_md5 = "37b59afd592725f9305e484a5d7f5168";
        let actual_md5 = calculate_md5(file.path()).unwrap();
        
        assert_eq!(actual_md5, expected_md5);
    }

    #[test]
    fn test_calculate_md5_nonexistent_file() {
        let path = Path::new("/nonexistent/file");
        let result = calculate_md5(path);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to open file"));
    }

    #[test]
    fn test_md5_consistency() {
        // Проверяем что два одинаковых файла дают одинаковый хеш
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        write!(file1, "same content").unwrap();
        write!(file2, "same content").unwrap();
        
        let md5_1 = calculate_md5(file1.path()).unwrap();
        let md5_2 = calculate_md5(file2.path()).unwrap();
        
        assert_eq!(md5_1, md5_2);
    }

    #[test]
    fn test_md5_different_for_different_content() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        write!(file1, "content1").unwrap();
        write!(file2, "content2").unwrap();
        
        let md5_1 = calculate_md5(file1.path()).unwrap();
        let md5_2 = calculate_md5(file2.path()).unwrap();
        
        assert_ne!(md5_1, md5_2);
    }
}