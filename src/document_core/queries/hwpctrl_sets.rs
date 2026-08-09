//! 웹한글컨트롤 ParameterSet 값 — `CharShape`·`ParaShape` (규격 §8.2.2, §8.2.11).
//!
//! 한글은 서식을 **파라미터셋**으로 돌려준다. 항목 이름과 단위가 rhwp 모델과 다르므로
//! (`Height` 는 HWPUNIT, `AlignType` 은 코드값) 그 번역을 데이터 곁인 여기에 둔다.
//! 좌표는 한글 커서 좌표(list/para/pos)를 쓴다 — 호출 측이 구역·문단으로 옮기려면
//! 리스트 표를 다시 만들어야 한다.

use crate::document_core::queries::field_query::{
    caret_stops, cell_path_to_list, char_idx_at_stream_pos, cursor_paragraph, json_escape,
    leading_anchor_pos, root_para_count, root_para_location, root_para_of, select_start_pos,
    shape_lists, stream_len, stream_pos, word_end_from, word_starts, ListEntry,
    EXTENDED_CTRL_UNITS, ROOT_LIST_ID,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{AutoNumber, AutoNumberType, Control};
use crate::model::page::ColumnDef;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::{Caption, ShapeObject, TextBox};
use crate::model::style::{Alignment, HeadType, LineSpacingType, UnderlineType};

/// 언어 일곱 갈래 — 항목 이름 접미사 순서가 모델 배열 순서와 같다.
const LANGS: [&str; 7] = [
    "Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User",
];

fn bit(value: bool) -> u8 {
    u8::from(value)
}

/// 자리·크기·순서를 가진 개체의 공통 속성. 없는 갈래(누름틀·구역 정의 따위)는 `None` 이다.
fn control_common(ctrl: &Control) -> Option<&crate::model::shape::CommonObjAttr> {
    match ctrl {
        Control::Table(t) => Some(&t.common),
        Control::Shape(s) => Some(s.common()),
        Control::Picture(p) => Some(&p.common),
        _ => None,
    }
}

fn control_common_mut(ctrl: &mut Control) -> Option<&mut crate::model::shape::CommonObjAttr> {
    match ctrl {
        Control::Table(t) => Some(&mut t.common),
        Control::Shape(s) => Some(s.common_mut()),
        Control::Picture(p) => Some(&mut p.common),
        _ => None,
    }
}

/// 문단 켜고끄기 비트 — 원본은 `attr1`, 5.0.1.7 이후 문서는 `attr2` 에 같은 뜻을 싣는다.
fn para_flag(shape: &crate::model::style::ParaShape, attr1_bit: u32, attr2_bit: u32) -> bool {
    (shape.attr1 >> attr1_bit) & 1 != 0 || (shape.attr2 >> attr2_bit) & 1 != 0
}

/// 한글 `AlignType` 코드 — 0 양쪽혼합 · 1 왼쪽 · 2 오른쪽 · 3 가운데 · 4 배분 · 5 나눔.
fn align_code(alignment: Alignment) -> u8 {
    match alignment {
        Alignment::Justify => 0,
        Alignment::Left => 1,
        Alignment::Right => 2,
        Alignment::Center => 3,
        Alignment::Distribute => 4,
        Alignment::Split => 5,
    }
}

/// 한글 `LineSpacingType` 코드 — 0 글자에 따라(%) · 1 고정값 · 2 여백만 지정.
fn line_spacing_code(kind: LineSpacingType) -> u8 {
    match kind {
        LineSpacingType::Percent => 0,
        LineSpacingType::Fixed => 1,
        LineSpacingType::SpaceOnly => 2,
        LineSpacingType::Minimum => 3,
    }
}

/// 한글 `HeadingType` 코드 — 0 없음 · 1 개요 · 2 번호 · 3 불릿.
fn heading_code(kind: HeadType) -> u8 {
    match kind {
        HeadType::None => 0,
        HeadType::Outline => 1,
        HeadType::Number => 2,
        HeadType::Bullet => 3,
    }
}

/// 한글 `UnderlineType` 코드 — 0 없음 · 1 아래 · 2 위.
fn underline_code(kind: UnderlineType) -> u8 {
    match kind {
        UnderlineType::None => 0,
        UnderlineType::Bottom => 1,
        UnderlineType::Top => 2,
    }
}

/// 컨트롤 하나의 `(CtrlID, CtrlCh, UserDesc)` — 한글이 주는 값 그대로다(실측).
///
/// `CtrlCh` 는 스트림에서의 글자 코드다: 구역·단 정의처럼 문단에 붙는 표식은 **2**, 표·그리기
/// 같은 개체는 **11**. 아직 못 본 갈래는 짐작으로 채우지 않고 빈 이름으로 둔다 — 없는 값을
/// 그럴듯하게 채우면 "모른다"가 사라진다.
fn control_identity(ctrl: &Control) -> (&'static str, u32, &'static str) {
    match ctrl {
        Control::SectionDef(_) => ("secd", 2, "구역 정의"),
        Control::ColumnDef(_) => ("cold", 2, "단 정의"),
        Control::Table(_) => ("tbl", 11, "표"),
        // 그리기 개체의 이름은 갈래마다 다르다("사각형"·"타원" …). rhwp 가 이미 같은 이름을
        // 들고 있어서 그대로 쓴다(오라클 실측과 일치). 묶음만 다르다 — rhwp 는 "묶음",
        // 한글은 **"그리기"** 다(실측).
        // 그림은 `shape_name()` 이 "그림(묶음내)" 를 준다 — 그 이름은 렌더 진단이 묶음 안에
        // 있음을 표시하려고 붙인 것이고, 한글이 부르는 이름은 묶음을 풀든 말든 **"그림"** 이다
        // (묶음 풀기 실측). 여기서만 걷어낸다 — `shape_name()` 은 다른 곳이 딛고 있다.
        Control::Shape(shape) => (
            "gso",
            11,
            match **shape {
                crate::model::shape::ShapeObject::Group(_) => "그리기",
                crate::model::shape::ShapeObject::Picture(_) => "그림",
                _ => shape.shape_name(),
            },
        ),
        Control::Picture(_) => ("gso", 11, "그림"),
        Control::Equation(_) => ("eqed", 11, "수식"),
        Control::Header(_) => ("head", 2, "머리말"),
        Control::Footer(_) => ("foot", 2, "꼬리말"),
        Control::Footnote(_) => ("fn", 2, "각주"),
        Control::Endnote(_) => ("en", 2, "미주"),
        // 한글이 부르는 이름은 "번호 넣기" 다(실측). rhwp 안에서 쓰는 이름과 다르다.
        // `CtrlCh` 는 스트림 글자 코드다 — 자동 번호는 `0x12` = 18 이다(실측).
        Control::AutoNumber(_) => ("atno", 18, "번호 넣기"),
        Control::NewNumber(_) => ("nwno", 2, "새 번호 지정"),
        Control::PageNumberPos(_) => ("pgnp", 2, "쪽 번호 위치"),
        Control::PageHide(_) => ("pghd", 2, "감추기"),
        Control::Bookmark(_) => ("bokm", 2, "책갈피"),
        Control::HiddenComment(_) => ("tcmt", 2, "숨은 설명"),
        _ => ("", 0, ""),
    }
}

/// 문단 하나가 담은 컨트롤을 사슬에 넣는다 — **자기 다음에 자기 속을** 넣는 깊이 우선이다.
///
/// 셀 안의 표도 사슬에 든다(한글 실측: 중첩 표가 있는 문서에서 `tbl` 이 하나 더 나온다).
/// 리스트 번호를 매기는 규칙(§4.9)과 같은 걸음이라 둘이 어긋나지 않는다.
fn collect_controls(
    para: &Paragraph,
    at: (u32, usize),
    lists: &[ListEntry],
    items: &mut Vec<String>,
) {
    let (list_id, para_in_list) = at;
    let control_positions = para.control_text_positions();
    // 자리표 글자가 **없는** 문단은 아래 식이 안 맞는다. 컨트롤 여럿이 같은 글자 번호를
    // 가리키면(0·0·0 …) 그것이 그 신호다 — 자리표가 있으면 번호가 서로 다르다.
    //
    // 그런 문단은 스트림을 되짚어 세운다: 자리 0 에서 시작해 다음 글자의 스트림 자리가 지금
    // 자리와 같으면 글자를 놓고 한 칸, 아니면 컨트롤을 놓고 여덟 칸 간다. 자리차지 개체 사이에
    // 공백이 한 칸씩 있는 문단(실측 앵커 16·25·34)이 이 길로 맞아떨어진다.
    let rebuilt: Option<Vec<usize>> = {
        let mut seen = control_positions.clone();
        seen.sort_unstable();
        let has_dup = seen.windows(2).any(|w| w[0] == w[1]);
        // **첫 글자 자리가 0 이면 대응표를 못 믿는다** — 앞머리에 컨트롤이 있는데 0 이라는 것은
        // 그 표가 컨트롤을 안 담았다는 뜻이다(`leading_anchor_pos` 와 같은 단서). 그때 이 되짚기를
        // 쓰면 자리가 통째로 어긋난다(그림을 넣은 문단에서 앵커가 20 대신 406 으로 나왔다).
        let offsets_trustworthy = para.char_offsets.first().is_some_and(|o| *o > 0);
        if has_dup && offsets_trustworthy {
            let mut out = Vec::with_capacity(para.controls.len());
            let mut p = 0usize;
            let mut chars = para.char_offsets.iter().peekable();
            for _ in 0..para.controls.len() {
                while chars.peek().is_some_and(|off| (**off as usize) <= p) {
                    chars.next();
                    p += 1;
                }
                out.push(p);
                p += EXTENDED_CTRL_UNITS;
            }
            Some(out)
        } else {
            None
        }
    };
    for (ci, ctrl) in para.controls.iter().enumerate() {
        let (id, ch, desc) = control_identity(ctrl);
        // 앵커 자리는 그 컨트롤이 **스트림에서 서 있는 자리**다(실측: 본문 첫 문단의 셋이
        // 0·8·16, 셀 안의 표는 그 문단의 글자 자리 그대로 `3/14/2`).
        //
        // `stream_pos` 로는 못 구한다 — 자리표 글자를 남기는 문단과 안 남기는 문단이 섞여
        // 있어 그 함수 하나로 갈리지 않는다. 여기서는 **앞선 것들만** 세면 된다:
        // 앞의 맨 글자 수 + 8 × 앞의 컨트롤 수.
        let pos = if let Some(rebuilt) = &rebuilt {
            rebuilt.get(ci).copied().unwrap_or(0)
        } else {
            control_positions
                .get(ci)
                .map(|char_idx| {
                    let placeholders_before = control_positions[..ci]
                        .iter()
                        .filter(|p| *p < char_idx)
                        .count();
                    let plain_before = char_idx.saturating_sub(placeholders_before);
                    plain_before + ci * EXTENDED_CTRL_UNITS
                })
                .unwrap_or(0)
        };
        items.push(format!(
            "{{\"ctrlId\":{},\"ctrlCh\":{},\"userDesc\":{},\"list\":{},\"para\":{},\"pos\":{},\"controlIndex\":{},\"props\":{}}}",
            json_escape(id),
            ch,
            json_escape(desc),
            list_id,
            para_in_list,
            pos,
            ci,
            control_props_json(ctrl),
        ));

        // 자기 다음에 자기 속으로 — 리스트 번호를 매기는 걸음(§4.9)과 같은 순서다.
        let child_of = |cell_index: usize| {
            lists
                .iter()
                .find(|l| {
                    l.host_list_id == list_id
                        && l.host_para_index == para_in_list
                        && l.control_index == ci
                        && l.cell_index == cell_index
                })
                .map(|l| l.list_id)
        };
        match ctrl {
            Control::Table(table) => {
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    let Some(child) = child_of(cell_index) else {
                        continue;
                    };
                    for (pi, cell_para) in cell.paragraphs.iter().enumerate() {
                        collect_controls(cell_para, (child, pi), lists, items);
                    }
                }
            }
            Control::Shape(shape) => {
                // 글상자와 **캡션** 둘 다 리스트를 연다 — 리스트 표와 같은 번호를 쓴다.
                for (node, paragraphs) in shape_lists(shape) {
                    let Some(child) = child_of(node) else {
                        continue;
                    };
                    for (pi, box_para) in paragraphs.iter().enumerate() {
                        collect_controls(box_para, (child, pi), lists, items);
                    }
                }
            }
            _ => {}
        }
    }
}

/// CP949 로 못 담는 글자를 `&#N;` 수치 참조로 바꾼다 — `GetTextFile("TEXT")` 의 규칙이다.
///
/// 판정은 **인코딩을 실제로 해 본다**. 표를 손으로 적으면 반드시 틀린다 — CP949 는 EUC-KR 에
/// 마이크로소프트 확장이 붙은 것이라 `€`·`①` 처럼 "없을 것 같은데 있는" 글자가 많다.
fn escape_outside_cp949(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut buf = [0u8; 4];
    for ch in text.chars() {
        let one = ch.encode_utf8(&mut buf);
        let (_, _, had_errors) = encoding_rs::EUC_KR.encode(one);
        if had_errors {
            out.push_str(&format!("&#{};", ch as u32));
        } else {
            out.push(ch);
        }
    }
    out
}

/// 스캔 항목 하나를 담는다. 상태는 **앞 항목과의 관계**라 넣는 자리에서 정한다.
///
/// - 문단이 바뀌면 3, 같은 문단 안에서 이어지면 2.
/// - 개체 리스트로 들어가는 첫 항목은 4, 나온 뒤 첫 항목은 5.
fn scan_push(items: &mut Vec<(u8, String, u8)>, state: u8, text: String) {
    items.push((state, text, 0));
}

/// 문단 하나를 스캔 차례로 푼다 — 글은 개체에서 끊기고, 개체 속을 돈 뒤 이어진다.
fn scan_paragraph(
    para: &Paragraph,
    at: (u32, usize),
    lists: &[ListEntry],
    items: &mut Vec<(u8, String, u8)>,
) {
    let (list_id, para_in_list) = at;
    // 그 리스트의 **첫 문단**이면 2, 아니면 3(같은 리스트의 다음 문단)이다. 셀은 저마다 다른
    // 리스트라 첫 문단뿐이어서 2 로 나온다 — 실측과 맞는다.
    let mut state: u8 = if para_in_list == 0 { 2 } else { 3 };

    // 구역·단 정의는 빈 항목을 하나씩 낸다. 그 뒤로는 같은 문단이므로 2 다.
    //
    // 이 항목들은 **표식**이라고 따로 적어 둔다 — `GetTextFile` 은 글을 이어 붙일 때 이것들만
    // 뺀다(실측: 표식 둘이 든 문서의 글이 `\r\n` 둘로 시작하지 넷이 아니다).
    for ctrl in para.controls.iter() {
        if matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)) {
            items.push((state, String::new(), 1));
            state = 2;
        }
    }

    let control_positions = para.control_text_positions();

    // **빈 누름틀은 안내문을 글로 낸다.** 한글은 파일을 열며 빈 필드에 안내문을 채우고, 그
    // 글이 스트림에도 들어간다(좌표 셈의 `guide_units` 가 이미 그것을 센다). 안 넣으면 서식
    // 문서의 글이 통째로 짧아진다.
    let mut inserts: Vec<(usize, String)> = Vec::new();
    for fr in para.field_ranges.iter() {
        if fr.start_char_idx != fr.end_char_idx {
            continue;
        }
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if let Some(guide) = field.guide_text() {
                if !guide.is_empty() {
                    inserts.push((fr.start_char_idx, guide.to_string()));
                }
            }
        }
    }
    inserts.sort_by_key(|(at, _)| *at);

    // 원래 글자 번호 → 안내문을 끼운 뒤의 번호. 컨트롤 자리를 옮길 때 쓴다.
    let raw: Vec<char> = para.text.chars().collect();
    let mut chars: Vec<char> = Vec::with_capacity(raw.len());
    let mut shift_at: Vec<(usize, usize)> = Vec::new(); // (원래 번호, 그 앞까지 밀린 양)
    let mut shift = 0usize;
    let mut next_insert = 0usize;
    for (i, ch) in raw.iter().enumerate() {
        while next_insert < inserts.len() && inserts[next_insert].0 == i {
            let text = &inserts[next_insert].1;
            chars.extend(text.chars());
            shift += text.chars().count();
            next_insert += 1;
        }
        shift_at.push((i, shift));
        chars.push(*ch);
    }
    while next_insert < inserts.len() {
        chars.extend(inserts[next_insert].1.chars());
        shift += inserts[next_insert].1.chars().count();
        next_insert += 1;
    }
    let shift_for = |orig: usize| -> usize {
        match shift_at.binary_search_by_key(&orig, |(i, _)| *i) {
            Ok(k) => shift_at[k].1,
            Err(_) => shift,
        }
    };
    let mut cut = 0usize;

    for (ci, ctrl) in para.controls.iter().enumerate() {
        let nested: Vec<(usize, &[Paragraph])> = match ctrl {
            Control::Table(table) => (0..table.cells.len())
                .filter_map(|cell| {
                    table
                        .cells
                        .get(cell)
                        .map(|c| (cell, c.paragraphs.as_slice()))
                })
                .collect(),
            Control::Shape(shape) => shape_lists(shape),
            _ => Vec::new(),
        };
        // 구역·단 정의는 위에서 이미 항목을 냈다.
        if matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)) {
            continue;
        }
        // **리스트가 없는 개체에서도 글은 끊긴다.** 수식이 그렇다(실측: 수식 앞 다섯 칸이
        // 한 항목으로 따로 난다). 리스트 있는 것만 끊다가 항목이 하나씩 모자랐다.
        //
        // 개체 앞까지의 글은 **비어 있어도 낸다**(글 없는 문단에 표만 있어도 빈 항목이 하나
        // 난다). 줄 끝은 아직 안 붙인다 — 문단이 안 끝났다.
        let here = control_positions
            .get(ci)
            .copied()
            .map(|orig| orig + shift_for(orig))
            .unwrap_or(chars.len())
            .min(chars.len());
        let run: String = chars[cut.min(here)..here].iter().collect();
        // 리스트를 여는 개체(표·글상자) 앞 조각은 줄이 되고, 인라인 개체(수식) 앞 조각은
        // 안 된다 — `GetTextFile` 이 그 둘을 다르게 잇는다(실측).
        items.push((state, run, if nested.is_empty() { 2 } else { 0 }));
        state = 2;
        cut = here;
        // 개체 속으로 — 첫 항목이 4, 나온 뒤 첫 항목이 5 다. 속이 없으면 여기서 끝이다.
        if nested.is_empty() {
            continue;
        }
        let mut entered = false;
        for (node, paragraphs) in nested {
            let child = lists
                .iter()
                .find(|l| {
                    l.host_list_id == list_id
                        && l.host_para_index == para_in_list
                        && l.control_index == ci
                        && l.cell_index == node
                })
                .map(|l| l.list_id);
            let Some(child_list) = child else { continue };
            for (pi, child_para) in paragraphs.iter().enumerate() {
                let before = items.len();
                scan_paragraph(child_para, (child_list, pi), lists, items);
                // **개체의 맨 첫 항목만** 진입 상태로 바꾼다. 그 뒤 문단들은 자기 자리대로
                // (같은 리스트의 다음 문단이면 3) 두어야 한다 — 여기서 전부 2 로 덮었다가
                // 글상자 안 문단들이 어긋났다.
                //
                // 진입 상태는 **어디서 들어가느냐**로 갈린다: 본문에서면 4, 이미 리스트 안이면
                // 5 다(실측: 같은 표라도 본문에 있으면 4, 글상자 안에 있으면 5).
                if !entered {
                    if let Some(first) = items.get_mut(before) {
                        first.0 = if list_id == ROOT_LIST_ID { 4 } else { 5 };
                    }
                    entered = true;
                }
            }
        }
        if entered {
            state = 5;
        }
    }

    // 남은 글 + 줄 끝.
    let tail: String = chars[cut.min(chars.len())..].iter().collect();
    scan_push(items, state, format!("{}\r\n", tail));
}

/// 컨트롤의 `Properties` 파라미터셋 — 채울 수 있는 항목만 낸다.
///
/// `Lock` 이 특히 중요하다: **잠긴 개체는 `SelectCtrlFront` 가 건너뛴다**(실측 — 이 표본의
/// 표 열둘 중 잠긴 셋만 정확히 빠진다). §4.34 가 못 풀던 "왜 어떤 개체는 안 골리는가"의 답이다.
///
/// `TextWrap`(본문과의 배치)은 `attr` **비트 21‥22** 다 — 짐작이 아니라 실측으로 뽑았다:
/// 오라클이 준 값 `0`·`1`·`3` 여섯 짝을 두 문서에서 모아 맞는 비트 구간을 찾았다.
/// 한때 이 값이 개체 고르기를 가른다고 봤으나 **반증됐다**(§4.44) — 다른 표본이 어긋났고,
/// 실제로는 그 개체가 캐럿보다 앞이라 안 걸린 것뿐이다. 값은 그대로 싣되 규칙에는 안 쓴다.
///
/// `VertRelTo`·`HorzRelTo` 같은 나머지는 아직 **넣지 않는다** — 짐작으로 채우면 "모른다"가
/// 사라진다(`CharShape` 와 같은 규칙).
fn control_props_json(ctrl: &Control) -> String {
    let common = match ctrl {
        Control::Table(t) => Some(&t.common),
        Control::Shape(s) => Some(s.common()),
        Control::Picture(p) => Some(&p.common),
        _ => None,
    };
    let Some(c) = common else {
        return "{}".to_string();
    };
    // 자리(`*Offset`)도 낸다 — 옮기기 액션의 유일한 관측창이다. 오라클의 이 셋은 32항목이라
    // 여기 있는 것은 그중 확인한 것뿐이다.
    format!(
        "{{\"Lock\":{},\"TreatAsChar\":{},\"AllowOverlap\":{},\"TextWrap\":{},\
         \"Width\":{},\"Height\":{},\"HorzOffset\":{},\"VertOffset\":{}}}",
        u8::from(c.locked),
        u8::from(c.treat_as_char),
        u8::from(c.allow_overlap),
        (c.attr >> 21) & 0x03,
        c.width,
        c.height,
        c.horizontal_offset,
        c.vertical_offset,
    )
}

impl DocumentCore {
    /// `HwpCtrl.CharShape` 가 돌려줄 값들 (규격 §8.2.2).
    ///
    /// 아직 못 채우는 항목(`FontType*`·`SmallCaps`·`BorderFill`)은 **넣지 않는다** —
    /// 없는 값을 0 으로 채우면 "모른다"와 "0이다"가 구별되지 않는다.
    pub fn char_shape_set_json(&self, list_id: u32, para_in_list: usize, pos: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        let char_idx = char_idx_at_stream_pos(para, pos);
        let shape_id = para.char_shape_id_at(char_idx).unwrap_or(0);
        let Some(raw) = self.document.doc_info.char_shapes.get(shape_id as usize) else {
            return "{}".to_string();
        };
        let style = self.styles.char_styles.get(shape_id as usize);

        let mut items: Vec<String> = Vec::new();
        for (i, lang) in LANGS.iter().enumerate() {
            if let Some(cs) = style {
                let raw_name = cs.font_family_for_lang(i);
                let name = crate::renderer::style_resolver::primary_font_name(&raw_name);
                items.push(format!("\"FaceName{}\":{}", lang, json_escape(name)));
            }
            items.push(format!("\"Size{}\":{}", lang, raw.relative_sizes[i]));
            items.push(format!("\"Ratio{}\":{}", lang, raw.ratios[i]));
            items.push(format!("\"Spacing{}\":{}", lang, raw.spacings[i]));
            items.push(format!("\"Offset{}\":{}", lang, raw.char_offsets[i]));
        }
        items.push(format!("\"Height\":{}", raw.base_size));
        items.push(format!("\"Bold\":{}", bit(raw.bold)));
        items.push(format!("\"Italic\":{}", bit(raw.italic)));
        items.push(format!("\"Emboss\":{}", bit(raw.emboss)));
        items.push(format!("\"Engrave\":{}", bit(raw.engrave)));
        items.push(format!("\"SuperScript\":{}", bit(raw.superscript)));
        items.push(format!("\"SubScript\":{}", bit(raw.subscript)));
        items.push(format!(
            "\"UnderlineType\":{}",
            underline_code(raw.underline_type)
        ));
        items.push(format!("\"UnderlineShape\":{}", raw.underline_shape));
        items.push(format!("\"OutlineType\":{}", raw.outline_type));
        items.push(format!("\"ShadowType\":{}", raw.shadow_type));
        items.push(format!("\"ShadowOffsetX\":{}", raw.shadow_offset_x));
        items.push(format!("\"ShadowOffsetY\":{}", raw.shadow_offset_y));
        items.push(format!("\"StrikeOutType\":{}", bit(raw.strikethrough)));
        items.push(format!("\"DiacSymMark\":{}", raw.emphasis_dot));
        items.push(format!("\"UseFontSpace\":{}", bit(raw.use_font_space)));
        items.push(format!("\"UseKerning\":{}", bit(raw.kerning)));
        items.push(format!("\"TextColor\":{}", raw.text_color));
        items.push(format!("\"ShadeColor\":{}", raw.shade_color));
        items.push(format!("\"UnderlineColor\":{}", raw.underline_color));
        items.push(format!("\"ShadowColor\":{}", raw.shadow_color));
        format!("{{{}}}", items.join(","))
    }

    /// `HwpCtrl.ParaShape` 가 돌려줄 값들 (규격 §8.2.11).
    pub fn para_shape_set_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        let Some(shape) = self
            .document
            .doc_info
            .para_shapes
            .get(para.para_shape_id as usize)
        else {
            return "{}".to_string();
        };
        let items = [
            format!("\"LeftMargin\":{}", shape.margin_left),
            format!("\"RightMargin\":{}", shape.margin_right),
            format!("\"Indentation\":{}", shape.indent),
            format!("\"PrevSpacing\":{}", shape.spacing_before),
            format!("\"NextSpacing\":{}", shape.spacing_after),
            format!("\"LineSpacing\":{}", shape.line_spacing),
            format!(
                "\"LineSpacingType\":{}",
                line_spacing_code(shape.line_spacing_type)
            ),
            format!("\"AlignType\":{}", align_code(shape.alignment)),
            format!("\"HeadingType\":{}", heading_code(shape.head_type)),
            format!("\"Level\":{}", shape.para_level),
            // 켜고 끄는 비트들 — attr1 이 원본이고 attr2 는 5.0.1.7 이후 확장이라 둘 다 본다.
            format!("\"WidowOrphan\":{}", bit(para_flag(shape, 16, 5))),
            format!("\"KeepWithNext\":{}", bit(para_flag(shape, 17, 6))),
            format!("\"KeepLinesTogether\":{}", bit(para_flag(shape, 18, 7))),
            format!("\"PagebreakBefore\":{}", bit(para_flag(shape, 19, 8))),
        ];
        format!("{{{}}}", items.join(","))
    }

    /// 커서 좌표(list/para/pos)로 글자 서식을 건다 — 웹한글컨트롤 `Run("CharShape*")` 용.
    ///
    /// `end_pos` 가 문단 길이를 넘으면 끝까지로 자른다(셀 블록처럼 "이 문단 전부"를 뜻할 때
    /// `u32::MAX` 를 주면 된다). `pos` 는 코드 유닛, rhwp 서식 API 는 글자 번호라 여기서 옮긴다.
    pub fn apply_char_format_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        start_pos: usize,
        end_pos: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let (start_char, end_char) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let last = para.text.chars().count();
            (
                char_idx_at_stream_pos(para, start_pos).min(last),
                char_idx_at_stream_pos(para, end_pos).min(last),
            )
        };
        if start_char >= end_char {
            return Ok(r#"{"ok":false,"reason":"빈 범위"}"#.to_string());
        }

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.apply_char_format_native(sec, para, start_char, end_char, props_json);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.apply_char_format_in_cell_by_path(
            section_index,
            host_para,
            &path,
            start_char,
            end_char,
            props_json,
        )
    }

    /// 커서 좌표(list/para/pos)에 글자를 끼운다 — 웹한글컨트롤 `Run("Insert*Space")` 용.
    ///
    /// 빈칸 세 가지가 스트림에서 저마다 다른 글자다(전부 한 칸): 보통 빈칸 `U+0020`,
    /// 묶음 빈칸 `U+001E`, 고정폭 빈칸 `U+001F`. 탭은 여기 없다 — 확장 컨트롤(8칸)이라
    /// 글자 끼우기로 다룰 수 없다.
    pub fn insert_text_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.insert_text_native(sec, para, char_idx, text);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.insert_text_in_cell_by_path(section_index, host_para, &path, char_idx, text)
    }

    /// 문서가 담은 컨트롤을 **문서 순서로** 늘어놓는다 — `HeadCtrl`·`LastCtrl` 과 `Next`·`Prev`
    /// 가 딛는 사슬이다(규격 §8.4 `CtrlCode`).
    ///
    /// 한글이 주는 값 셋을 그대로 낸다(실측): `CtrlID` 는 네 글자 코드, `CtrlCh` 는 그 컨트롤이
    /// 스트림에서 갖는 글자 코드(구역·단 정의 같은 표식은 2, 개체는 11), `UserDesc` 는 사람이
    /// 읽는 이름("구역 정의"·"표"·"사각형").
    ///
    /// 개체 목록([`objects_json`](Self::objects_json))과 달리 **표식까지 전부** 담는다 —
    /// 한글의 사슬이 그렇다.
    pub fn controls_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<String> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                collect_controls(para, (ROOT_LIST_ID, para_in_body), &lists, &mut items);
                para_in_body += 1;
            }
        }
        format!("[{}]", items.join(","))
    }

    /// 문서 글을 **한글 스캔 차례**로 늘어놓는다 — `InitScan`·`GetText`·`ReleaseScan` 이 쓴다.
    ///
    /// 각 항목은 `{state, text}` 다. 실측으로 세운 규칙(§4.54, 표본 넷):
    ///
    /// | 상태 | 뜻 |
    /// | --- | --- |
    /// | 2 | 같은 문단이 이어지거나 리스트가 바뀜(셀 → 셀) |
    /// | 3 | 같은 리스트에서 **다음 문단** |
    /// | 4 | 개체 리스트로 **들어감** |
    /// | 5 | 개체 리스트에서 **나옴** |
    ///
    /// 방출 규칙도 실측이다. **구역·단 정의는 빈 항목을 하나씩 낸다.** 표·그리기는 항목을
    /// 안 내고 대신 그 속 리스트로 들어간다(진입 4, 탈출 5). 문단의 글은 개체를 만나면
    /// **거기서 끊기고**, 개체를 다 돈 뒤 남은 글과 줄 끝이 이어진다.
    pub fn scan_items_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<(u8, String, u8)> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                scan_paragraph(para, (ROOT_LIST_ID, para_in_body), &lists, &mut items);
                para_in_body += 1;
            }
        }
        let body: Vec<String> = items
            .iter()
            .map(|(state, text, kind)| {
                format!(
                    "{{\"state\":{},\"kind\":{},\"text\":{}}}",
                    state,
                    kind,
                    json_escape(text)
                )
            })
            .collect();
        format!("[{}]", body.join(","))
    }

    /// 문서 글 전체를 GetTextFile 공통 순서로 조립한다.
    ///
    /// 훑기([`scan_items_json`](Self::scan_items_json))와 **같은 뿌리**다. 표식 항목(구역·단
    /// 정의)만 빼고 각 조각이 줄 끝으로 끝나게 이어 붙이되, **마지막 문단 뒤에는 줄 끝을 안
    /// 붙인다**(실측: 오라클이 정확히 두 글자 짧다).
    ///
    fn text_file_content(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<(u8, String, u8)> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                scan_paragraph(para, (ROOT_LIST_ID, para_in_body), &lists, &mut items);
                para_in_body += 1;
            }
        }
        let mut out = String::new();
        for (_, text, kind) in items.iter() {
            if *kind == 1 {
                continue; // 표식(구역·단 정의)은 글에 안 실린다
            }
            out.push_str(text);
            // 인라인 개체(수식 따위) 앞 조각은 **줄을 만들지 않는다** — 그 개체는 줄 안에 있다.
            // 표·글상자처럼 리스트를 여는 개체 앞 조각만 줄이 된다.
            if *kind != 2 && !text.ends_with("\r\n") {
                out.push_str("\r\n");
            }
        }
        if out.ends_with("\r\n") {
            out.truncate(out.len() - 2);
        }
        out
    }

    /// 문서 글 전체 — 웹한글컨트롤 `GetTextFile("TEXT")`.
    ///
    /// 이 형식만의 규칙이 하나 더 있다. **CP949 로 못 담는 글자는 `&#N;` 수치 참조가 된다** —
    /// `◦`(U+25E6)는 바뀌고 `€`·`①`·`㈜` 는 그대로다(여덟 글자로 예측 전수 적중). 훑기와
    /// `UNICODE` 형식은 escape 하지 않는다.
    pub fn text_file_json(&self) -> String {
        json_escape(&escape_outside_cp949(&self.text_file_content()))
    }

    /// 문서 글 전체 — 웹한글컨트롤 `GetTextFile("UNICODE")`.
    ///
    /// 문자열을 CP949로 왕복시키지 않고 원문 Unicode를 그대로 JSON 문자열로 내보낸다.
    pub fn text_file_unicode_json(&self) -> String {
        json_escape(&self.text_file_content())
    }

    /// 컨트롤 하나를 지운다 — 웹한글컨트롤 `DeleteCtrl`.
    ///
    /// 자리는 컨트롤 사슬이 준 `(list, para, controlIndex)` 다. 본문만 다룬다 — 셀·글상자 안의
    /// 컨트롤은 아래 삭제 API 가 `(구역, 문단, 컨트롤)` 셋만 받아 짚지 못한다.
    pub fn delete_control_at(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        control_index: usize,
    ) -> Result<String, HwpError> {
        if list_id != ROOT_LIST_ID {
            return Ok(
                r#"{"ok":false,"reason":"본문 밖 컨트롤은 아직 다루지 않는다"}"#.to_string(),
            );
        }
        let (sec, para) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        self.delete_control_native(sec, para, control_index)
    }

    /// 개체의 잠금을 켜고 끈다 — 웹한글컨트롤 `ShapeObjLock`·`ShapeObjUnlockAll`.
    ///
    /// `control_index` 가 `None` 이면 **본문 전체**를 푼다(`ShapeObjUnlockAll`). 잠금은 HWP5
    /// `attr` **비트 30** 이라 파서가 읽고 직렬화기가 도로 쓴다 — 값 하나만 뒤집으면 된다.
    ///
    /// 잠금은 고르기를 가른다(잠긴 개체는 `SelectCtrlFront` 가 건너뛴다). 그래서 이 뮤테이터는
    /// 고르기 규칙과 짝이고, 하니스는 잠근 뒤 고르기가 달라지는 것으로 둘을 한꺼번에 검증한다.
    pub fn set_control_lock(
        &mut self,
        para_in_list: Option<usize>,
        control_index: Option<usize>,
        locked: bool,
    ) -> Result<String, HwpError> {
        let target = match para_in_list {
            Some(p) => Some(
                root_para_location(self, p)
                    .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", p)))?,
            ),
            None => None,
        };
        let mut touched = 0usize;
        let mut touched_sections: Vec<usize> = Vec::new();
        for (si, section) in self.document.sections.iter_mut().enumerate() {
            for (pi, para) in section.paragraphs.iter_mut().enumerate() {
                if let Some((ts, tp)) = target {
                    if si != ts || pi != tp {
                        continue;
                    }
                }
                for (ci, ctrl) in para.controls.iter_mut().enumerate() {
                    if let Some(want) = control_index {
                        if ci != want {
                            continue;
                        }
                    }
                    let common = match ctrl {
                        Control::Table(t) => Some(&mut t.common),
                        Control::Shape(s) => Some(s.common_mut()),
                        Control::Picture(p) => Some(&mut p.common),
                        _ => None,
                    };
                    let Some(c) = common else { continue };
                    touched_sections.push(si);
                    c.locked = locked;
                    // 저장으로도 살아남게 원본 비트까지 맞춘다(파서가 읽는 자리와 같다).
                    if locked {
                        c.attr |= 1 << 30;
                    } else {
                        c.attr &= !(1u32 << 30);
                    }
                    touched += 1;
                }
            }
        }
        // 잠금은 `attr` 비트라 저장기가 다시 써야 한다 — 손댄 구역의 원본 스트림을 버린다.
        for si in touched_sections {
            if let Some(section) = self.document.sections.get_mut(si) {
                section.raw_stream = None;
            }
        }
        Ok(format!("{{\"ok\":true,\"touched\":{}}}", touched))
    }

    /// 커서가 든 필드의 상태 — 웹한글컨트롤 `CurFieldState`(규격 §8.2).
    ///
    /// 값은 **갈래 + 필드 비트**다(오라클 실측 넷으로 세운 규칙):
    ///
    /// | 자리 | 값 | 풀이 |
    /// | --- | --- | --- |
    /// | 본문 | 0 | 갈래 0, 필드 밖 |
    /// | 표 셀 안(필드 아님) | 1 | 갈래 1 |
    /// | 셀 필드 안 | 17 | 갈래 1 + `0x10` |
    /// | 누름틀 안 | 18 | 갈래 2 + `0x10` |
    ///
    /// 셀 필드는 **셀 전체가 필드**라 그 셀 안이면 어디든 해당한다(`start_pos`·`end_pos` 가 0).
    pub fn cur_field_state(&self, list_id: u32, para_in_list: usize, pos: usize) -> u32 {
        const IN_FIELD: u32 = 0x10;
        const KIND_CELL: u32 = 1;
        const KIND_CLICK_HERE: u32 = 2;

        let (fields, lists) = self.collect_fields_and_lists();
        let in_cell = lists.iter().any(|l| l.list_id == list_id && l.is_cell);
        let mut state = if in_cell { KIND_CELL } else { 0 };
        for f in &fields {
            if f.list_id != list_id {
                continue;
            }
            // 셀 필드는 범위가 없다 — 그 셀 안이면 어디든 필드 안이다.
            if f.start_pos == 0 && f.end_pos == 0 {
                state = IN_FIELD | KIND_CELL;
                continue;
            }
            if f.para_in_list == para_in_list && pos >= f.start_pos && pos <= f.end_pos {
                return IN_FIELD | KIND_CLICK_HERE;
            }
        }
        state
    }

    /// 커서가 든 셀의 모양 — 웹한글컨트롤 `CellShape` 파라미터셋(규격 §8.2).
    ///
    /// 오라클이 답하는 항목은 `Width`·`Height`·`VertAlign` 이다(나머지 이름은 전부 `null` —
    /// `CellWidth`·`MarginLeft` 따위는 이 컨트롤에 없다). 셀이 아니면 빈 셋이다.
    pub fn cell_shape_set_json(&self, list_id: u32) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let Some(entry) = lists.iter().find(|l| l.list_id == list_id) else {
            return "{}".to_string();
        };
        if !entry.is_cell {
            return "{}".to_string();
        }
        let Some(section) = self.document.sections.get(entry.section_index) else {
            return "{}".to_string();
        };
        let Some(para) = section.paragraphs.get(entry.host_para_index) else {
            return "{}".to_string();
        };
        let Some(Control::Table(table)) = para.controls.get(entry.control_index) else {
            return "{}".to_string();
        };
        let Some(cell) = table.cells.get(entry.cell_index) else {
            return "{}".to_string();
        };
        format!(
            "{{\"Width\":{},\"Height\":{},\"VertAlign\":{}}}",
            cell.width, cell.height, cell.vertical_align as u8,
        )
    }

    /// 본문에 놓인 **개체** 목록 — `Run("ShapeObjNextObject")` 따위가 딛는다.
    ///
    /// 개체는 그림·그리기·수식과 **표**다. 표를 빼면 한글이 고르는 자리와 안 맞는다 —
    /// 실측: 오라클은 `0/0/16`(그리기)뿐 아니라 `0/1/0`·`0/4/0` 도 고르는데 그 둘이 표다.
    /// 개체를 고르면 캐럿이 `(문단, 8 × 컨트롤 번호)` 에 선다.
    ///
    /// `listId` 는 그 개체가 글자를 담는 리스트가 있을 때만 있다(글상자). `ShapeObjTextBoxEdit`
    /// 가 그 안으로 들어간다.
    pub fn objects_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<String> = Vec::new();
        // 본문은 구역을 가로질러 이어진다 — 문단 번호도 이어서 센다.
        let mut para_base = 0usize;
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_off, para) in section.paragraphs.iter().enumerate() {
                let para_idx = para_base + para_off;
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let kind = match ctrl {
                        Control::Shape(_) => "shape",
                        Control::Picture(_) => "picture",
                        Control::Equation(_) => "equation",
                        Control::Table(_) => "table",
                        _ => continue,
                    };
                    // 리스트 표의 `host_para_index` 는 **구역 안 번호**다(본문 번호가 아니다).
                    let list_id = lists
                        .iter()
                        .find(|l| {
                            !l.is_cell
                                && l.host_list_id == ROOT_LIST_ID
                                && l.section_index == sec_idx
                                && l.host_para_index == para_off
                                && l.control_index == ci
                        })
                        .map(|l| l.list_id.to_string())
                        .unwrap_or_else(|| "null".to_string());
                    // 자리차지(어울림)인지 글자처럼 다루는지 — 한글이 개체로 고르는 것과 갈릴 수 있다.
                    let anchored = match ctrl {
                        Control::Table(t) => !t.common.treat_as_char,
                        Control::Shape(s) => !s.common().treat_as_char,
                        _ => true,
                    };
                    items.push(format!(
                    "{{\"para\":{},\"controlIndex\":{},\"kind\":\"{}\",\"listId\":{},\"anchored\":{}}}",
                    para_idx, ci, kind, list_id, anchored
                ));
                }
            }
            para_base += section.paragraphs.len();
        }
        format!("[{}]", items.join(","))
    }

    /// 커서 자리에서 문단을 가른다 — 웹한글컨트롤 `Run("BreakPara")`.
    ///
    /// 캐럿은 새 문단의 처음으로 간다(실측: 6/0/1 에서 걸면 6/1/0).
    pub fn split_para_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
    ) -> Result<String, HwpError> {
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.split_paragraph_native(sec, para, char_idx, None);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.split_paragraph_in_cell_by_path(section_index, host_para, &path, char_idx, None)
    }

    /// 개체 크기를 한 걸음 늘이거나 줄인다 — 웹한글컨트롤 `Run("ShapeObjResize*")`.
    ///
    /// 걸음은 **283 HWPUNIT**(≈1mm)로 일정하다(실측: 25704 → 25987 → 26270, 되돌리면 그대로
    /// 되짚는다). 표의 크기 조절과 달리 **결정적**이다 — 같은 값을 두 번 읽어도 같다(§4.47 의
    /// 표 계열은 읽을 때마다 달라 판정 불가였다).
    ///
    /// `Right`·`Left` 는 폭을, `Down`·`Up` 은 높이를 바꾼다. 방향 이름이 **가장자리를 미는 쪽**
    /// 이라 `Left`·`Up` 은 줄인다.
    pub fn resize_control_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
        d_width: i32,
        d_height: i32,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let ctrl = para
            .controls
            .get_mut(control_index)
            .ok_or_else(|| HwpError::InvalidField(format!("컨트롤 {} 없음", control_index)))?;
        let common = match ctrl {
            Control::Table(t) => Some(&mut t.common),
            Control::Shape(s) => Some(s.common_mut()),
            Control::Picture(p) => Some(&mut p.common),
            _ => None,
        };
        let Some(c) = common else {
            return Ok(r#"{"ok":false,"reason":"크기를 가진 개체가 아니다"}"#.to_string());
        };
        c.width = (c.width as i64 + d_width as i64).max(0) as u32;
        c.height = (c.height as i64 + d_height as i64).max(0) as u32;
        let (w, h) = (c.width, c.height);
        // **`SHAPE_COMPONENT` 도 함께 정착시킨다** — 한글은 두 단계다(리사이즈 직후 저장엔
        // `common` 만, 다음 저장에서 `current_*`·배율 행렬 `sx=cur/org`·`rotation_center
        // = cur/2 가 따라온다 — `probes/pQ-settle.json` 실측). rhwp 가 `common` 만 바꾸면
        // 그 정착이 영원히 안 와 저장본의 행렬이 옛 크기로 남는다(계획서 §4.23).
        let sa = match para
            .controls
            .get_mut(control_index)
            .expect("바로 위에서 확인했다")
        {
            Control::Shape(s) => Some(s.shape_attr_mut()),
            Control::Picture(p) => Some(&mut p.shape_attr),
            _ => None,
        };
        if let Some(sa) = sa {
            sa.current_width = w;
            sa.current_height = h;
            if sa.original_width > 0 {
                let sx = f64::from(w) / f64::from(sa.original_width);
                sa.render_sx = if sa.render_sx < 0.0 { -sx } else { sx };
            }
            if sa.original_height > 0 {
                let sy = f64::from(h) / f64::from(sa.original_height);
                sa.render_sy = if sa.render_sy < 0.0 { -sy } else { sy };
            }
            sa.rotation_center.x = (w / 2) as i32;
            sa.rotation_center.y = (h / 2) as i32;
            // 원본 바이트를 비워야 직렬화가 행렬을 새 크기로 다시 만든다.
            sa.raw_rendering = Vec::new();
        }
        section.raw_stream = None;
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 개체를 한 걸음 옮긴다 — 웹한글컨트롤 `Run("ShapeObjMove*")`.
    ///
    /// 걸음은 **56 HWPUNIT**(≈0.2mm)로 일정하고 결정적이다(실측: 23040 → 23096 → 23152,
    /// 되돌리면 그대로 되짚는다). 크기 조절의 걸음(283)과 **다르다** — 같은 개체 액션이라고
    /// 같은 걸음일 것이라 넘겨짚으면 틀린다.
    ///
    /// **글자처럼 배치인 개체는 안 움직인다**(실측). 자리를 문단 흐름이 잡기 때문이다.
    pub fn move_control_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
        dx: i32,
        dy: i32,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let ctrl = para
            .controls
            .get_mut(control_index)
            .ok_or_else(|| HwpError::InvalidField(format!("컨트롤 {} 없음", control_index)))?;
        let common = match ctrl {
            Control::Table(t) => Some(&mut t.common),
            Control::Shape(s) => Some(s.common_mut()),
            Control::Picture(p) => Some(&mut p.common),
            _ => None,
        };
        let Some(c) = common else {
            return Ok(r#"{"ok":false,"reason":"자리를 가진 개체가 아니다"}"#.to_string());
        };
        if c.treat_as_char {
            // 글자처럼 배치는 문단 흐름이 자리를 잡는다 — 옮겨지지 않는다.
            return Ok(r#"{"ok":true,"moved":false}"#.to_string());
        }
        c.horizontal_offset = (c.horizontal_offset as i64 + dx as i64).max(0) as u32;
        c.vertical_offset = (c.vertical_offset as i64 + dy as i64).max(0) as u32;
        section.raw_stream = None;
        Ok(r#"{"ok":true,"moved":true}"#.to_string())
    }

    /// 개체의 **앞뒤 순서**를 바꾼다 — 웹한글컨트롤 `Run("ShapeObj{BringToFront,SendToBack,
    /// BringForward,SendBack,BringInFrontOfText,CtrlSendBehindText}")`.
    ///
    /// 규칙은 한글 저장본의 앞뒤 두 벌을 견줘 실측했다(`probes/pZ2-*.json`). 어느 API 도
    /// 결과를 안 비추지만 파일에는 `CTRL_HEADER` 의 `z_order` 로 적힌다.
    ///
    /// | 갈래 | 잰 것 |
    /// |---|---|
    /// | `front` | 고른 개체가 맨 위로, **그 위에 있던 것들은 한 칸씩 내려온다**(0,1,2 에서 z=0 을 올리면 1,2,0 이 아니라 → 2 이고 나머지가 0,1) |
    /// | `back` | 맨 아래로, 그 아래 있던 것들이 한 칸씩 올라간다 |
    /// | `forward` | 바로 위의 것과 **자리를 맞바꾼다**(한 칸) |
    /// | `backward` | 바로 아래의 것과 맞바꾼다 |
    /// | `behindText`·`inFrontOfText` | `z_order` 가 아니라 **`text_wrap`** 이다 |
    ///
    /// 마지막 둘이 순서가 아니라 배치라는 것은 이름만 보면 안 갈린다 — 실측으로 갈렸다.
    ///
    /// 겨루는 무리는 **같은 구역의 모든 개체**로 잡는다. 실측은 한 문단 안의 셋으로만 했으니
    /// 문단을 넘는 무리 짓기는 아직 안 잰 자리다.
    pub fn set_control_z_order_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
        mode: &str,
    ) -> Result<String, HwpError> {
        use crate::model::shape::TextWrap;

        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;

        // 배치(글 앞/뒤)는 순서와 다른 축이다 — 고른 개체 하나만 건드린다.
        if let Some(wrap) = match mode {
            "behindText" => Some(TextWrap::BehindText),
            "inFrontOfText" => Some(TextWrap::InFrontOfText),
            _ => None,
        } {
            let para = section
                .paragraphs
                .get_mut(para_idx)
                .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
            let ctrl = para
                .controls
                .get_mut(control_index)
                .ok_or_else(|| HwpError::InvalidField(format!("컨트롤 {} 없음", control_index)))?;
            let Some(c) = control_common_mut(ctrl) else {
                return Ok(r#"{"ok":false,"reason":"배치를 가진 개체가 아니다"}"#.to_string());
            };
            c.text_wrap = wrap;
            // packed `attr` 을 함께 고쳐야 저장에 실린다 — enum 만 바꾸면 묻힌다.
            crate::serializer::control::sync_text_wrap_bits(c);
            section.raw_stream = None;
            return Ok(r#"{"ok":true}"#.to_string());
        }

        // 구역의 개체를 전부 모아 순서를 다시 매긴다.
        let mut slots: Vec<(usize, usize, i32)> = Vec::new();
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Some(c) = control_common(ctrl) {
                    slots.push((pi, ci, c.z_order));
                }
            }
        }
        let Some(&(_, _, target_z)) = slots
            .iter()
            .find(|&&(pi, ci, _)| pi == para_idx && ci == control_index)
        else {
            return Ok(r#"{"ok":false,"reason":"순서를 가진 개체가 아니다"}"#.to_string());
        };
        let top = slots.iter().map(|&(_, _, z)| z).max().unwrap_or(target_z);
        let bottom = slots.iter().map(|&(_, _, z)| z).min().unwrap_or(target_z);

        // 새 순서를 계산한다. 바뀌는 것이 없으면(이미 맨 위에서 더 올리기 따위) 그대로 둔다.
        let renumber = |z: i32| -> i32 {
            match mode {
                "front" => {
                    if z == target_z {
                        top
                    } else if z > target_z {
                        z - 1
                    } else {
                        z
                    }
                }
                "back" => {
                    if z == target_z {
                        bottom
                    } else if z < target_z {
                        z + 1
                    } else {
                        z
                    }
                }
                // 한 칸은 **맞바꾸기**다 — 위/아래 하나와만 자리를 바꾼다.
                "forward" => {
                    if z == target_z && target_z < top {
                        z + 1
                    } else if z == target_z + 1 && target_z < top {
                        z - 1
                    } else {
                        z
                    }
                }
                "backward" => {
                    if z == target_z && target_z > bottom {
                        z - 1
                    } else if z == target_z - 1 && target_z > bottom {
                        z + 1
                    } else {
                        z
                    }
                }
                _ => z,
            }
        };
        if !matches!(mode, "front" | "back" | "forward" | "backward") {
            return Ok(format!(
                r#"{{"ok":false,"reason":"모르는 갈래: {}"}}"#,
                mode
            ));
        }

        let mut moved = false;
        for (pi, ci, z) in slots {
            let new_z = renumber(z);
            if new_z == z {
                continue;
            }
            moved = true;
            if let Some(c) = section
                .paragraphs
                .get_mut(pi)
                .and_then(|p| p.controls.get_mut(ci))
                .and_then(control_common_mut)
            {
                c.z_order = new_z;
            }
        }
        if moved {
            section.raw_stream = None;
        }
        Ok(format!(r#"{{"ok":true,"moved":{}}}"#, moved))
    }

    /// 개체를 뒤집는다 — 웹한글컨트롤 `Run("ShapeObj{Horz,Vert}Flip[OrgState]")`.
    ///
    /// 저장본 앞뒤 대조로 실측했다(`probes/pM*.json`, 계획서 §4.20·§4.22). 파일에는
    /// `SHAPE_COMPONENT` 의 뒤집기 비트와 **변환 행렬**로 적힌다.
    ///
    /// - 뒤집기는 그 축 비트(0x01/0x02)를 토글한다. `OrgState` 는 켜져 있으면 끄고 아니면
    ///   무동작이다.
    /// - 행렬: 켜면 그 축의 배율이 **−(cur/org)** 가 되고 이동량이 붙는다. 이동량은
    ///   `even_ceil(cur) − 2·(org % 2)` 다 — 여덟 관측(배율 0~3걸음 × 두 축 × 홀짝 org)이
    ///   전부 맞고, 같은 크기를 다른 이력으로 만들어도 같은 값이라 **상태의 함수**다
    ///   (`probes/pM8-path.json`).
    /// - 한글이 함께 세우는 `0x0003_0000` 두 비트는 **흉내 내지 않는다.** 같은 파일 상태에서
    ///   이력에 따라 지워지기도 남기도 하는 세션 부산물이고(§4.20), 한글이 저장한 HWPX 의
    ///   `<hp:flip>` 에는 horizontal·vertical 두 속성뿐이라 담을 자리도 없다. 대조도 그
    ///   비트를 거른다(`shape_attr.flip` 은 축이 `horz_flip`/`vert_flip` 제 줄로 따로 보인다).
    pub fn set_control_flip_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
        vertical: bool,
        org_state: bool,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let Some(Control::Shape(shape)) = para.controls.get_mut(control_index) else {
            return Ok(r#"{"ok":false,"reason":"뒤집을 수 있는 개체가 아니다"}"#.to_string());
        };
        let sa = shape.shape_attr_mut();

        let axis_bit: u32 = if vertical { 0x02 } else { 0x01 };
        if org_state && sa.flip & axis_bit == 0 {
            // 이미 원래 상태다. 한글은 이때 세션 부산물 비트를 지우기도 하지만 그 비트는
            // 흉내 내지 않으므로 여기서는 정말 아무 일도 없다.
            return Ok(r#"{"ok":true,"moved":false}"#.to_string());
        }
        sa.flip ^= axis_bit;
        let on = sa.flip & axis_bit != 0;

        let (cur, org) = if vertical {
            (sa.current_height, sa.original_height)
        } else {
            (sa.current_width, sa.original_width)
        };
        let scale = if org > 0 {
            f64::from(cur) / f64::from(org)
        } else {
            1.0
        };
        // 이동량 — `even_ceil(cur) − 2·(org % 2)`(실측 §4.22).
        let shift = if on {
            f64::from(cur + (cur & 1) - 2 * (org & 1))
        } else {
            0.0
        };
        let sign = if on { -1.0 } else { 1.0 };
        if vertical {
            sa.vert_flip = on;
            sa.render_sy = sign * scale;
            sa.render_ty = shift;
        } else {
            sa.horz_flip = on;
            sa.render_sx = sign * scale;
            sa.render_tx = shift;
        }
        // 원본 바이트를 비워야 직렬화가 행렬을 다시 만든다 — `attr` 과 같은 덫이다.
        sa.raw_rendering = Vec::new();
        section.raw_stream = None;
        Ok(r#"{"ok":true,"moved":true}"#.to_string())
    }

    /// 쪽마다 **캐럿이 설 수 있는 첫 자리** — 웹한글컨트롤 `Run("MovePage*")` 용.
    ///
    /// 줄과 달리 쪽 경계는 **파일이 안 알려 준다.** 저장 vpos 가 되돌아가는 자리로 두 곳은
    /// 짚히는데(실측 `20250130-hongbo` 의 15/122 · 26/0), 셋째는 못 짚는다 — 표가 쪽을
    /// 넘어가는 자리이고 셀 안 `vpos` 는 **셀 기준**이라 되돌아감이 안 남는다. 그래서 이 값은
    /// rhwp 조판기가 답한다.
    ///
    /// 항목 갈래를 가려야 한글과 같아진다:
    ///
    /// - 문단이 이어지는 쪽(`PartialParagraph`)은 **그 줄의 시작**이 답이다(15/122).
    /// - 표가 이어지는 쪽(`PartialTable` 이어짐)은 캐럿이 설 자리가 **본문에 없다** — 셀 안이다.
    ///   건너뛰고 그 다음 항목을 본다(그래서 넷째 쪽이 29 가 아니라 **30/0** 이다).
    /// - 첫 쪽의 시작은 앞머리 자리차지 뒤다(줄 이동과 같은 규칙).
    ///
    /// **조판 정밀도를 물려받는다.** 쪽 나눔이 한글과 갈리는 문서에서는 이 값도 갈린다.
    pub fn page_caret_starts(&self) -> Result<String, HwpError> {
        use crate::renderer::pagination::PageItem;
        let mut out: Vec<String> = Vec::new();
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for page in pr.pages.iter() {
                let mut found: Option<(usize, usize)> = None;
                'items: for col in &page.column_contents {
                    for item in &col.items {
                        match item {
                            PageItem::PartialTable {
                                is_continuation: true,
                                ..
                            } => continue,
                            PageItem::PartialParagraph {
                                para_index,
                                start_line,
                                ..
                            } => {
                                let ts = self
                                    .document
                                    .sections
                                    .get(sec_idx)
                                    .and_then(|s| s.paragraphs.get(*para_index))
                                    .and_then(|p| p.line_segs.get(*start_line))
                                    .map(|seg| seg.text_start as usize)
                                    .unwrap_or(0);
                                found = Some((*para_index, ts));
                                break 'items;
                            }
                            PageItem::FullParagraph { para_index }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => {
                                found = Some((*para_index, 0));
                                break 'items;
                            }
                            _ => continue,
                        }
                    }
                }
                // 쪽 전체가 **이어지는 표**뿐이면 본문에는 설 자리가 없다 — 캐럿은 그 쪽에
                // 보이는 **첫 칸 안**으로 들어간다(실측: 리스트 52·118 의 0/0). 빠뜨리면 목록이
                // 밀려 뒤가 다 어긋나므로 그 칸의 리스트를 찾아 채운다.
                let Some((para_index, pos)) = found else {
                    let cell = page
                        .column_contents
                        .iter()
                        .flat_map(|col| col.items.iter())
                        .find_map(|item| match item {
                            PageItem::PartialTable {
                                para_index,
                                control_index,
                                start_row,
                                ..
                            } => self.first_cell_list_of_row(
                                sec_idx,
                                *para_index,
                                *control_index,
                                *start_row,
                            ),
                            _ => None,
                        });
                    match cell {
                        Some(list_id) => {
                            out.push(format!("{{\"list\":{},\"para\":0,\"pos\":0}}", list_id))
                        }
                        None => out.push("null".to_string()),
                    }
                    continue;
                };
                // 본문 문단 번호는 구역을 가로질러 이어 센다 — 캐럿 좌표가 그렇다.
                let before: usize = self
                    .document
                    .sections
                    .iter()
                    .take(sec_idx)
                    .map(|s| s.paragraphs.len())
                    .sum();
                let para_in_list = before + para_index;
                let start = self
                    .document
                    .sections
                    .get(sec_idx)
                    .and_then(|s| s.paragraphs.get(para_index))
                    .map(leading_anchor_pos)
                    .unwrap_or(0);
                out.push(format!(
                    "{{\"list\":0,\"para\":{},\"pos\":{}}}",
                    para_in_list,
                    pos.max(start)
                ));
            }
        }
        Ok(format!("[{}]", out.join(",")))
    }

    /// 스트림 자리를 **글자 번호**로 옮긴다 — 글자 번호를 받는 코어 API(`createTable` 따위)에
    /// 커서 좌표를 넘길 때 쓴다. 둘을 헷갈리면 글 한가운데에 꽂힌다(`SetTextFile` 에서 겪었다).
    pub fn char_index_at(
        &self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
    ) -> Result<String, HwpError> {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return Ok("{\"charIndex\":0}".to_string());
        };
        Ok(format!(
            "{{\"charIndex\":{}}}",
            char_idx_at_stream_pos(para, pos)
        ))
    }

    /// 쪽 하나의 글 — 웹한글컨트롤 `GetPageText`.
    ///
    /// 규칙은 실측이다(`20250130-hongbo`):
    ///
    /// - **본문 문단만** 담는다. 표 안 글은 안 들어간다 — 표만 있는 문단은 **빈 줄**이 된다
    ///   (3쪽이 표 넷뿐이라 빈 줄 넷이다).
    /// - 문단 사이는 `\r\n` 이다. 마지막 문단 뒤에는 안 붙인다.
    /// - **쪽 경계에서 문단을 자른다.** 1쪽이 `…현장 문` 으로 끝나고 2쪽이 `화로…` 로 시작한다 —
    ///   문단 15 의 줄 3(`text_start` 122)에서 정확히 갈린다.
    pub fn page_text(&self, page_index: usize) -> Result<String, HwpError> {
        let starts: Vec<(usize, usize)> = {
            let raw = self.page_caret_starts()?;
            // `[{"list":0,"para":N,"pos":M}, …]` — 본문 밖에서 시작하는 쪽은 그 문단으로 친다.
            let mut out = Vec::new();
            for chunk in raw.split("{\"list\":").skip(1) {
                let para = chunk
                    .split("\"para\":")
                    .nth(1)
                    .and_then(|s| s.split([',', '}']).next())
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                let pos = chunk
                    .split("\"pos\":")
                    .nth(1)
                    .and_then(|s| s.split([',', '}']).next())
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                out.push((para, pos));
            }
            out
        };
        let Some(&(from_para, from_pos)) = starts.get(page_index) else {
            return Ok(json_escape(""));
        };
        let last_para = root_para_count(self).saturating_sub(1);
        let (to_para, to_pos) = starts
            .get(page_index + 1)
            .copied()
            .unwrap_or((last_para + 1, 0));

        let mut lines: Vec<String> = Vec::new();
        for p in from_para..=to_para.min(last_para) {
            let Some(para) = self.cursor_paragraph_ref(ROOT_LIST_ID, p) else {
                continue;
            };
            let chars: Vec<char> = para.text.chars().collect();
            let begin = if p == from_para {
                char_idx_at_stream_pos(para, from_pos)
            } else {
                0
            };
            let end = if p == to_para {
                char_idx_at_stream_pos(para, to_pos)
            } else {
                chars.len()
            };
            // 경계 문단은 **양쪽 쪽에 다 들어간다**(다음 쪽이 그 문단의 처음부터라 이 쪽에는
            // 빈 줄로 들어간다) — 실측: 2쪽 끝이 `끝.` 뒤 빈 줄 넷으로 끝나는데 그 마지막이
            // 3쪽 첫 문단이다. 빼면 쪽마다 한 줄씩 모자란다.
            lines.push(
                chars[begin.min(chars.len())..end.min(chars.len())]
                    .iter()
                    .collect(),
            );
        }
        // 마지막 쪽에는 다음 쪽이 없는데도 **경계 항목이 하나 더** 붙는다(실측: 마지막 쪽이
        // 줄바꿈 하나다 — 문단 하나뿐인데 항목은 둘이다). 문서 끝을 경계로 치는 셈이다.
        if page_index + 1 >= starts.len() {
            lines.push(String::new());
        }
        Ok(json_escape(&lines.join("\r\n")))
    }

    /// 개체 사이를 도는 차례 — 웹한글컨트롤 `Run("ShapeObjNext/PrevObject")` 용.
    ///
    /// **쪽 단위로 돈다.** 실측(`20250130-hongbo`): 1쪽에서 걸면 문단 0 → 5 → 2 → 0 만 돌고,
    /// 3쪽에서 걸면 26 ↔ 29 만, 2쪽에서 걸면 24 ↔ 25 만 돈다. 문서 전체를 도는 것이 아니라
    /// **그 쪽에 놓인 개체**끼리 돈다 — 앞서 "일곱 중 셋만 돈다"고 적힌 수수께끼가 이것이었다.
    ///
    /// 쪽 안의 차례는 문단 순서가 아니라 **z 순서**다(0 → 5 → 2 는 문단 순서가 아니다).
    ///
    /// 쪽을 조판기에 물으므로 **조판 정밀도를 물려받는다**.
    pub fn object_cycle_json(&self) -> Result<String, HwpError> {
        use crate::renderer::pagination::PageItem;
        // 문단 → 쪽. 한 문단이 여러 쪽에 걸치면 **처음 나온 쪽**으로 친다.
        let mut page_of: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        let mut page_no = 0usize;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for page in pr.pages.iter() {
                for col in &page.column_contents {
                    for item in &col.items {
                        let para = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = para {
                            page_of.entry((sec_idx, p)).or_insert(page_no);
                        }
                    }
                }
                page_no += 1;
            }
        }

        let mut out: Vec<String> = Vec::new();
        let mut para_in_list = 0usize;
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let common = match ctrl {
                        Control::Table(t) => Some(&t.common),
                        Control::Shape(s) => Some(s.common()),
                        Control::Picture(p) => Some(&p.common),
                        _ => None,
                    };
                    let Some(c) = common else { continue };
                    let page = page_of.get(&(sec_idx, para_idx)).copied().unwrap_or(0);
                    out.push(format!(
                        "{{\"para\":{},\"controlIndex\":{},\"page\":{},\"z\":{}}}",
                        para_in_list, ci, page, c.z_order
                    ));
                }
                para_in_list += 1;
            }
        }
        Ok(format!("[{}]", out.join(",")))
    }

    /// 표의 어떤 **행에서 첫 칸**의 리스트 번호. 쪽을 넘어 이어지는 표의 시작 칸을 짚는다.
    fn first_cell_list_of_row(
        &self,
        section_index: usize,
        host_para: usize,
        control_index: usize,
        row: usize,
    ) -> Option<u32> {
        let (_, lists) = self.collect_fields_and_lists();
        lists
            .iter()
            .filter(|l| {
                l.is_cell
                    && l.section_index == section_index
                    && l.host_para_index == host_para
                    && l.control_index == control_index
                    && l.grid.is_some_and(|g| g.row as usize == row)
            })
            .min_by_key(|l| l.grid.map(|g| g.col).unwrap_or(u16::MAX))
            .map(|l| l.list_id)
    }

    /// 글상자 붙이기·떼기 — 웹한글컨트롤 `Run("ShapeObjAttach/DetachTextBox")`.
    ///
    /// 캡션과 달리 **빈 채로 생긴다**(붙이면 캐럿이 `list 2, para 0, pos 0`). 떼면 그 리스트가
    /// 사라져서 `SetPos(2,0,0)` 이 본문으로 되돌아간다 — 그 되돌아감이 판별자다.
    pub fn set_text_box_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
        attach: bool,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let Some(Control::Shape(shape)) = para.controls.get_mut(control_index) else {
            return Ok(r#"{"ok":false,"reason":"그리기 개체가 아니다"}"#.to_string());
        };
        let Some(drawing) = shape.drawing_mut() else {
            return Ok(r#"{"ok":false,"reason":"글상자를 담을 수 없는 개체다"}"#.to_string());
        };
        let changed = if attach {
            if drawing.text_box.is_some() {
                false
            } else {
                drawing.text_box = Some(TextBox {
                    paragraphs: vec![Paragraph::new_empty()],
                    ..Default::default()
                });
                true
            }
        } else {
            drawing.text_box.take().is_some()
        };
        if changed {
            section.raw_stream = None;
        }
        Ok(format!("{{\"ok\":{}}}", changed))
    }

    /// 캡션 붙이기 — 웹한글컨트롤 `Run("ShapeObjAttachCaption")`.
    ///
    /// 한글은 **빈 캡션을 만들지 않는다.** 붙이는 순간 `그림 ` + 번호 + 공백이 들어가 있고
    /// 캐럿이 그 끝(12칸)에 선다. 캡션 문단의 자리 지도를 재서 알아낸 구성이다 — `SetPos` 를
    /// 0~13 으로 훑으면 0·1·2·3 은 그대로 서고 4~10 은 **11 로 스냅**한 뒤 11·12 가 다시 선다:
    ///
    /// | 자리 | 무엇 |
    /// | --- | --- |
    /// | 0–2 | 글자 `그`·`림`·공백 |
    /// | 3–10 | 번호 컨트롤 **8칸** |
    /// | 11 | 공백 하나 |
    /// | 12 | 문단 끝 |
    ///
    /// 개체 갈래와 무관하게 말머리는 `그림` 이다(사각형에 붙여도 그렇다 — 실측).
    pub fn attach_caption_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let Some(Control::Shape(shape)) = para.controls.get_mut(control_index) else {
            return Ok(r#"{"ok":false,"reason":"그리기 개체가 아니다"}"#.to_string());
        };
        if Self::shape_caption_slot(shape).is_some() {
            return Ok(r#"{"ok":false,"reason":"이미 캡션이 있다"}"#.to_string());
        }

        // 말머리 세 글자와 꼬리 공백 하나를 먼저 놓고, 번호를 그 사이에 끼운다. 자리표 글자는
        // `insert_text_at` 이 넣게 둔다 — `char_offsets` 를 함께 갱신해 주기 때문이다.
        let mut caption_para = Paragraph::new_empty();
        // 글자 다섯: `그`·`림`·공백 · **번호 자리표** · 공백. 자리표 뒤 글자는 컨트롤 몫 8칸을
        // 건너뛴 자리(11)에 있으므로 대응표를 직접 놓는다 — `insert_text_at` 은 컨트롤을 모른다.
        caption_para.insert_text_at(0, "그림   ");
        caption_para.char_offsets = vec![0, 1, 2, 3, 11];
        caption_para.controls.push(Control::AutoNumber(AutoNumber {
            number_type: AutoNumberType::Picture,
            ..Default::default()
        }));
        caption_para.char_count = 12;

        *Self::shape_caption_slot(shape) = Some(Caption {
            paragraphs: vec![caption_para],
            ..Default::default()
        });
        shape.common_mut().attr |= 1 << 29;
        section.raw_stream = None;
        Ok(r#"{"ok":true,"pos":12}"#.to_string())
    }

    /// 캡션 떼기 — 웹한글컨트롤 `Run("ShapeObjDetachCaption")`. 캐럿은 개체 앵커로 돌아온다.
    pub fn detach_caption_at(
        &mut self,
        para_in_list: usize,
        control_index: usize,
    ) -> Result<String, HwpError> {
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;
        let Some(Control::Shape(shape)) = para.controls.get_mut(control_index) else {
            return Ok(r#"{"ok":false,"reason":"그리기 개체가 아니다"}"#.to_string());
        };
        let had = Self::shape_caption_slot(shape).take().is_some();
        if had {
            shape.common_mut().attr &= !(1u32 << 29);
            section.raw_stream = None;
        }
        Ok(format!("{{\"ok\":{}}}", had))
    }

    /// 캡션이 담기는 자리 — 갈래마다 `drawing` 밑이거나 제 몸에 붙어 있다.
    fn shape_caption_slot(shape: &mut ShapeObject) -> &mut Option<Caption> {
        match shape {
            ShapeObject::Line(s) => &mut s.drawing.caption,
            ShapeObject::Rectangle(s) => &mut s.drawing.caption,
            ShapeObject::Ellipse(s) => &mut s.drawing.caption,
            ShapeObject::Arc(s) => &mut s.drawing.caption,
            ShapeObject::Polygon(s) => &mut s.drawing.caption,
            ShapeObject::Curve(s) => &mut s.drawing.caption,
            ShapeObject::Group(s) => &mut s.caption,
            ShapeObject::Picture(s) => &mut s.caption,
            ShapeObject::Chart(s) => &mut s.caption,
            ShapeObject::Ole(s) => &mut s.caption,
        }
    }

    /// 나누기 — 웹한글컨트롤 `Run("BreakPage"·"BreakColumn"·"BreakColDef"·"BreakSection")`.
    ///
    /// 넷은 **한 규칙**이다(실측 15건, 계획서 §4.45). 표식이 앉는 문단은 캐럿이 문단의 어디에
    /// 있느냐로 갈린다 — 문단을 가르는 것은 **한가운데일 때뿐**이다:
    ///
    /// | 캐럿 | 하는 일 | 표식이 앉는 문단 |
    /// | --- | --- | --- |
    /// | 문단 끝 | 안 가름 | 다음 문단 |
    /// | 문단 처음 | 안 가름 | 그 문단 |
    /// | 한가운데 | 가름 | 뒤쪽 문단 |
    ///
    /// 시작과 끝이 같은 문단(자리차지 하나뿐인 문단)은 한글이 **끝으로 친다** — 그래서 끝 가지를
    /// 먼저 본다. 표식은 갈래마다 크기가 다르다: 쪽·단은 문단 속성이라 0칸, `BreakColDef` 는
    /// `ColumnDef` 하나로 8칸, `BreakSection` 은 `SectionDef`+`ColumnDef` 로 16칸이다.
    ///
    /// 캐럿은 `max(표식칸, 대상 문단의 원래 시작)` 에 선다. 이 `max` 는 **맞춘 식**이지 밝힌
    /// 기전이 아니다 — 액션 넷 × 자리 셋 + 판별 자리 셋, 열다섯 관측에 전부 맞는다.
    ///
    /// 처음엔 빈 문단에서 재다가 앞의 둘이 "아무 일도 안 한다"고 볼 뻔했다.
    /// **자를 빈 곳에 대면 눈금이 안 보인다.**
    pub fn break_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        kind: &str,
    ) -> Result<String, HwpError> {
        let marker_units = match kind {
            "page" | "column" => 0usize,
            "colDef" => EXTENDED_CTRL_UNITS,
            "section" => 2 * EXTENDED_CTRL_UNITS,
            other => return Err(HwpError::InvalidField(format!("모르는 나누기 {}", other))),
        };
        let (start, end) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            (leading_anchor_pos(para), stream_len(para))
        };

        // 대상 문단을 고른다. 한가운데일 때만 가른다.
        //
        // 시작과 끝이 같은 문단은 `end > 0` 으로 갈린다: 자리차지가 하나라도 있으면(끝 8) 끝으로
        // 치고, **아무것도 없는 빈 문단**(끝 0)은 자기가 표식을 진다. 이 갈림은 오라클이 캐럿을
        // 어디 두는지로만 보인다 — 쪽 나눔은 0칸이라 문단 자에는 아무 자국도 안 남는다.
        let at_end = pos >= end && end > 0;
        let target_in_list = if at_end {
            para_in_list + 1
        } else if pos <= start {
            para_in_list
        } else {
            let raw = self.split_para_at_cursor(list_id, para_in_list, pos)?;
            if raw.contains("\"ok\":false") {
                return Ok(raw);
            }
            para_in_list + 1
        };
        let at_start = !at_end && target_in_list == para_in_list;

        // 대상 문단이 **이미 표식을 지고 있으면** 겹쳐 얹을 수 없다 — 한글은 그 자리에 빈 문단을
        // 새로 끼운다(실측: 쪽 나눔이 앉은 문단에 단 나눔을 걸면 문단이 하나 는다). 그래서
        // "가르지 않는다"는 앞의 표는 **대상이 비어 있을 때** 이야기다.
        let occupied = self
            .cursor_paragraph_ref(list_id, target_in_list)
            .map(|p| {
                p.column_type != ColumnBreakType::None
                    || matches!(
                        p.controls.first(),
                        Some(Control::SectionDef(_) | Control::ColumnDef(_))
                    )
            })
            .unwrap_or(false);
        if occupied {
            self.insert_empty_paragraph_at(list_id, target_in_list)?;
        }

        // 캐럿은 **글을 따라간다.** 문단 처음에서 걸어 빈 문단을 끼웠다면 캐럿이 있던 글은
        // 한 칸 뒤로 밀렸으니 캐럿도 따라간다. 문단 끝에서 끼웠다면 새 문단이 캐럿 뒤에 오므로
        // 캐럿이 그 안으로 들어간다(실측: 앞은 9/8, 뒤는 7/0).
        let caret_para = if at_start && occupied {
            target_in_list + 1
        } else {
            target_in_list
        };
        // 표식을 얹기 **전에** 캐럿이 설 문단의 시작을 재 둔다.
        let caret_start = self
            .cursor_paragraph_ref(list_id, caret_para)
            .map(leading_anchor_pos)
            .unwrap_or(0);
        let caret = marker_units.max(caret_start);

        if list_id != ROOT_LIST_ID {
            // 본문 밖은 구역·단 정의를 둘 곳이 없다 — 표식은 안 단다.
            return Ok(format!(
                "{{\"ok\":true,\"para\":{},\"pos\":{}}}",
                caret_para, caret
            ));
        }
        let Some((sec, para)) = root_para_location(self, target_in_list) else {
            // 마지막 문단 끝에서 걸면 다음 문단이 없다 — 아직 다루지 않는다.
            return Ok(r#"{"ok":false,"reason":"대상 문단이 없다"}"#.to_string());
        };
        let target = self
            .document
            .sections
            .get_mut(sec)
            .and_then(|s| s.paragraphs.get_mut(para))
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", target_in_list)))?;
        match kind {
            "page" => target.column_type = ColumnBreakType::Page,
            "column" => target.column_type = ColumnBreakType::Column,
            // 표식 컨트롤은 글자를 안 남기고 `char_count` 로만 8칸을 센다(모델 규약).
            "colDef" => {
                target.column_type = ColumnBreakType::MultiColumn;
                target
                    .controls
                    .insert(0, Control::ColumnDef(ColumnDef::default()));
                target.char_count += EXTENDED_CTRL_UNITS as u32;
            }
            _ => {
                target.column_type = ColumnBreakType::Section;
                target
                    .controls
                    .insert(0, Control::ColumnDef(ColumnDef::default()));
                target
                    .controls
                    .insert(0, Control::SectionDef(Box::default()));
                target.char_count += 2 * EXTENDED_CTRL_UNITS as u32;
            }
        }
        // 표식은 문단 레코드에 얹히므로 그 구역의 원본 스트림을 버려야 저장에 실린다.
        if let Some(section) = self.document.sections.get_mut(sec) {
            section.raw_stream = None;
        }
        Ok(format!(
            "{{\"ok\":true,\"para\":{},\"pos\":{}}}",
            caret_para, caret
        ))
    }

    /// 자동 번호를 캐럿 자리에 끼운다 — 웹한글컨트롤 `InsertPageNum`·`InsertCpNo`·`InsertTpNo`.
    ///
    /// 셋 다 사슬에 `atno` 하나를 더하고 스트림에서 **8칸**을 차지한다(실측: 문단 끝 7 → 15).
    /// 갈래는 `page`(쪽 번호)·`current`(현재 쪽)·`total`(전체 쪽수)이고, 컨트롤 아이디로는
    /// 안 갈린다 — 셋이 같은 `atno` 다.
    ///
    /// 파서가 이 컨트롤에 **자리표 글자 한 칸**을 남기므로(`parse_para_text` 의 `0x0012` 가지)
    /// 여기서도 그 한 칸을 함께 넣는다. 그래야 저장·조판이 파일에서 온 문서와 같은 꼴이 된다.
    pub fn insert_auto_number_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        kind: &str,
    ) -> Result<String, HwpError> {
        let number_type = match kind {
            "page" | "current" => AutoNumberType::Page,
            "total" => AutoNumberType::TotalPage,
            other => {
                return Err(HwpError::InvalidField(format!(
                    "모르는 번호 갈래 {}",
                    other
                )))
            }
        };
        if list_id != ROOT_LIST_ID {
            return Ok(r#"{"ok":false,"reason":"본문 밖은 아직 다루지 않는다"}"#.to_string());
        }
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;

        // 컨트롤은 문단 안에서 **글자 차례대로** 놓인다 — 앞선 컨트롤 수가 끼울 자리다.
        let control_index = para
            .control_text_positions()
            .iter()
            .filter(|p| **p < char_idx)
            .count();
        // 자리표 글자는 문단이 스스로 넣게 둔다 — `char_offsets`·`char_shapes`·`line_segs` 를
        // 함께 갱신해 준다. 여기서 지우거나 직접 만지면 `control_text_positions` 가 이 컨트롤을
        // 문단 맨 앞으로 오해해 캐럿 클램프까지 어긋난다(실제로 한 번 그랬다).
        para.insert_text_at(char_idx, " ");
        para.controls.insert(
            control_index,
            Control::AutoNumber(AutoNumber {
                number_type,
                ..Default::default()
            }),
        );
        // 글자 한 칸은 `insert_text_at` 이 이미 셌다 — 컨트롤 몫 일곱만 더한다(합 8칸).
        para.char_count += (EXTENDED_CTRL_UNITS - 1) as u32;
        section.raw_stream = None;
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 구역마다 **첫 본문 문단 번호** — `MoveSectionUp`·`MoveSectionDown` 이 딛는 자리다.
    ///
    /// 본문 리스트는 구역을 가로질러 이어지므로([`root_para_location`]) 구역 경계는 이 표로만
    /// 안다. 구역이 셋인 문서면 `[0, 8, 15]` 꼴이다.
    ///
    /// 경계는 `document.sections` 의 칸막이가 아니라 **문단이 진 구역 표식**으로 센다.
    /// `BreakSection` 이 만든 구역은 문단에 `SectionDef` 를 얹을 뿐 `sections` 를 안 가르는데,
    /// 한글은 그 문단부터 새 구역으로 본다(실측: 나눈 뒤 `MoveSectionDown` 이 그 문단을 짚는다).
    /// 칸막이로 세면 방금 만든 구역이 안 보인다.
    pub fn section_starts_json(&self) -> String {
        let mut starts: Vec<String> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                let marks_section = para_in_body == 0
                    || matches!(para.controls.first(), Some(Control::SectionDef(_)));
                if marks_section {
                    starts.push(para_in_body.to_string());
                }
                para_in_body += 1;
            }
        }
        format!("[{}]", starts.join(","))
    }

    /// 빈 문단 하나를 그 자리에 끼운다 — 나누기가 표식을 놓을 자리를 만들 때 쓴다.
    ///
    /// 서식은 **뒤 이웃**에서 물려받는다. 나누기로 생기는 문단은 뒤따르는 글의 앞머리라
    /// 그쪽을 닮는 것이 맞다.
    fn insert_empty_paragraph_at(
        &mut self,
        list_id: u32,
        para_in_list: usize,
    ) -> Result<(), HwpError> {
        if list_id != ROOT_LIST_ID {
            return Err(HwpError::InvalidField(
                "본문 밖에는 아직 문단을 끼우지 않는다".into(),
            ));
        }
        let (sec, para) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let fresh = match section.paragraphs.get(para) {
            Some(neighbor) => Paragraph::new_empty_like(neighbor),
            None => Paragraph::new_empty(),
        };
        section.paragraphs.insert(para, fresh);
        section.raw_stream = None;
        Ok(())
    }

    /// 커서 좌표(list/para/pos)로 글자를 지운다 — 웹한글컨트롤 `Run("Delete*")` 용.
    ///
    /// [`apply_char_format_at_cursor`](Self::apply_char_format_at_cursor) 와 같은 자를 쓴다 —
    /// 인자는 코드 유닛이고 여기서 글자 번호로 옮긴다. 빈 범위면 아무 일도 하지 않는다.
    pub fn delete_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        start_pos: usize,
        end_pos: usize,
    ) -> Result<String, HwpError> {
        let (start_char, end_char) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let last = para.text.chars().count();
            (
                char_idx_at_stream_pos(para, start_pos).min(last),
                char_idx_at_stream_pos(para, end_pos).min(last),
            )
        };
        if start_char >= end_char {
            return Ok(r#"{"ok":false,"reason":"빈 범위"}"#.to_string());
        }

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.delete_text_native(sec, para, start_char, end_char - start_char);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.delete_range_in_cell_by_path(
            section_index,
            host_para,
            &path,
            para_in_list,
            start_char,
            para_in_list,
            end_char,
        )
    }

    /// 커서가 든 셀을 기준으로 표를 고친다 — 웹한글컨트롤 `Run("TableInsert*"·"TableDelete*")`.
    ///
    /// 리스트 아이디만 주면 구역·문단·컨트롤·행·열을 여기서 풀어 준다. 캐럿을 어디로 옮길지는
    /// 호출 측(호환 층)이 정한다 — 표가 바뀐 **뒤의** 격자를 봐야 알 수 있기 때문이다.
    ///
    /// 중첩 표는 아직 다루지 않는다. 아래 표 편집 API 가 `(구역, 문단, 컨트롤)` 세 값만 받아서
    /// 셀 안의 표까지 짚지 못한다.
    pub fn table_edit_at_cursor(&mut self, list_id: u32, op: &str) -> Result<String, HwpError> {
        let (section, host_para, control_index, row, col) = {
            let (_, lists) = self.collect_fields_and_lists();
            let entry = lists
                .iter()
                .find(|l| l.list_id == list_id)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let grid = entry
                .grid
                .ok_or_else(|| HwpError::InvalidField("표 셀이 아니다".into()))?;
            if entry.host_list_id != ROOT_LIST_ID {
                return Ok(r#"{"ok":false,"reason":"중첩 표는 아직 다루지 않는다"}"#.to_string());
            }
            (
                entry.section_index,
                entry.host_para_index,
                entry.control_index,
                grid.row,
                grid.col,
            )
        };
        match op {
            "insertRowAbove" => {
                self.insert_table_row_native(section, host_para, control_index, row, false)
            }
            // `TableAppendRow` 도 같은 자리에 끼운다 — 다른 것은 캐럿이 새 줄로 간다는 점뿐이고
            // 그 판단은 호환 층이 한다.
            "insertRowBelow" | "appendRow" | "appendRowAtEnd" => {
                self.insert_table_row_native(section, host_para, control_index, row, true)
            }
            "insertColLeft" => {
                self.insert_table_column_native(section, host_para, control_index, col, false)
            }
            "insertColRight" => {
                self.insert_table_column_native(section, host_para, control_index, col, true)
            }
            "deleteRow" => self.delete_table_row_native(section, host_para, control_index, row),
            "deleteCol" => self.delete_table_column_native(section, host_para, control_index, col),
            // 셀을 두 줄·두 칸으로 나눈다. 한글의 `TableSplitCellRow2`·`Col2` 는 대화상자 없이
            // 곧바로 반씩 나눈다(실측: 셀 하나가 늘고 캐럿은 제자리).
            "splitRow2" => self.split_table_cell_into_native(
                section,
                host_para,
                control_index,
                row,
                col,
                2,
                1,
                true,
                false,
            ),
            "splitCol2" => self.split_table_cell_into_native(
                section,
                host_para,
                control_index,
                row,
                col,
                1,
                2,
                true,
                false,
            ),
            // 한 칸만 크기 조절 — 경계가 어긋나며 격자가 갈라진다(§4.21).
            "resizeCellRight" | "resizeCellLeft" | "resizeCellDown" | "resizeCellUp" => {
                let dir = op.trim_start_matches("resizeCell");
                let vertical = matches!(dir, "Down" | "Up");
                let forward = matches!(dir, "Right" | "Down");
                self.resize_table_cell_native(
                    section,
                    host_para,
                    control_index,
                    row,
                    col,
                    vertical,
                    forward,
                )
            }
            // 칸 크기 조절 열둘 — `Ex` 는 평범한 것과 자취가 같아 같은 갈래로 보낸다(§4.21).
            "resizeRight" | "resizeLeft" | "resizeDown" | "resizeUp" | "resizeLineRight"
            | "resizeLineLeft" | "resizeLineDown" | "resizeLineUp" => {
                let line_mode = op.starts_with("resizeLine");
                let dir = op
                    .trim_start_matches("resizeLine")
                    .trim_start_matches("resize");
                let vertical = matches!(dir, "Down" | "Up");
                let forward = matches!(dir, "Right" | "Down");
                self.resize_table_native(
                    section,
                    host_para,
                    control_index,
                    row,
                    col,
                    vertical,
                    forward,
                    line_mode,
                )
            }
            _ => Err(HwpError::InvalidField(format!("모르는 표 편집 '{}'", op))),
        }
    }

    /// 커서가 든 셀에서 `(end_row, end_col)` 까지를 하나로 합친다 — `Run("TableMergeCell")`.
    ///
    /// 셀 블록의 범위는 호환 층이 들고 있다. 오라클에서 블록은 `GetSelectedPos` 로 안 보이니
    /// (글자 범위가 아니다) 이 층이 기억한 범위를 그대로 넘겨받는다.
    pub fn table_merge_at_cursor(
        &mut self,
        list_id: u32,
        end_row: u16,
        end_col: u16,
    ) -> Result<String, HwpError> {
        let (section, host_para, control_index, row, col) = {
            let (_, lists) = self.collect_fields_and_lists();
            let entry = lists
                .iter()
                .find(|l| l.list_id == list_id)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let grid = entry
                .grid
                .ok_or_else(|| HwpError::InvalidField("표 셀이 아니다".into()))?;
            if entry.host_list_id != ROOT_LIST_ID {
                return Ok(r#"{"ok":false,"reason":"중첩 표는 아직 다루지 않는다"}"#.to_string());
            }
            (
                entry.section_index,
                entry.host_para_index,
                entry.control_index,
                grid.row,
                grid.col,
            )
        };
        self.merge_table_cells_native(
            section,
            host_para,
            control_index,
            row.min(end_row),
            col.min(end_col),
            row.max(end_row),
            col.max(end_col),
        )
    }

    /// 셀 블록이 덮은 칸들의 **글을 비운다** — 웹한글컨트롤 `Run("TableDeleteCell")`.
    ///
    /// 이름과 달리 칸을 지우는 것이 아니다(실측, 계획서 §4.21). 블록 직사각형 안 모든 칸의
    /// 내용이 **빈 문단 하나**가 되고 격자·캐럿은 그대로다. 원래 빈 칸은 자취를 안 남긴다.
    /// 블록이 없으면 무동작이다(저장본 차이 0).
    ///
    /// 규약은 `table_merge_at_cursor` 와 같다 — 블록 첫 칸의 리스트와 끝 칸의 (행, 열).
    pub fn clear_table_cells_at_cursor(
        &mut self,
        list_id: u32,
        end_row: u16,
        end_col: u16,
    ) -> Result<String, HwpError> {
        let (section, host_para, control_index, row, col) = {
            let (_, lists) = self.collect_fields_and_lists();
            let entry = lists
                .iter()
                .find(|l| l.list_id == list_id)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let grid = entry
                .grid
                .ok_or_else(|| HwpError::InvalidField("표 셀이 아니다".into()))?;
            if entry.host_list_id != ROOT_LIST_ID {
                return Ok(r#"{"ok":false,"reason":"중첩 표는 아직 다루지 않는다"}"#.to_string());
            }
            (
                entry.section_index,
                entry.host_para_index,
                entry.control_index,
                grid.row,
                grid.col,
            )
        };
        let (r1, r2) = (row.min(end_row), row.max(end_row));
        let (c1, c2) = (col.min(end_col), col.max(end_col));
        let table = self.get_table_mut(section, host_para, control_index)?;
        let mut cleared = false;
        for cell in table.cells.iter_mut() {
            if cell.row < r1 || cell.row > r2 || cell.col < c1 || cell.col > c2 {
                continue;
            }
            // 실측한 자취 그대로다: 글·좌표·줄만 비고 문단 객체(서식·보존 바이트)는 남는다.
            // 문단 여럿·컨트롤 든 칸은 잰 적이 없다 — "빈 문단 하나"가 빈 칸의 정의라
            // (`Cell::new_empty` 도 그렇다) 그 꼴로 줄인다.
            cell.paragraphs.truncate(1);
            let Some(para) = cell.paragraphs.first_mut() else {
                continue;
            };
            if para.text.is_empty() && para.controls.is_empty() {
                continue; // 원래 빈 칸 — 자취를 안 남긴다(실측).
            }
            cleared = true;
            para.text.clear();
            para.char_count = 1;
            para.char_offsets.clear();
            para.char_shapes.truncate(1);
            if let Some(first) = para.char_shapes.first_mut() {
                first.start_pos = 0;
            }
            para.range_tags.clear();
            para.line_segs.clear();
            para.tab_extended.clear();
            para.controls.clear();
            para.has_para_text = false;
        }
        if cleared {
            table.dirty = true;
            self.document.sections[section].raw_stream = None;
        }
        Ok(format!(r#"{{"ok":true,"cleared":{}}}"#, cleared))
    }

    /// 문단 하나의 캐럿 경계 — 웹한글컨트롤 `MoveParaBegin`·`MoveParaEnd` 가 가는 자리.
    ///
    /// `start` 는 앞머리 자리차지 컨트롤을 건너뛴 자리다(본문 첫 문단은 0 이 아니다).
    /// `end` 는 문단 부호를 뺀 코드 유닛 길이다. 없는 자리면 `{}`.
    pub fn para_bounds_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        format!(
            "{{\"start\":{},\"end\":{},\"selectStart\":{}}}",
            leading_anchor_pos(para),
            stream_len(para),
            select_start_pos(para),
        )
    }

    /// 줄이 시작하는 자리들 — `MoveLineBegin`·`MoveLineEnd` 가 딛는 값.
    ///
    /// `LineSeg::text_start` 는 파일이 그대로 준 **코드 유닛** 위치라 한글 좌표와 같은 자다.
    /// 옮길 것이 없다.
    pub fn line_starts_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let starts: Vec<String> = para
            .line_segs
            .iter()
            .map(|seg| seg.text_start.to_string())
            .collect();
        format!("[{}]", starts.join(","))
    }

    /// 지금 단어의 끝 — `MoveWordEnd` 가 가는 자리(다음 공백 글자의 자리).
    pub fn word_end_json(&self, list_id: u32, para_in_list: usize, pos: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "null".to_string();
        };
        word_end_from(para, pos).to_string()
    }

    /// 단어가 시작하는 자리들 — `MoveNextWord`·`MovePrevWord`·`MoveWordBegin/End` 가 딛는 눈금.
    pub fn word_starts_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let starts: Vec<String> = word_starts(para).iter().map(|p| p.to_string()).collect();
        format!("[{}]", starts.join(","))
    }

    /// 캐럿이 설 수 있는 자리들 — 한 글자 이동이 딛는 눈금(`MoveNextChar` 류).
    pub fn caret_stops_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let stops: Vec<String> = caret_stops(para).iter().map(|p| p.to_string()).collect();
        format!("[{}]", stops.join(","))
    }

    /// 커서 좌표(list/para)로 문단 서식을 건다 — 웹한글컨트롤 `Run("ParagraphShape*")` 용.
    ///
    /// 문단 서식은 셀 경로가 깊으면 걸 수 없다 — 코어에 by-path 짝이 아직 없다. 그 경우
    /// 조용히 넘기지 않고 오류로 알린다.
    pub fn apply_para_format_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.apply_para_format_native(sec, para, props_json);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        if path.len() != 1 {
            return Err(HwpError::InvalidField(
                "중첩 셀의 문단 서식은 아직 다루지 않는다".into(),
            ));
        }
        let (control_idx, cell_idx, cell_para_idx) = path[0];
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.apply_para_format_in_cell_native(
            section_index,
            host_para,
            control_idx,
            cell_idx,
            cell_para_idx,
            props_json,
        )
    }

    /// 커서 좌표가 가리키는 문단 — 리스트 표를 한 번만 만든다.
    fn cursor_paragraph_ref(&self, list_id: u32, para_in_list: usize) -> Option<&Paragraph> {
        if list_id == ROOT_LIST_ID {
            // 본문은 구역을 가로질러 이어진다 — `root_para_location` 주석 참고.
            let (sec, para) = root_para_location(self, para_in_list)?;
            return self.document.sections.get(sec)?.paragraphs.get(para);
        }
        let (_, lists) = self.collect_fields_and_lists();
        cursor_paragraph(self, &lists, list_id, para_in_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Section;
    use crate::model::style::ParaShape;

    /// 문단 모양이 한글 코드값과 단위로 나가는지 — 이름만 같고 값이 rhwp 내부 표현이면
    /// 오라클과 어긋난다.
    #[test]
    fn para_shape_set_uses_hwp_codes() {
        let mut core = DocumentCore::new_empty();
        core.document.doc_info.para_shapes = vec![ParaShape {
            alignment: Alignment::Center,
            line_spacing_type: LineSpacingType::Percent,
            line_spacing: 160,
            margin_left: 100,
            head_type: HeadType::Outline,
            para_level: 2,
            ..Default::default()
        }];
        core.document.sections.push(Section {
            paragraphs: vec![Paragraph {
                para_shape_id: 0,
                ..Default::default()
            }],
            ..Default::default()
        });

        let json = core.para_shape_set_json(ROOT_LIST_ID, 0);

        // 가운데 정렬은 3, 글자에 따라(%)는 0 — rhwp 열거형 순서(Justify=0, Center=3)와
        // 우연히 같은 자리가 아니라 한글 코드표를 따른 값이다.
        assert!(json.contains("\"AlignType\":3"), "{json}");
        assert!(json.contains("\"LineSpacingType\":0"), "{json}");
        assert!(json.contains("\"LineSpacing\":160"), "{json}");
        assert!(json.contains("\"LeftMargin\":100"), "{json}");
        assert!(json.contains("\"HeadingType\":1"), "{json}");
        assert!(json.contains("\"Level\":2"), "{json}");
    }

    /// 없는 자리를 물으면 빈 셋이다 — 0 으로 채우면 "모른다"와 "0이다"가 뭉개진다.
    #[test]
    fn missing_cursor_gives_empty_set() {
        let core = DocumentCore::new_empty();
        assert_eq!(core.para_shape_set_json(ROOT_LIST_ID, 99), "{}");
        assert_eq!(core.char_shape_set_json(ROOT_LIST_ID, 99, 0), "{}");
    }

    /// TEXT는 CP949 밖 글자를 수치 참조로 바꾸고 UNICODE는 원문을 보존한다.
    #[test]
    fn text_file_formats_keep_distinct_encoding_contracts() {
        let source = "가◦€";
        assert_eq!(escape_outside_cp949(source), "가&#9702;€");
        assert_eq!(json_escape(source), "\"가◦€\"");
        assert_eq!(json_escape(&escape_outside_cp949(source)), "\"가&#9702;€\"");
    }

    /// 앞뒤 순서 — 한글 저장본으로 잰 규칙(`scenarios/pL-zorder.json`)을 코어 단위로 굳힌다.
    mod z_order {
        use super::*;
        use crate::model::shape::{RectangleShape, TextWrap};

        /// 순서만 다른 개체 셋을 한 문단에 세운다.
        fn core_with_three() -> DocumentCore {
            let mut core = DocumentCore::new_empty();
            let controls = (0..3)
                .map(|z| {
                    let mut r = RectangleShape::default();
                    r.common.z_order = z;
                    Control::Shape(Box::new(ShapeObject::Rectangle(r)))
                })
                .collect();
            core.document.sections.push(Section {
                paragraphs: vec![Paragraph {
                    controls,
                    ..Default::default()
                }],
                ..Default::default()
            });
            core
        }

        fn orders(core: &DocumentCore) -> Vec<i32> {
            core.document.sections[0].paragraphs[0]
                .controls
                .iter()
                .filter_map(control_common)
                .map(|c| c.z_order)
                .collect()
        }

        /// 맨 위로 보내면 **위에 있던 것들이 한 칸씩 내려온다** — 자리만 맞바꾸는 것이 아니다.
        #[test]
        fn bring_to_front_pushes_the_rest_down() {
            let mut core = core_with_three();
            core.set_control_z_order_at(0, 0, "front").unwrap();
            assert_eq!(orders(&core), vec![2, 0, 1]);
        }

        /// 맨 아래로 보내면 아래 있던 것들이 한 칸씩 올라온다.
        #[test]
        fn send_to_back_pulls_the_rest_up() {
            let mut core = core_with_three();
            core.set_control_z_order_at(0, 2, "back").unwrap();
            assert_eq!(orders(&core), vec![1, 2, 0]);
        }

        /// 한 칸은 **이웃과 맞바꾸기**다. 맨 위에서 더 올리면 아무 일도 없다.
        #[test]
        fn one_step_swaps_with_the_neighbour() {
            let mut core = core_with_three();
            core.set_control_z_order_at(0, 0, "forward").unwrap();
            assert_eq!(orders(&core), vec![1, 0, 2]);

            let mut core = core_with_three();
            core.set_control_z_order_at(0, 2, "backward").unwrap();
            assert_eq!(orders(&core), vec![0, 2, 1]);

            let mut core = core_with_three();
            let out = core.set_control_z_order_at(0, 2, "forward").unwrap();
            assert_eq!(orders(&core), vec![0, 1, 2]);
            assert!(out.contains("\"moved\":false"), "{out}");
        }

        /// 리사이즈는 `SHAPE_COMPONENT` 까지 정착시켜야 한다 — 한글의 두 번째 저장이
        /// 만드는 상태가 정답지다(§4.23). `common` 만 바꾸면 행렬이 옛 크기로 남는다.
        #[test]
        fn resize_settles_the_shape_component_too() {
            let mut core = core_with_three();
            match &mut core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => {
                    let c = s.common_mut();
                    c.width = 8475;
                    c.height = 6750;
                    let a = s.shape_attr_mut();
                    a.original_width = 8475;
                    a.current_width = 8475;
                    a.original_height = 6750;
                    a.current_height = 6750;
                    a.raw_rendering = vec![1, 2, 3];
                }
                _ => unreachable!(),
            }
            core.resize_control_at(0, 0, 283, 0).unwrap();
            match &core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => {
                    let a = s.shape_attr();
                    assert_eq!(a.current_width, 8758);
                    assert!(
                        (a.render_sx - 8758.0 / 8475.0).abs() < 1e-12,
                        "{}",
                        a.render_sx
                    );
                    assert_eq!(a.rotation_center.x, 4379, "cur/2 — 실측 4237→4379");
                    assert!(a.raw_rendering.is_empty(), "행렬 원본을 비워야 재생성된다");
                }
                _ => unreachable!(),
            }
        }

        /// 뒤집기 — 축 토글·OrgState 복원·행렬. 이동량 식은 여덟 관측의 요약이다(§4.22).
        #[test]
        fn flip_matrix_follows_the_measured_formula() {
            let mut core = core_with_three();
            let sa = |c: &DocumentCore| match &c.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => {
                    let a = s.shape_attr();
                    (a.flip, a.horz_flip, a.render_sx, a.render_tx)
                }
                _ => unreachable!(),
            };
            match &mut core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => {
                    let a = s.shape_attr_mut();
                    a.original_width = 8475;
                    a.current_width = 8475;
                }
                _ => unreachable!(),
            }

            // 배율 없음·홀수 폭: even_ceil(8475) − 2 = 8474.
            core.set_control_flip_at(0, 0, false, false).unwrap();
            assert_eq!(sa(&core), (0x01, true, -1.0, 8474.0));

            // OrgState — 켜져 있으면 끈다. 표시 비트는 흉내 내지 않으므로 축만 진다.
            core.set_control_flip_at(0, 0, false, true).unwrap();
            assert_eq!(sa(&core), (0x00, false, 1.0, 0.0));

            // 이미 원래 상태면 무동작이다.
            let out = core.set_control_flip_at(0, 0, false, true).unwrap();
            assert!(out.contains("\"moved\":false"), "{out}");

            // 배율 걸림·짝수 cur: even_ceil(8758) − 2 = 8756, sx = −8758/8475 (실측 r1).
            match &mut core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => s.shape_attr_mut().current_width = 8758,
                _ => unreachable!(),
            }
            core.set_control_flip_at(0, 0, false, false).unwrap();
            let (_, on, sx, tx) = sa(&core);
            assert!(on);
            assert_eq!(tx, 8756.0);
            assert!((sx - (-(8758.0 / 8475.0))).abs() < 1e-12, "{sx}");

            // 홀수 cur·짝수 org: even_ceil(7033) − 0 = 7034 (실측 v1).
            match &mut core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => {
                    let a = s.shape_attr_mut();
                    a.original_height = 6750;
                    a.current_height = 7033;
                }
                _ => unreachable!(),
            }
            core.set_control_flip_at(0, 0, true, false).unwrap();
            let ty = match &core.document.sections[0].paragraphs[0].controls[0] {
                Control::Shape(s) => s.shape_attr().render_ty,
                _ => unreachable!(),
            };
            assert_eq!(ty, 7034.0);
        }

        /// 글 앞/뒤는 이름과 달리 **순서가 아니라 배치**다 — `z_order` 를 건드리면 안 된다.
        #[test]
        fn text_wrap_modes_do_not_touch_the_order() {
            let mut core = core_with_three();
            core.set_control_z_order_at(0, 1, "behindText").unwrap();
            assert_eq!(orders(&core), vec![0, 1, 2]);
            let common = control_common(&core.document.sections[0].paragraphs[0].controls[1]);
            assert_eq!(common.map(|c| c.text_wrap), Some(TextWrap::BehindText));

            core.set_control_z_order_at(0, 1, "inFrontOfText").unwrap();
            let common = control_common(&core.document.sections[0].paragraphs[0].controls[1]);
            assert_eq!(common.map(|c| c.text_wrap), Some(TextWrap::InFrontOfText));
        }
    }
}
