//! [#2740] IR 필드 전수 스윕 — `Debug` 표현 기반 왕복 불변식 비교기.
//!
//! ## 왜 비교기를 하나 더 만드는가
//!
//! `serializer::hwpx::roundtrip::diff_documents` 는 이미 성숙한 왕복 비교기이고
//! `tests/hwp5_roundtrip_baseline.rs`·`tests/hwpx_roundtrip_baseline.rs` 가 코퍼스
//! 전수 게이트를 XFAIL 래칫과 함께 이미 돌리고 있다. **코퍼스 스윕 자체는 이미 있다.**
//!
//! 없는 것은 *비교 대상의 망라성*이다. `diff_documents` 는 사건 대응으로 누적된
//! 수작업 화이트리스트(22종 variant, #1378~#1403)라서 다음이 통째로 비교되지 않는다.
//!
//! - `CharShape`/`ParaShape`/`BorderFill`/`Numbering`/`Bullet`/`Style`/`Font` 의 **내용**
//!   (개수만 비교하고 필드는 하나도 보지 않는다)
//! - `Paragraph.text`·`para_shape_id`·`style_id`
//! - 표/셀의 속성 전부 (`attr`·`row_count`·`cell_spacing`·`padding`·`zones`·셀 병합/여백/
//!   테두리 등) — 셀은 문단 재귀를 위해 순회만 하고 속성은 보지 않는다
//! - `CommonObjAttr` 의 배치 모델 대부분 (`width_criterion`·`vert_rel_to`·`text_wrap`·
//!   `z_order`·오프셋 등)
//! - 그림 외 도형의 크기·기하
//! - `footnote_shape`/`endnote_shape`/`page_border_fill`
//!
//! 최근 반복 수정된 1속성 왕복 소실(`hp:tbl` widthRelTo/protect/numberingType,
//! `charPr` outline/shadow, 그림·도형 `hp:sz`)이 정확히 이 사각지대에 있다. 즉
//! **게이트가 통과하는데도 같은 부류의 결함이 계속 나오는 이유는 게이트가 그 필드를
//! 볼 수 없기 때문**이다. 사람이 눈으로 찾는 한 이 부류는 계속 재발한다.
//!
//! ## 설계 — 손으로 나열하지 않는 비교
//!
//! 비교 단위를 손으로 적는 대신 각 IR 노드의 `Debug` 파생 표현을 문자열로 얻어
//! 구조적으로 재귀 분해한다. `Debug` 파생은 구조체의 **모든 필드**를 출력하므로
//! IR 에 필드를 추가하면 별도 조치 없이 비교 대상에 편입된다(무유지보수).
//!
//! 척추(순회 경로)는 명시적이되, 큰 노드는 **철저 구조 분해**(`let Paragraph { .. } = p`
//! 를 `..` 없이)로 잡아 필드가 추가되면 **컴파일이 깨지도록** 했다. 즉 사각지대가
//! 조용히 생기지 않는다:
//!
//! - `Paragraph`·`Table`·`Cell` — 철저 구조 분해 (복제 비용 회피 + 컴파일 타임 관문)
//! - 그 밖의 모든 잎 노드 — `Debug` 전수 비교 (런타임 망라)
//!
//! ## 비용 통제
//!
//! `Debug` 문자열화는 [`DEBUG_CAP`] 바이트에서 잘린다([`CappedWriter`] 가 `Err` 를
//! 돌려 파생 `Debug` 를 조기 중단시킨다). 초과분은 비교되지 않으며 이는 명시된 한계다
//! (문단 텍스트/`char_offsets` 가 큰 문단에서 발생 가능).
//!
//! ## 범위 밖 (의도)
//!
//! `raw_stream`·`bin_data_content`·`extra_streams`·`hwpx_aux_entries` 는 원본 바이트
//! 보존 버퍼라 왕복 후 달라지는 것이 **정상**이다(재직렬화 결과이므로). 바이트 보존은
//! `hwp5_roundtrip_batch` 의 C2 BinData 지문이 이미 담당한다.

use std::fmt::{Debug, Write as _};

use crate::model::control::Control;
use crate::model::document::{DocInfo, Document, SectionDef};
use crate::model::paragraph::Paragraph;
use crate::model::shape::ShapeObject;
use crate::model::table::{Cell, Table};

/// `Debug` 문자열화 상한(바이트). 초과 시 그 노드는 앞부분만 비교된다.
const DEBUG_CAP: usize = 96 * 1024;

/// 구조 재귀 최대 깊이 — 병적인 중첩에서 폭주 방지.
const MAX_DEPTH: usize = 14;

/// 발산 보고 시 값 문자열 절단 길이.
const VALUE_CAP: usize = 160;

/// 문서 1건당 수집 상한 — 심하게 깨진 문서에서 메모리 폭주 방지.
pub const MAX_DIVERGENCES: usize = 2000;

/// 왕복 전후 IR 의 단일 필드 발산.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDivergence {
    /// 인덱스를 포함한 정확 경로 (`sections[0].paragraphs[3].controls[1].common.z_order`).
    pub path: String,
    /// 원본 IR 값 (절단됨).
    pub left: String,
    /// 재파싱 IR 값 (절단됨).
    pub right: String,
}

impl FieldDivergence {
    /// 인덱스를 지운 정규화 경로 — baseline 키.
    ///
    /// `sections[0].paragraphs[37].controls[1].common.z_order`
    /// → `sections[].paragraphs[].controls[].common.z_order`
    ///
    /// 샘플마다 인덱스는 다르지만 **결함의 종류**는 경로 모양으로 식별되므로,
    /// baseline 을 인덱스가 아니라 이 모양으로 고정해야 문서가 조금 바뀌어도
    /// 래칫이 오작동하지 않는다.
    pub fn normalized_path(&self) -> String {
        let mut out = String::with_capacity(self.path.len());
        let mut in_index = false;
        for ch in self.path.chars() {
            match ch {
                '[' => {
                    in_index = true;
                    out.push('[');
                }
                ']' => {
                    in_index = false;
                    out.push(']');
                }
                _ if in_index => {}
                _ => out.push(ch),
            }
        }
        out
    }
}

impl std::fmt::Display for FieldDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} → {}", self.path, self.left, self.right)
    }
}

/// 상한을 넘으면 `Err` 를 돌려 파생 `Debug` 의 문자열화를 조기 중단시키는 writer.
///
/// 파생 `Debug` 구현은 `write!` 결과에 `?` 를 걸므로 첫 `Err` 에서 즉시 반환된다.
/// 덕분에 거대한 `Vec<u8>`·긴 텍스트를 만나도 비용이 `cap` 으로 묶인다.
struct CappedWriter {
    buf: String,
    cap: usize,
    overflow: bool,
}

impl std::fmt::Write for CappedWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.buf.len() + s.len() > self.cap {
            self.overflow = true;
            return Err(std::fmt::Error);
        }
        self.buf.push_str(s);
        Ok(())
    }
}

/// `Debug` 표현을 [`DEBUG_CAP`] 이내로 문자열화. 두 번째 값은 절단 여부.
fn debug_capped<T: Debug + ?Sized>(v: &T) -> (String, bool) {
    let mut w = CappedWriter {
        buf: String::new(),
        cap: DEBUG_CAP,
        overflow: false,
    };
    // 결과 무시 — 절단은 overflow 로 전달된다.
    let _ = write!(w, "{v:?}");
    (w.buf, w.overflow)
}

/// 최상위(중첩·문자열 리터럴 밖) 구분자 기준으로 `body` 를 콤마 분리.
/// 괄호 짝이 맞지 않으면 `None`.
fn split_top_level(body: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut quote: Option<u8> = None;
    let b = body.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == b'\\' {
                // 이스케이프 다음 바이트는 항상 ASCII (Rust Debug 출력 규약).
                i += 2;
                continue;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' => quote = Some(c),
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            b',' if depth == 0 => {
                parts.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if depth != 0 || quote.is_some() {
        return None;
    }
    parts.push(&body[start..]);
    Some(parts)
}

/// `name: value` 를 최상위 첫 `:` 에서 분리.
fn split_field(part: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let b = part.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' => quote = Some(c),
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b':' if depth == 0 => return Some((&part[..i], &part[i + 1..])),
            _ => {}
        }
        i += 1;
    }
    None
}

/// `Debug` 문자열 한 개를 (헤드 이름, 자식 목록) 으로 분해.
///
/// - `Name { a: 1, b: 2 }` → `("Name", [("a","1"), ("b","2")])`
/// - `[x, y]` → `("", [("0","x"), ("1","y")])`
/// - `Name(x, y)` → `("Name", [("0","x"), ("1","y")])`
///
/// 스칼라·문자열 리터럴·유닛 variant 는 분해 불가(`None`) — 잎으로 취급된다.
fn split_debug(s: &str) -> Option<(&str, Vec<(&str, &str)>)> {
    let s = s.trim();
    if s.len() < 2 || s.starts_with('"') {
        return None;
    }
    let open_idx = s.find(['{', '[', '('])?;
    let open = s.as_bytes()[open_idx];
    let close = match open {
        b'{' => '}',
        b'[' => ']',
        _ => ')',
    };
    if !s.ends_with(close) {
        return None;
    }
    let head = s[..open_idx].trim();
    let body = &s[open_idx + 1..s.len() - 1];
    let parts = split_top_level(body)?;
    let named = open == b'{';
    let mut out = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        let part = part.trim();
        if part.is_empty() {
            // `Name {}` / `[]` 의 빈 본문
            continue;
        }
        if named {
            let (k, v) = split_field(part)?;
            out.push((k.trim(), v.trim()));
        } else {
            // 인덱스 키는 경로 조립에서 `[i]` 로 쓰이므로 정적 문자열이 필요 없다.
            let key = INDEX_KEYS.get(i).copied().unwrap_or("n");
            out.push((key, part));
        }
    }
    Some((head, out))
}

/// 시퀀스 자식 키 — `split_debug` 가 인덱스를 문자열로 만들지 않도록 미리 준비.
/// 범위를 넘으면 `"n"` 으로 축약되며, 경로에는 실제 인덱스가 따로 붙는다.
const INDEX_KEYS: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
];

/// 양쪽이 모두 `Some(..)` 이면 안쪽을 벗겨 경로를 투명하게 유지한다.
fn unwrap_some<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    let peel = |s: &'a str| -> Option<&'a str> {
        let s = s.trim();
        if s.starts_with("Some(") && s.ends_with(')') {
            Some(s["Some(".len()..s.len() - 1].trim())
        } else {
            None
        }
    };
    match (peel(a), peel(b)) {
        (Some(x), Some(y)) => (x, y),
        _ => (a, b),
    }
}

fn truncate(s: &str) -> String {
    if s.chars().count() <= VALUE_CAP {
        return s.to_string();
    }
    let cut: String = s.chars().take(VALUE_CAP).collect();
    format!("{cut}…")
}

fn push_leaf(path: &str, a: &str, b: &str, out: &mut Vec<FieldDivergence>) {
    if out.len() >= MAX_DIVERGENCES {
        return;
    }
    out.push(FieldDivergence {
        path: path.to_string(),
        left: truncate(a),
        right: truncate(b),
    });
}

/// 두 `Debug` 문자열을 구조적으로 비교해 발산 잎을 수집한다.
fn compare_node(path: &str, a: &str, b: &str, depth: usize, out: &mut Vec<FieldDivergence>) {
    if a == b || out.len() >= MAX_DIVERGENCES {
        return;
    }
    if depth >= MAX_DEPTH {
        push_leaf(path, a, b, out);
        return;
    }
    let (a, b) = unwrap_some(a, b);
    if a == b {
        return;
    }
    let (Some((head_a, kids_a)), Some((head_b, kids_b))) = (split_debug(a), split_debug(b)) else {
        push_leaf(path, a, b, out);
        return;
    };
    if head_a != head_b {
        // 다른 enum variant / 다른 타입 — 잎으로 보고한다.
        push_leaf(path, a, b, out);
        return;
    }
    if kids_a.len() != kids_b.len() {
        // 길이 불일치는 그 자체로 보고하고(기존 비교기의 zip 사각지대), 공통 구간만 계속 본다.
        push_leaf(
            &format!("{path}.len"),
            &kids_a.len().to_string(),
            &kids_b.len().to_string(),
            out,
        );
    }
    for (idx, ((ka, va), (kb, vb))) in kids_a.iter().zip(kids_b.iter()).enumerate() {
        if ka != kb {
            push_leaf(path, a, b, out);
            return;
        }
        let child = if ka.as_bytes().first().is_some_and(|c| c.is_ascii_digit()) {
            format!("{path}[{idx}]")
        } else {
            format!("{path}.{ka}")
        };
        compare_node(&child, va, vb, depth + 1, out);
    }
}

/// 값 2개를 `Debug` 로 문자열화해 구조 비교한다.
fn cmp_debug<T: Debug + ?Sized>(path: &str, a: &T, b: &T, out: &mut Vec<FieldDivergence>) {
    if out.len() >= MAX_DIVERGENCES {
        return;
    }
    let (sa, _) = debug_capped(a);
    let (sb, _) = debug_capped(b);
    compare_node(path, &sa, &sb, 0, out);
}

/// 개수 비교 — 시퀀스는 개수를 먼저 보고해야 zip 이 뒤를 가리지 않는다.
fn cmp_count(path: &str, a: usize, b: usize, out: &mut Vec<FieldDivergence>) {
    if a != b {
        push_leaf(&format!("{path}.len"), &a.to_string(), &b.to_string(), out);
    }
}

// ---------------------------------------------------------------------------
// 척추 — 순회 경로
// ---------------------------------------------------------------------------

/// 왕복 전후 두 `Document` 의 IR 필드를 전수 비교한다.
pub fn sweep_documents(a: &Document, b: &Document) -> Vec<FieldDivergence> {
    let mut out = Vec::new();
    sweep_doc_info(&a.doc_info, &b.doc_info, &mut out);
    cmp_count("sections", a.sections.len(), b.sections.len(), &mut out);
    for (i, (sa, sb)) in a.sections.iter().zip(b.sections.iter()).enumerate() {
        if out.len() >= MAX_DIVERGENCES {
            break;
        }
        let base = format!("sections[{i}]");
        sweep_section_def(&base, &sa.section_def, &sb.section_def, &mut out);
        sweep_paragraphs(
            &format!("{base}.paragraphs"),
            &sa.paragraphs,
            &sb.paragraphs,
            &mut out,
        );
    }
    out
}

fn sweep_doc_info(a: &DocInfo, b: &DocInfo, out: &mut Vec<FieldDivergence>) {
    macro_rules! table {
        ($f:ident) => {{
            let base = concat!("doc_info.", stringify!($f));
            cmp_count(base, a.$f.len(), b.$f.len(), out);
            for (i, (x, y)) in a.$f.iter().zip(b.$f.iter()).enumerate() {
                cmp_debug(&format!("{base}[{i}]"), x, y, out);
            }
        }};
    }
    table!(char_shapes);
    table!(para_shapes);
    table!(border_fills);
    table!(tab_defs);
    table!(numberings);
    table!(bullets);
    table!(styles);
    table!(bin_data_list);

    // font_faces 는 언어별 2차원 목록.
    cmp_count(
        "doc_info.font_faces",
        a.font_faces.len(),
        b.font_faces.len(),
        out,
    );
    for (i, (xs, ys)) in a.font_faces.iter().zip(b.font_faces.iter()).enumerate() {
        let base = format!("doc_info.font_faces[{i}]");
        cmp_count(&base, xs.len(), ys.len(), out);
        for (j, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
            cmp_debug(&format!("{base}[{j}]"), x, y, out);
        }
    }

    cmp_debug(
        "doc_info.bullet_count",
        &a.bullet_count,
        &b.bullet_count,
        out,
    );
    cmp_debug(
        "doc_info.memo_shape_count",
        &a.memo_shape_count,
        &b.memo_shape_count,
        out,
    );
    cmp_debug(
        "doc_info.hwpml_version",
        &a.hwpml_version,
        &b.hwpml_version,
        out,
    );
    // extra_records 는 미모델링 레코드의 원본 바이트 — 개수만 본다.
    cmp_count(
        "doc_info.extra_records",
        a.extra_records.len(),
        b.extra_records.len(),
        out,
    );
}

fn sweep_section_def(base: &str, a: &SectionDef, b: &SectionDef, out: &mut Vec<FieldDivergence>) {
    // 원본 보존 버퍼/렌더 파생물은 개수만 (내용 비교는 왕복 의미가 없다).
    cmp_count(
        &format!("{base}.section_def.extra_child_records"),
        a.extra_child_records.len(),
        b.extra_child_records.len(),
        out,
    );
    cmp_count(
        &format!("{base}.section_def.master_pages"),
        a.master_pages.len(),
        b.master_pages.len(),
        out,
    );
    let mut sa = a.clone();
    let mut sb = b.clone();
    sa.extra_child_records.clear();
    sa.master_pages.clear();
    sb.extra_child_records.clear();
    sb.master_pages.clear();
    cmp_debug(&format!("{base}.section_def"), &sa, &sb, out);
}

fn sweep_paragraphs(base: &str, a: &[Paragraph], b: &[Paragraph], out: &mut Vec<FieldDivergence>) {
    cmp_count(base, a.len(), b.len(), out);
    for (i, (pa, pb)) in a.iter().zip(b.iter()).enumerate() {
        if out.len() >= MAX_DIVERGENCES {
            return;
        }
        sweep_paragraph(&format!("{base}[{i}]"), pa, pb, out);
    }
}

/// 문단 1건 — `..` 없는 철저 구조 분해로 필드 누락을 컴파일 타임에 막는다.
/// (`Paragraph` 에 필드를 추가하면 여기서 컴파일이 깨진다.)
fn sweep_paragraph(base: &str, a: &Paragraph, b: &Paragraph, out: &mut Vec<FieldDivergence>) {
    let Paragraph {
        char_count,
        control_mask,
        para_shape_id,
        style_id,
        column_type,
        raw_break_type,
        text,
        char_offsets,
        char_shapes,
        line_segs,
        range_tags,
        field_ranges,
        orphan_field_ends,
        controls,
        ctrl_data_records,
        char_count_msb,
        raw_header_extra,
        has_para_text,
        tab_extended,
        numbering_restart,
    } = a;

    macro_rules! f {
        ($n:ident) => {
            cmp_debug(&format!("{base}.{}", stringify!($n)), $n, &b.$n, out)
        };
    }
    f!(char_count);
    f!(control_mask);
    f!(para_shape_id);
    f!(style_id);
    f!(column_type);
    f!(raw_break_type);
    f!(text);
    f!(char_offsets);
    f!(char_shapes);
    f!(line_segs);
    f!(range_tags);
    f!(field_ranges);
    f!(orphan_field_ends);
    f!(ctrl_data_records);
    f!(char_count_msb);
    f!(raw_header_extra);
    f!(has_para_text);
    f!(tab_extended);
    f!(numbering_restart);

    sweep_controls(&format!("{base}.controls"), controls, &b.controls, out);
}

fn sweep_controls(base: &str, a: &[Control], b: &[Control], out: &mut Vec<FieldDivergence>) {
    cmp_count(base, a.len(), b.len(), out);
    for (i, (ca, cb)) in a.iter().zip(b.iter()).enumerate() {
        if out.len() >= MAX_DIVERGENCES {
            return;
        }
        sweep_control(&format!("{base}[{i}]"), ca, cb, out);
    }
}

/// 컨트롤 1건 — 속성은 전수 비교하고, 하위 문단은 따로 재귀한다.
///
/// 표는 셀 문단까지 복제하면 비용이 커지므로 철저 구조 분해로 처리하고,
/// 나머지는 하위 문단만 비워낸 사본을 `Debug` 로 전수 비교한다.
fn sweep_control(base: &str, a: &Control, b: &Control, out: &mut Vec<FieldDivergence>) {
    use Control::*;
    match (a, b) {
        (Table(ta), Table(tb)) => sweep_table(base, ta, tb, out),
        (Header(ha), Header(hb)) => {
            let (mut x, mut y) = (ha.clone(), hb.clone());
            x.paragraphs.clear();
            y.paragraphs.clear();
            cmp_debug(base, &x, &y, out);
            sweep_paragraphs(
                &format!("{base}.paragraphs"),
                &ha.paragraphs,
                &hb.paragraphs,
                out,
            );
        }
        (Footer(ha), Footer(hb)) => {
            let (mut x, mut y) = (ha.clone(), hb.clone());
            x.paragraphs.clear();
            y.paragraphs.clear();
            cmp_debug(base, &x, &y, out);
            sweep_paragraphs(
                &format!("{base}.paragraphs"),
                &ha.paragraphs,
                &hb.paragraphs,
                out,
            );
        }
        (Footnote(fa), Footnote(fb)) => {
            let (mut x, mut y) = (fa.clone(), fb.clone());
            x.paragraphs.clear();
            y.paragraphs.clear();
            cmp_debug(base, &x, &y, out);
            sweep_paragraphs(
                &format!("{base}.paragraphs"),
                &fa.paragraphs,
                &fb.paragraphs,
                out,
            );
        }
        (Endnote(fa), Endnote(fb)) => {
            let (mut x, mut y) = (fa.clone(), fb.clone());
            x.paragraphs.clear();
            y.paragraphs.clear();
            cmp_debug(base, &x, &y, out);
            sweep_paragraphs(
                &format!("{base}.paragraphs"),
                &fa.paragraphs,
                &fb.paragraphs,
                out,
            );
        }
        (Field(fa), Field(fb)) => {
            let (mut x, mut y) = (fa.clone(), fb.clone());
            x.memo_paragraphs.clear();
            y.memo_paragraphs.clear();
            cmp_debug(base, &x, &y, out);
            sweep_paragraphs(
                &format!("{base}.memo_paragraphs"),
                &fa.memo_paragraphs,
                &fb.memo_paragraphs,
                out,
            );
        }
        (Picture(pa), Picture(pb)) => {
            let (mut x, mut y) = (pa.clone(), pb.clone());
            if let Some(c) = x.caption.as_mut() {
                c.paragraphs.clear();
            }
            if let Some(c) = y.caption.as_mut() {
                c.paragraphs.clear();
            }
            cmp_debug(base, &x, &y, out);
            sweep_caption_paragraphs(base, &pa.caption, &pb.caption, out);
        }
        (Shape(sa), Shape(sb)) => sweep_shape(base, sa, sb, out),
        (Form(fa), Form(fb)) => sweep_form(base, fa, fb, out),
        (HiddenComment(ha), HiddenComment(hb)) => sweep_paragraphs(
            &format!("{base}.paragraphs"),
            &ha.paragraphs,
            &hb.paragraphs,
            out,
        ),
        // 하위 문단이 없는 컨트롤 — 통째로 전수 비교.
        (SectionDef(x), SectionDef(y)) => sweep_section_def(base, x, y, out),
        (ColumnDef(x), ColumnDef(y)) => cmp_debug(base, x, y, out),
        (AutoNumber(x), AutoNumber(y)) => cmp_debug(base, x, y, out),
        (NewNumber(x), NewNumber(y)) => cmp_debug(base, x, y, out),
        (PageNumberPos(x), PageNumberPos(y)) => cmp_debug(base, x, y, out),
        (Bookmark(x), Bookmark(y)) => cmp_debug(base, x, y, out),
        (Hyperlink(x), Hyperlink(y)) => cmp_debug(base, x, y, out),
        (Ruby(x), Ruby(y)) => cmp_debug(base, x, y, out),
        (CharOverlap(x), CharOverlap(y)) => cmp_debug(base, x, y, out),
        (PageHide(x), PageHide(y)) => cmp_debug(base, x, y, out),
        (Equation(x), Equation(y)) => cmp_debug(base, x, y, out),
        (Unknown(x), Unknown(y)) => cmp_debug(base, x, y, out),
        // 컨트롤 종류 자체가 달라진 경우 — 왕복 소실 중 가장 큰 종류.
        _ => push_leaf(base, control_kind(a), control_kind(b), out),
    }
}

/// 컨트롤 종류 이름 — 종류 자체가 뒤바뀐 발산을 읽기 쉽게 보고한다.
fn control_kind(c: &Control) -> &'static str {
    use Control::*;
    match c {
        SectionDef(_) => "SectionDef",
        ColumnDef(_) => "ColumnDef",
        Table(_) => "Table",
        Shape(_) => "Shape",
        Picture(_) => "Picture",
        Header(_) => "Header",
        Footer(_) => "Footer",
        Footnote(_) => "Footnote",
        Endnote(_) => "Endnote",
        AutoNumber(_) => "AutoNumber",
        NewNumber(_) => "NewNumber",
        PageNumberPos(_) => "PageNumberPos",
        Bookmark(_) => "Bookmark",
        Hyperlink(_) => "Hyperlink",
        Ruby(_) => "Ruby",
        CharOverlap(_) => "CharOverlap",
        PageHide(_) => "PageHide",
        HiddenComment(_) => "HiddenComment",
        Equation(_) => "Equation",
        Field(_) => "Field",
        Form(_) => "Form",
        Unknown(_) => "Unknown",
    }
}

/// 양식 개체 — `properties` 만 `HashMap` 이라 `Debug` 순서가 비결정적이다.
/// 그대로 비교하면 실행마다 결과가 달라지는 위양성이 나오므로 정렬해서 비교한다.
/// (`..` 없는 철저 구조 분해로 필드 추가를 컴파일 타임에 잡는다.)
fn sweep_form(
    base: &str,
    a: &crate::model::control::FormObject,
    b: &crate::model::control::FormObject,
    out: &mut Vec<FieldDivergence>,
) {
    let crate::model::control::FormObject {
        form_type,
        name,
        caption,
        text,
        width,
        height,
        fore_color,
        back_color,
        value,
        enabled,
        properties,
    } = a;

    macro_rules! f {
        ($n:ident) => {
            cmp_debug(&format!("{base}.{}", stringify!($n)), $n, &b.$n, out)
        };
    }
    f!(form_type);
    f!(name);
    f!(caption);
    f!(text);
    f!(width);
    f!(height);
    f!(fore_color);
    f!(back_color);
    f!(value);
    f!(enabled);

    let sorted = |m: &std::collections::HashMap<String, String>| {
        m.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    cmp_debug(
        &format!("{base}.properties"),
        &sorted(properties),
        &sorted(&b.properties),
        out,
    );
}

fn sweep_caption_paragraphs(
    base: &str,
    a: &Option<crate::model::shape::Caption>,
    b: &Option<crate::model::shape::Caption>,
    out: &mut Vec<FieldDivergence>,
) {
    if let (Some(x), Some(y)) = (a, b) {
        sweep_paragraphs(
            &format!("{base}.caption.paragraphs"),
            &x.paragraphs,
            &y.paragraphs,
            out,
        );
    }
}

/// 표 — `..` 없는 철저 구조 분해. 필드가 추가되면 컴파일이 깨진다.
fn sweep_table(base: &str, a: &Table, b: &Table, out: &mut Vec<FieldDivergence>) {
    let Table {
        attr,
        row_count,
        col_count,
        cell_spacing,
        padding,
        row_sizes,
        border_fill_id,
        zones,
        cells,
        cell_grid,
        page_break,
        repeat_header,
        caption,
        common,
        outer_margin_left,
        outer_margin_right,
        outer_margin_top,
        outer_margin_bottom,
        raw_ctrl_data,
        raw_table_record_attr,
        raw_table_record_extra,
        dirty,
        local_resize_rows,
        local_resize_cols,
        local_resize_cell_widths,
        local_resize_cell_heights,
    } = a;

    macro_rules! f {
        ($n:ident) => {
            cmp_debug(&format!("{base}.{}", stringify!($n)), $n, &b.$n, out)
        };
    }
    f!(attr);
    f!(row_count);
    f!(col_count);
    f!(cell_spacing);
    f!(padding);
    f!(row_sizes);
    f!(border_fill_id);
    f!(zones);
    f!(cell_grid);
    f!(page_break);
    f!(repeat_header);
    f!(common);
    f!(outer_margin_left);
    f!(outer_margin_right);
    f!(outer_margin_top);
    f!(outer_margin_bottom);
    f!(raw_ctrl_data);
    f!(raw_table_record_attr);
    f!(raw_table_record_extra);
    f!(dirty);
    f!(local_resize_rows);
    f!(local_resize_cols);
    f!(local_resize_cell_widths);
    f!(local_resize_cell_heights);

    // 캡션: 속성은 문단을 뺀 사본으로, 문단은 재귀로.
    {
        let strip = |c: &Option<crate::model::shape::Caption>| {
            c.clone().map(|mut v| {
                v.paragraphs.clear();
                v
            })
        };
        cmp_debug(
            &format!("{base}.caption"),
            &strip(caption),
            &strip(&b.caption),
            out,
        );
        sweep_caption_paragraphs(base, caption, &b.caption, out);
    }

    let cells_path = format!("{base}.cells");
    cmp_count(&cells_path, cells.len(), b.cells.len(), out);
    for (i, (ca, cb)) in cells.iter().zip(b.cells.iter()).enumerate() {
        if out.len() >= MAX_DIVERGENCES {
            return;
        }
        sweep_cell(&format!("{cells_path}[{i}]"), ca, cb, out);
    }
}

/// 셀 — `..` 없는 철저 구조 분해.
fn sweep_cell(base: &str, a: &Cell, b: &Cell, out: &mut Vec<FieldDivergence>) {
    let Cell {
        col,
        row,
        col_span,
        row_span,
        width,
        height,
        padding,
        border_fill_id,
        paragraphs,
        list_header_width_ref,
        text_direction,
        vertical_align,
        apply_inner_margin,
        is_header,
        raw_list_extra,
        field_name,
    } = a;

    macro_rules! f {
        ($n:ident) => {
            cmp_debug(&format!("{base}.{}", stringify!($n)), $n, &b.$n, out)
        };
    }
    f!(col);
    f!(row);
    f!(col_span);
    f!(row_span);
    f!(width);
    f!(height);
    f!(padding);
    f!(border_fill_id);
    f!(list_header_width_ref);
    f!(text_direction);
    f!(vertical_align);
    f!(apply_inner_margin);
    f!(is_header);
    f!(raw_list_extra);
    f!(field_name);

    sweep_paragraphs(
        &format!("{base}.paragraphs"),
        paragraphs,
        &b.paragraphs,
        out,
    );
}

/// 도형 — 하위 문단(글상자·캡션)을 비운 사본으로 속성 전수 비교 후 문단 재귀.
fn sweep_shape(base: &str, a: &ShapeObject, b: &ShapeObject, out: &mut Vec<FieldDivergence>) {
    let mut x = a.clone();
    let mut y = b.clone();
    strip_shape(&mut x);
    strip_shape(&mut y);
    cmp_debug(base, &x, &y, out);

    // 글상자 문단
    if let (Some(ta), Some(tb)) = (shape_text_box(a), shape_text_box(b)) {
        sweep_paragraphs(
            &format!("{base}.text_box.paragraphs"),
            &ta.paragraphs,
            &tb.paragraphs,
            out,
        );
    }
    // 캡션 문단
    sweep_caption_paragraphs(base, shape_caption(a), shape_caption(b), out);
    // 묶음 자식
    if let (ShapeObject::Group(ga), ShapeObject::Group(gb)) = (a, b) {
        let path = format!("{base}.children");
        cmp_count(&path, ga.children.len(), gb.children.len(), out);
        for (i, (ca, cb)) in ga.children.iter().zip(gb.children.iter()).enumerate() {
            if out.len() >= MAX_DIVERGENCES {
                return;
            }
            sweep_shape(&format!("{path}[{i}]"), ca, cb, out);
        }
    }
}

/// 도형에서 하위 문단·자식을 비운다(속성 전수 비교용 사본 준비).
fn strip_shape(s: &mut ShapeObject) {
    use ShapeObject::*;
    macro_rules! drawing {
        ($x:expr) => {{
            if let Some(tb) = $x.drawing.text_box.as_mut() {
                tb.paragraphs.clear();
            }
            if let Some(c) = $x.drawing.caption.as_mut() {
                c.paragraphs.clear();
            }
        }};
    }
    match s {
        Line(x) => drawing!(x),
        Rectangle(x) => drawing!(x),
        Ellipse(x) => drawing!(x),
        Arc(x) => drawing!(x),
        Polygon(x) => drawing!(x),
        Curve(x) => drawing!(x),
        Chart(x) => {
            drawing!(x);
            if let Some(c) = x.caption.as_mut() {
                c.paragraphs.clear();
            }
        }
        Ole(x) => {
            drawing!(x);
            if let Some(c) = x.caption.as_mut() {
                c.paragraphs.clear();
            }
        }
        Group(x) => {
            if let Some(c) = x.caption.as_mut() {
                c.paragraphs.clear();
            }
            // 자식은 따로 재귀 비교하므로 사본에서는 비운다.
            x.children.clear();
        }
        Picture(x) => {
            if let Some(c) = x.caption.as_mut() {
                c.paragraphs.clear();
            }
        }
    }
}

fn shape_text_box(s: &ShapeObject) -> Option<&crate::model::shape::TextBox> {
    use ShapeObject::*;
    match s {
        Line(x) => x.drawing.text_box.as_ref(),
        Rectangle(x) => x.drawing.text_box.as_ref(),
        Ellipse(x) => x.drawing.text_box.as_ref(),
        Arc(x) => x.drawing.text_box.as_ref(),
        Polygon(x) => x.drawing.text_box.as_ref(),
        Curve(x) => x.drawing.text_box.as_ref(),
        Chart(x) => x.drawing.text_box.as_ref(),
        Ole(x) => x.drawing.text_box.as_ref(),
        Group(_) | Picture(_) => None,
    }
}

fn shape_caption(s: &ShapeObject) -> &Option<crate::model::shape::Caption> {
    use ShapeObject::*;
    match s {
        Line(x) => &x.drawing.caption,
        Rectangle(x) => &x.drawing.caption,
        Ellipse(x) => &x.drawing.caption,
        Arc(x) => &x.drawing.caption,
        Polygon(x) => &x.drawing.caption,
        Curve(x) => &x.drawing.caption,
        Chart(x) => &x.caption,
        Ole(x) => &x.caption,
        Group(x) => &x.caption,
        Picture(x) => &x.caption,
    }
}

// ---------------------------------------------------------------------------
// 왕복 실행 진입점
// ---------------------------------------------------------------------------

/// HWP5 왕복(`parse → serialize → reparse`) 후 IR 필드 전수 스윕.
pub fn sweep_hwp5_roundtrip(bytes: &[u8]) -> Result<Vec<FieldDivergence>, String> {
    use crate::parser::parse_document;
    use crate::serializer::serialize_document;
    let doc1 = parse_document(bytes).map_err(|e| format!("파싱 실패: {e}"))?;
    let out = serialize_document(&doc1).map_err(|e| format!("직렬화 실패: {e}"))?;
    let doc2 = parse_document(&out).map_err(|e| format!("재파싱 실패: {e}"))?;
    Ok(sweep_documents(&doc1, &doc2))
}

/// HWP5 **레코드 재생성** 경로 왕복 후 IR 필드 전수 스윕.
///
/// [`sweep_hwp5_roundtrip`] 은 실제로 직렬화기를 거의 검사하지 못한다.
/// `serializer/body_text.rs:28`·`serializer/doc_info.rs:25` 가 `raw_stream` 이
/// 있으면 **원본 바이트를 그대로 되돌려주기** 때문이다. 즉 편집하지 않은 문서의
/// HWP5 왕복은 바이트 재생(replay)이고, IR 이 같은 것은 당연하다.
///
/// 제품에서 문서를 한 글자라도 고치면 `document_core/commands/*` 가
/// `section.raw_stream = None` / `doc_info.raw_stream_dirty = true` 로 무효화하고
/// **레코드를 다시 만드는 경로**로 저장한다. 반복돼 온 1속성 소실은 전부 이 경로에서
/// 난다. 이 함수는 그 무효화를 그대로 재현해 진짜 저장 경로를 측정한다.
pub fn sweep_hwp5_rebuild_roundtrip(bytes: &[u8]) -> Result<Vec<FieldDivergence>, String> {
    use crate::parser::parse_document;
    use crate::serializer::serialize_document;
    let doc1 = parse_document(bytes).map_err(|e| format!("파싱 실패: {e}"))?;

    // 편집이 일어난 문서와 동일한 무효화 (document_core/commands 관례).
    let mut edited = doc1.clone();
    edited.doc_info.raw_stream_dirty = true;
    for s in &mut edited.sections {
        s.raw_stream = None;
    }

    let out = serialize_document(&edited).map_err(|e| format!("직렬화 실패: {e}"))?;
    let doc2 = parse_document(&out).map_err(|e| format!("재파싱 실패: {e}"))?;
    Ok(sweep_documents(&doc1, &doc2))
}

/// HWPX 왕복(`parse → serialize → reparse`) 후 IR 필드 전수 스윕.
pub fn sweep_hwpx_roundtrip(bytes: &[u8]) -> Result<Vec<FieldDivergence>, String> {
    use crate::parser::hwpx::parse_hwpx;
    use crate::serializer::hwpx::serialize_hwpx;
    let doc1 = parse_hwpx(bytes).map_err(|e| format!("파싱 실패: {e}"))?;
    let out = serialize_hwpx(&doc1).map_err(|e| format!("직렬화 실패: {e}"))?;
    let doc2 = parse_hwpx(&out).map_err(|e| format!("재파싱 실패: {e}"))?;
    Ok(sweep_documents(&doc1, &doc2))
}

/// 발산 목록을 정규화 경로별 건수로 집계 (baseline 키 단위).
pub fn tally(divs: &[FieldDivergence]) -> std::collections::BTreeMap<String, usize> {
    let mut map = std::collections::BTreeMap::new();
    for d in divs {
        *map.entry(d.normalized_path()).or_insert(0) += 1;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_debug_struct() {
        let (head, kids) = split_debug("CharShape { bold: true, size: 1000 }").unwrap();
        assert_eq!(head, "CharShape");
        assert_eq!(kids, vec![("bold", "true"), ("size", "1000")]);
    }

    #[test]
    fn split_debug_seq() {
        let (head, kids) = split_debug("[1, 2, 3]").unwrap();
        assert_eq!(head, "");
        assert_eq!(kids, vec![("0", "1"), ("1", "2"), ("2", "3")]);
    }

    #[test]
    fn split_debug_nested_commas_are_not_split() {
        let (_, kids) = split_debug("T { a: [1, 2], b: S { c: 3 } }").unwrap();
        assert_eq!(kids, vec![("a", "[1, 2]"), ("b", "S { c: 3 }")]);
    }

    #[test]
    fn split_debug_string_with_comma_and_braces() {
        let (_, kids) = split_debug(r#"T { name: "a, b { c", n: 1 }"#).unwrap();
        assert_eq!(kids, vec![("name", r#""a, b { c""#), ("n", "1")]);
    }

    #[test]
    fn split_debug_rejects_scalar() {
        assert!(split_debug("42").is_none());
        assert!(split_debug("true").is_none());
        assert!(split_debug(r#""hello""#).is_none());
        assert!(split_debug("None").is_none());
    }

    #[test]
    fn compare_node_reports_leaf_path() {
        let mut out = Vec::new();
        compare_node(
            "t",
            "T { a: 1, b: S { c: 2 } }",
            "T { a: 1, b: S { c: 9 } }",
            0,
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "t.b.c");
        assert_eq!(out[0].left, "2");
        assert_eq!(out[0].right, "9");
    }

    #[test]
    fn compare_node_reports_seq_length_and_common_prefix() {
        let mut out = Vec::new();
        compare_node("v", "[1, 2, 3]", "[1, 9]", 0, &mut out);
        // 길이 발산 1건 + 인덱스 1 값 발산 1건
        let paths: Vec<&str> = out.iter().map(|d| d.path.as_str()).collect();
        assert!(paths.contains(&"v.len"), "길이 발산 누락: {paths:?}");
        assert!(paths.contains(&"v[1]"), "값 발산 누락: {paths:?}");
    }

    #[test]
    fn compare_node_unwraps_matching_some() {
        let mut out = Vec::new();
        compare_node("c", "Some(S { x: 1 })", "Some(S { x: 2 })", 0, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "c.x", "Some 은 경로에 투명해야 함");
    }

    #[test]
    fn compare_node_some_vs_none_is_leaf() {
        let mut out = Vec::new();
        compare_node("c", "Some(S { x: 1 })", "None", 0, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "c");
        assert_eq!(out[0].right, "None");
    }

    #[test]
    fn compare_node_different_variant_is_leaf() {
        let mut out = Vec::new();
        compare_node("s", "Line { a: 1 }", "Rectangle { a: 1 }", 0, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "s");
    }

    #[test]
    fn identical_values_produce_nothing() {
        let mut out = Vec::new();
        compare_node("x", "T { a: 1 }", "T { a: 1 }", 0, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn normalized_path_strips_indices() {
        let d = FieldDivergence {
            path: "sections[0].paragraphs[37].controls[1].common.z_order".to_string(),
            left: "1".into(),
            right: "2".into(),
        };
        assert_eq!(
            d.normalized_path(),
            "sections[].paragraphs[].controls[].common.z_order"
        );
    }

    #[test]
    fn capped_writer_truncates_large_debug() {
        let big: Vec<u32> = (0..200_000).collect();
        let (s, overflow) = debug_capped(&big);
        assert!(overflow, "상한 초과가 감지되어야 함");
        assert!(s.len() <= DEBUG_CAP, "상한을 넘겨 문자열화하면 안 됨");
    }

    #[test]
    fn debug_capped_small_value_is_complete() {
        let (s, overflow) = debug_capped(&(1u8, 2u8));
        assert!(!overflow);
        assert_eq!(s, "(1, 2)");
    }

    #[test]
    fn tally_groups_by_normalized_path() {
        let divs = vec![
            FieldDivergence {
                path: "sections[0].paragraphs[1].text".into(),
                left: "a".into(),
                right: "b".into(),
            },
            FieldDivergence {
                path: "sections[0].paragraphs[9].text".into(),
                left: "a".into(),
                right: "b".into(),
            },
        ];
        let t = tally(&divs);
        assert_eq!(t.get("sections[].paragraphs[].text"), Some(&2));
    }

    /// 실제 IR 타입 위에서 동작하는지 — 표 속성 1개만 바꾸면 그 경로만 나와야 한다.
    #[test]
    fn sweep_detects_single_table_attribute_change() {
        use crate::model::control::Control;
        use crate::model::document::{Document, Section};
        use crate::model::paragraph::Paragraph;
        use crate::model::table::Table;

        let mut tbl = Table::default();
        tbl.row_count = 2;
        let mut para = Paragraph::default();
        para.controls.push(Control::Table(Box::new(tbl)));
        let mut doc_a = Document::default();
        doc_a.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });

        let mut doc_b = doc_a.clone();
        if let Control::Table(t) = &mut doc_b.sections[0].paragraphs[0].controls[0] {
            t.row_count = 3;
        }

        let divs = sweep_documents(&doc_a, &doc_b);
        assert_eq!(divs.len(), 1, "예상 밖 발산: {divs:?}");
        assert_eq!(
            divs[0].path,
            "sections[0].paragraphs[0].controls[0].row_count"
        );
        assert_eq!(divs[0].left, "2");
        assert_eq!(divs[0].right, "3");
    }

    /// 동일 문서는 발산 0 — 위양성 방지 기본 보증.
    #[test]
    fn sweep_identical_documents_is_empty() {
        use crate::model::document::{Document, Section};
        use crate::model::paragraph::Paragraph;

        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs: vec![Paragraph {
                text: "가나다".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        });
        let copy = doc.clone();
        assert!(sweep_documents(&doc, &copy).is_empty());
    }
}
