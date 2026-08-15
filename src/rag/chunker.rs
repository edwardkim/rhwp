//! (leaf) HWP/HWPX 문서를 **LLM-ready RAG 청크**로 조립하는 엔진.
//!
//! 상위 개요는 [`crate::rag`] 모듈 문서를 본다. 이 파일은 순수 로직(토큰 추정·표
//! 선형화·구조 인지 청킹)만 담아 `rustfmt` 대상이 된다.
//!
//! 재파싱하지 않는다 — rhwp 가 이미 만든 IR 을 그대로 소비한다.
//! - 제목 계층: [`build_structure`] (조판부호·개요/조문 판정 그대로)
//! - 표 격자: [`extract_tables`] (앵커 셀 + 병합 span, 픽셀 추측 없음)

use serde::Serialize;

use crate::document_core::queries::structure::{build_structure, StructureMode, StructureNode};
use crate::document_core::queries::table_extract::{extract_tables, TableGrid};
use crate::model::document::Document;

/// `tokenEstimate` 가 쓰는 결정론적 휴리스틱의 이름.
///
/// **실제 토크나이저가 아니다.** 코드포인트 하나가 CJK 면 1 토큰, 그 밖의 비공백
/// 문자는 4 글자당 1 토큰으로 센다. 그래서 봉투의 필드 이름도 `tokens` 가 아니라
/// `tokenEstimate` 다 — 값은 근삿값이다.
pub const TOKEN_ESTIMATOR: &str = "cjk1-latin4-v1";

/// 표 하나를 조밀 격자로 펼칠 때 허용하는 최대 칸 수.
///
/// 손상·악의적 문서가 `row_count`/`col_count` 에 거대한 값을 넣어 두면 조밀 격자
/// 할당이 메모리를 터뜨린다. 상한을 넘으면 앵커 셀만 나열하는 폴백으로 내려간다.
const DENSE_GRID_CELL_CAP: usize = 200_000;

/// 청크 조립 옵션.
#[derive(Debug, Clone, Copy)]
pub struct ChunkOptions {
    /// 청크 하나의 `text` 가 목표로 하는 토큰 예산(추정치 기준).
    pub max_tokens: usize,
    /// 제목 계층 판정 방식 — `export-structure` 와 같은 모드를 그대로 쓴다.
    pub mode: StructureMode,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            mode: StructureMode::Auto,
        }
    }
}

/// 코드포인트가 CJK(한중일) 계열인지 — 토큰 추정에서 1 글자 = 1 토큰으로 센다.
fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x11FF   // 한글 자모
        | 0x3040..=0x30FF // 히라가나·가타카나
        | 0x3130..=0x318F // 한글 호환 자모
        | 0x3400..=0x4DBF // CJK 확장 A
        | 0x4E00..=0x9FFF // CJK 통합 한자
        | 0xAC00..=0xD7A3 // 한글 음절
        | 0xF900..=0xFAFF // CJK 호환 한자
        | 0xFF00..=0xFFEF // 반각·전각 형태
    )
}

/// 결정론적 토큰 수 **추정**. [`TOKEN_ESTIMATOR`] 참조 — 실제 토크나이저가 아니다.
pub fn estimate_tokens(text: &str) -> usize {
    let mut cjk = 0usize;
    let mut other = 0usize;
    for c in text.chars() {
        if is_cjk(c) {
            cjk += 1;
        } else if !c.is_whitespace() {
            other += 1;
        }
    }
    cjk + other.div_ceil(4)
}

/// 청크 내용 구성.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkKind {
    /// 문단 텍스트만.
    Text,
    /// 표(선형화)만.
    Table,
    /// 문단과 표가 함께.
    Mixed,
}

/// 청크에 실린 표 하나의 **메타데이터**(문서 텍스트는 담지 않는다 — 본문은 `text` 에 있다).
#[derive(Debug, Clone, Serialize)]
pub struct ChunkTableRef {
    /// `export-tables` 의 문서 내 표 순번(0부터).
    pub index: usize,
    /// 표가 놓인 구역 인덱스.
    pub section: usize,
    /// 표를 담은 문단 인덱스 — 역참조·인용용 주소.
    pub paragraph: usize,
    /// 행 수.
    pub rows: u16,
    /// 열 수.
    pub cols: u16,
    /// 이 파트에서 반복해 실은 머리 행 수.
    #[serde(rename = "headerRowCount")]
    pub header_row_count: usize,
    /// 큰 표가 여러 청크로 쪼개졌을 때의 1 기준 파트 번호.
    pub part: usize,
    /// 이 표의 총 파트 수.
    #[serde(rename = "partCount")]
    pub part_count: usize,
    /// 표가 쪼개져 머리 행을 되풀이했는가.
    #[serde(rename = "headerRepeated")]
    pub header_repeated: bool,
}

/// RAG 청크 하나.
#[derive(Debug, Clone, Serialize)]
pub struct LlmChunk {
    /// 문서 전체에서의 0 기준 청크 순번.
    #[serde(rename = "chunkIndex")]
    pub chunk_index: usize,
    /// 루트부터 이 청크가 속한 제목까지의 경로(예: `["제3장", "제2절"]`).
    /// 청크를 페이지 밖에서도 자기완결로 만든다.
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
    /// 소속 제목의 계층 깊이(서문은 0).
    #[serde(rename = "headingLevel")]
    pub heading_level: u8,
    /// 소속 제목이 놓인 구역 인덱스(서문 청크는 없음).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<usize>,
    /// 소속 제목이 놓인 문단 인덱스(서문 청크는 없음).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<usize>,
    /// 내용 구성.
    pub kind: ChunkKind,
    /// `text` 의 토큰 수 **추정치**([`TOKEN_ESTIMATOR`]).
    #[serde(rename = "tokenEstimate")]
    pub token_estimate: usize,
    /// 같은 제목(절)이 여러 청크로 나뉠 때의 1 기준 파트 번호.
    pub part: usize,
    /// 그 제목이 나뉜 총 청크 수.
    #[serde(rename = "partCount")]
    pub part_count: usize,
    /// 청크 본문 — 문단 텍스트와 선형화된 표. **문서 파생(신뢰 불가)** 값이다.
    pub text: String,
    /// 이 청크가 품은 표들의 메타데이터.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<ChunkTableRef>,
}

impl LlmChunk {
    /// 이 청크에 **실제로 실린** 문서 파생 필드 경로들.
    ///
    /// 봉투 출처 계약(`mydocs/tech/envelope_provenance.md`)을 소비한다 — 값을 담은
    /// 경로만 남긴다(선언을 그대로 베끼지 않는다). RAG 청크는 프롬프트에 이어 붙는
    /// 주입면 그 자체이므로, 소비자가 이 값을 **데이터로 격리**하도록 표지를 싣는다.
    pub fn untrusted_fields(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if !self.heading_path.is_empty() {
            fields.push("headingPath");
        }
        if !self.text.is_empty() {
            fields.push("text");
        }
        fields
    }
}

// ── 내부 조립 ───────────────────────────────────────────────────────────────

/// 제목 트리를 평탄화한 한 조각(제목 하나 + 그에 귀속된 본문·표).
struct Segment {
    heading_path: Vec<String>,
    heading_level: u8,
    section: Option<usize>,
    paragraph: Option<usize>,
    paras: Vec<String>,
    /// `tables` 벡터에서의 인덱스.
    tables: Vec<usize>,
}

impl Segment {
    /// 표 배치·정렬에 쓰는 앵커. 서문(주소 없음)은 문서 맨 앞 `(0, 0)` 으로 본다.
    fn anchor(&self) -> (usize, usize) {
        (self.section.unwrap_or(0), self.paragraph.unwrap_or(0))
    }
}

/// 제목 트리를 DFS 전위 순회로 평탄화한다 — 제목의 문서 순서를 그대로 보존한다.
fn flatten_nodes(nodes: &[StructureNode], path: &[String], out: &mut Vec<Segment>) {
    for node in nodes {
        let mut heading_path = path.to_vec();
        heading_path.push(node.heading.clone());
        out.push(Segment {
            heading_path: heading_path.clone(),
            heading_level: node.level,
            section: Some(node.section),
            paragraph: Some(node.paragraph),
            paras: node.body.clone(),
            tables: Vec::new(),
        });
        flatten_nodes(&node.children, &heading_path, out);
    }
}

/// 각 표를 "그 위치를 읽기 순서상 감싸는 가장 깊은 제목"에 귀속시킨다.
///
/// 세그먼트는 앵커 오름차순(= 제목 문서 순서)이므로, 표 위치 이하인 **마지막**
/// 세그먼트가 주인이다. 첫 제목보다 앞선 표를 받을 서문 세그먼트가 없으면 만든다.
fn assign_tables(segments: &mut Vec<Segment>, tables: &[TableGrid]) {
    if tables.is_empty() {
        return;
    }
    let earliest = tables
        .iter()
        .map(|t| (t.section, t.paragraph))
        .min()
        .expect("tables non-empty");
    let need_leading_preamble = segments
        .first()
        .is_none_or(|s| s.section.is_some() && earliest < s.anchor());
    if need_leading_preamble {
        segments.insert(
            0,
            Segment {
                heading_path: Vec::new(),
                heading_level: 0,
                section: None,
                paragraph: None,
                paras: Vec::new(),
                tables: Vec::new(),
            },
        );
    }
    for (table_index, table) in tables.iter().enumerate() {
        let pos = (table.section, table.paragraph);
        let owner = segments
            .iter()
            .rposition(|s| s.anchor() <= pos)
            .unwrap_or(0);
        segments[owner].tables.push(table_index);
    }
}

/// 셀·캡션 텍스트를 Markdown 표 한 칸에 안전하게 넣도록 정리한다.
fn sanitize_cell(text: &str) -> String {
    text.replace('\r', "")
        .replace('\n', " ")
        .replace('|', "\\|")
        .trim()
        .to_string()
}

/// 표 하나를 Markdown 텍스트 파트들로 선형화한다.
///
/// - 머리 행을 보존하고, 병합 셀은 앵커 칸에 `[병합 R×C]` 로 주석한다(덮인 칸은 빈 칸).
/// - 예산을 넘는 큰 표는 **행 단위로만** 쪼개고(절대 행 중간을 자르지 않는다) 파트마다
///   머리 행을 되풀이한다.
///
/// 반환: `(파트 텍스트, 표 메타, 토큰 추정)` 목록. 항상 최소 1개.
fn linearize_table_parts(
    grid: &TableGrid,
    max_tokens: usize,
) -> Vec<(String, ChunkTableRef, usize)> {
    let rows = grid.rows as usize;
    let cols = grid.cols as usize;
    let caption = grid
        .caption
        .as_deref()
        .map(sanitize_cell)
        .filter(|s| !s.is_empty());

    // 폴백 — 격자가 비었거나 병적으로 크면 앵커 셀만 나열한다(행 단위 원자, 미분할).
    if rows == 0 || cols == 0 || rows.saturating_mul(cols) > DENSE_GRID_CELL_CAP {
        let mut lines = Vec::new();
        if let Some(cap) = &caption {
            lines.push(format!("[표] {cap}"));
        }
        for cell in &grid.cells {
            lines.push(format!(
                "({}, {}) {}",
                cell.row,
                cell.col,
                sanitize_cell(&cell.text)
            ));
        }
        let text = lines.join("\n");
        let tokens = estimate_tokens(&text);
        return vec![(
            text,
            ChunkTableRef {
                index: grid.index,
                section: grid.section,
                paragraph: grid.paragraph,
                rows: grid.rows,
                cols: grid.cols,
                header_row_count: 0,
                part: 1,
                part_count: 1,
                header_repeated: false,
            },
            tokens,
        )];
    }

    // 조밀 격자로 펼친다 — 병합 앵커 텍스트를 제자리에, 덮인 칸은 빈 문자열.
    let mut dense = vec![vec![String::new(); cols]; rows];
    let mut header_flags = vec![false; rows];
    for cell in &grid.cells {
        let r = cell.row as usize;
        let c = cell.col as usize;
        if r >= rows || c >= cols {
            continue;
        }
        let mut text = sanitize_cell(&cell.text);
        if cell.row_span > 1 || cell.col_span > 1 {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&format!("[병합 {}×{}]", cell.row_span, cell.col_span));
        }
        dense[r][c] = text;
        if cell.is_header {
            let end = (r + cell.row_span as usize).min(rows);
            for flag in header_flags.iter_mut().take(end).skip(r) {
                *flag = true;
            }
        }
    }

    let row_line = |cells: &[String]| -> String { format!("| {} |", cells.join(" | ")) };
    let all_rows: Vec<String> = dense.iter().map(|r| row_line(r)).collect();

    // 선두의 연속된 머리 행 수. 없으면 첫 행을 머리로 삼아(맥락 보존) 항상 유효한 Markdown 표를 낸다.
    let mut header_rows = 0usize;
    while header_rows < rows && header_flags[header_rows] {
        header_rows += 1;
    }
    if header_rows == 0 {
        header_rows = 1; // rows >= 1 은 위에서 보장됨
    }

    let separator = format!("| {} |", vec!["---"; cols].join(" | "));
    let mut header_block: Vec<String> = Vec::new();
    if let Some(cap) = &caption {
        header_block.push(format!("[표] {cap}"));
    }
    header_block.extend(all_rows[..header_rows].iter().cloned());
    header_block.push(separator);
    let header_text = header_block.join("\n");
    let header_tokens = estimate_tokens(&header_text);

    // 데이터 행을 예산 안에서 파트로 묶는다 — 행 중간은 절대 자르지 않는다.
    let data_rows = &all_rows[header_rows..];
    let mut parts_rows: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut cur_tokens = header_tokens;
    for line in data_rows {
        let line_tokens = estimate_tokens(line);
        if !cur.is_empty() && cur_tokens + line_tokens > max_tokens {
            parts_rows.push(std::mem::take(&mut cur));
            cur_tokens = header_tokens;
        }
        cur.push(line.clone());
        cur_tokens += line_tokens;
    }
    if !cur.is_empty() || parts_rows.is_empty() {
        parts_rows.push(cur);
    }

    let part_count = parts_rows.len();
    let header_repeated = part_count > 1;
    parts_rows
        .into_iter()
        .enumerate()
        .map(|(k, rows_slice)| {
            let mut lines = header_block.clone();
            lines.extend(rows_slice);
            let text = lines.join("\n");
            let tokens = estimate_tokens(&text);
            (
                text,
                ChunkTableRef {
                    index: grid.index,
                    section: grid.section,
                    paragraph: grid.paragraph,
                    rows: grid.rows,
                    cols: grid.cols,
                    header_row_count: header_rows,
                    part: k + 1,
                    part_count,
                    header_repeated,
                },
                tokens,
            )
        })
        .collect()
}

/// 조립 중인 청크 버퍼.
struct ChunkBuf {
    heading_path: Vec<String>,
    heading_level: u8,
    section: Option<usize>,
    paragraph: Option<usize>,
    pieces: Vec<String>,
    tokens: usize,
    tables: Vec<ChunkTableRef>,
    has_text: bool,
    has_table: bool,
}

impl ChunkBuf {
    fn new(seg: &Segment) -> Self {
        Self {
            heading_path: seg.heading_path.clone(),
            heading_level: seg.heading_level,
            section: seg.section,
            paragraph: seg.paragraph,
            pieces: Vec::new(),
            tokens: 0,
            tables: Vec::new(),
            has_text: false,
            has_table: false,
        }
    }

    fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    fn push_text(&mut self, text: String, tokens: usize) {
        self.pieces.push(text);
        self.tokens += tokens;
        self.has_text = true;
    }

    fn push_table(&mut self, text: String, table_ref: ChunkTableRef, tokens: usize) {
        self.pieces.push(text);
        self.tokens += tokens;
        self.tables.push(table_ref);
        self.has_table = true;
    }

    /// 버퍼를 청크로 굳혀 `out` 에 밀어 넣고 버퍼를 비운다.
    fn flush(&mut self, out: &mut Vec<LlmChunk>) {
        if self.is_empty() {
            return;
        }
        let kind = match (self.has_text, self.has_table) {
            (true, true) => ChunkKind::Mixed,
            (false, true) => ChunkKind::Table,
            _ => ChunkKind::Text,
        };
        let text = self.pieces.join("\n\n");
        let token_estimate = estimate_tokens(&text);
        out.push(LlmChunk {
            chunk_index: 0,
            heading_path: self.heading_path.clone(),
            heading_level: self.heading_level,
            section: self.section,
            paragraph: self.paragraph,
            kind,
            token_estimate,
            part: 0,
            part_count: 0,
            text,
            tables: std::mem::take(&mut self.tables),
        });
        self.pieces.clear();
        self.tokens = 0;
        self.has_text = false;
        self.has_table = false;
    }
}

/// 세그먼트 하나를 청크들로 내보낸다.
///
/// 자연 경계(제목/문단/표)에서만 나눈다. 문단은 문단 경계에서만, 표는 절대 행 중간을
/// 자르지 않는다. 한 세그먼트가 여러 청크가 되면 각 청크가 같은 `headingPath` 를
/// 되풀이해 자기완결을 유지한다.
fn emit_segment(seg: &Segment, tables: &[TableGrid], max_tokens: usize, out: &mut Vec<LlmChunk>) {
    let start = out.len();
    let mut buf = ChunkBuf::new(seg);

    for para in &seg.paras {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tokens = estimate_tokens(trimmed);
        if !buf.is_empty() && buf.tokens + tokens > max_tokens {
            buf.flush(out);
        }
        buf.push_text(trimmed.to_string(), tokens);
        if buf.tokens > max_tokens {
            // 단일 문단이 예산을 넘으면 그 자체로 한 청크(문단 경계는 지킨다).
            buf.flush(out);
        }
    }

    // 표는 세그먼트 안에서 문서 순서(표 index)대로. 본문 뒤에 온다.
    let mut table_indices = seg.tables.clone();
    table_indices.sort_unstable();
    for table_index in table_indices {
        let parts = linearize_table_parts(&tables[table_index], max_tokens);
        if parts.len() == 1 {
            let (text, table_ref, tokens) = parts.into_iter().next().expect("one part");
            if !buf.is_empty() && buf.tokens + tokens > max_tokens {
                buf.flush(out);
            }
            buf.push_table(text, table_ref, tokens);
        } else {
            // 여러 파트로 쪼개진 큰 표는 각 파트가 독립 청크다.
            buf.flush(out);
            for (text, table_ref, tokens) in parts {
                let mut standalone = ChunkBuf::new(seg);
                standalone.push_table(text, table_ref, tokens);
                standalone.flush(out);
            }
        }
    }

    buf.flush(out);

    // 이 세그먼트가 만든 청크들에 파트 번호를 매긴다.
    let produced = out.len() - start;
    for (k, chunk) in out[start..].iter_mut().enumerate() {
        chunk.part = k + 1;
        chunk.part_count = produced;
    }
}

/// 문서를 결정론적 RAG 청크 목록으로 조립한다.
///
/// 재파싱하지 않는다 — [`build_structure`] 의 제목 계층과 [`extract_tables`] 의 표
/// 격자를 그대로 소비한다. 같은 입력·옵션이면 바이트까지 같은 결과를 낸다.
pub fn build_chunks(doc: &Document, opts: &ChunkOptions) -> Vec<LlmChunk> {
    let max_tokens = opts.max_tokens.max(1);
    let structure = build_structure(doc, opts.mode);
    let tables = extract_tables(doc);

    let mut segments: Vec<Segment> = Vec::new();
    if !structure.preamble.is_empty() {
        segments.push(Segment {
            heading_path: Vec::new(),
            heading_level: 0,
            section: None,
            paragraph: None,
            paras: structure.preamble.clone(),
            tables: Vec::new(),
        });
    }
    flatten_nodes(&structure.roots, &[], &mut segments);
    assign_tables(&mut segments, &tables);

    let mut chunks: Vec<LlmChunk> = Vec::new();
    for seg in &segments {
        emit_segment(seg, &tables, max_tokens, &mut chunks);
    }
    for (i, chunk) in chunks.iter_mut().enumerate() {
        chunk.chunk_index = i;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::Control;
    use crate::model::document::{Document, Section};
    use crate::model::paragraph::Paragraph;
    use crate::model::style::{HeadType, ParaShape};
    use crate::model::table::{Cell, Table};

    /// para_shape 0 = 본문, 1 = 개요(Outline) 제목.
    fn doc_with(paras: Vec<Paragraph>) -> Document {
        let mut doc = Document::default();
        doc.doc_info.para_shapes.push(ParaShape::default()); // id 0: 본문
        doc.doc_info.para_shapes.push(ParaShape {
            head_type: HeadType::Outline,
            para_level: 0,
            ..ParaShape::default()
        }); // id 1: 제목
        doc.sections.push(Section {
            paragraphs: paras,
            ..Section::default()
        });
        doc
    }

    fn para(text: &str, shape: u16) -> Paragraph {
        Paragraph {
            text: text.to_string(),
            para_shape_id: shape,
            ..Paragraph::new_empty()
        }
    }

    fn heading(text: &str) -> Paragraph {
        para(text, 1)
    }

    fn body(text: &str) -> Paragraph {
        para(text, 0)
    }

    /// 앵커 셀만 가진 표를 하나 담은 문단.
    fn para_with_table(rows: u16, cols: u16, cells: Vec<Cell>) -> Paragraph {
        let table = Table {
            row_count: rows,
            col_count: cols,
            cells,
            ..Table::default()
        };
        let mut p = Paragraph::new_empty();
        p.controls.push(Control::Table(Box::new(table)));
        p
    }

    fn cell(row: u16, col: u16, text: &str, is_header: bool) -> Cell {
        Cell {
            row,
            col,
            row_span: 1,
            col_span: 1,
            is_header,
            paragraphs: vec![body(text)],
            ..Cell::default()
        }
    }

    #[test]
    fn empty_document_yields_no_chunks_without_panicking() {
        let doc = Document::default();
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        assert!(chunks.is_empty());
    }

    #[test]
    fn degenerate_paragraphs_do_not_panic() {
        // 빈/공백 문단만 있는 문서.
        let doc = doc_with(vec![body(""), body("   "), body("\n")]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        assert!(chunks.iter().all(|c| !c.text.is_empty()));
    }

    #[test]
    fn token_estimate_is_labelled_a_heuristic() {
        // CJK 는 글자당 1, 라틴은 4글자당 1.
        assert_eq!(estimate_tokens("가나다"), 3);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(TOKEN_ESTIMATOR, "cjk1-latin4-v1");
    }

    #[test]
    fn multi_paragraph_chunks_stay_within_budget() {
        // 짧은 문단 여러 개가 예산 안에서 묶이되, 묶인 청크는 예산(± 휴리스틱)을
        // 넘지 않는다. "\n\n" 이 있으면 여러 문단이 한 청크로 묶였다는 뜻이다.
        let mut paras = vec![heading("장")];
        for _ in 0..20 {
            paras.push(body("가나다라마")); // 각 5 토큰
        }
        let doc = doc_with(paras);
        let opts = ChunkOptions {
            max_tokens: 12,
            mode: StructureMode::Auto,
        };
        let chunks = build_chunks(&doc, &opts);
        assert!(chunks.len() > 1, "예산이 쪼개기를 유발해야 한다");
        for c in &chunks {
            if c.text.contains("\n\n") {
                assert!(
                    c.token_estimate <= opts.max_tokens,
                    "묶인 청크가 예산 초과: {} > {}",
                    c.token_estimate,
                    opts.max_tokens
                );
            }
        }
    }

    #[test]
    fn chunk_boundaries_respect_headings() {
        let doc = doc_with(vec![
            heading("제1장 총칙"),
            body("가나다라마바사"),
            heading("제2장 벌칙"),
            body("아자차카타파하"),
        ]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        // 서로 다른 제목의 본문이 한 청크에 섞이지 않는다.
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].heading_path, vec!["제1장 총칙"]);
        assert!(chunks[0].text.contains("가나다라마바사"));
        assert!(!chunks[0].text.contains("아자차카타파하"));
        assert_eq!(chunks[1].heading_path, vec!["제2장 벌칙"]);
        // 전역 순번은 0,1.
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
    }

    #[test]
    fn nested_headings_build_a_path() {
        let mut doc = doc_with(vec![heading("제1장"), heading("제1절"), body("본문")]);
        // 제1절을 한 단계 더 깊은 개요 수준으로.
        doc.doc_info.para_shapes.push(ParaShape {
            head_type: HeadType::Outline,
            para_level: 1,
            ..ParaShape::default()
        });
        doc.sections[0].paragraphs[1].para_shape_id = 2;
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        let deep = chunks
            .iter()
            .find(|c| c.text.contains("본문"))
            .expect("본문 청크");
        assert_eq!(deep.heading_path, vec!["제1장", "제1절"]);
        assert_eq!(deep.heading_level, 2);
    }

    #[test]
    fn large_section_splits_into_parts_with_repeated_heading_path() {
        // 예산을 작게 잡아 본문이 여러 문단에서 쪼개지게 한다.
        let doc = doc_with(vec![
            heading("장"),
            body("가나다라마바사아자차"), // 10 토큰
            body("카타파하거너더러머버"), // 10 토큰
            body("서어저처커터퍼허고노"), // 10 토큰
        ]);
        let opts = ChunkOptions {
            max_tokens: 12,
            mode: StructureMode::Auto,
        };
        let chunks = build_chunks(&doc, &opts);
        assert!(chunks.len() >= 2, "쪼개져야 한다: {}", chunks.len());
        // 모든 파트가 같은 headingPath 를 되풀이한다(자기완결).
        for c in &chunks {
            assert_eq!(c.heading_path, vec!["장"]);
        }
        assert_eq!(chunks[0].part, 1);
        assert_eq!(chunks[0].part_count, chunks.len());
    }

    #[test]
    fn table_never_splits_mid_row_and_repeats_header() {
        // 머리 1행 + 데이터 6행, 예산을 작게 잡아 표를 쪼갠다.
        let mut cells = vec![cell(0, 0, "이름", true), cell(0, 1, "값", true)];
        for r in 1..=6u16 {
            cells.push(cell(r, 0, &format!("행{r}"), false));
            cells.push(cell(r, 1, &format!("데이터{r}"), false));
        }
        let doc = doc_with(vec![heading("표 절"), para_with_table(7, 2, cells)]);
        let opts = ChunkOptions {
            max_tokens: 20,
            mode: StructureMode::Auto,
        };
        let chunks = build_chunks(&doc, &opts);
        let table_chunks: Vec<&LlmChunk> = chunks.iter().filter(|c| !c.tables.is_empty()).collect();
        assert!(table_chunks.len() >= 2, "표가 쪼개져야 한다");
        for c in &table_chunks {
            // 각 파트가 머리 행("이름"/"값")을 되풀이한다.
            assert!(c.text.contains("이름"), "머리 반복 누락: {}", c.text);
            assert!(c.tables[0].header_repeated);
            // 셀 값이 행 단위로 온전하다 — "행N" 과 "데이터N" 이 같은 줄에 있다.
            for line in c.text.lines().filter(|l| l.contains("행")) {
                if let Some(num) = line.split('행').nth(1).and_then(|s| s.chars().next()) {
                    assert!(
                        line.contains(&format!("데이터{num}")),
                        "행이 중간에서 잘렸다: {line}"
                    );
                }
            }
        }
    }

    #[test]
    fn merged_cells_are_annotated() {
        let cells = vec![
            Cell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 2,
                is_header: true,
                paragraphs: vec![body("병합머리")],
                ..Cell::default()
            },
            cell(1, 0, "좌", false),
            cell(1, 1, "우", false),
        ];
        let doc = doc_with(vec![heading("표"), para_with_table(2, 2, cells)]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        let table_chunk = chunks
            .iter()
            .find(|c| !c.tables.is_empty())
            .expect("표 청크");
        assert!(
            table_chunk.text.contains("[병합 1×2]"),
            "병합 주석 누락: {}",
            table_chunk.text
        );
    }

    #[test]
    fn every_chunk_declares_untrusted_content() {
        let doc = doc_with(vec![heading("장"), body("본문 텍스트")]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        assert!(!chunks.is_empty());
        for c in &chunks {
            let fields = c.untrusted_fields();
            assert!(fields.contains(&"text"), "text 표지 누락");
            assert!(fields.contains(&"headingPath"), "headingPath 표지 누락");
        }
    }

    #[test]
    fn output_is_deterministic_byte_for_byte() {
        let doc = doc_with(vec![
            heading("제1장"),
            body("가나다라마바사"),
            para_with_table(
                2,
                2,
                vec![cell(0, 0, "머리", true), cell(1, 0, "값", false)],
            ),
            heading("제2장"),
            body("아자차카타파하"),
        ]);
        let opts = ChunkOptions::default();
        let a = serde_json::to_string(&build_chunks(&doc, &opts)).unwrap();
        let b = serde_json::to_string(&build_chunks(&doc, &opts)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn all_body_text_and_tables_are_covered() {
        // 라운드트립: 구조·표에서 나온 모든 문자열이 청크 어딘가에 있다(무손실).
        let doc = doc_with(vec![
            body("서문문단"),
            heading("제1장 제목"),
            body("첫째 본문"),
            body("둘째 본문"),
            para_with_table(
                2,
                1,
                vec![cell(0, 0, "표머리", true), cell(1, 0, "표값", false)],
            ),
        ]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        let haystack: String = chunks
            .iter()
            .map(|c| format!("{} {}", c.heading_path.join(" "), c.text))
            .collect::<Vec<_>>()
            .join("\n");
        for needle in [
            "서문문단",
            "제1장 제목",
            "첫째 본문",
            "둘째 본문",
            "표머리",
            "표값",
        ] {
            assert!(haystack.contains(needle), "누락: {needle}");
        }
    }

    #[test]
    fn table_before_first_heading_lands_in_a_preamble_chunk() {
        let doc = doc_with(vec![
            para_with_table(1, 1, vec![cell(0, 0, "선행표", true)]),
            heading("제1장"),
            body("본문"),
        ]);
        let chunks = build_chunks(&doc, &ChunkOptions::default());
        let table_chunk = chunks
            .iter()
            .find(|c| !c.tables.is_empty())
            .expect("표 청크");
        assert!(
            table_chunk.heading_path.is_empty(),
            "서문 표는 제목 경로가 비어야 한다"
        );
        assert!(table_chunk.text.contains("선행표"));
    }
}
