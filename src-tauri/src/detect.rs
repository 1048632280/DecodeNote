use crate::encoding::EncodingId;

pub struct DetectionResult {
    pub encoding: EncodingId,
    pub bom: Option<String>,
    pub confidence: DetectionConfidence,
}

#[derive(Clone, Debug)]
pub enum DetectionConfidence {
    High,
    Medium,
    Low,
}

impl DetectionConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

pub fn detect_encoding(bytes: &[u8]) -> DetectionResult {
    if bytes.is_empty() {
        return DetectionResult {
            encoding: EncodingId::Utf8,
            bom: None,
            confidence: DetectionConfidence::High,
        };
    }

    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return DetectionResult {
            encoding: EncodingId::Utf8Bom,
            bom: Some("UTF-8 BOM".to_string()),
            confidence: DetectionConfidence::High,
        };
    }

    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return DetectionResult {
            encoding: EncodingId::Utf16Le,
            bom: Some("UTF-16 LE BOM".to_string()),
            confidence: DetectionConfidence::High,
        };
    }

    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        return DetectionResult {
            encoding: EncodingId::Utf16Be,
            bom: Some("UTF-16 BE BOM".to_string()),
            confidence: DetectionConfidence::High,
        };
    }

    if is_valid_utf8(bytes) {
        return DetectionResult {
            encoding: EncodingId::Utf8,
            bom: None,
            confidence: DetectionConfidence::High,
        };
    }

    if let Some(result) = detect_utf16_heuristic(bytes) {
        return result;
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let (name, confidence) = detector.guess_assess(None, true);

    let encoding = map_chardetng_name(name);
    let confidence = if confidence < 0.5 {
        DetectionConfidence::Low
    } else if confidence < 0.8 {
        DetectionConfidence::Medium
    } else {
        DetectionConfidence::High
    };

    DetectionResult {
        encoding,
        bom: None,
        confidence,
    }
}

fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

fn detect_utf16_heuristic(bytes: &[u8]) -> Option<DetectionResult> {
    if bytes.len() < 2 {
        return None;
    }

    let mut le_zero_count = 0usize;
    let mut be_zero_count = 0usize;
    let sample_len = bytes.len().min(4096);

    for i in (0..sample_len.saturating_sub(1)).step_by(2) {
        if bytes[i + 1] == 0 {
            le_zero_count += 1;
        }
        if bytes[i] == 0 {
            be_zero_count += 1;
        }
    }

    let total_pairs = sample_len / 2;
    if total_pairs == 0 {
        return None;
    }

    let le_ratio = le_zero_count as f64 / total_pairs as f64;
    let be_ratio = be_zero_count as f64 / total_pairs as f64;

    if le_ratio > 0.5 && le_ratio > be_ratio {
        return Some(DetectionResult {
            encoding: EncodingId::Utf16Le,
            bom: None,
            confidence: DetectionConfidence::Medium,
        });
    }

    if be_ratio > 0.5 && be_ratio > le_ratio {
        return Some(DetectionResult {
            encoding: EncodingId::Utf16Be,
            bom: None,
            confidence: DetectionConfidence::Medium,
        });
    }

    None
}

fn map_chardetng_name(name: &str) -> EncodingId {
    match name {
        "UTF-8" => EncodingId::Utf8,
        "GBK" | "GB2312" => EncodingId::Gbk,
        "GB18030" => EncodingId::Gb18030,
        "Big5" => EncodingId::Big5,
        "SHIFT_JIS" => EncodingId::ShiftJis,
        "EUC-JP" => EncodingId::EucJp,
        "EUC-KR" => EncodingId::EucKr,
        "windows-1252" => EncodingId::Windows1252,
        "windows-1250" => EncodingId::Windows1250,
        "windows-1251" => EncodingId::Windows1251,
        "windows-1256" => EncodingId::Windows1256,
        "ISO-8859-2" => EncodingId::Iso88592,
        "ISO-8859-5" => EncodingId::Iso88595,
        "KOI8-R" => EncodingId::Koi8R,
        "KOI8-U" => EncodingId::Koi8U,
        "macintosh" => EncodingId::Macintosh,
        _ => EncodingId::Gb18030,
    }
}
