//! IR comparison query adapters and their presentation-only diff helpers.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{classify_hwp_error, cli_password, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

fn control_tag(c: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match c {
        Control::SectionDef(_) => "secd",
        Control::ColumnDef(_) => "cold",
        Control::Table(_) => "tbl",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "pic",
        Control::Header(_) => "head",
        Control::Footer(_) => "foot",
        Control::Footnote(_) => "fn",
        Control::Endnote(_) => "en",
        Control::AutoNumber(_) => "atno",
        Control::NewNumber(_) => "nwno",
        Control::PageNumberPos(_) => "pgnp",
        Control::Bookmark(_) => "bokm",
        Control::IndexMark(_) => "idxm",
        Control::PageNumCtrl(_) => "pgct",
        Control::Hyperlink(_) => "hlk",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "tcps",
        Control::PageHide(_) => "pghd",
        Control::HiddenComment(_) => "tcmt",
        Control::Equation(_) => "eqed",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

fn diff_table(
    diffs: &mut Vec<String>,
    ci: usize,
    a: &rhwp::model::table::Table,
    b: &rhwp::model::table::Table,
) {
    if a.row_count != b.row_count {
        diffs.push(format!(
            "ctrl[{}] tbl rows: A={} vs B={}",
            ci, a.row_count, b.row_count
        ));
    }
    if a.col_count != b.col_count {
        diffs.push(format!(
            "ctrl[{}] tbl cols: A={} vs B={}",
            ci, a.col_count, b.col_count
        ));
    }
    if a.page_break != b.page_break {
        diffs.push(format!(
            "ctrl[{}] tbl page_break: A={:?} vs B={:?}",
            ci, a.page_break, b.page_break
        ));
    }
    if a.repeat_header != b.repeat_header {
        diffs.push(format!(
            "ctrl[{}] tbl repeat_header: A={} vs B={}",
            ci, a.repeat_header, b.repeat_header
        ));
    }
    if a.cell_spacing != b.cell_spacing {
        diffs.push(format!(
            "ctrl[{}] tbl cell_spacing: A={} vs B={}",
            ci, a.cell_spacing, b.cell_spacing
        ));
    }
    if a.border_fill_id != b.border_fill_id {
        diffs.push(format!(
            "ctrl[{}] tbl border_fill_id: A={} vs B={}",
            ci, a.border_fill_id, b.border_fill_id
        ));
    }
    if a.outer_margin_left != b.outer_margin_left
        || a.outer_margin_right != b.outer_margin_right
        || a.outer_margin_top != b.outer_margin_top
        || a.outer_margin_bottom != b.outer_margin_bottom
    {
        diffs.push(format!(
            "ctrl[{}] tbl outer_margin: A=({},{},{},{}) vs B=({},{},{},{})",
            ci,
            a.outer_margin_left,
            a.outer_margin_top,
            a.outer_margin_right,
            a.outer_margin_bottom,
            b.outer_margin_left,
            b.outer_margin_top,
            b.outer_margin_right,
            b.outer_margin_bottom,
        ));
    }
    diff_common_obj(diffs, ci, "tbl", &a.common, &b.common);
    // [#3469] 셀 문단 재귀 비교 — 표 속성만 보면 셀 안의 텍스트 변경이 보이지 않는다.
    // 글상자는 #1807 이 같은 구멍(#1795 "소거망 구멍")을 이미 막았는데 표는 열려 있었다.
    // ir-diff 는 `convert --verify` 게이트의 근거이고 한국 문서는 표가 본체라,
    // 이 구멍은 변환이 표 안의 모든 텍스트를 손상시켜도 통과시킨다.
    diff_table_cells(diffs, ci, a, b);
}

/// [#3469] 표 셀 안의 문단을 재귀 비교한다.
///
/// 셀 목록 길이가 다르면 그 사실만 보고하고, 공통 구간의 셀은 문단 단위로
/// `diff_textbox_paragraph_lists`(글상자와 같은 비교기)로 내려간다. 셀 문단 안의
/// 중첩 표는 그 안에서 다시 이 경로를 타므로 임의 깊이가 자연히 커버된다.
fn diff_table_cells(
    diffs: &mut Vec<String>,
    ci: usize,
    a: &rhwp::model::table::Table,
    b: &rhwp::model::table::Table,
) {
    use rhwp::model::control::Control;

    if a.cells.len() != b.cells.len() {
        diffs.push(format!(
            "ctrl[{}] tbl 셀 수: A={} vs B={}",
            ci,
            a.cells.len(),
            b.cells.len()
        ));
    }
    for (k, (ca, cb)) in a.cells.iter().zip(b.cells.iter()).enumerate() {
        let prefix = format!("ctrl[{}] tbl cell[{}:{},{}]", ci, k, ca.row, ca.col);
        diff_textbox_paragraph_lists(diffs, &prefix, &ca.paragraphs, &cb.paragraphs);
        // 셀 문단이 품은 중첩 표도 같은 규칙으로 내려간다.
        for (pi, (pa, pb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate() {
            for (cj, (na, nb)) in pa.controls.iter().zip(pb.controls.iter()).enumerate() {
                if let (Control::Table(ta), Control::Table(tb)) = (na, nb) {
                    diff_table_cells(diffs, ci, ta, tb);
                    let _ = (pi, cj);
                }
            }
        }
    }
}

fn diff_common_obj(
    diffs: &mut Vec<String>,
    ci: usize,
    tag: &str,
    a: &rhwp::model::shape::CommonObjAttr,
    b: &rhwp::model::shape::CommonObjAttr,
) {
    if a.treat_as_char != b.treat_as_char {
        diffs.push(format!(
            "ctrl[{}] {} tac: A={} vs B={}",
            ci, tag, a.treat_as_char, b.treat_as_char
        ));
    }
    if a.text_wrap != b.text_wrap {
        diffs.push(format!(
            "ctrl[{}] {} wrap: A={:?} vs B={:?}",
            ci, tag, a.text_wrap, b.text_wrap
        ));
    }
    if a.width != b.width || a.height != b.height {
        diffs.push(format!(
            "ctrl[{}] {} size: A={}x{} vs B={}x{}",
            ci, tag, a.width, a.height, b.width, b.height
        ));
    }
    if a.vertical_offset != b.vertical_offset {
        diffs.push(format!(
            "ctrl[{}] {} v_offset: A={} vs B={}",
            ci, tag, a.vertical_offset, b.vertical_offset
        ));
    }
    if a.horizontal_offset != b.horizontal_offset {
        diffs.push(format!(
            "ctrl[{}] {} h_offset: A={} vs B={}",
            ci, tag, a.horizontal_offset, b.horizontal_offset
        ));
    }
    if a.vert_rel_to != b.vert_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} vert_rel: A={:?} vs B={:?}",
            ci, tag, a.vert_rel_to, b.vert_rel_to
        ));
    }
    if a.horz_rel_to != b.horz_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} horz_rel: A={:?} vs B={:?}",
            ci, tag, a.horz_rel_to, b.horz_rel_to
        ));
    }
}

/// [#1807] 글상자 문단 한 쌍의 핵심 필드 비교 — 본문 문단 비교의 축약판.
/// 직렬화 결함(#1795: FIELD_END 갭 선점 → char_offsets 시프트)이 글상자 안에서
/// 발생해도 ir-diff 가 검출하도록 text/cc/char_offsets/char_shapes/line_segs/
/// field_ranges 를 비교한다.
fn diff_textbox_paragraph_fields(
    diffs: &mut Vec<String>,
    prefix: &str,
    pa: &rhwp::model::paragraph::Paragraph,
    pb: &rhwp::model::paragraph::Paragraph,
) {
    if pa.text != pb.text {
        diffs.push(format!(
            "{} text: A={:?} vs B={:?}",
            prefix,
            pa.text.chars().take(30).collect::<String>(),
            pb.text.chars().take(30).collect::<String>()
        ));
    }
    if pa.char_count != pb.char_count {
        diffs.push(format!(
            "{} cc: A={} vs B={}",
            prefix, pa.char_count, pb.char_count
        ));
    }
    if pa.char_offsets != pb.char_offsets {
        if pa.char_offsets.len() != pb.char_offsets.len() {
            diffs.push(format!(
                "{} char_offsets len: A={} vs B={}",
                prefix,
                pa.char_offsets.len(),
                pb.char_offsets.len()
            ));
        } else if let Some((idx, (a, b))) = pa
            .char_offsets
            .iter()
            .zip(pb.char_offsets.iter())
            .enumerate()
            .find(|(_, (a, b))| a != b)
        {
            diffs.push(format!(
                "{} char_offsets[{}]: A={} vs B={}",
                prefix, idx, a, b
            ));
        }
    }
    if pa.char_shapes.len() != pb.char_shapes.len() {
        diffs.push(format!(
            "{} char_shapes count: A={} vs B={}",
            prefix,
            pa.char_shapes.len(),
            pb.char_shapes.len()
        ));
    } else if let Some((idx, (ca, cb))) = pa
        .char_shapes
        .iter()
        .zip(pb.char_shapes.iter())
        .enumerate()
        .find(|(_, (ca, cb))| ca.start_pos != cb.start_pos || ca.char_shape_id != cb.char_shape_id)
    {
        diffs.push(format!(
            "{} cs[{}]: A=({},{}) vs B=({},{})",
            prefix, idx, ca.start_pos, ca.char_shape_id, cb.start_pos, cb.char_shape_id
        ));
    }
    if pa.line_segs.len() != pb.line_segs.len() {
        diffs.push(format!(
            "{} line_segs count: A={} vs B={}",
            prefix,
            pa.line_segs.len(),
            pb.line_segs.len()
        ));
    } else if let Some((idx, (la, lb))) = pa
        .line_segs
        .iter()
        .zip(pb.line_segs.iter())
        .enumerate()
        .find(|(_, (la, lb))| la.text_start != lb.text_start || la.vertical_pos != lb.vertical_pos)
    {
        diffs.push(format!(
            "{} ls[{}]: A=(ts={},vpos={}) vs B=(ts={},vpos={})",
            prefix, idx, la.text_start, la.vertical_pos, lb.text_start, lb.vertical_pos
        ));
    }
    if pa.field_ranges.len() != pb.field_ranges.len() {
        diffs.push(format!(
            "{} field_ranges count: A={} vs B={}",
            prefix,
            pa.field_ranges.len(),
            pb.field_ranges.len()
        ));
    } else if let Some((idx, (fa, fb))) = pa
        .field_ranges
        .iter()
        .zip(pb.field_ranges.iter())
        .enumerate()
        .find(|(_, (fa, fb))| {
            fa.start_char_idx != fb.start_char_idx
                || fa.end_char_idx != fb.end_char_idx
                || fa.control_idx != fb.control_idx
        })
    {
        diffs.push(format!(
            "{} field_ranges[{}]: A=({}..{},c{}) vs B=({}..{},c{})",
            prefix,
            idx,
            fa.start_char_idx,
            fa.end_char_idx,
            fa.control_idx,
            fb.start_char_idx,
            fb.end_char_idx,
            fb.control_idx
        ));
    }
}

/// [#1807] 글상자 문단 목록 재귀 비교. 중첩 글상자(Shape in Shape)도 재귀한다.
fn diff_textbox_paragraph_lists(
    diffs: &mut Vec<String>,
    prefix: &str,
    pas: &[rhwp::model::paragraph::Paragraph],
    pbs: &[rhwp::model::paragraph::Paragraph],
) {
    use rhwp::model::control::Control;
    if pas.len() != pbs.len() {
        diffs.push(format!(
            "{} tb 문단 수: A={} vs B={}",
            prefix,
            pas.len(),
            pbs.len()
        ));
    }
    for (k, (pa, pb)) in pas.iter().zip(pbs.iter()).enumerate() {
        let p = format!("{} tb_p[{}]", prefix, k);
        diff_textbox_paragraph_fields(diffs, &p, pa, pb);
        for (cj, (ca, cb)) in pa.controls.iter().zip(pb.controls.iter()).enumerate() {
            if let (Control::Shape(sa), Control::Shape(sb)) = (ca, cb) {
                diff_shape_textbox(diffs, &format!("{}.ctrl[{}]", p, cj), sa, sb);
            }
        }
    }
}

/// [#1807] Shape 글상자 유무 + 내부 문단 재귀 비교 진입점.
fn diff_shape_textbox(
    diffs: &mut Vec<String>,
    prefix: &str,
    sa: &rhwp::model::shape::ShapeObject,
    sb: &rhwp::model::shape::ShapeObject,
) {
    let ta = sa.drawing().and_then(|d| d.text_box.as_ref());
    let tb = sb.drawing().and_then(|d| d.text_box.as_ref());
    match (ta, tb) {
        (Some(ta), Some(tb)) => {
            diff_textbox_paragraph_lists(diffs, prefix, &ta.paragraphs, &tb.paragraphs);
        }
        (Some(_), None) | (None, Some(_)) => {
            diffs.push(format!(
                "{} text_box 유무: A={} vs B={}",
                prefix,
                ta.is_some(),
                tb.is_some()
            ));
        }
        (None, None) => {}
    }
}

/// `tab_extended`(`[u16; 7]`) 두 인라인 탭 레코드가 **의미 있는** 필드에서 다른지 판정.
///
/// HWPX 파서(`parse_tab_extension`)는 인라인 탭을 `ext[0]`=width,
/// `ext[2]`=`type<<8 | leader`(leader 는 low byte), `ext[6]`=0x0009 마커로만 채우고
/// `ext[1]`·`ext[3]`·`ext[4]`·`ext[5]`는 0 으로 둔다. HWPX 직렬화(`render_hp_t_content`)도
/// width/leader/type 를 오직 `ext[0]`·`ext[2]`에서만 읽는다. 반면 HWP5 인라인 탭(8 WCHAR
/// 블록)은 `ext[1]`을 leader/fill 슬롯으로, `ext[3]`·`ext[4]`·`ext[5]`를 WCHAR 4~6 원본
/// 바이트(보통 0x20)로 채운다 — 이들은 HWPX `<hp:tab>`에 대응 속성이 없어 HWPX 쪽이 항상
/// 0 이라, HWPX↔HWP5 parity 비교에서 거의 모든 탭에 거짓 차이(0 vs leader, 0 vs 32)를 만들어
/// 실제 차이(width/type/leader)를 가린다. 따라서 두 포맷이 공통으로 쓰는 필드
/// [0]=width, [2]=type/leader 팩, [6]=마커만 비교하고 [1],[3],[4],[5]는 제외한다.
/// (HWP5 직렬화는 [1],[3..6]을 그대로 보존하므로 self-roundtrip 충실도에는 영향 없음 —
/// 도구 비교에서만 제외.)
fn tab_ext_semantic_differs(a: &[u16; 7], b: &[u16; 7]) -> bool {
    // 두 포맷 공통 필드만: [0]=width, [2]=type<<8|leader, [6]=0x0009 마커.
    // [1](HWP5 leader/fill 슬롯, HWPX=0)·[3]·[4]·[5](HWP5 예약 바이트, HWPX=0)는 제외.
    const SEMANTIC: [usize; 3] = [0, 2, 6];
    SEMANTIC.iter().any(|&k| a[k] != b[k])
}

/// [Task #2122] ir-diff 출력 상태 — 종전 fn-지역 macro(emit_header/emit_diff) 본문을
/// 메서드로 이관 (동작·출력 불변, macro 확장 인라인 제거).
struct IrDiffEmitter {
    summary_mode: bool,
    max_lines: Option<usize>,
    printed_lines: usize,
    truncated: bool,
    summary_buckets: std::collections::BTreeMap<String, u32>,
}

impl IrDiffEmitter {
    fn println_guarded(&mut self, line: String) {
        match self.max_lines {
            Some(limit) if self.printed_lines >= limit => {
                if !self.truncated {
                    println!("... 이하 생략 (--max-lines {} 도달)", limit);
                    self.truncated = true;
                }
            }
            _ => {
                println!("{}", line);
                self.printed_lines += 1;
            }
        }
    }
    /// paragraph/섹션 헤더. summary 모드에서는 출력 안 함, max_lines 초과 시 truncate.
    fn header(&mut self, line: String) {
        if !self.summary_mode {
            self.println_guarded(line);
        }
    }
    /// 차이 라인. summary 모드에서는 카테고리별 카운트, 일반 모드에서는 "  [차이] {}" 형식.
    /// 카테고리 추출: ":" 앞쪽 첫 토큰. controls[N].xxx 는 ".xxx" 만 추출.
    fn diff(&mut self, body: String) {
        if self.summary_mode {
            let prefix = body.split(':').next().unwrap_or(&body);
            let cat = if let Some(pos) = prefix.rfind(']') {
                prefix[pos + 1..].trim_start_matches('.').trim().to_string()
            } else {
                prefix.trim().to_string()
            };
            let key = if cat.is_empty() { body.clone() } else { cat };
            *self.summary_buckets.entry(key).or_insert(0) += 1;
        } else {
            self.println_guarded(format!("  [차이] {}", body));
        }
    }
}

/// [Task #2122] ir-diff 문단 단위 필드 비교 — 차이 문자열 목록 생산 (원본 무변경 이동).
fn ir_diff_paragraph_fields(
    pa: &rhwp::model::paragraph::Paragraph,
    pb: &rhwp::model::paragraph::Paragraph,
    doc_a: &rhwp::model::document::Document,
    doc_b: &rhwp::model::document::Document,
) -> Vec<String> {
    let mut diffs: Vec<String> = Vec::new();

    // 텍스트 비교
    if pa.text != pb.text {
        diffs.push(format!(
            "text: A={:?} vs B={:?}",
            pa.text.chars().take(30).collect::<String>(),
            pb.text.chars().take(30).collect::<String>()
        ));
    }

    // char_count 비교
    if pa.char_count != pb.char_count {
        diffs.push(format!("cc: A={} vs B={}", pa.char_count, pb.char_count));
    }

    // char_offsets 비교
    if pa.char_offsets != pb.char_offsets {
        let len_a = pa.char_offsets.len();
        let len_b = pb.char_offsets.len();
        if len_a != len_b {
            diffs.push(format!("char_offsets len: A={} vs B={}", len_a, len_b));
        } else {
            let first_diff = pa
                .char_offsets
                .iter()
                .zip(pb.char_offsets.iter())
                .enumerate()
                .find(|(_, (a, b))| a != b);
            if let Some((idx, (a, b))) = first_diff {
                diffs.push(format!("char_offsets[{}]: A={} vs B={}", idx, a, b));
            }
        }
    }

    // para_shape_id 비교
    if pa.para_shape_id != pb.para_shape_id {
        diffs.push(format!(
            "ps_id: A={} vs B={}",
            pa.para_shape_id, pb.para_shape_id
        ));
    }

    // tab_extended 비교
    if pa.tab_extended.len() != pb.tab_extended.len() {
        diffs.push(format!(
            "tab_ext count: A={} vs B={}",
            pa.tab_extended.len(),
            pb.tab_extended.len()
        ));
    } else {
        for (ti, (ta, tb)) in pa
            .tab_extended
            .iter()
            .zip(pb.tab_extended.iter())
            .enumerate()
        {
            if tab_ext_semantic_differs(ta, tb) {
                diffs.push(format!("tab_ext[{}]: A={:?} vs B={:?}", ti, ta, tb));
                break;
            }
        }
    }

    // LINE_SEG 비교
    if pa.line_segs.len() != pb.line_segs.len() {
        diffs.push(format!(
            "line_segs count: A={} vs B={}",
            pa.line_segs.len(),
            pb.line_segs.len()
        ));
    } else {
        for (li, (la, lb)) in pa.line_segs.iter().zip(pb.line_segs.iter()).enumerate() {
            if la.text_start != lb.text_start {
                diffs.push(format!(
                    "ls[{}].ts: A={} vs B={}",
                    li, la.text_start, lb.text_start
                ));
            }
            if la.vertical_pos != lb.vertical_pos {
                diffs.push(format!(
                    "ls[{}].vpos: A={} vs B={}",
                    li, la.vertical_pos, lb.vertical_pos
                ));
            }
            if la.line_height != lb.line_height {
                diffs.push(format!(
                    "ls[{}].lh: A={} vs B={}",
                    li, la.line_height, lb.line_height
                ));
            }
            if la.text_height != lb.text_height {
                diffs.push(format!(
                    "ls[{}].th: A={} vs B={}",
                    li, la.text_height, lb.text_height
                ));
            }
            if la.baseline_distance != lb.baseline_distance {
                diffs.push(format!(
                    "ls[{}].bl: A={} vs B={}",
                    li, la.baseline_distance, lb.baseline_distance
                ));
            }
            if la.line_spacing != lb.line_spacing {
                diffs.push(format!(
                    "ls[{}].ls: A={} vs B={}",
                    li, la.line_spacing, lb.line_spacing
                ));
            }
            if la.column_start != lb.column_start {
                diffs.push(format!(
                    "ls[{}].cs: A={} vs B={}",
                    li, la.column_start, lb.column_start
                ));
            }
            if la.segment_width != lb.segment_width {
                diffs.push(format!(
                    "ls[{}].sw: A={} vs B={}",
                    li, la.segment_width, lb.segment_width
                ));
            }
        }
    }

    // 컨트롤 식별 비교
    if pa.controls.len() != pb.controls.len() {
        diffs.push(format!(
            "controls count: A={} vs B={}",
            pa.controls.len(),
            pb.controls.len()
        ));
    }
    {
        use rhwp::model::control::Control;
        let ctrl_count = pa.controls.len().min(pb.controls.len());
        for ci in 0..ctrl_count {
            let ca = &pa.controls[ci];
            let cb = &pb.controls[ci];
            match (ca, cb) {
                (Control::Table(ta), Control::Table(tb)) => {
                    diff_table(&mut diffs, ci, ta, tb);
                }
                (Control::Picture(pic_a), Control::Picture(pic_b)) => {
                    diff_common_obj(&mut diffs, ci, "pic", &pic_a.common, &pic_b.common);
                }
                (Control::Shape(sa), Control::Shape(sb)) => {
                    diff_common_obj(&mut diffs, ci, "shape", sa.common(), sb.common());
                    // [#1807] 글상자 내부 문단 재귀 비교 — 직렬화 결함이
                    // 글상자 안에서 발생해도 검출되도록 (#1795 소거망 구멍)
                    diff_shape_textbox(&mut diffs, &format!("ctrl[{}] shape", ci), sa, sb);
                }
                _ if control_tag(ca) != control_tag(cb) => {
                    diffs.push(format!(
                        "ctrl[{}] type: A={} vs B={}",
                        ci,
                        control_tag(ca),
                        control_tag(cb)
                    ));
                }
                _ => {}
            }
        }
    }

    // char_shapes 비교
    if pa.char_shapes.len() != pb.char_shapes.len() {
        diffs.push(format!(
            "char_shapes count: A={} vs B={}",
            pa.char_shapes.len(),
            pb.char_shapes.len()
        ));
    } else {
        for (ci, (ca, cb)) in pa.char_shapes.iter().zip(pb.char_shapes.iter()).enumerate() {
            if ca.start_pos != cb.start_pos {
                diffs.push(format!(
                    "cs[{}].pos: A={} vs B={}",
                    ci, ca.start_pos, cb.start_pos
                ));
                break;
            }
            if ca.char_shape_id != cb.char_shape_id {
                diffs.push(format!(
                    "cs[{}].id: A={} vs B={}",
                    ci, ca.char_shape_id, cb.char_shape_id
                ));
                break;
            }
        }
    }
    diffs
}

/// [#4113 / #3918 승격 2호] `verify` — 편집 파이프라인의 독립 사후검증 게이트.
///
/// 기대 조건 집합을 문서 실측과 대조해 전부 만족이면 exit 0, 하나라도 어긋나면
/// **봉투를 먼저 내고** exit 3(판정 — #2707) — 판정은 데이터다(규칙 3). 실행
/// 실패는 stdout 을 비우고 exit 1, 조립 오류는 exit 2. 실측은 전부 기존 코어
/// 재사용이다: `page_count`·`grep`·`collect_field_records`·`detect_format`(규칙 2).

/// 두 문서의 IR 을 **전수** 대조한다 — `diagnostics::ir_field_sweep` 을 CLI 로 낸 것.
///
/// `ir-diff` 와 갈리는 점은 **비교 대상이 손으로 나열되지 않는다**는 것이다. `ir-diff` 는
/// 사건 대응으로 쌓인 화이트리스트라 `z_order`·도형 변환 행렬·표 속성 같은 것을 아예 보지
/// 않는다. 실제로 한글이 `ShapeObjBringToFront` 를 저장본에 적어 두었는데 `ir-diff` 는
/// "동일" 이라 답했고, 이 스윕은 `common.z_order` 가 1↔2 로 뒤바뀐 것을 그대로 짚었다.
///
/// 쓰임새는 **편집 액션의 자취를 재는 것**이다. 어떤 API 도 결과를 안 비추는 액션이라도
/// 저장본은 적으므로, 같은 문서의 앞뒤 저장본을 이걸로 대조하면 관측창이 생긴다
/// (`tools/hwpctrl_compat` 의 L3).

pub(crate) fn ir_sweep(args: &[String]) -> i32 {
    use rhwp::diagnostics::ir_field_sweep::{sweep_documents, tally};

    if args.len() < 2 {
        eprintln!("사용법: rhwp ir-sweep <파일A> <파일B> [--json] [--max-lines <N>]");
        return EXIT_USAGE;
    }
    let (file_a, file_b) = (&args[0], &args[1]);
    let mut json_mode = false;
    let mut max_lines: Option<usize> = None;
    let is_value = |idx: usize| idx < args.len() && !args[idx].starts_with('-');
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--max-lines" if is_value(i + 1) => {
                max_lines = args[i + 1].parse().ok();
                i += 2;
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
    }

    let mut load = |path: &String| match std::fs::read(path) {
        Ok(bytes) => match rhwp::parser::parse_document(&bytes) {
            Ok(doc) => Some(doc),
            Err(e) => {
                eprintln!("파싱 실패: {path} — {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("읽기 실패: {path} — {e}");
            None
        }
    };
    let (Some(doc_a), Some(doc_b)) = (load(file_a), load(file_b)) else {
        return EXIT_RUNTIME;
    };

    let report = match sweep_documents(&doc_a, &doc_b) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("전수 비교 실패: {e}");
            return EXIT_RUNTIME;
        }
    };
    // `examples()` 는 진단용 표본이라 상한이 있다 — 건수는 반드시 `total()` 을 쓴다.
    let total = report.total();
    let examples = report.examples();
    if json_mode {
        let rows: Vec<serde_json::Value> = examples
            .iter()
            .take(max_lines.unwrap_or(usize::MAX))
            .map(|d| serde_json::json!({ "path": d.path, "left": d.left, "right": d.right }))
            .collect();
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "a": file_a,
            "b": file_b,
            "identical": report.is_empty(),
            "diffCount": total,
            "truncated": rows.len() < total,
            "categories": tally(&report),
            "divergences": rows,
        });
        println!("{}", provenance::marked(envelope, "ir-sweep"));
        // `ir-diff` 와 같은 규약 — 차이가 있으면 3.
        return if report.is_empty() { EXIT_OK } else { 3 };
    }

    for d in examples.iter().take(max_lines.unwrap_or(200)) {
        println!("{} : {} → {}", d.path, d.left, d.right);
    }
    println!("\n=== 전수 비교 완료: 차이 {total} 건 ===");
    EXIT_OK
}

pub(crate) fn ir_diff(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("사용법: rhwp ir-diff <파일A> <파일B> [-s <구역>] [-p <문단>] [--summary] [--max-lines <N>] [--json]");
        // [#3274] 인자 부족은 사용법 오류다 — 종전엔 0 으로 끝나 스크립트가 감지 못했다.
        return EXIT_USAGE;
    }

    let file_a = &args[0];
    let file_b = &args[1];
    let mut section_filter: Option<usize> = None;
    let mut para_filter: Option<usize> = None;
    // [Task #653 보강] 출력 가드 옵션
    let mut summary_mode = false;
    let mut max_lines: Option<usize> = None;
    // [#3274] --json: 계약 봉투 한 줄(카테고리 요약 포함), 차이 발견 시 exit 3.
    let mut json_mode = false;

    // [#3274] 값을 받는 옵션은 다음 토큰이 플래그(`-` 시작)면 값으로 삼키지 않는다.
    // 종전엔 `--max-lines --json` 처럼 값을 빠뜨리면 "--json" 이 값으로 소비돼
    // json 모드가 조용히 꺼지고, 게이트를 기대한 스크립트가 차이를 통과로 오판했다.
    // (-s/-p/--max-lines 는 모두 비음수만 받으므로 `-` 로 시작하는 값은 없다.)
    let is_value = |idx: usize| idx < args.len() && !args[idx].starts_with('-');
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--section" if is_value(i + 1) => {
                section_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "-p" | "--para" if is_value(i + 1) => {
                para_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "--summary" => {
                summary_mode = true;
                i += 1;
            }
            "--max-lines" if is_value(i + 1) => {
                max_lines = args[i + 1].parse().ok();
                i += 2;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // [#3274] 읽기·파싱 실패는 exit 1 (#2707 정렬) — 종전엔 0 으로 끝나
    // "비교했고 차이 없음"과 "비교 자체를 못 함"을 구별할 수 없었다.
    let data_a = match fs::read(file_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_a, e);
            return EXIT_RUNTIME;
        }
    };
    let data_b = match fs::read(file_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_b, e);
            return EXIT_RUNTIME;
        }
    };

    // 일반 열기·내보내기 명령과 동일하게 전역 --password/--password-stdin을
    // 적용한다. 종전에는 ir-diff만 parse_document를 직접 호출해, 암호 문서가
    // 비교 대상이면 복호화 지원이 있어도 EncryptedDocument로 즉시 종료했다.
    // 비암호 문서는 parse_document_with_password가 비밀번호를 무시하므로, 암호/
    // 평문 counterpart 비교에도 하나의 입력 경로를 사용할 수 있다.
    let password = cli_password();
    let parse_for_ir_diff = |data: &[u8]| match password.as_deref() {
        Some(password) => rhwp::parser::parse_document_with_password(data, password.as_bytes()),
        None => rhwp::parser::parse_document(data),
    };

    let doc_a = match parse_for_ir_diff(&data_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패", file_a);
            return classify_hwp_error(&e.to_string()).report();
        }
    };
    let doc_b = match parse_for_ir_diff(&data_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패", file_b);
            return classify_hwp_error(&e.to_string()).report();
        }
    };

    let name_a = Path::new(file_a)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let name_b = Path::new(file_b)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    if !summary_mode && !json_mode {
        println!("=== IR 비교: {} vs {} ===", name_a, name_b);
    }

    // [Task #653 보강] 출력 가드 상태 — IrDiffEmitter 로 통합 (#2122)
    // [#3274] json 모드는 summary 와 같은 수집 전용 경로(버킷만 쌓고 무출력)를 탄다 —
    // stdout 순수성을 위해 텍스트 라인을 한 줄도 내면 안 된다.
    let mut em = IrDiffEmitter {
        summary_mode: summary_mode || json_mode,
        max_lines,
        printed_lines: 0,
        truncated: false,
        summary_buckets: std::collections::BTreeMap::new(),
    };

    let mut total_diffs = 0u32;

    // 구역 수 비교
    // [#3274] 종전엔 total_diffs 선언이 이 블록 뒤에 있어 구역 수 차이가 집계되지
    // 않았다. 텍스트 모드에선 차이 라인이 화면에 보여 무해했으나, --json 게이트에서는
    // 구역 하나가 덧붙은 변환본이 diffCount=0·identical:true·exit 0 으로 통과하는
    // 치명적 누락이었다(봉투 자기모순). 선언을 앞으로 올리고 여기서도 집계한다.
    if doc_a.sections.len() != doc_b.sections.len() {
        em.diff(format!(
            "구역 수: A={} vs B={}",
            doc_a.sections.len(),
            doc_b.sections.len()
        ));
        total_diffs += 1;
    }

    let sec_count = doc_a.sections.len().min(doc_b.sections.len());

    for sec_idx in 0..sec_count {
        if let Some(sf) = section_filter {
            if sec_idx != sf {
                continue;
            }
        }

        let sec_a = &doc_a.sections[sec_idx];
        let sec_b = &doc_b.sections[sec_idx];

        if sec_a.paragraphs.len() != sec_b.paragraphs.len() {
            em.diff(format!(
                "구역 {}: 문단 수 A={} vs B={}",
                sec_idx,
                sec_a.paragraphs.len(),
                sec_b.paragraphs.len()
            ));
            total_diffs += 1;
        }

        let para_count = sec_a.paragraphs.len().min(sec_b.paragraphs.len());
        for pi in 0..para_count {
            if let Some(pf) = para_filter {
                if pi != pf {
                    continue;
                }
            }

            let pa = &sec_a.paragraphs[pi];
            let pb = &sec_b.paragraphs[pi];
            let diffs = ir_diff_paragraph_fields(pa, pb, &doc_a, &doc_b);

            if !diffs.is_empty() {
                let text_preview: String = pa.text.chars().take(30).collect();
                em.header(format!(
                    "\n--- 문단 {}.{} --- \"{}\"",
                    sec_idx, pi, text_preview
                ));
                for d in &diffs {
                    em.diff(format!("{}", d));
                }
                total_diffs += diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: ParaShape
    {
        let ps_a = &doc_a.doc_info.para_shapes;
        let ps_b = &doc_b.doc_info.para_shapes;
        if ps_a.len() != ps_b.len() {
            em.diff(format!(
                "ParaShape 수: A={} vs B={}",
                ps_a.len(),
                ps_b.len()
            ));
            total_diffs += 1;
        }
        let ps_count = ps_a.len().min(ps_b.len());
        for i in 0..ps_count {
            let a = &ps_a[i];
            let b = &ps_b[i];
            let mut ps_diffs: Vec<String> = Vec::new();
            if a.margin_left != b.margin_left {
                ps_diffs.push(format!("ml: {}vs{}", a.margin_left, b.margin_left));
            }
            if a.margin_right != b.margin_right {
                ps_diffs.push(format!("mr: {}vs{}", a.margin_right, b.margin_right));
            }
            if a.indent != b.indent {
                ps_diffs.push(format!("indent: {}vs{}", a.indent, b.indent));
            }
            if a.tab_def_id != b.tab_def_id {
                ps_diffs.push(format!("tab_def: {}vs{}", a.tab_def_id, b.tab_def_id));
            }
            if a.spacing_before != b.spacing_before {
                ps_diffs.push(format!("sb: {}vs{}", a.spacing_before, b.spacing_before));
            }
            if a.spacing_after != b.spacing_after {
                ps_diffs.push(format!("sa: {}vs{}", a.spacing_after, b.spacing_after));
            }
            if a.line_spacing != b.line_spacing {
                ps_diffs.push(format!("ls: {}vs{}", a.line_spacing, b.line_spacing));
            }
            if !ps_diffs.is_empty() {
                em.diff(format!("PS[{}] {}", i, ps_diffs.join(", ")));
                total_diffs += ps_diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: TabDef
    {
        let td_a = &doc_a.doc_info.tab_defs;
        let td_b = &doc_b.doc_info.tab_defs;
        if td_a.len() != td_b.len() {
            em.diff(format!("TabDef 수: A={} vs B={}", td_a.len(), td_b.len()));
            total_diffs += 1;
        }
        let td_count = td_a.len().min(td_b.len());
        for i in 0..td_count {
            let a = &td_a[i];
            let b = &td_b[i];
            if a.tabs.len() != b.tabs.len() {
                em.diff(format!(
                    "TD[{}] 탭 수: A={} vs B={}",
                    i,
                    a.tabs.len(),
                    b.tabs.len()
                ));
                total_diffs += 1;
            } else {
                for (ti, (ta, tb)) in a.tabs.iter().zip(b.tabs.iter()).enumerate() {
                    if ta.position != tb.position
                        || ta.tab_type != tb.tab_type
                        || ta.fill_type != tb.fill_type
                    {
                        em.diff(format!(
                            "TD[{}][{}] pos: {}vs{}, type: {}vs{}, fill: {}vs{}",
                            i,
                            ti,
                            ta.position,
                            tb.position,
                            ta.tab_type,
                            tb.tab_type,
                            ta.fill_type,
                            tb.fill_type
                        ));
                        total_diffs += 1;
                    }
                }
            }
        }
    }

    // [Task #653 보강] 요약 모드 출력 — 카테고리별 카운트 (내림차순 → 알파벳)
    // [#3274] --summary --json 병용 시 JSON 이 이긴다 — stdout 순수성 우선.
    if summary_mode && !json_mode {
        println!("=== 카테고리별 차이 요약 ===");
        let mut entries: Vec<(String, u32)> = em.summary_buckets.clone().into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (cat, count) in &entries {
            println!("  {:>5}건  {}", count, cat);
        }
    }

    if json_mode {
        // [#3274] 계약 봉투 한 줄 — 카테고리 버킷(BTreeMap)은 키 정렬이 결정적이다.
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "a": file_a,
            "b": file_b,
            "identical": total_diffs == 0,
            "diffCount": total_diffs,
            "categories": em.summary_buckets,
        });
        println!("{}", provenance::marked(envelope, "ir-diff"));
        // 차이 발견 = 3: #2707 의 "--verify IR 차이" 코드와 같은 의미의 게이트 신호.
        return if total_diffs == 0 { EXIT_OK } else { 3 };
    }

    println!("\n=== 비교 완료: 차이 {} 건 ===", total_diffs);
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::tab_ext_semantic_differs;

    #[test]
    fn tab_ext_reserved_fields_ignored() {
        // 같은 문서의 HWPX(파서가 [1],[3..6]=0) vs HWP5([1]=leader/fill 슬롯, [3..6]=원본 바이트).
        // 이 포맷 비대칭 슬롯들은 모두 무시 → 의미 차이 없음.
        let hwpx = [1640, 0, 256, 0, 0, 0, 9];
        let hwp5 = [1640, 5, 256, 32, 32, 32, 9];
        assert!(!tab_ext_semantic_differs(&hwpx, &hwp5));
    }

    #[test]
    fn tab_ext_semantic_fields_detected() {
        let base = [1640, 0, 256, 0, 0, 0, 9];
        assert!(!tab_ext_semantic_differs(&base, &base));
        // width([0]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1641, 0, 256, 0, 0, 0, 9]));
        // type([2] high byte) 차이 검출 — 256(0x0100)→512(0x0200)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 512, 0, 0, 0, 9]));
        // leader([2] low byte, 두 포맷 공통) 차이 검출 — 256(0x0100)→257(0x0101)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 257, 0, 0, 0, 9]));
        // HWP5 leader/fill 슬롯([1], HWPX는 항상 0)은 포맷 비대칭이라 무시 — 차이로 치지 않음
        assert!(!tab_ext_semantic_differs(
            &base,
            &[1640, 1, 256, 0, 0, 0, 9]
        ));
        // marker([6]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 256, 0, 0, 0, 0]));
    }
}
