//! Residual page catalog. Rows are data; classifiers do not branch on names.

use std::path::Path;
use std::str::FromStr;

use super::ResidualClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorpusId {
    AdminHandbookHwp,
    AdminHandbookHwpx,
    Regulatory76076,
    Issue4090,
    Hwp3Sample16,
    Hwp3ToHwp5_2010,
    Hwp3ToHwp5_2018,
    Hwp3ToHwp5_2020,
    Hwp3ToHwp5_2022,
    Hwp3ToHwp5_2024,
    NoteTailHwp,
    NoteTailHwpx,
    Issue2006,
    Issue4490,
    Issue4491,
}

impl CorpusId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdminHandbookHwp => "admin_hwp",
            Self::AdminHandbookHwpx => "admin_hwpx",
            Self::Regulatory76076 => "reg_76076",
            Self::Issue4090 => "issue4090",
            Self::Hwp3Sample16 => "hwp3_sample16",
            Self::Hwp3ToHwp5_2010 => "hwp3_hwp5_2010",
            Self::Hwp3ToHwp5_2018 => "hwp3_hwp5_2018",
            Self::Hwp3ToHwp5_2020 => "hwp3_hwp5_2020",
            Self::Hwp3ToHwp5_2022 => "hwp3_hwp5_2022",
            Self::Hwp3ToHwp5_2024 => "hwp3_hwp5_2024",
            Self::NoteTailHwp => "note_tail_hwp",
            Self::NoteTailHwpx => "note_tail_hwpx",
            Self::Issue2006 => "issue2006",
            Self::Issue4490 => "issue4490",
            Self::Issue4491 => "issue4491",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "admin_hwp" => Some(Self::AdminHandbookHwp),
            "admin_hwpx" => Some(Self::AdminHandbookHwpx),
            "reg_76076" => Some(Self::Regulatory76076),
            "issue4090" => Some(Self::Issue4090),
            "hwp3_sample16" => Some(Self::Hwp3Sample16),
            "hwp3_hwp5_2010" => Some(Self::Hwp3ToHwp5_2010),
            "hwp3_hwp5_2018" => Some(Self::Hwp3ToHwp5_2018),
            "hwp3_hwp5_2020" => Some(Self::Hwp3ToHwp5_2020),
            "hwp3_hwp5_2022" => Some(Self::Hwp3ToHwp5_2022),
            "hwp3_hwp5_2024" => Some(Self::Hwp3ToHwp5_2024),
            "note_tail_hwp" => Some(Self::NoteTailHwp),
            "note_tail_hwpx" => Some(Self::NoteTailHwpx),
            "issue2006" => Some(Self::Issue2006),
            "issue4490" => Some(Self::Issue4490),
            "issue4491" => Some(Self::Issue4491),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::AdminHandbookHwp,
            Self::AdminHandbookHwpx,
            Self::Regulatory76076,
            Self::Issue4090,
            Self::Hwp3Sample16,
            Self::Hwp3ToHwp5_2010,
            Self::Hwp3ToHwp5_2018,
            Self::Hwp3ToHwp5_2020,
            Self::Hwp3ToHwp5_2022,
            Self::Hwp3ToHwp5_2024,
            Self::NoteTailHwp,
            Self::NoteTailHwpx,
            Self::Issue2006,
            Self::Issue4490,
            Self::Issue4491,
        ]
    }
}

impl FromStr for CorpusId {
    type Err = CatalogLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| CatalogLoadError::UnknownCorpus(s.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Hwp,
    Hwpx,
    PdfOracle,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hwp => "hwp",
            Self::Hwpx => "hwpx",
            Self::PdfOracle => "pdf_oracle",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "hwp" => Some(Self::Hwp),
            "hwpx" => Some(Self::Hwpx),
            "pdf_oracle" | "pdf" => Some(Self::PdfOracle),
            _ => None,
        }
    }
}

/// How a page participates in residual compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsolationPolicy {
    /// Compare this page even if a neighbor fails.
    Independent,
    /// Skip raster if decode fails; keep the document report.
    SkipOnDecodeFail,
    /// Fatal only when every page is skipped.
    DocumentIfAllSkipped,
}

impl IsolationPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::SkipOnDecodeFail => "skip_on_decode_fail",
            Self::DocumentIfAllSkipped => "document_if_all_skipped",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "independent" => Some(Self::Independent),
            "skip_on_decode_fail" => Some(Self::SkipOnDecodeFail),
            "document_if_all_skipped" => Some(Self::DocumentIfAllSkipped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRecord {
    pub corpus: CorpusId,
    pub source_kind: SourceKind,
    pub page_index: u16,
    pub human_page: u16,
    pub locked_count: u16,
    pub residual: ResidualClass,
    pub isolation: IsolationPolicy,
    pub ink_budget_ppm: u32,
    pub hist_l1_budget: u32,
    pub bbox_delta_hu: u32,
    pub font_env_sensitive: bool,
    pub wrap_exclusion_risk: bool,
    pub glyph_paint_risk: bool,
    pub table_place_risk: bool,
    pub stage_ref: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLoadError {
    Empty,
    MissingHeader,
    UnknownCorpus(String),
    BadRow { line: usize, reason: String },
    Io(String),
}

impl core::fmt::Display for CatalogLoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "catalog is empty"),
            Self::MissingHeader => write!(f, "catalog is missing the header row"),
            Self::UnknownCorpus(name) => write!(f, "unknown corpus id: {name}"),
            Self::BadRow { line, reason } => write!(f, "catalog line {line}: {reason}"),
            Self::Io(err) => write!(f, "catalog io: {err}"),
        }
    }
}

impl std::error::Error for CatalogLoadError {}

const REQUIRED_COLUMNS: &[&str] = &[
    "corpus",
    "source_kind",
    "page_index",
    "human_page",
    "locked_count",
    "residual",
    "isolation",
    "ink_budget_ppm",
    "hist_l1",
    "bbox_hu",
    "font_env",
    "wrap_risk",
    "glyph_risk",
    "table_risk",
    "stage",
    "note",
];

pub fn load_page_catalog(text: &str) -> Result<Vec<PageRecord>, CatalogLoadError> {
    let mut lines = text.lines();
    let header = lines.next().ok_or(CatalogLoadError::MissingHeader)?;
    let columns: Vec<&str> = header.split('\t').collect();
    if columns.len() < REQUIRED_COLUMNS.len() {
        return Err(CatalogLoadError::MissingHeader);
    }
    for (index, expected) in REQUIRED_COLUMNS.iter().enumerate() {
        if columns.get(index).copied() != Some(*expected) {
            return Err(CatalogLoadError::BadRow {
                line: 1,
                reason: format!("expected column {expected} at {index}"),
            });
        }
    }

    let mut records = Vec::new();
    for (offset, line) in lines.enumerate() {
        let line_no = offset + 2;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        records.push(parse_row(line_no, line)?);
    }
    if records.is_empty() {
        return Err(CatalogLoadError::Empty);
    }
    Ok(records)
}

pub fn load_page_catalog_from_path(path: &Path) -> Result<Vec<PageRecord>, CatalogLoadError> {
    let text =
        std::fs::read_to_string(path).map_err(|err| CatalogLoadError::Io(err.to_string()))?;
    load_page_catalog(&text)
}

fn parse_row(line_no: usize, line: &str) -> Result<PageRecord, CatalogLoadError> {
    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() < REQUIRED_COLUMNS.len() {
        return Err(CatalogLoadError::BadRow {
            line: line_no,
            reason: format!(
                "expected {} columns, got {}",
                REQUIRED_COLUMNS.len(),
                cols.len()
            ),
        });
    }
    let bad = |reason: String| CatalogLoadError::BadRow {
        line: line_no,
        reason,
    };
    let corpus = CorpusId::parse(cols[0]).ok_or_else(|| bad(format!("corpus {}", cols[0])))?;
    let source_kind =
        SourceKind::parse(cols[1]).ok_or_else(|| bad(format!("source_kind {}", cols[1])))?;
    let page_index = cols[2]
        .parse::<u16>()
        .map_err(|_| bad(format!("page_index {}", cols[2])))?;
    let human_page = cols[3]
        .parse::<u16>()
        .map_err(|_| bad(format!("human_page {}", cols[3])))?;
    let locked_count = cols[4]
        .parse::<u16>()
        .map_err(|_| bad(format!("locked_count {}", cols[4])))?;
    let residual =
        ResidualClass::parse(cols[5]).ok_or_else(|| bad(format!("residual {}", cols[5])))?;
    let isolation =
        IsolationPolicy::parse(cols[6]).ok_or_else(|| bad(format!("isolation {}", cols[6])))?;
    Ok(PageRecord {
        corpus,
        source_kind,
        page_index,
        human_page,
        locked_count,
        residual,
        isolation,
        ink_budget_ppm: parse_u32(cols[7], "ink_budget_ppm", line_no)?,
        hist_l1_budget: parse_u32(cols[8], "hist_l1", line_no)?,
        bbox_delta_hu: parse_u32(cols[9], "bbox_hu", line_no)?,
        font_env_sensitive: parse_flag(cols[10], line_no)?,
        wrap_exclusion_risk: parse_flag(cols[11], line_no)?,
        glyph_paint_risk: parse_flag(cols[12], line_no)?,
        table_place_risk: parse_flag(cols[13], line_no)?,
        stage_ref: cols[14].to_string(),
        note: cols[15].to_string(),
    })
}

fn parse_u32(value: &str, label: &str, line: usize) -> Result<u32, CatalogLoadError> {
    value.parse::<u32>().map_err(|_| CatalogLoadError::BadRow {
        line,
        reason: format!("{label} {value}"),
    })
}

fn parse_flag(value: &str, line: usize) -> Result<bool, CatalogLoadError> {
    match value {
        "0" | "false" | "no" => Ok(false),
        "1" | "true" | "yes" => Ok(true),
        other => Err(CatalogLoadError::BadRow {
            line,
            reason: format!("flag {other}"),
        }),
    }
}
