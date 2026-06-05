use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EncodingId {
    #[serde(rename = "UTF-8")]
    Utf8,
    #[serde(rename = "UTF-8 BOM")]
    Utf8Bom,
    #[serde(rename = "GBK")]
    Gbk,
    #[serde(rename = "GB18030")]
    Gb18030,
    #[serde(rename = "BIG5")]
    Big5,
    #[serde(rename = "UTF-16 LE")]
    Utf16Le,
    #[serde(rename = "UTF-16 BE")]
    Utf16Be,
    #[serde(rename = "GB2312")]
    Gb2312,
    #[serde(rename = "Shift_JIS")]
    ShiftJis,
    #[serde(rename = "EUC-JP")]
    EucJp,
    #[serde(rename = "ISO-2022-JP")]
    Iso2022Jp,
    #[serde(rename = "EUC-KR")]
    EucKr,
    #[serde(rename = "windows-1252")]
    Windows1252,
    #[serde(rename = "windows-1250")]
    Windows1250,
    #[serde(rename = "windows-1251")]
    Windows1251,
    #[serde(rename = "windows-1256")]
    Windows1256,
    #[serde(rename = "ISO-8859-2")]
    Iso88592,
    #[serde(rename = "ISO-8859-5")]
    Iso88595,
    #[serde(rename = "KOI8-R")]
    Koi8R,
    #[serde(rename = "KOI8-U")]
    Koi8U,
    #[serde(rename = "macintosh")]
    Macintosh,
}

#[derive(Clone, Debug, Serialize)]
pub struct EncodingOption {
    pub id: EncodingId,
    pub label: String,
    pub category: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecodeResult {
    pub text: String,
    pub encoding: EncodingId,
    pub detected_encoding: Option<EncodingId>,
    pub file_size: u64,
    pub had_errors: bool,
    pub replacement_count: usize,
    pub total_chars: usize,
    pub bom: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SaveResult {
    pub path: String,
    pub file_size: u64,
    pub revision: u64,
}

impl EncodingId {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Gbk => "GBK",
            Self::Gb18030 => "GB18030",
            Self::Big5 => "BIG5",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16Be => "UTF-16 BE",
            Self::Gb2312 => "GB2312",
            Self::ShiftJis => "Shift_JIS",
            Self::EucJp => "EUC-JP",
            Self::Iso2022Jp => "ISO-2022-JP",
            Self::EucKr => "EUC-KR",
            Self::Windows1252 => "windows-1252",
            Self::Windows1250 => "windows-1250",
            Self::Windows1251 => "windows-1251",
            Self::Windows1256 => "windows-1256",
            Self::Iso88592 => "ISO-8859-2",
            Self::Iso88595 => "ISO-8859-5",
            Self::Koi8R => "KOI8-R",
            Self::Koi8U => "KOI8-U",
            Self::Macintosh => "macintosh",
        }
    }

    pub fn common_encodings() -> Vec<EncodingId> {
        vec![
            Self::Utf8,
            Self::Utf8Bom,
            Self::Gbk,
            Self::Gb18030,
            Self::Big5,
            Self::Utf16Le,
            Self::Utf16Be,
        ]
    }

    pub fn extra_encodings() -> Vec<EncodingId> {
        vec![
            Self::Gb2312,
            Self::ShiftJis,
            Self::EucJp,
            Self::Iso2022Jp,
            Self::EucKr,
            Self::Windows1252,
            Self::Windows1250,
            Self::Windows1251,
            Self::Windows1256,
            Self::Iso88592,
            Self::Iso88595,
            Self::Koi8R,
            Self::Koi8U,
            Self::Macintosh,
        ]
    }

    pub fn all_options() -> Vec<EncodingOption> {
        let mut options: Vec<EncodingOption> = Self::common_encodings()
            .into_iter()
            .map(|id| {
                let label = id.label().to_string();
                EncodingOption {
                    id,
                    label,
                    category: "common".to_string(),
                }
            })
            .collect();

        let extra: Vec<EncodingOption> = Self::extra_encodings()
            .into_iter()
            .map(|id| {
                let label = id.label().to_string();
                EncodingOption {
                    id,
                    label,
                    category: "extra".to_string(),
                }
            })
            .collect();

        options.extend(extra);
        options
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "UTF-8" => Some(Self::Utf8),
            "UTF-8 BOM" => Some(Self::Utf8Bom),
            "GBK" => Some(Self::Gbk),
            "GB18030" => Some(Self::Gb18030),
            "BIG5" => Some(Self::Big5),
            "UTF-16 LE" => Some(Self::Utf16Le),
            "UTF-16 BE" => Some(Self::Utf16Be),
            "GB2312" => Some(Self::Gb2312),
            "Shift_JIS" => Some(Self::ShiftJis),
            "EUC-JP" => Some(Self::EucJp),
            "ISO-2022-JP" => Some(Self::Iso2022Jp),
            "EUC-KR" => Some(Self::EucKr),
            "windows-1252" => Some(Self::Windows1252),
            "windows-1250" => Some(Self::Windows1250),
            "windows-1251" => Some(Self::Windows1251),
            "windows-1256" => Some(Self::Windows1256),
            "ISO-8859-2" => Some(Self::Iso88592),
            "ISO-8859-5" => Some(Self::Iso88595),
            "KOI8-R" => Some(Self::Koi8R),
            "KOI8-U" => Some(Self::Koi8U),
            "macintosh" => Some(Self::Macintosh),
            _ => None,
        }
    }

    pub fn encoding_rs_label(&self) -> Option<&'static str> {
        match self {
            Self::Utf8 | Self::Utf8Bom => Some("UTF-8"),
            Self::Gbk | Self::Gb2312 => Some("GBK"),
            Self::Gb18030 => Some("GB18030"),
            Self::Big5 => Some("BIG5"),
            Self::ShiftJis => Some("SHIFT_JIS"),
            Self::EucJp => Some("EUC-JP"),
            Self::Iso2022Jp => Some("ISO-2022-JP"),
            Self::EucKr => Some("EUC-KR"),
            Self::Windows1252 => Some("windows-1252"),
            Self::Windows1250 => Some("windows-1250"),
            Self::Windows1251 => Some("windows-1251"),
            Self::Windows1256 => Some("windows-1256"),
            Self::Iso88592 => Some("ISO-8859-2"),
            Self::Iso88595 => Some("ISO-8859-5"),
            Self::Koi8R => Some("KOI8-R"),
            Self::Koi8U => Some("KOI8-U"),
            Self::Macintosh => Some("macintosh"),
            Self::Utf16Le | Self::Utf16Be => None,
        }
    }

    pub fn writes_bom(&self) -> bool {
        matches!(self, Self::Utf8Bom | Self::Utf16Le | Self::Utf16Be)
    }
}
