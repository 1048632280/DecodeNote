use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("不是文件: {0}")]
    NotAFile(String),

    #[error("文件过大 ({0} bytes)，超过50MB限制")]
    FileTooLarge(u64),

    #[error("读取错误: {0}")]
    ReadError(String),

    #[error("写入错误: {0}")]
    WriteError(String),

    #[error("解码错误: {0}")]
    DecodeError(String),

    #[error("编码错误: {0}")]
    EncodeError(String),

    #[error("没有打开的文件")]
    NoDocument,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
