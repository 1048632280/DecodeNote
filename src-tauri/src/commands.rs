use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use encoding_rs::Encoding;

use crate::detect::detect_encoding;
use crate::document::DocumentSession;
use crate::encoding::{DecodeResult, EncodingId, SaveResult};
use crate::error::AppError;
use crate::file_io;

pub struct AppState {
    pub document: Mutex<Option<DocumentSession>>,
}

fn decode_bytes(bytes: &[u8], encoding: &EncodingId) -> Result<(String, bool, usize), AppError> {
    match encoding {
        EncodingId::Utf16Le => decode_utf16(bytes, false),
        EncodingId::Utf16Be => decode_utf16(bytes, true),
        _ => {
            let label = encoding
                .encoding_rs_label()
                .ok_or_else(|| AppError::DecodeError("不支持的编码".to_string()))?;
            let enc = Encoding::for_label(label.as_bytes())
                .ok_or_else(|| AppError::DecodeError(format!("未知编码: {}", label)))?;
            let (text, had_errors) = if *encoding == EncodingId::Utf8Bom {
                let stripped = if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
                    &bytes[3..]
                } else {
                    bytes
                };
                let (cow, _enc, had) = enc.decode(stripped);
                (cow.into_owned(), had)
            } else {
                let (cow, _enc, had) = enc.decode(bytes);
                (cow.into_owned(), had)
            };
            let replacement_count = text.chars().filter(|&c| c == '\u{FFFD}').count();
            Ok((text, had_errors, replacement_count))
        }
    }
}

fn decode_utf16(bytes: &[u8], big_endian: bool) -> Result<(String, bool, usize), AppError> {
    if bytes.len() < 2 {
        return Ok((String::new(), false, 0));
    }

    let has_bom = (big_endian && bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF)
        || (!big_endian && bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE);

    let start = if has_bom { 2 } else { 0 };
    let data = &bytes[start..];

    if data.len() % 2 != 0 {
        return Err(AppError::DecodeError(
            "UTF-16 字节长度不是偶数".to_string(),
        ));
    }

    let mut units = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        let unit = if big_endian {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_le_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }

    let text = String::from_utf16(&units).map_err(|e| {
        AppError::DecodeError(format!("UTF-16 解码失败: {}", e))
    })?;

    let replacement_count = text.chars().filter(|&c| c == '\u{FFFD}').count();
    let had_errors = replacement_count > 0;

    Ok((text, had_errors, replacement_count))
}

fn encode_text(text: &str, encoding: &EncodingId) -> Result<Vec<u8>, AppError> {
    match encoding {
        EncodingId::Utf16Le => {
            let mut result = Vec::new();
            result.extend_from_slice(&[0xFF, 0xFE]);
            for unit in text.encode_utf16() {
                result.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(result)
        }
        EncodingId::Utf16Be => {
            let mut result = Vec::new();
            result.extend_from_slice(&[0xFE, 0xFF]);
            for unit in text.encode_utf16() {
                result.extend_from_slice(&unit.to_be_bytes());
            }
            Ok(result)
        }
        _ => {
            let label = encoding
                .encoding_rs_label()
                .ok_or_else(|| AppError::EncodeError("不支持的编码".to_string()))?;
            let enc = Encoding::for_label(label.as_bytes())
                .ok_or_else(|| AppError::EncodeError(format!("未知编码: {}", label)))?;

            let mut result: Vec<u8> = Vec::new();

            if *encoding == EncodingId::Utf8Bom {
                result.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            }

            let (cow, _enc, had_errors) = enc.encode(text);
            if had_errors {
                return Err(AppError::EncodeError(
                    "文本包含无法编码为此编码的字符".to_string(),
                ));
            }
            result.extend_from_slice(&cow);
            Ok(result)
        }
    }
}

#[tauri::command]
pub async fn open_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<DecodeResult, AppError> {
    let path_buf = PathBuf::from(&path);
    let path_for_read = path_buf.clone();
    let bytes = tokio::task::spawn_blocking(move || file_io::read_file_bytes(&path_for_read))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    let file_size = bytes.len() as u64;
    let detection = detect_encoding(&bytes);
    let (text, had_errors, replacement_count) = decode_bytes(&bytes, &detection.encoding)?;

    let bom = detection.bom.clone();
    let detected = detection.encoding.clone();

    let revision = {
        let mut doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let doc = DocumentSession::new(
            path_buf.clone(),
            bytes,
            detection.encoding.clone(),
            Some(detection.encoding.clone()),
        );
        *doc_lock = Some(doc.clone());
        doc.revision
    };

    Ok(DecodeResult {
        text,
        encoding: detected.clone(),
        detected_encoding: Some(detected),
        file_size,
        had_errors,
        replacement_count,
        bom,
        revision,
    })
}

#[tauri::command]
pub async fn decode_current_as(
    state: State<'_, AppState>,
    encoding: EncodingId,
) -> Result<DecodeResult, AppError> {
    let (bytes, current_doc) = {
        let doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let doc = doc_lock
            .as_ref()
            .ok_or(AppError::NoDocument)?
            .clone();
        (doc.original_bytes.clone(), doc)
    };

    let (text, had_errors, replacement_count) = decode_bytes(&bytes, &encoding)?;

    let revision = {
        let mut doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(ref mut doc) = *doc_lock {
            doc.active_encoding = encoding.clone();
            doc.save_encoding = encoding.clone();
            doc.bump_revision()
        } else {
            return Err(AppError::NoDocument);
        }
    };

    Ok(DecodeResult {
        text,
        encoding: encoding.clone(),
        detected_encoding: current_doc.detected_encoding.clone(),
        file_size: bytes.len() as u64,
        had_errors,
        replacement_count,
        bom: None,
        revision,
    })
}

#[tauri::command]
pub async fn save_current(
    state: State<'_, AppState>,
    text: String,
    encoding: EncodingId,
) -> Result<SaveResult, AppError> {
    let (path, current_revision) = {
        let doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let doc = doc_lock.as_ref().ok_or(AppError::NoDocument)?;
        (doc.path.clone(), doc.revision)
    };

    let bytes = encode_text(&text, &encoding)?;

    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || file_io::write_file_bytes(&path_clone, &bytes))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    let file_size = bytes.len() as u64;
    let revision = {
        let mut doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(ref mut doc) = *doc_lock {
            doc.update_after_save(bytes, path.clone())
        } else {
            return Err(AppError::NoDocument);
        }
    };

    Ok(SaveResult {
        path: path.display().to_string(),
        file_size,
        revision,
    })
}

#[tauri::command]
pub async fn save_as(
    state: State<'_, AppState>,
    path: String,
    text: String,
    encoding: EncodingId,
) -> Result<SaveResult, AppError> {
    let path = PathBuf::from(&path);
    let bytes = encode_text(&text, &encoding)?;

    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || file_io::write_file_bytes(&path_clone, &bytes))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    let file_size = bytes.len() as u64;
    let revision = {
        let mut doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(ref mut doc) = *doc_lock {
            doc.update_after_save(bytes, path.clone())
        } else {
            let doc = DocumentSession::new(
                path.clone(),
                bytes,
                encoding.clone(),
                Some(encoding.clone()),
            );
            *doc_lock = Some(doc.clone());
            doc.revision
        }
    };

    Ok(SaveResult {
        path: path.display().to_string(),
        file_size,
        revision,
    })
}

#[tauri::command]
pub async fn get_supported_encodings() -> Result<Vec<crate::encoding::EncodingOption>, AppError> {
    Ok(EncodingId::all_options())
}
