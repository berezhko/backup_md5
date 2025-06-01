use anyhow::{Context, Result};
use md5;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn calculate_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}