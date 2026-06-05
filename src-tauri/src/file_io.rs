use std::path::PathBuf;
use crate::error::AppError;

pub fn read_file_bytes(path: &PathBuf) -> Result<Vec<u8>, AppError> {
    if !path.exists() {
        return Err(AppError::FileNotFound(path.display().to_string()));
    }
    if !path.is_file() {
        return Err(AppError::NotAFile(path.display().to_string()));
    }
    let metadata = std::fs::metadata(path).map_err(|e| {
        AppError::ReadError(format!("无法读取文件信息: {}", e))
    })?;
    let max_size: u64 = 50 * 1024 * 1024;
    if metadata.len() > max_size {
        return Err(AppError::FileTooLarge(metadata.len()));
    }
    std::fs::read(path).map_err(|e| {
        AppError::ReadError(format!("无法读取文件: {}", e))
    })
}

pub fn write_file_bytes(path: &PathBuf, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(AppError::WriteError(format!(
                "目录不存在: {}",
                parent.display()
            )));
        }
    }
    std::fs::write(path, bytes).map_err(|e| {
        AppError::WriteError(format!("无法写入文件: {}", e))
    })
}
