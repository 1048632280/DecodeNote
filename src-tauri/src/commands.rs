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
        EncodingId::Utf16Le | EncodingId::Utf16Be => decode_utf16_via_encoding_rs(bytes, *encoding == EncodingId::Utf16Be),
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

fn decode_utf16_via_encoding_rs(bytes: &[u8], big_endian: bool) -> Result<(String, bool, usize), AppError> {
    let label = if big_endian { "UTF-16BE" } else { "UTF-16LE" };
    let enc = Encoding::for_label(label.as_bytes())
        .ok_or_else(|| AppError::DecodeError(format!("未知编码: {}", label)))?;
    let (cow, _enc, had_errors) = enc.decode(bytes);
    let mut text = cow.into_owned();
    if text.starts_with('\u{FEFF}') {
        text.remove(0);
    }
    let replacement_count = text.chars().filter(|&c| c == '\u{FFFD}').count();
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

    let total_chars = text.chars().count();
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
        total_chars,
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
    let total_chars = text.chars().count();

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
        total_chars,
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
    let (path, _current_revision) = {
        let doc_lock = state.document.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let doc = doc_lock.as_ref().ok_or(AppError::NoDocument)?;
        (doc.path.clone(), doc.revision)
    };

    let bytes = encode_text(&text, &encoding)?;
    let file_size = bytes.len() as u64;

    let path_clone = path.clone();
    let bytes_for_write = bytes.clone();
    tokio::task::spawn_blocking(move || file_io::write_file_bytes(&path_clone, &bytes_for_write))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
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
    let file_size = bytes.len() as u64;

    let path_clone = path.clone();
    let bytes_for_write = bytes.clone();
    tokio::task::spawn_blocking(move || file_io::write_file_bytes(&path_clone, &bytes_for_write))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
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
