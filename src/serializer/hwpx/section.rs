//! Contents/section{N}.xml — Section 본문 직렬화
//!
//! Stage 2 (#182): 기존 템플릿 기반 구조를 유지하되, `<hp:p>` 와 `<hp:run>` 의 속성을
//! IR에서 가져와 동적으로 생성한다. `secPr`/`pagePr`/`grid` 등 섹션 정의는 템플릿 보존
//! (IR에 대응 필드가 더 담길 때까지 점진적으로 동적화 예정).
//!
//! Stage #177 (2026-04-18): `<hp:lineseg>` 직렬화를 IR 기반으로 전환.
//! `Paragraph.line_segs` 의 9개 필드(textpos, vertpos, vertsize, textheight, baseline,
//! spacing, horzpos, horzsize, flags)를 그대로 출력하여 **원본 lineseg 값 보존**.
//! rhwp 는 자신의 문서에서 새로 부정확한 값을 생산하지 않는다.
//!
//! IR 매핑 관행:
//!   - `section.paragraphs` 여러 개 = 하드 문단 경계 (`<hp:p>` 여러 개)
//!   - `paragraph.text` 내 `\n` = 소프트 라인브레이크 (`<hp:lineBreak/>`, 같은 문단 내)
//!   - `paragraph.text` 내 `\t` = 탭 (`<hp:tab width=... leader="0" type="1"/>`)
//!   - `paragraph.text` 내 `U+2007` = 고정폭 빈칸 (`<hp:fwSpace/>`, #4675)
//!   - `paragraph.para_shape_id` → `<hp:p paraPrIDRef>`
//!   - `paragraph.style_id` → `<hp:p styleIDRef>`
//!   - `paragraph.column_type` → `<hp:p pageBreak/columnBreak>`
//!   - `paragraph.char_shapes[0].char_shape_id` → 첫 `<hp:run charPrIDRef>`
//!   - `paragraph.line_segs[i]` → 각 `<hp:lineseg>` 속성 (9개 필드 그대로 출력)

use quick_xml::Writer;

use crate::model::control::{
    AutoNumber, AutoNumberType, CharOverlap, Control, Equation, Field, NewNumber, PageHide,
    PageNumCtrl, PageNumberPos, PageStartsOn, Ruby, EQUATION_LINE_MODE_BIT,
};
use crate::model::document::{Document, Section, SectionDef};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType};
use crate::model::paragraph::{
    ColumnBreakType, FieldRange, LineSeg, OrphanFieldEnd, Paragraph, TitleMark,
};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, ShapeObject, SizeCriterion, TextWrap, VertAlign, VertRelTo,
};

use super::context::SerializeContext;
use super::field::{
    write_bookmark, write_field_begin, write_field_end, write_field_end_full, write_hyperlink_begin,
};
use super::utils::xml_escape;
use super::SerializeError;
use super::{picture, table};

const EMPTY_SECTION_XML: &str = include_str!("templates/empty_section0.xml");

/// MEMO subList 여는 태그 (#1391) — 실물(aift) 고정 속성.
/// `textDirection` 만 원본값 보존(가변); 나머지는 실측 고정 속성.
fn render_sub_list_open(text_direction: Option<&str>) -> String {
    format!(
        r#"<hp:subList id="" textDirection="{}" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">"#,
        text_direction.unwrap_or("HORIZONTAL"),
    )
}
const LINESEG_SLOT_OPEN: &str = "<hp:linesegarray>";
const LINESEG_SLOT_CLOSE: &str = "</hp:linesegarray>";
const PARA_CLOSE: &str = "</hp:p></hs:sec>";

// 템플릿 내 첫 <hp:p> 태그의 실제 문자열 (id="3121190098" 랜덤 해시 포함).
// 템플릿은 정적이므로 이 문자열이 고정 위치에 있음이 보장됨.
const TEMPLATE_FIRST_P_TAG: &str = r#"<hp:p id="3121190098" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">"#;
// 템플릿 첫 문단의 텍스트 run 전체 — 템플릿 내 유일하게 1회 등장 (#1378).
const TEMPLATE_TEXT_RUN: &str = r#"<hp:run charPrIDRef="0"><hp:t/></hp:run>"#;
// 템플릿 첫 run(secPr/colPr 전용)의 여는 태그 + secPr 시작 — run id 정비용 anchor (#1378).
const TEMPLATE_SECPR_RUN_OPEN: &str = r#"<hp:run charPrIDRef="0"><hp:secPr "#;

// [#1407] 템플릿의 하드코딩 본문 colPr(단 정의) — 단일 단 기본값. 첫 문단 IR 에
// ColumnDef 가 있으면 이 anchor 를 IR 값으로 치환한다 (#1388 secPr 동형).
const TEMPLATE_BODY_COL_PR: &str = r#"<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" sameGap="0"/></hp:ctrl>"#;

// [#1637] 템플릿의 하드코딩 secPr visibility — 전부 기본값(미숨김/SHOW_ALL). 원본의
// visibility 를 IR(SectionDef) 값으로 치환하지 않으면 hideFirstEmptyLine 등이 드롭되어
// (특히 ="1" → "0") 선두 빈줄이 가시화되며 본문이 밀려 페이지네이션이 달라진다 (#1388 동형).
const TEMPLATE_VISIBILITY: &str = r#"<hp:visibility hideFirstHeader="0" hideFirstFooter="0" hideFirstMasterPage="0" border="SHOW_ALL" fill="SHOW_ALL" hideFirstPageNum="0" hideFirstEmptyLine="0" showLineNumber="0"/>"#;

/// SectionDef 의 visibility 플래그를 `<hp:visibility>` 요소로 직렬화.
///
/// 파서가 IR 에 보존하는 6필드(hideFirstHeader/Footer/MasterPage·border·fill·
/// hideFirstEmptyLine)만 치환하고, IR 미보존 필드(hideFirstPageNum·showLineNumber)는
/// 템플릿 기본값("0")을 유지한다.
fn render_visibility(sd: &SectionDef) -> String {
    let b = |v: bool| if v { "1" } else { "0" };
    let bf = |v: bool| if v { "HIDE_ALL" } else { "SHOW_ALL" };
    format!(
        r#"<hp:visibility hideFirstHeader="{}" hideFirstFooter="{}" hideFirstMasterPage="{}" border="{}" fill="{}" hideFirstPageNum="0" hideFirstEmptyLine="{}" showLineNumber="0"/>"#,
        b(sd.hide_header),
        b(sd.hide_footer),
        b(sd.hide_master_page),
        bf(sd.hide_border),
        bf(sd.hide_fill),
        b(sd.hide_empty_line),
    )
}

/// 템플릿의 visibility 고정 문자열을 IR 기반 값으로 1회 치환.
fn replace_visibility(xml: &str, sd: &SectionDef) -> String {
    xml.replacen(TEMPLATE_VISIBILITY, &render_visibility(sd), 1)
}

/// [#1987] 템플릿 secPr 의 하드코딩 스칼라(spaceColumns, outlineShapeIDRef, memoShapeIDRef 등)를
/// IR 값으로 치환.
/// 각 속성은 템플릿 secPr 여는 태그에 정확히 1회 등장하므로 replacen(1)로 안전하다.
fn replace_secpr_scalars(xml: &str, sd: &SectionDef) -> String {
    let out = xml.replacen(
        r#"spaceColumns="1134""#,
        &format!(r#"spaceColumns="{}""#, sd.column_spacing),
        1,
    );
    // 구역 세로쓰기: 파서는 textDirection="VERTICAL" → text_direction=1 로 읽지만
    // (parser/hwpx/section.rs) 직렬화기는 이 필드를 재출력하지 않아, 세로쓰기 구역이
    // .hwpx 저장 시 HORIZONTAL 로 유실됐다. text_direction==1 일 때만 치환한다
    // (기본 0 문서는 템플릿 HORIZONTAL 유지). 템플릿엔 secPr 에만 등장하므로 replacen(1) 안전.
    let out = if sd.text_direction == 1 {
        out.replacen(
            r#"textDirection="HORIZONTAL""#,
            r#"textDirection="VERTICAL""#,
            1,
        )
    } else {
        out
    };
    let out = out.replacen(
        r#"outlineShapeIDRef="1""#,
        &format!(r#"outlineShapeIDRef="{}""#, sd.outline_numbering_id),
        1,
    );

    // [#2779] 메모 모양 참조. 파서가 secPr@memoShapeIDRef 를 memo_shape_id 로 읽지만
    // 직렬화가 템플릿 상수 "0" 만 방출해, 메모 모양이 지정된 구역이 저장마다 0 으로
    // 리셋됐다(실측 14 secPr/9 파일). 템플릿 secPr 여는 태그에 정확히 1회 등장하고
    // 기본 SectionDef 도 0 이라 기본 문서 출력은 바이트 동일하다.
    let out = out.replacen(
        r#"memoShapeIDRef="0""#,
        &format!(r#"memoShapeIDRef="{}""#, sd.memo_shape_id),
        1,
    );

    // 기본 탭 폭. HWPX 파서는 secPr 의 tabStop 속성을 default_tab_spacing 으로 읽는다
    // (parser/hwpx/section.rs). 치환하지 않으면 열었던 값과 무관하게 늘 템플릿 상수
    // 8000 으로 저장돼, 탭 정렬이 원본과 어긋난다.
    //
    // 다만 SectionDef 는 derive(Default) 라 파싱을 거치지 않은 문서(신규 작성 등)에서는
    // 0 이다. 0 을 그대로 내보내면 탭 폭이 0 이 되어 지금보다 나빠지므로, 값이 있을 때만
    // 치환하고 없으면 템플릿 기본값을 유지한다.
    let out = if sd.default_tab_spacing != 0 {
        out.replacen(
            r#"tabStop="8000""#,
            &format!(r#"tabStop="{}""#, sd.default_tab_spacing),
            1,
        )
    } else {
        out
    };

    // 구역 그리드와 시작 번호. 템플릿 상수가 모두 0 이고 SectionDef 기본값도 0 이라
    // 기본 문서의 출력은 그대로이며, 파싱된 문서에서만 실제 값이 살아난다.
    let out = out.replacen(
        r#"<hp:grid lineGrid="0" charGrid="0" wonggojiFormat="0"/>"#,
        &format!(
            r#"<hp:grid lineGrid="{}" charGrid="{}" wonggojiFormat="0"/>"#,
            sd.line_grid, sd.char_grid
        ),
        1,
    );
    out.replacen(
        r#"<hp:startNum pageStartsOn="BOTH" page="0" pic="0" tbl="0" equation="0"/>"#,
        &format!(
            r#"<hp:startNum pageStartsOn="{}" page="{}" pic="{}" tbl="{}" equation="{}"/>"#,
            page_starts_on_str(sd.page_num_type),
            sd.page_num,
            sd.picture_num,
            sd.table_num,
            sd.equation_num
        ),
        1,
    )
}

/// 쪽 번호 시작 종류(SectionDef.page_num_type, 0/1/2) → HWPX `pageStartsOn` 토큰.
/// 파서 `parse_start_num` 의 역매핑. 0(이어서)=BOTH, 1(홀수)=ODD, 2(짝수)=EVEN.
fn page_starts_on_str(page_num_type: u8) -> &'static str {
    match page_num_type {
        1 => "ODD",
        2 => "EVEN",
        _ => "BOTH",
    }
}

/// [#1984] noteLine `type` u8 → HWPX 문자열 (parser noteLine type 역매핑).
fn note_line_type_str(t: u8) -> &'static str {
    match t {
        0 => "NONE",
        2 => "DASH",
        3 => "DOT",
        4 => "DASH_DOT",
        5 => "DASH_DOT_DOT",
        6 => "LONG_DASH",
        7 => "CIRCLE",
        8 => "DOUBLE_SLIM",
        9 => "SLIM_THICK",
        10 => "THICK_SLIM",
        11 => "SLIM_THICK_SLIM",
        _ => "SOLID",
    }
}

/// [#1984] separator_color(0xBBGGRR LE) → "#RRGGBB" (parser noteLine color 역매핑).
fn note_color_hex(c: u32) -> String {
    let r = c & 0xFF;
    let g = (c >> 8) & 0xFF;
    let b = (c >> 16) & 0xFF;
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// [#1984] FootnoteShape → `<hp:noteLine .../>` + `<hp:noteSpacing .../>` 두 요소.
fn render_note_line_spacing(shape: &crate::model::footnote::FootnoteShape) -> (String, String) {
    let note_line = format!(
        r#"<hp:noteLine length="{}" type="{}" width="{} mm" color="{}"/>"#,
        shape.separator_length,
        note_line_type_str(shape.separator_line_type),
        line_width_mm(shape.separator_line_width),
        note_color_hex(shape.separator_color),
    );
    let note_spacing = format!(
        r#"<hp:noteSpacing betweenNotes="{}" belowLine="{}" aboveLine="{}"/>"#,
        shape.between_notes_margin_hu(),
        shape.separator_below_margin_hu(),
        shape.separator_above_margin_hu(),
    );
    (note_line, note_spacing)
}

/// FootnoteNumbering → HWPX `type` 토큰. 템플릿 계열(CONTINUOUS/EACH_COLUMN/DIGIT)과
/// 동일한 UPPER_SNAKE 표기를 쓰며, 파서도 이 토큰을 수용한다.
fn note_numbering_str(numbering: crate::model::footnote::FootnoteNumbering) -> &'static str {
    use crate::model::footnote::FootnoteNumbering::*;
    match numbering {
        Continue => "CONTINUOUS",
        RestartSection => "RESTART_SECTION",
        RestartPage => "RESTART_PAGE",
    }
}

fn render_note_numbering(shape: &crate::model::footnote::FootnoteShape) -> String {
    format!(
        r#"<hp:numbering type="{}" newNum="{}"/>"#,
        note_numbering_str(shape.numbering),
        shape.start_number,
    )
}

/// [#2779] FootnotePlacement → HWPX `placement/@place` 토큰 (컨텍스트 키 역매핑).
///
/// OWPML 스키마(ParaList: footNotePr/endNotePr 의 placement@place)는 각주와 미주에
/// 서로 다른 열거를 두지만, 두 열거는 HWP5 `attr` bits 8-9 의 같은 코드 공간을 쓴다
/// (모델 주석 「각 단마다 따로 배열 / 문서의 마지막」과 동일한 대응):
///
/// | 코드 | FootnotePlacement | 각주 토큰          | 미주 토큰       |
/// |------|-------------------|--------------------|-----------------|
/// | 0    | EachColumn        | EACH_COLUMN        | END_OF_DOCUMENT |
/// | 1    | BelowText         | MERGED_COLUMN      | END_OF_SECTION  |
/// | 2    | RightColumn       | RIGHT_MOST_COLUMN  | (없음)          |
///
/// 즉 코드↔토큰은 컨텍스트를 아는 한 각 컨텍스트 안에서 전단사이므로, 호출자가
/// 자기가 각주/미주 중 무엇을 렌더하는지 알려주면 무손실 역매핑이 된다.
/// 미주에 코드 2(RightColumn)가 들어오는 비정상 IR 은 스키마에 대응 토큰이 없어
/// 기본값 `END_OF_DOCUMENT` 로 강등한다.
fn note_place_str(
    placement: crate::model::footnote::FootnotePlacement,
    is_end_note: bool,
) -> &'static str {
    use crate::model::footnote::FootnotePlacement::*;
    if is_end_note {
        match placement {
            BelowText => "END_OF_SECTION",
            // EachColumn 및 스키마 밖 RightColumn → 기본값.
            _ => "END_OF_DOCUMENT",
        }
    } else {
        match placement {
            BelowText => "MERGED_COLUMN",
            RightColumn => "RIGHT_MOST_COLUMN",
            EachColumn => "EACH_COLUMN",
        }
    }
}

/// [#2779] FootnoteShape → `<hp:placement .../>` (place + beneathText).
fn render_note_placement(
    shape: &crate::model::footnote::FootnoteShape,
    is_end_note: bool,
) -> String {
    format!(
        r#"<hp:placement place="{}" beneathText="{}"/>"#,
        note_place_str(shape.placement, is_end_note),
        u8::from(shape.print_inline_after_text),
    )
}

/// [#2742] NumberFormat → HWPX `autoNumFormat/@type` 토큰.
///
/// 파서 `FootnoteShape::number_format_from_name` 의 UPPER_SNAKE 분기 역매핑이며,
/// 같은 파일의 `note_line_type_str`·`note_numbering_str` 와 동형이다. 이름이 모델
/// variant 와 다른 항목(UpperRoman=ROMAN_CAPITAL, HangulDigit=HANGUL_PHONETIC,
/// HanjaDigit=IDEOGRAPH, HanjaGapEul=DECAGON_CIRCLE, FourSymbol=SYMBOL)은 파서 표기를
/// 그대로 따른다.
fn note_number_format_str(format: crate::model::footnote::NumberFormat) -> &'static str {
    use crate::model::footnote::NumberFormat::*;
    match format {
        Digit => "DIGIT",
        CircledDigit => "CIRCLED_DIGIT",
        UpperRoman => "ROMAN_CAPITAL",
        LowerRoman => "ROMAN_SMALL",
        UpperAlpha => "LATIN_CAPITAL",
        LowerAlpha => "LATIN_SMALL",
        CircledUpperAlpha => "CIRCLED_LATIN_CAPITAL",
        CircledLowerAlpha => "CIRCLED_LATIN_SMALL",
        HangulSyllable => "HANGUL_SYLLABLE",
        CircledHangulSyllable => "CIRCLED_HANGUL_SYLLABLE",
        HangulJamo => "HANGUL_JAMO",
        CircledHangulJamo => "CIRCLED_HANGUL_JAMO",
        HangulDigit => "HANGUL_PHONETIC",
        HanjaDigit => "IDEOGRAPH",
        CircledHanjaDigit => "CIRCLED_IDEOGRAPH",
        HanjaGapEul => "DECAGON_CIRCLE",
        HanjaGapEulHanja => "DECAGON_CIRCLE_HANJA",
        FourSymbol => "SYMBOL",
        UserChar => "USER_CHAR",
    }
}

/// [#2742] 주석 장식 문자(IR `char`) → HWPX 속성값.
///
/// `'\0'` 은 "미지정" 규약이므로(`object_ops/note.rs` 가 `suffix_char == '\0'` 을 기본값
/// 폴백으로 해석) 템플릿 기본값 `fallback` 을 유지한다. 파싱을 거치지 않은 문서에서
/// IR 이 0 이면 템플릿 상수를 남기는 `tabStop`/`textDirection` 치환과 같은 패턴이다.
///
/// `'\0'` 외의 제어문자(< 0x20)도 같은 폴백으로 보낸다. HWP5 의 장식 문자는 WCHAR 원값이라
/// 이론상 제어문자가 올 수 있고, 그대로 방출하면 XML 1.0 이 금지하는 문자가 되어 저장본을
/// 한컴이 열지 못한다. 같은 파일 `render_hp_t_content` 도 `< 0x20` 을 방출 대상에서 뺀다.
/// (코퍼스 828 note shape 의 장식 문자는 전부 `'\0'` 또는 출력 가능 문자였다.)
fn note_deco_char_attr(c: char, fallback: &str) -> String {
    if (c as u32) < 0x20 {
        fallback.to_string()
    } else {
        xml_escape(&c.to_string())
    }
}

/// [#2742] FootnoteShape → `<hp:autoNumFormat .../>`.
/// 속성 순서는 템플릿·한컴 실물과 같이 type → userChar → prefixChar → suffixChar → supscript.
fn render_auto_num_format(shape: &crate::model::footnote::FootnoteShape) -> String {
    format!(
        r#"<hp:autoNumFormat type="{}" userChar="{}" prefixChar="{}" suffixChar="{}" supscript="{}"/>"#,
        note_number_format_str(shape.number_format),
        note_deco_char_attr(shape.user_char, ""),
        note_deco_char_attr(shape.prefix_char, ""),
        note_deco_char_attr(shape.suffix_char, ")"),
        u8::from(shape.number_code_superscript),
    )
}

/// [#2742] 템플릿의 하드코딩 autoNumFormat — 각주/미주 두 슬롯의 문자열이 동일하다.
const TEMPLATE_AUTO_NUM_FORMAT: &str =
    r#"<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>"#;

/// `needle` 의 처음 두 출현을 각각 `first`/`second` 로 치환한다. 치환 결과가 needle
/// 과 같아도 안전하다(연쇄 replacen 은 replacement==needle 일 때 두 번째가 첫 슬롯을
/// 다시 잡는 버그가 있어 각주/미주 numbering 처럼 fn/en 템플릿 문자열이 동일한 경우
/// 위치 기반 분할이 필요하다). needle 이 2회 미만이면 원본을 반환한다.
fn replace_first_two(haystack: &str, needle: &str, first: &str, second: &str) -> String {
    let mut parts = haystack.splitn(3, needle);
    match (parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), Some(c)) => format!("{a}{first}{b}{second}{c}"),
        _ => haystack.to_string(),
    }
}

/// [#1984] 템플릿 footNotePr/endNotePr 의 하드코딩 noteLine·noteSpacing 을 IR 값으로 치환.
/// 미치환 시 각주 구분선 위/아래 여백·주석간격이 항상 기본값(aboveLine=850 등)으로 방출돼
/// 각주 zone 높이가 달라지고, 각주 있는 페이지의 본문 가용높이가 어긋나 표 분할·페이지 수가
/// 갈린다(1543000: 각주 overhead 32.83→19.39px → p141 표 1행/3행 → 192/190쪽). 파서는
/// 값을 FootnoteShape 로 수집하나 직렬화가 템플릿 상수만 방출하던 결함.
fn replace_footnote_shape(xml: &str, sd: &SectionDef) -> String {
    // [#2742] 번호 모양·사용자 기호·앞/뒤 장식 문자·위첨자. 파서는 HWPX(autoNumFormat)와
    // HWP5(FOOTNOTE_SHAPE attr) 양쪽에서 이 5필드를 FootnoteShape 로 읽지만 직렬화가
    // 템플릿 상수만 방출해, 저장할 때마다 구역 각주/미주 모양이 한컴 기본값으로 리셋됐다.
    // 새 주석 삽입은 이 모양을 기본값으로 쓰므로(object_ops/note.rs) 저장본에서는 미주가
    // 「문1）」 대신 「1)」로 매겨진다(실측: 코퍼스 330파일 중 18파일 · note shape 19개).
    // 각주/미주 템플릿 문자열이 완전히 같아 연쇄 replacen 은 두 번째 슬롯을 못 잡는다 —
    // numbering 과 같은 사유로 위치 기반 2회 치환을 쓴다.
    let xml = replace_first_two(
        xml,
        TEMPLATE_AUTO_NUM_FORMAT,
        &render_auto_num_format(&sd.footnote_shape),
        &render_auto_num_format(&sd.endnote_shape),
    );
    // 템플릿: 첫 noteLine(length="-1")·noteSpacing(betweenNotes="283") = 각주,
    // 둘째(length="14692344"·betweenNotes="0") = 미주.
    let (fn_line, fn_spacing) = render_note_line_spacing(&sd.footnote_shape);
    let (en_line, en_spacing) = render_note_line_spacing(&sd.endnote_shape);
    let out = xml
        .replacen(
            r##"<hp:noteLine length="-1" type="SOLID" width="0.12 mm" color="#000000"/>"##,
            &fn_line,
            1,
        )
        .replacen(
            r#"<hp:noteSpacing betweenNotes="283" belowLine="567" aboveLine="850"/>"#,
            &fn_spacing,
            1,
        )
        .replacen(
            r##"<hp:noteLine length="14692344" type="SOLID" width="0.12 mm" color="#000000"/>"##,
            &en_line,
            1,
        )
        .replacen(
            r#"<hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>"#,
            &en_spacing,
            1,
        );
    // 번호 매기기(각주 번호 종류 + 시작 번호). 템플릿은 fn/en 모두
    // `type="CONTINUOUS" newNum="1"` 로 동일해, 미치환 시 페이지/구역마다
    // 새로 매기거나 시작 번호가 1이 아닌 문서가 저장 때 연속·1로 되돌아간다.
    // fn/en 템플릿 문자열이 같으므로 위치 기반 2회 치환을 쓴다.
    let out = replace_first_two(
        &out,
        r#"<hp:numbering type="CONTINUOUS" newNum="1"/>"#,
        &render_note_numbering(&sd.footnote_shape),
        &render_note_numbering(&sd.endnote_shape),
    );

    // placement — 배치 방법(place)과 beneathText(본문 아래 바로 이어 출력).
    //
    // beneathText 는 같은 요소의 독립 bool 이라 종전에도 IR 반영됐다(템플릿 "0" 고정
    // 이라 저장 때마다 꺼지던 결함). [#2779] place 는 종전에 템플릿 상수
    // (EACH_COLUMN/END_OF_DOCUMENT)를 그대로 방출해, 통단·오른쪽단 각주와 구역끝
    // 미주가 저장마다 기본 배치로 되돌아갔다. 컨텍스트 키 역매핑(note_place_str)으로
    // 각주/미주 각각의 스키마 토큰을 방출한다.
    //
    // 각주 슬롯의 방출 토큰은 EACH_COLUMN/MERGED_COLUMN/RIGHT_MOST_COLUMN 셋 뿐이라
    // 미주 앵커(END_OF_DOCUMENT)를 새로 만들어내지 않는다 — 연쇄 replacen(1) 이
    // 서로의 슬롯을 훔칠 수 없다(기존 코드와 동일 근거).
    out.replacen(
        r#"<hp:placement place="EACH_COLUMN" beneathText="0"/>"#,
        &render_note_placement(&sd.footnote_shape, false),
        1,
    )
    .replacen(
        r#"<hp:placement place="END_OF_DOCUMENT" beneathText="0"/>"#,
        &render_note_placement(&sd.endnote_shape, true),
        1,
    )
}

/// 레퍼런스 기준 줄 레이아웃 파라미터.
const VERT_STEP: u32 = 1600; // vertsize(1000) + spacing(600)

/// 탭 확장 데이터(`tab_extended`)가 없는 "암묵적 기본 탭"을 위한 `<hp:tab width>` 마커.
///
/// OWPML `<hp:tab>` 은 구조상 `width`/`leader`/`type` 이 모두 필수라 값 없음을 표현할 수
/// 없다(#4403). 예전에는 여기 고정 상수 `TAB_DEFAULT_WIDTH = 4000`(한컴 실제 기본 탭 간격,
/// `secPr@tabStopVal` 실측·HWP5 스펙 "기본 탭 간격" 필드와 일치 — 상수 자체는 정확했다)을
/// 채웠는데, 렌더러(`renderer/layout/text_measurement.rs` 인라인 탭 처리)는 `tab_extended`
/// 항목이 있으면 그 `width` 를 "이 탭의 실제 계산된 전진량"으로 신뢰해 커서 위치에 그대로 더한다
/// (`total + width`) — 탭 앞 텍스트 폭이나 문단의 실제 `TabDef`(좌/우/가운데 정렬, 커스텀 위치)
/// 를 무시한다. 그 결과 재적재 후 탭 뒤 텍스트가 원래와 다른 위치에 그려진다: 목차처럼 "제목 +
/// 탭 + 쪽번호"인 문단에서 원본은 문단의 우측 정렬 `TabDef` 로 쪽번호가 우측 끝에 정렬되는데,
/// 라운드트립 후에는 이 탭이 명시적 LEFT 로 굳어져 제목 바로 뒤에 고정 거리만큼만 전진한다
/// (실측: `samples/SO-SUEOP.hwp` 자기 라운드트립 `render-diff`, 목차 페이지 최대 변위 470px).
///
/// `width=0` 은 실제 탭에서 나올 수 없는 값이다(폭 0인 탭은 시각적으로 아무 효과가 없어 한컴도
/// 만들지 않는다) — 그래서 "원본에 계산된 탭 폭 데이터가 없었다"는 마커로 안전하게 쓴다.
/// HWPX 파서(`parser/hwpx/section.rs`)는 `width=0` 인 `<hp:tab>` 을 만나면 `tab_extended`
/// 항목을 만들지 않고 비워 둔다 — HWP5 바이너리 직렬화기의 동형 널 마커(`serializer/body_text.rs`,
/// #1892)와 같은 규약이다. 그러면 렌더러는 이 문단의 실제 `TabDef`/커서 위치 기준으로
/// `find_next_tab_stop` 을 통해 탭 정지를 다시 계산해, 원본과 같은 경로를 탄다. 한컴 앱 자신도
/// 이 값을 열 때 재계산하는 것으로 보여(주석 원문 "한컴이 열면서 재계산하지만 초기값으로 필요"),
/// `width=0` 은 실제 한컴에서 다시 열 때도 안전하다.
const TAB_NO_DATA_WIDTH_MARKER: u32 = 0;

/// Stage 2 진입점. `ctx` 는 Stage 3+ 에서 파라미터 검증에 사용.
pub fn write_section(
    section: &Section,
    doc: &Document,
    index: usize,
    ctx: &mut SerializeContext,
) -> Result<Vec<u8>, SerializeError> {
    let mut vert_cursor: u32 = 0;

    let first_para = section.paragraphs.first();
    // [#1584] 첫 문단 렌더 직전 set — 본문 첫 ColumnDef(섹션 템플릿 흡수분)의 인라인
    // XML 방출만 1회 억제한다(슬롯 위치는 보존). 렌더 직후 reset 하여 추가 문단 누설 방지.
    ctx.body_coldef_template_pending = true;
    let (first_runs, first_linesegs, first_advance) = match first_para {
        Some(p) => render_paragraph_parts(p, vert_cursor, ctx),
        // 문단이 없는 섹션(비파싱 IR) — linesegarray 방출 생략 (#1380)
        None => (String::new(), String::new(), vert_cursor),
    };
    ctx.body_coldef_template_pending = false;
    vert_cursor = first_advance;

    // 치환은 모두 pristine 템플릿의 고정 anchor 에 대해 수행한다 (#1378):
    // linesegarray 치환을 콘텐츠(run 시퀀스) 삽입보다 먼저 두어, 콘텐츠에 포함된
    // 중첩 linesegarray(각주·머리말 등)가 anchor 탐색을 오염시키지 않도록 한다.
    let mut out = replace_first_linesegs(EMPTY_SECTION_XML, &first_linesegs);
    out = replace_page_pr(&out, &section.section_def.page_def);
    // 쪽 테두리/배경 — 템플릿의 하드코딩 borderFillIDRef="1"(테두리 없음)을 IR 값으로
    // 치환한다. 누락 시 문서의 쪽 테두리가 소실되어 외곽 4선 노드가 사라진다(#1388 동형).
    out = replace_page_border_fill(&out, &section.section_def);
    // [#1637] secPr visibility — 템플릿 고정값을 IR 값으로 치환(hideFirstEmptyLine 등 보존).
    out = replace_visibility(&out, &section.section_def);

    // [#1987] secPr 스칼라 필드 — 템플릿 하드코딩(spaceColumns="1134", outlineShapeIDRef="1")을
    // IR 값으로 치환한다. 미치환 시 원본이 spaceColumns=1130·outlineShapeIDRef=0 등이어도
    // 1134/1 로 방출돼 단 간격·개요번호 문단모양 참조가 어긋난다. ir-diff 는 secPr 스칼라를
    // 비교하지 않아 못 잡던 저장 충실도 결함.
    out = replace_secpr_scalars(&out, &section.section_def);

    // [#1984] 각주/미주 모양(구분선 여백·주석간격)을 IR 값으로 치환 — 미치환 시 각주 zone
    // 높이가 기본값으로 고정돼 각주 있는 페이지의 표 분할·페이지 수가 갈린다.
    out = replace_footnote_shape(&out, &section.section_def);

    // 바탕쪽(masterPage) — secPr 의 masterPageCnt 치환 + secPr 내부 끝에 idRef 참조 삽입.
    // 누락 시 라운드트립에서 바탕쪽 전체(그 안의 그림/표/문단 노드 포함)가 소실된다.
    // id 인덱스는 전 섹션 누적 전역값으로, mod.rs 의 파일 생성 인덱스와 정합한다.
    let master_pages = &section.section_def.master_pages;
    if !master_pages.is_empty() {
        let base: usize = doc.sections[..index]
            .iter()
            .map(|s| s.section_def.master_pages.len())
            .sum();
        let ids: Vec<String> = (0..master_pages.len())
            .map(|k| format!("masterpage{}", base + k))
            .collect();
        out = out.replacen(
            r#"masterPageCnt="0""#,
            &format!(r#"masterPageCnt="{}""#, master_pages.len()),
            1,
        );
        let refs = super::master_page::render_master_page_refs(&ids);
        out = out.replacen("</hp:secPr>", &format!("{refs}</hp:secPr>"), 1);
    }

    // [#1407] 본문 단 정의(colPr) — 첫 문단 IR 의 ColumnDef 를 템플릿 하드코딩
    // colPr(colCount=1)에 치환한다. 본문(depth 0) ColumnDef 는 render_runs 인라인
    // 슬롯에서 제외되므로(#1379) 여기서 받지 않으면 단 정의가 손실된다(2단→1단 →
    // 페이지 넘침). #1388 secPr 여백 치환과 동형.
    if let Some(p) = first_para {
        if let Some(Control::ColumnDef(cd)) = p
            .controls
            .iter()
            .find(|c| matches!(c, Control::ColumnDef(_)))
        {
            out = out.replacen(TEMPLATE_BODY_COL_PR, &render_col_pr_ctrl(cd), 1);
        }
    }

    if let Some(p) = first_para {
        // 첫 문단 `<hp:p>` 태그를 IR 기반 속성으로 교체
        // (#1933) 본문 경로는 종전에 style_id 를 reference 하지 않았다 — emit 만
        // 강등해 미등록 ID 방출을 막고, 참조 집합/assert 동작은 종전 유지한다.
        let pid = ctx.next_para_id();
        let sid = ctx.effective_style_id(p.style_id);
        let new_p_tag = render_hp_p_open(p, pid, sid);
        out = out.replacen(TEMPLATE_FIRST_P_TAG, &new_p_tag, 1);

        // 섹션 첫 run(secPr/colPr 전용)의 charPrIDRef 를 첫 텍스트 run id 와 일치시킨다.
        // 파서는 이 run 시작에서도 (0, id) 를 기록하므로, id 가 다르면 재파싱 시
        // 가짜 (0, 0) entry 가 생긴다 (#1378 stage1 양상 ②).
        let first_cs = first_run_char_shape_id(p);
        if first_cs != 0 {
            let new_secpr_run = format!(r#"<hp:run charPrIDRef="{}"><hp:secPr "#, first_cs);
            out = out.replacen(TEMPLATE_SECPR_RUN_OPEN, &new_secpr_run, 1);
        }

        // 템플릿의 텍스트 run 전체를 문단의 run 시퀀스(다중 run 분할 포함)로 1회 치환.
        out = out.replacen(TEMPLATE_TEXT_RUN, &first_runs, 1);
    }

    // 추가 문단: `</hp:p></hs:sec>` 직전에 `<hp:p>` 요소를 삽입.
    if section.paragraphs.len() > 1 {
        let mut extra = String::new();
        for p in section.paragraphs.iter().skip(1) {
            let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
            vert_cursor = advance;
            let pid = ctx.next_para_id();
            let sid = ctx.effective_style_id(p.style_id);
            extra.push_str(&render_hp_p_open(p, pid, sid));
            extra.push_str(&runs);
            extra.push_str(&linesegs);
            extra.push_str("</hp:p>");
        }
        out = out.replacen(PARA_CLOSE, &format!("</hp:p>{}</hs:sec>", extra), 1);
    }

    Ok(out.into_bytes())
}

/// IR의 Paragraph를 기반으로 `<hp:p>` 시작 태그를 생성.
///
/// `id` 는 문단 순서 기반(0, 1, 2, ...)로 할당한다. 한컴 샘플은 랜덤 해시도 쓰지만
/// 파서는 id 를 무시하므로 순차값으로 충분.
///
/// `style_id_ref` 는 호출자가 `ctx.effective_style_id(p.style_id)` 로 강등한 값
/// (미등록 스타일 → 0, #1933). 전 문단 경로가 이 함수를 거치므로 여기가 단일
/// 방출 지점이다.
pub(crate) fn render_hp_p_open(p: &Paragraph, id: u32, style_id_ref: u8) -> String {
    let page_break = if matches!(p.column_type, ColumnBreakType::Page) {
        1
    } else {
        0
    };
    let column_break = if matches!(p.column_type, ColumnBreakType::Column) {
        1
    } else {
        0
    };
    format!(
        r#"<hp:p id="{}" paraPrIDRef="{}" styleIDRef="{}" pageBreak="{}" columnBreak="{}" merged="0">"#,
        id, p.para_shape_id, style_id_ref, page_break, column_break,
    )
}

/// 문단 첫 run 의 charPrIDRef. IR의 `char_shapes[0].char_shape_id` 사용.
/// 비어있으면 0 (기본 글자모양) 반환.
fn first_run_char_shape_id(p: &Paragraph) -> u32 {
    p.char_shapes.first().map(|r| r.char_shape_id).unwrap_or(0)
}

/// Paragraph 하나를 (완전한 `<hp:run>` 시퀀스 XML, `<hp:linesegarray>` 요소 XML,
/// 다음 vert_cursor)로 변환.
///
/// `<hp:lineseg>` 출력 원칙 (#177, #1380):
/// - `para.line_segs` 가 비어있지 않으면 **IR 값 그대로** `<hp:linesegarray>` 요소로 출력
/// - 비어있으면 **요소 자체를 방출 생략** (빈 문자열 반환) — 원본에 linesegarray 가
///   없는 문단의 보존 + rhwp 는 lineseg 를 새로 생산하지 않음. 한컴은 열 때 재계산
///
pub(crate) fn render_paragraph_parts(
    para: &Paragraph,
    vert_start: u32,
    ctx: &mut SerializeContext,
) -> (String, String, u32) {
    let (runs_xml, position_axis_intact) = render_runs(para, ctx);

    // [#4778] 위치 축이 무너진 문단(파서가 담지 못한 8유닛 슬롯 — 예: 차례표지
    // 0x0008 — 이 있거나 mismatch 폴백으로 컨트롤을 말미에 몰아쓴 문단)에는 저장
    // lineseg 를 방출하지 않는다. 방출 텍스트와 textpos 사다리가 어긋난 lineseg 를
    // 한글 2022 가 만나면 **그 문단부터 문서 끝까지 본문을 통째로 폐기**한다
    // (성년후견 h2x: -112,075자 실측 — lineseg 억제만으로 전량 회복). 방출을
    // 생략하면 한글이 열 때 재계산한다(#1380 과 같은 계약).
    if !para.line_segs.is_empty() && position_axis_intact {
        // IR 기반 출력 — 원본 lineseg 값 보존 (#177)
        //
        let linesegs = format!(
            "{}{}{}",
            LINESEG_SLOT_OPEN,
            render_lineseg_array_from_ir(&para.line_segs),
            LINESEG_SLOT_CLOSE
        );
        let vert_end = next_vert_cursor_from_ir(&para.line_segs, vert_start);
        (runs_xml, linesegs, vert_end)
    } else {
        // IR 에 line_segs 없음 — linesegarray 방출 생략 (#1380)
        (runs_xml, String::new(), vert_start)
    }
}

/// 원본에서 글자들이 0 부터 연속으로 놓여 있었는가 — 곧 컨트롤이 전부 텍스트 뒤에 있어
/// mismatch 경로로 다시 써도 글자 위치가 밀리지 않는가.
///
/// mismatch 경로는 텍스트를 0 부터 연속으로 방출한다. 원본 `char_offsets` 가 그 누적 폭과
/// 같으면 방출 전후 좌표가 일치하므로 lineseg 의 `text_start` 가 그대로 유효하다.
fn text_positions_unshifted(para: &Paragraph) -> bool {
    let mut expected = 0u32;
    for (i, c) in para.text.chars().enumerate() {
        match para.char_offsets.get(i) {
            Some(&off) if off == expected => {}
            // char_offsets 가 없는 합성 IR 은 갭이 없다고 본다(종전 동작 유지).
            None => {}
            _ => return false,
        }
        expected = expected.saturating_add(char_utf16_width(c));
    }
    true
}

/// 문단 하나를 여러 `<hp:t>` 조각으로 나눠 방출해도 위치가 이어지는 커서.
///
/// 탭 확장·제목 차례 표시는 둘 다 "문단 축 위 n번째" 로만 식별되므로, run 분할·필드
/// 방출로 조각이 갈려도 같은 커서를 물려줘야 제자리에 실린다.
#[derive(Default)]
pub(crate) struct InlineCursor<'a> {
    /// 다음에 소비할 `tab_extended` 인덱스
    pub tab_idx: usize,
    /// 문단의 제목 차례 표시 전체 (문자 인덱스 오름차순)
    pub title_marks: &'a [TitleMark],
    /// 다음에 방출할 `title_marks` 인덱스
    pub mark_idx: usize,
    /// 지금까지 방출한 문단 텍스트의 문자 수
    pub char_idx: usize,
}

impl InlineCursor<'_> {
    /// 현재 문자 위치에 걸린 제목 차례 표시를 전부 방출한다.
    fn flush_marks_at_cursor(&mut self, t_xml: &mut String, buf: &mut String) {
        while let Some(m) = self.title_marks.get(self.mark_idx) {
            if m.char_idx > self.char_idx {
                break;
            }
            flush_buf(t_xml, buf);
            t_xml.push_str(&format!(
                r#"<hp:titleMark ignore="{}"/>"#,
                if m.ignore { 1 } else { 0 }
            ));
            self.mark_idx += 1;
        }
    }
}

/// `<hp:t>...</hp:t>` 본문 생성 — 탭/소프트브레이크/XML escape 포함.
///
/// `tab_extended`: IR의 탭 확장 정보 목록. `cursor.tab_idx` 를 통해 탭 문자마다 순서대로
/// 참조한다. 항목이 없으면 "데이터 없음" 마커(width=`TAB_NO_DATA_WIDTH_MARKER`=0, leader=0,
/// type=1)를 방출한다(#4403) — 파서가 이를 인식해 `tab_extended` 를 만들지 않아야 렌더러가
/// 실제 `TabDef` 기준으로 탭 정지를 다시 계산한다.
pub(crate) fn render_hp_t_content(
    text: &str,
    tab_extended: &[[u16; 7]],
    cursor: &mut InlineCursor<'_>,
) -> String {
    let mut t_xml = String::from("<hp:t>");
    let mut buf = String::new();
    // 조각 첫머리에서도 훑는다 — char_shape 경계가 표시 자리에 걸리면 그 표시만 담은
    // 빈 run 이 나오는데(한컴 실측: `<hp:t><hp:titleMark ignore="1"/></hp:t>`),
    // 문자 루프만으로는 문자가 없어 그 run 을 그냥 지나친다.
    cursor.flush_marks_at_cursor(&mut t_xml, &mut buf);
    for c in text.chars() {
        // 제목 차례 표시는 이 문자 **앞**에 놓인다.
        cursor.flush_marks_at_cursor(&mut t_xml, &mut buf);
        cursor.char_idx += 1;
        match c {
            '\t' => {
                flush_buf(&mut t_xml, &mut buf);
                let (width, leader, tab_type) = if let Some(ext) = tab_extended.get(cursor.tab_idx)
                {
                    cursor.tab_idx += 1;
                    (ext[0] as u32, ext[2] & 0x00ff, (ext[2] >> 8) & 0x00ff)
                } else {
                    (TAB_NO_DATA_WIDTH_MARKER, 0u16, 1u16)
                };
                t_xml.push_str(&format!(
                    r#"<hp:tab width="{}" leader="{}" type="{}"/>"#,
                    width, leader, tab_type
                ));
            }
            '\n' => {
                flush_buf(&mut t_xml, &mut buf);
                t_xml.push_str("<hp:lineBreak/>");
            }
            // 고정폭 빈칸은 `<hp:fwSpace/>` 요소로 복원한다(#4675). 파서가
            // `<hp:fwSpace/>`→U+2007 로 읽으므로 리터럴 방출은 표현 강등이다 — 한글은
            // 요소를 텍스트 추출에 싣지 않지만 리터럴은 문자로 실어, 저장본의 추출
            // 텍스트·재조판이 원본과 달라진다(10k 스윕 figure-space-only 1,970건).
            //
            // 한컴 원본 실측(hwpx 300건): U+2007 은 fwSpace 요소 530회 · 리터럴 0회로
            // **항상 요소**다. 반면 U+00A0 은 nbSpace 요소 15회 · 리터럴 9회로 섞여 있어
            // 요소로 강제하면 리터럴이던 원본에서 한글 추출 텍스트가 사라진다(실측 23건).
            // IR 이 두 표기를 구분하지 못하는 한 U+00A0 은 기존대로 리터럴로 둔다.
            '\u{2007}' => {
                flush_buf(&mut t_xml, &mut buf);
                t_xml.push_str("<hp:fwSpace/>");
            }
            // 소프트 하이픈도 같은 계약이다 — 리터럴 '-' 로 내리면 한글이 실제
            // 하이픈으로 읽어 단어가 갈라진다(`pertinent` → `per-tinent`).
            // 스키마의 `<hp:hyphen>`(ParaList XML schema.xml:291)으로 되돌린다.
            '\u{00AD}' => {
                flush_buf(&mut t_xml, &mut buf);
                t_xml.push_str("<hp:hyphen/>");
            }
            c if (c as u32) < 0x20 => { /* 기타 제어문자 무시 */ }
            c => buf.push(c),
        }
    }
    flush_buf(&mut t_xml, &mut buf);
    t_xml.push_str("</hp:t>");
    t_xml
}

/// 문단 콘텐츠를 `char_shapes` 경계 기준 다중 `<hp:run>` 으로 분할 출력하는 빌더 (#1378).
///
/// 파서(`src/parser/hwpx/section.rs`)는 각 `<hp:run charPrIDRef>` 시작 위치에서
/// `(utf16_pos, char_shape_id)` 를 기록한다. 동일 id라도 위치가 다른 경계는 보존한다.
/// 이 빌더는 그 역방향: `segs[i].0` (i ≥ 1) 위치에서 run 을 닫고 새 run 을 연다.
///
/// 경계 규칙 (구현계획서 1.2):
/// 1. 경계와 슬롯/문자가 같은 위치면 경계 먼저 — 해당 콘텐츠는 새 run 소속 (`cut_before`)
/// 2. 연속 동일 id 경계도 run으로 방출 — start_pos 보존 (#3739)
/// 3. `char_shapes` 가 비어있으면 단일 run `charPrIDRef="0"`
/// 4. `segs[0].start_pos > 0` 인 비정상 IR 도 첫 run 은 위치 0 부터 시작 (관용 처리)
/// 5. 빈 세그먼트는 `<hp:t></hp:t>` 로 방출 — 재파싱 시 run 시작 entry 위치 보존
struct RunSplitter {
    /// `(start_pos, char_shape_id)` — `[0]` 은 첫 run, `[1..]` 은 cut 경계.
    segs: Vec<(u32, u32)>,
    /// 다음 적용할 경계 인덱스.
    next: usize,
    /// 완성된 run 시퀀스.
    runs: String,
    /// 현재 run 의 내부 콘텐츠 버퍼.
    content: String,
}

impl RunSplitter {
    fn new(para: &Paragraph) -> Self {
        let mut segs: Vec<(u32, u32)> = Vec::new();
        for cs in &para.char_shapes {
            segs.push((cs.start_pos, cs.char_shape_id));
        }
        if segs.is_empty() {
            segs.push((0, 0)); // 규칙 3
        }
        Self {
            segs,
            next: 1,
            runs: String::new(),
            content: String::new(),
        }
    }

    /// 경계가 없는 단일 run 문단인지.
    fn single_run(&self) -> bool {
        self.segs.len() == 1
    }

    /// `pos` 위치의 콘텐츠 방출 전에 run 경계 적용이 필요한지 (flush 판단용).
    fn needs_cut(&self, pos: u32) -> bool {
        self.next < self.segs.len() && self.segs[self.next].0 <= pos
    }

    /// `pos` 이하 위치의 경계를 모두 적용 — 규칙 1 (경계 먼저, 콘텐츠는 새 run 소속).
    fn cut_before(&mut self, pos: u32) {
        while self.needs_cut(pos) {
            self.cut_one();
        }
    }

    /// 경계 1개만 적용한다 — 같은 위치에 여러 경계가 겹칠 때 그중 특정 run 에
    /// 콘텐츠를 넣어야 하는 경우에 쓴다 (#3545 안내문 잔재 복원).
    fn cut_one(&mut self) {
        self.close_run();
        self.next += 1;
    }

    /// 현재 열려 있는 run 의 `charPrIDRef`.
    fn current_shape_id(&self) -> u32 {
        self.segs[self.next - 1].1
    }

    /// 현재 run 을 `<hp:run charPrIDRef>` 로 감싸 완성 목록에 추가.
    fn close_run(&mut self) {
        self.runs.push_str(&format!(
            r#"<hp:run charPrIDRef="{}">"#,
            self.segs[self.next - 1].1
        ));
        if self.content.is_empty() {
            // 규칙 5 — 빈 run 도 <hp:t></hp:t> 로 방출해 재파싱 시 entry 보존
            self.runs.push_str("<hp:t></hp:t>");
        } else {
            self.runs.push_str(&self.content);
            self.content.clear();
        }
        self.runs.push_str("</hp:run>");
    }

    /// 잔여 경계(콘텐츠 끝 이후 시작)를 빈 run 으로 방출하고 마지막 run 을 닫는다.
    fn finish(mut self) -> String {
        while self.next < self.segs.len() {
            self.close_run();
            self.next += 1;
        }
        self.close_run();
        self.runs
    }
}

/// `<hp:ctrl><hp:fieldEnd beginIDRef=".."/></hp:ctrl>` 방출 공통 경로.
fn emit_field_end(out: &mut String, para: &Paragraph, fr: &FieldRange) {
    if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
        // [Task #bookmark-hyperlink] 짝(matched) fieldEnd 자신의 fieldid 는 fieldBegin 의
        // id(f.field_id, beginIDRef 로 사용)와 별개 값 — 파싱된 fr.end_field_id 를 그대로
        // 되돌려 써야 한다. 과거엔 write_field_end 로 beginIDRef 만 쓰고 fieldid 는 항상
        // 누락시켰다(고아 fieldEnd 경로만 write_field_end_full 로 보존, 비대칭).
        let xml_result = if fr.end_field_id == 0 {
            writer_to_string(|w| write_field_end(w, f.field_id))
        } else {
            writer_to_string(|w| write_field_end_full(w, f.field_id, fr.end_field_id))
        };
        if let Ok(xml) = xml_result {
            out.push_str("<hp:ctrl>");
            out.push_str(&xml);
            out.push_str("</hp:ctrl>");
        }
    }
}

/// [#3545] 적재 정규화가 지운 **초기 상태 누름틀의 안내문 본문 run** 을 되살린다.
///
/// 한컴은 미기입 누름틀(`dirty="0"`)의 안내문을 파일에는 begin~end 사이 본문 run 으로
/// 유지하고 렌더·인쇄에서만 구분 취급한다 (`samples/hwpx/form-01.hwpx` 의
/// `<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>`). rhwp 는 적재 시
/// `clear_initial_field_texts` 로 이를 빈 필드로 정규화하므로, 저장에서 되살리지 않으면
/// 파일 차원에서 텍스트가 영구 소실된다(XSD 는 통과하는 조용한 내용 소실).
///
/// IR 위치 축(`expected_utf16_pos`)은 건드리지 않고 **방출 XML 에만** 텍스트를 되돌린다.
/// 재적재하면 같은 정규화가 다시 지우므로 IR 은 저장→적재 고정점을 유지한다.
fn emit_guide_residue(splitter: &mut RunSplitter, para: &Paragraph, fr: &FieldRange, pos: u32) {
    // 값이 채워진 필드는 본문 run 이 이미 있다 — 중복 주입 금지.
    if fr.start_char_idx != fr.end_char_idx {
        return;
    }
    let Some(Control::Field(f)) = para.controls.get(fr.control_idx) else {
        return;
    };
    // 수정됨(bit 15) 표식이 선 필드는 초기 상태가 아니다 — 사용자가 비운 값을 되살리면 안 된다.
    if f.is_dirty() {
        return;
    }
    let Some(residue) = f.guide_residue.as_ref() else {
        return;
    };
    if residue.text.is_empty() {
        return;
    }
    // 잔재를 담던 run 의 경계까지만 먼저 끊는다 — 삭제 수술이 같은 위치에 접어 둔
    // zero-width run 들이 원본 서식(charPrIDRef)의 유일한 근거다.
    while splitter.needs_cut(pos) && splitter.current_shape_id() != residue.char_shape_id {
        splitter.cut_one();
    }
    // 안내문 잔재는 합성 텍스트라 탭 확장·제목 차례 표시가 없다 — 빈 커서로 낸다.
    let mut cursor = InlineCursor::default();
    splitter
        .content
        .push_str(&render_hp_t_content(&residue.text, &[], &mut cursor));
}

/// `pos` 위치에서 경계를 적용하고 `fieldEnd` 를 방출한다.
/// 0-length 필드면 그 직전에 안내문 잔재를 복원한다 (#3545).
fn emit_field_end_at(splitter: &mut RunSplitter, para: &Paragraph, fr: &FieldRange, pos: u32) {
    emit_guide_residue(splitter, para, fr, pos);
    splitter.cut_before(pos);
    emit_field_end(&mut splitter.content, para, fr);
}

/// 고아(다단락) fieldEnd 를 `<hp:ctrl><hp:fieldEnd .../></hp:ctrl>` 로 방출 (Task #1556).
fn emit_orphan_field_end(out: &mut String, ofe: &OrphanFieldEnd) {
    if let Ok(xml) = writer_to_string(|w| write_field_end_full(w, ofe.begin_id_ref, ofe.field_id)) {
        out.push_str("<hp:ctrl>");
        out.push_str(&xml);
        out.push_str("</hp:ctrl>");
    }
}

/// 문단 텍스트 전체를 char_shapes 경계로 분할하며 `splitter` 에 누적한다.
///
/// `char_offsets` 로 문자 idx → UTF-16 위치를 매핑하므로 IR 내 컨트롤(8 유닛 갭)이
/// 있어도 경계 위치가 어긋나지 않는다.
fn split_text_into(splitter: &mut RunSplitter, para: &Paragraph, cursor: &mut InlineCursor<'_>) {
    let mut text_buf = String::new();
    let mut running_pos = 0u32;
    for (idx, c) in para.text.chars().enumerate() {
        let char_pos = para.char_offsets.get(idx).copied().unwrap_or(running_pos);
        if splitter.needs_cut(char_pos) {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                cursor,
            );
            splitter.cut_before(char_pos);
        }
        text_buf.push(c);
        running_pos = char_pos
            .max(running_pos)
            .saturating_add(char_utf16_width(c));
    }
    flush_text_fragment(
        &mut splitter.content,
        &mut text_buf,
        &para.tab_extended,
        cursor,
    );
    // 마지막 문자 뒤에 남은 제목 차례 표시 — 빈 조각으로 한 번 더 낸다.
    flush_trailing_title_marks(&mut splitter.content, &para.tab_extended, cursor);
}

/// Paragraph 본문을 완전한 `<hp:run>` 시퀀스로 직렬화한다 (#1378 다중 run 분할).
///
/// 반환: (run 시퀀스 XML, **위치 축 보존 여부**).
///
/// [#4778] 두 번째 값이 `false` 면 방출된 텍스트 스트림이 원본 8유닛 슬롯 축과
/// 어긋난 상태다(파서 미수용 슬롯 또는 mismatch 폴백). 호출부는 이때 저장
/// lineseg 방출을 억제해야 한다 — textpos 사다리와 어긋난 lineseg 는 한글이
/// 그 문단부터 본문을 통째 폐기하는 트리거다.
fn render_runs(para: &Paragraph, ctx: &mut SerializeContext) -> (String, bool) {
    // ID 참조 무결성 (구현계획서 1.5): 실제 char_shapes entry 만 reference.
    // 빈 IR 의 fallback 0 은 제외 — char_shapes 미등록 문서(`Document::default()`)의
    // 직렬화를 깨지 않도록.
    for cs in &para.char_shapes {
        ctx.char_shape_ids.reference(cs.char_shape_id);
    }

    // [#1592] 완전 빈 문단(원본에 <hp:run> 없음)은 run 을 방출하지 않는다. char_shapes=[]
    // 인 문단에 RunSplitter 가 기본 (0,0) 세그먼트로 charPrIDRef="0" 빈 run 을 추가하면,
    // 재파싱 시 spurious (0,0) char_shape 가 생긴다(원본은 run 없어 char_shapes=[]).
    // char_shapes 가 있으면(예: [(0,0)] 명시) 종전대로 run 을 방출한다(linesegarray 는 별도).
    if para.text.is_empty()
        && para.char_shapes.is_empty()
        && para.controls.is_empty()
        && para.field_ranges.is_empty()
        && para.orphan_field_ends.is_empty()
        // 표시만 있고 텍스트가 없는 문단도 8유닛을 점유한다 — 여기서 빠지면 축이 밀린다.
        && para.title_marks.is_empty()
    {
        // 방출할 것이 없는 문단 — 옮길 슬롯도 없으므로 위치 축은 그대로다.
        return (String::new(), true);
    }

    let mut splitter = RunSplitter::new(para);

    let slot_count = inferred_control_slot_count(para);
    // [#4778] U+FFFC 마커는 컨트롤이 없어도 위치 축의 정규 시민이다(HWP3 암호 변환본:
    // 마커 리터럴이 그대로 방출돼 재파싱 고정점 유지 — #3739 --verify 계약). 억제는
    // **마커로 설명되지 않는 8유닛 구멍**(차례표지 0x0008 등 파서 미수용 슬롯)에만 건다.
    let marker_count = para.text.chars().filter(|c| *c == '\u{fffc}').count();
    // slots 와 각 slot 의 para.controls 인덱스(slot_ctrl_indices)를 병행 수집 —
    // [Task #1627] empty-text 문단의 bookmark in-order 방출에서 slot 사이 위치 계산에 사용.
    let (slots, slot_ctrl_indices): (Vec<&Control>, Vec<usize>) =
        if slot_count == para.controls.len() {
            // 전 컨트롤이 위치 슬롯인 경로. 본문 첫 문단의 첫 ColumnDef 도 슬롯으로 남겨
            // char-offset 정합을 보존하고, 그 XML 만 render_control_slot 의 consume-once
            // 플래그로 억제한다(템플릿이 이미 방출 — 중복 방지). 2번째+ 는 인라인 방출.
            (
                para.controls.iter().collect(),
                (0..para.controls.len()).collect(),
            )
        } else {
            // [Task #1379] 셀·글상자 subList(depth>0) 경로에서는 ColumnDef 도 인라인 슬롯으로
            // 취급한다 (원본 XML 에 <hp:ctrl><hp:colPr/></hp:ctrl> 인라인 존재).
            // [Task #1584] 본문(depth 0) 경로에서도 ColumnDef 를 인라인 슬롯에 포함하되,
            // 첫 문단의 첫 ColumnDef(섹션 템플릿 흡수분)는 슬롯에서 기본 제외한다.
            // 2번째+ 본문 ColumnDef 는 포함하여 드롭을 방지한다.
            let suppress_first_col = ctx.sub_list_depth == 0 && ctx.body_coldef_template_pending;
            let mut col_seen = 0u32;
            let mut s: Vec<&Control> = Vec::new();
            let mut si: Vec<usize> = Vec::new();
            // 제외된 hidden 슬롯 후보(SectionDef·템플릿 흡수 첫 ColumnDef)의 인덱스.
            let mut hidden: Vec<usize> = Vec::new();
            for (i, c) in para.controls.iter().enumerate() {
                let keep = if matches!(c, Control::ColumnDef(_)) {
                    col_seen += 1;
                    if suppress_first_col && col_seen == 1 {
                        hidden.push(i);
                        false
                    } else {
                        true
                    }
                } else if matches!(c, Control::SectionDef(_)) {
                    hidden.push(i);
                    false
                } else {
                    // [#4677] 위치 축은 책갈피까지 포함한다(`occupies_hwpx_slot_axis`).
                    let slot = occupies_hwpx_slot_axis(c);
                    // [#4388] Unknown 은 슬롯이 아니므로 이 실제 배제 지점에서 직접
                    // 경고한다. HWP3 Hyperlink은 occupies_hwpx_slot_axis가 HWPX field로
                    // 승격해 보존한다. HiddenComment 등 다른 non-slot 컨트롤은 이 헬퍼가
                    // 내부적으로 무시한다.
                    if !slot {
                        warn_if_unrepresentable_in_hwpx(c);
                    }
                    slot
                };
                if keep {
                    s.push(c);
                    si.push(i);
                }
            }
            // [Task #1591 v2] hidden 슬롯 정합: cc 증거(slot_count)가 hidden 후보
            // (secPr/템플릿 흡수 colPr — 원본 XML 에서 첫 run 이 점유하는 8유닛 슬롯)의
            // 점유를 보여주면 위치 슬롯으로 편입한다. XML 은 방출되지 않고
            // (SectionDef: 방출 arm 없음, 첫 ColumnDef: consume-once 억제) 위치 축만
            // 8유닛씩 전진 — 첫 문단이 mismatch 폴백 대신 메인 경로(위치 정확)로
            // 진입해 후위 슬롯(pageNum 등)·fieldEnd 가 char-offset 위치에 방출된다
            // (Class C1 +8 시프트·C2 fieldEnd 드롭의 근원 교정). 증거 불일치(합성 IR,
            // hidden 이 cc 를 점유하지 않는 문서)는 종전 동작 그대로.
            if !hidden.is_empty() && slot_count == s.len() + hidden.len() {
                let mut s2: Vec<&Control> = Vec::new();
                let mut si2: Vec<usize> = Vec::new();
                let mut keep_iter = si.iter().peekable();
                for (i, c) in para.controls.iter().enumerate() {
                    let kept = keep_iter.peek() == Some(&&i);
                    if kept {
                        keep_iter.next();
                    }
                    if kept || hidden.contains(&i) {
                        s2.push(c);
                        si2.push(i);
                    }
                }
                // 첫 ColumnDef 를 슬롯으로 되살렸으므로 consume-once 플래그를 유지해
                // render_control_slot 이 XML 방출만 1회 건너뛰게 한다(첫 분기와 동형).
                (s2, si2)
            } else {
                if suppress_first_col {
                    // 템플릿 흡수분을 슬롯에서 제외한 채 진행하므로, render_control_slot 의
                    // consume-once 억제가 2번째 ColumnDef 를 잘못 건너뛰지 않도록 플래그 해제.
                    ctx.body_coldef_template_pending = false;
                }
                (s, si)
            }
        };

    // [#4677] 책갈피는 이제 위치 슬롯이라(`occupies_hwpx_slot_axis`) 슬롯 루프가 제자리에
    // 방출한다. 종전의 별도 방출(문단 시작 hoist / slot 사이 in-order, Task #1591·#1627)은
    // 슬롯 축이 책갈피를 잡지 못하던 시절의 우회였고, 그대로 두면 이중 방출이 된다.
    let mut cursor = InlineCursor {
        title_marks: &para.title_marks,
        ..Default::default()
    };

    // fast path: 슬롯·필드·고아 fieldEnd·경계 없음 — 텍스트 전체를 단일 run 으로
    if slots.is_empty()
        && para.field_ranges.is_empty()
        && para.orphan_field_ends.is_empty()
        && splitter.single_run()
    {
        let t = render_hp_t_content(&para.text, &para.tab_extended, &mut cursor);
        splitter.content.push_str(&t);
        flush_trailing_title_marks(&mut splitter.content, &para.tab_extended, &mut cursor);
        // 슬롯이 하나도 없는데 char_count 는 슬롯을 주장하면(파서가 못 읽은 컨트롤 —
        // 예: 차례표지 0x0008) 방출 축이 원본보다 짧다 — 이때 lineseg 를 그대로 쓰면
        // 한글이 그 문단부터 본문을 폐기한다(#4778). 축 붕괴를 호출부에 알린다.
        // U+FFFC 마커가 슬롯 주장을 전부 설명하면 종전 계약 유지(#3739).
        return (
            splitter.finish(),
            slot_count == 0 || marker_count >= slot_count,
        );
    }

    // mismatch 경로: 슬롯 위치 추정 불가 — 텍스트(경계 분할 포함) 후 슬롯 일괄 방출
    if slot_count != slots.len() {
        split_text_into(&mut splitter, para, &mut cursor);
        for slot in slots.iter() {
            render_control_slot(&mut splitter.content, slot, ctx);
        }
        // [Task #1591 v2] 균형 field_ranges 의 닫는 fieldEnd 도 말미 복원 — 이 경로는
        // fieldBegin(슬롯)만 방출하고 fieldEnd 방출 코드가 없어 same-para 균형 필드가
        // 1/0 으로 깨지며 cc −8 (#1593). #1556 고아 복원과 동형(위치 대신 말미 일괄).
        for fr in &para.field_ranges {
            emit_field_end(&mut splitter.content, para, fr);
        }
        // [Task #1556] 위치 추정 불가 경로에서도 고아 fieldEnd 의 8유닛 슬롯은 복원한다
        // (정확한 위치 대신 말미 일괄 — 최소한 char_count 보존).
        for ofe in &para.orphan_field_ends {
            emit_orphan_field_end(&mut splitter.content, ofe);
        }
        // 컨트롤을 말미로 몰았으므로 원본 lineseg 좌표계는 더 이상 유효하지 않다 —
        // 호출부가 저장 lineseg 방출을 억제하게 한다(#4778). 단, 슬롯 부족분이
        // U+FFFC 마커로 전부 설명되면 종전 계약(방출 유지, #3739 고정점)을 지킨다.
        let shortfall = slot_count.saturating_sub(slots.len());
        return (splitter.finish(), marker_count >= shortfall);
    }

    // 슬롯으로 세어 놓고 XML 은 내지 않는 컨트롤이 섞여 있으면, 방출 축은 그만큼 짧다.
    // 위치는 여전히 최선을 다해 맞추되 **축 정합을 주장하지는 않는다** — lineseg 를 그대로
    // 쓰면 한글이 범위 밖 textpos 를 만나 파일을 아예 열지 못한다.
    let axis_faithful = slots.iter().all(|c| emits_hwpx_slot_xml(c));

    // 메인 경로 — UTF-16 위치 축 위에서 슬롯/필드/문자/경계를 함께 처리
    let mut text_buf = String::new();
    let mut slot_idx = 0usize;
    let mut expected_utf16_pos = 0u32;
    let mut field_end_emitted = vec![false; para.field_ranges.len()];
    // [Task #1556] 고아 fieldEnd 방출 추적.
    let mut orphan_emitted = vec![false; para.orphan_field_ends.len()];
    let text_char_count = para.text.chars().count();

    // 빈 문단(text == "")의 0-length 필드: 메인 루프가 실행되지 않아
    // pre-char 검사를 통과하지 못한다. 따라서 슬롯마다 `inner_slot_count`를
    // 확인해 fieldEnd를 제자리에 방출한다. 모든 슬롯 뒤에 fieldEnd를 몰면
    // 연속 또는 중첩 필드의 닫는 순서가 뒤바뀐다.
    if para.text.is_empty() {
        while slot_idx < slots.len() {
            splitter.cut_before(expected_utf16_pos);
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            let emitted_ctrl_idx = slot_ctrl_indices[slot_idx];
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            for (i, fr) in para.field_ranges.iter().enumerate() {
                if !field_end_emitted[i]
                    && fr.start_char_idx == fr.end_char_idx
                    && fr.control_idx + fr.inner_slot_count == emitted_ctrl_idx
                {
                    emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
                    expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                    field_end_emitted[i] = true;
                }
            }
        }
        // [Task #1556] 빈 문단의 고아 fieldEnd (char_idx == 0).
        for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
            if ofe.char_idx == 0 && !orphan_emitted[i] {
                splitter.cut_before(expected_utf16_pos);
                emit_orphan_field_end(&mut splitter.content, ofe);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                orphan_emitted[i] = true;
            }
        }
    }

    for (idx, c) in para.text.chars().enumerate() {
        let char_pos = para
            .char_offsets
            .get(idx)
            .copied()
            .unwrap_or(expected_utf16_pos);
        // [#1407] 이 idx 위치에서 닫혀야 할(미방출) fieldEnd 가 있으면, 그 8유닛 갭은
        // 슬롯이 아니라 fieldEnd 소유다. 슬롯 방출을 양보해 텍스트-끝 슬롯(newNum 등)이
        // fieldEnd 자리를 가로채지 못하게 한다 (0-length 필드는 아래 pre-char 경로가 처리).
        while slot_idx < slots.len() && char_pos >= expected_utf16_pos.saturating_add(8) {
            // [Issue #1948] 이 갭(expected_utf16_pos)이 미방출 **고아(교차 문단)
            // fieldEnd** 소유면 슬롯 방출을 양보한다. 종전엔 field_ranges 의 fieldEnd
            // 만 갭을 지켰고(위 주석 #1407) 고아 fieldEnd 는 while 뒤(char_idx==idx)에서
            // 방출돼, 말미 슬롯(표 등)이 고아 fieldEnd 의 8유닛 갭을 먼저 가로채
            // char_offsets 가 +8 밀렸다(36380743 문단 0.10: 표가 fieldEnd 자리로 당겨짐).
            // 갭 소유자인 고아 fieldEnd 를 먼저 방출하고 while 을 재평가한다.
            if let Some(oi) = para
                .orphan_field_ends
                .iter()
                .enumerate()
                .position(|(i, ofe)| ofe.char_idx == idx && !orphan_emitted[i])
            {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut cursor,
                );
                splitter.cut_before(expected_utf16_pos);
                emit_orphan_field_end(&mut splitter.content, &para.orphan_field_ends[oi]);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                orphan_emitted[oi] = true;
                continue;
            }
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut cursor,
            );
            // 슬롯 시작 위치의 경계 — 슬롯은 새 run 소속 (규칙 1)
            splitter.cut_before(expected_utf16_pos);
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            let emitted_ctrl_idx = slot_ctrl_indices[slot_idx];
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            // [Task #1893] 이 슬롯이 0-length 필드(start==end==idx)의 fieldBegin 이면
            // 그 fieldEnd 를 즉시 이어서 방출 — 같은 갭의 end 몫 8유닛을 다음 슬롯이
            // 가로채 begin 들이 연속 배치되면 재파스 LIFO 페어링이 교차된다
            // (fr(0,0)+(50,50) → fr(0,50)+(0,0), 빈 누름틀 placeholder 소실/줄바꿈 분기).
            // 필드가 표·그림을 감쌌으면 안쪽 슬롯을 지나서 닫는다 — 자기 슬롯 직후에
            // 닫으면 개체가 필드 밖으로 밀려 빈 누름틀이 되고, 한글이 안내문을 본문에
            // 찍는다(G-순수증식). `inner_slot_count` 가 0 이면 종전과 동일하다.
            for (i, fr) in para.field_ranges.iter().enumerate() {
                if !field_end_emitted[i]
                    && fr.start_char_idx == fr.end_char_idx
                    && fr.end_char_idx == idx
                    && fr.control_idx + fr.inner_slot_count == emitted_ctrl_idx
                {
                    emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
                    expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                    field_end_emitted[i] = true;
                }
            }
        }

        // [Task #1556] 고아 fieldEnd (char_idx == idx): 문자 push 전에 8유닛 슬롯 방출.
        for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
            if ofe.char_idx == idx && !orphan_emitted[i] {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut cursor,
                );
                splitter.cut_before(expected_utf16_pos);
                emit_orphan_field_end(&mut splitter.content, ofe);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                orphan_emitted[i] = true;
            }
        }

        // HWP3 원본은 개체가 있는 위치를 U+FFFC 하나로 `text`에도 남기면서,
        // char_offsets에서는 그 한 글자를 HWP5와 같은 8 UTF-16 단위 슬롯으로 센다.
        // HWPX에서는 개체 태그 자체가 그 슬롯이므로 U+FFFC를 <hp:t>로 다시 쓰면
        // 문자 1단위가 중복되고, 다음 문자까지는 슬롯이 감지되지 않아 제어가 문단
        // 끝으로 밀린다. 현재 위치가 슬롯의 시작이고 소비할 control이 있을 때만
        // 표시 문자를 그 control의 HWPX 표현으로 직접 바꾼다. 실제 리터럴 U+FFFC나
        // 위치가 불명확한 합성 IR은 기존 텍스트 경로를 유지한다.
        if c == '\u{fffc}' && slot_idx < slots.len() && char_pos == expected_utf16_pos {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut cursor,
            );
            splitter.cut_before(expected_utf16_pos);
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            continue;
        }

        // 0-length 필드(start == end == idx): fieldBegin 방출 직후, 문자 push 전에 fieldEnd 방출.
        // post-char 검사(next_idx 기준)는 end-1 번째 문자 처리 후 방출하므로 0-length 필드에서
        // fieldEnd가 fieldBegin 앞에 나오거나 텍스트 뒤로 밀리는 문제가 생긴다.
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.start_char_idx == fr.end_char_idx
                && fr.end_char_idx == idx
                && !field_end_emitted[i]
            {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut cursor,
                );
                emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
                field_end_emitted[i] = true;
            }
        }

        // AutoNumber 슬롯 (#1382): placeholder 공백이 슬롯 8유닛의 첫 유닛을 점유
        // (HWP5/HWPX 파서 공통 규약 — placeholder at P, 다음 문자 P+8). 슬롯 위치
        // (char_pos == expected)의 placeholder 에서 ctrl 을 방출하고, placeholder 는
        // 파서가 IR 에 합성한 문자이므로 텍스트로 내보내지 않는다 (한컴 원본 XML 동형:
        // `<hp:ctrl><hp:autoNum/></hp:ctrl>` 뒤에 실제 텍스트만 이어진다).
        // placeholder 판별: 일반 공백과 구분하기 위해 직후 offset 의 +8 jump 를 본다
        // (마지막 문자면 jump 가 char_offsets 에 없으므로 placeholder 로 간주).
        let is_autonum_placeholder = c == ' '
            && char_pos == expected_utf16_pos
            && para
                .char_offsets
                .get(idx + 1)
                .copied()
                .unwrap_or_else(|| char_pos.saturating_add(8))
                >= char_pos.saturating_add(8);
        if slot_idx < slots.len()
            && matches!(slots[slot_idx], Control::AutoNumber(_))
            && is_autonum_placeholder
        {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut cursor,
            );
            splitter.cut_before(expected_utf16_pos);
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            continue;
        }

        // 문자 위치의 경계 — 문자는 새 run 소속 (규칙 1)
        if splitter.needs_cut(char_pos) {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut cursor,
            );
            splitter.cut_before(char_pos);
        }

        text_buf.push(c);
        let width = char_utf16_width(c);
        if char_pos >= expected_utf16_pos {
            expected_utf16_pos = char_pos.saturating_add(width);
        } else {
            expected_utf16_pos = expected_utf16_pos.saturating_add(width);
        }

        // end_char_idx는 미포함(exclusive): 현재 문자가 필드 범위의 마지막이면 fieldEnd 삽입.
        // 0-length 필드(start == end)는 위의 pre-char 검사에서 처리하므로 제외한다.
        let next_idx = idx + 1;
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.end_char_idx == next_idx
                && !field_end_emitted[i]
                && fr.start_char_idx < fr.end_char_idx
            {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut cursor,
                );
                emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
                // [#1407] fieldEnd 는 8유닛 슬롯을 소비한다. expected 를 +8 진행하지
                // 않으면 다음 idx 에서 텍스트-끝 슬롯(newNum 등)이 이 8유닛 갭을
                // 가로채 텍스트가 +8 밀린다 (143E 문단 0.14: char_offsets[3] 27→35).
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                field_end_emitted[i] = true;
            }
        }
    }

    flush_text_fragment(
        &mut splitter.content,
        &mut text_buf,
        &para.tab_extended,
        &mut cursor,
    );
    flush_trailing_title_marks(&mut splitter.content, &para.tab_extended, &mut cursor);

    // end_char_idx >= text.len() 인 경우 루프에서 감지되지 않으므로 루프 후에 처리.
    // [Task #1893] 단, 문단 끝의 0-length 필드(start == end == text.len())는 자기
    // fieldBegin 슬롯이 아직 아래 잔여 슬롯 루프에 남아 있다 — 여기서 먼저 방출하면
    // fieldEnd 가 fieldBegin 앞에 놓여 재파스가 고아 end + 미닫힘 begin 으로 해석,
    // field_range 가 소실된다(빈 누름틀 안내문 placeholder 미렌더 → 라운드트립 렌더
    // 분기). begin 슬롯 방출 직후로 지연한다(중간 위치 0-length 는 pre-char 경로가
    // 동일 규칙으로 처리 — #1407).
    for (i, fr) in para.field_ranges.iter().enumerate() {
        if !field_end_emitted[i] {
            let begin_slot_pending = fr.start_char_idx == fr.end_char_idx
                && slot_ctrl_indices[slot_idx..].contains(&fr.control_idx);
            if begin_slot_pending {
                continue;
            }
            emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            field_end_emitted[i] = true;
        }
    }

    // [Task #1556] 텍스트 끝(char_idx == text_char_count) 의 고아 fieldEnd — para 0.16 케이스.
    for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
        if !orphan_emitted[i] {
            debug_assert!(
                ofe.char_idx >= text_char_count,
                "미방출 고아 fieldEnd 는 텍스트 끝이어야 함"
            );
            splitter.cut_before(expected_utf16_pos);
            emit_orphan_field_end(&mut splitter.content, ofe);
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            orphan_emitted[i] = true;
        }
    }

    while slot_idx < slots.len() {
        splitter.cut_before(expected_utf16_pos);
        render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
        let emitted_ctrl_idx = slot_ctrl_indices[slot_idx];
        slot_idx += 1;
        expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        // [Task #1893] 위에서 지연한 문단 끝 0-length 필드의 fieldEnd 를 자기
        // fieldBegin 슬롯 **뒤**에 방출 — begin→end 순서 보존.
        //
        // 필드가 표·그림을 감싸면 텍스트 축은 0길이지만 안쪽에 컨트롤 슬롯이 있다.
        // `inner_slot_count` 만큼 지나서 닫아야 개체가 필드 안에 남는다. 자기 슬롯
        // 직후에 닫으면 개체가 밖으로 밀려 빈 누름틀이 되고, 한글이 안내문을 본문에
        // 찍는다(G-순수증식 16경로).
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if !field_end_emitted[i]
                && fr.start_char_idx == fr.end_char_idx
                && fr.control_idx + fr.inner_slot_count == emitted_ctrl_idx
            {
                emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                field_end_emitted[i] = true;
            }
        }
    }
    // [Task #1893] 방어: 지연분이 슬롯 루프에서 매칭되지 못했으면 말미에 방출(종전 동작).
    for (i, fr) in para.field_ranges.iter().enumerate() {
        if !field_end_emitted[i] {
            emit_field_end_at(&mut splitter, para, fr, expected_utf16_pos);
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            field_end_emitted[i] = true;
        }
    }
    (splitter.finish(), axis_faithful)
}

fn inferred_control_slot_count(para: &Paragraph) -> usize {
    // [#1382] AutoNumber 는 placeholder 공백이 슬롯 8유닛의 첫 유닛을 점유한다
    // (HWP5 body_text / HWPX offsets 조립 공통 규약). placeholder 가 텍스트 폭에
    // 이미 1유닛으로 집계되므로 두 축 모두에서 autoNum 1개당 잉여가 8이 아닌 7로
    // 측정된다 → autoNum 수만큼 더해 보정한 뒤 8로 나눈다. placeholder 없는
    // 합성 IR(편집기 생성)은 잉여 0이라 보정해도 0 — 기존 mismatch 경로 유지.
    let autonum_count = para
        .controls
        .iter()
        .filter(|c| matches!(c, Control::AutoNumber(_)))
        .count() as u32;

    let text_units: u32 = para.text.chars().map(char_utf16_width).sum();
    // 암호 HWP3 parser는 개체 자리에 U+FFFC 하나를 남기되, 실제 offset은 HWP5와
    // 같은 8단위로 전진시킨다. 이 표식은 HWPX에서 개체 슬롯으로 치환되므로, 1단위
    // 텍스트로만 세면 `(char_count - text_units) / 8`가 0으로 내림되어 mismatch
    // fallback(텍스트 뒤에 control 몰아쓰기)으로 빠진다. control이 없는 리터럴 U+FFFC는
    // 제외하기 위해 가능한 control 수까지만 슬롯 증거로 반영한다.
    let hwp3_object_marker_slots = para
        .text
        .chars()
        .filter(|c| *c == '\u{fffc}')
        .count()
        .min(para.controls.len()) as u32;
    let from_char_count = para
        .char_count
        .saturating_sub(1)
        // U+FFFC 자체 1단위를 빼면 실제 8단위 control 슬롯만 남는다.
        .saturating_sub(text_units.saturating_sub(hwp3_object_marker_slots))
        .saturating_add(autonum_count)
        / 8;

    let mut offsets_gap = 0u32;
    let mut expected = 0u32;
    for (idx, c) in para.text.chars().enumerate() {
        let pos = para.char_offsets.get(idx).copied().unwrap_or(expected);
        if pos > expected {
            offsets_gap += pos - expected;
        }
        expected = pos.max(expected).saturating_add(char_utf16_width(c));
    }
    // marker 다음 offset gap은 8이 아니라 7(표식 텍스트 1단위를 이미 센 뒤)이므로,
    // 나누기 전에 marker 수를 더해 실제 8단위 슬롯 수로 복원한다.
    let from_offsets = offsets_gap
        .saturating_add(autonum_count)
        .saturating_add(hwp3_object_marker_slots)
        / 8;

    // fieldEnd는 8 code unit 슬롯이지만 para.controls[]에 대응 컨트롤이 없다.
    // field_ranges.len()이 fieldEnd 수와 정확히 일치하므로 빼서 보정한다.
    // [Task #1556] 고아(다단락) fieldEnd 도 컨트롤 없는 8유닛 슬롯이므로 동일하게 차감.
    // 제목 차례 표시(`Mtit`/`Mign`)도 CTRL_HEADER 가 없는 8유닛 슬롯이라 같은 계열이다 —
    // 차감하지 않으면 슬롯 수가 `controls.len()` 보다 커져 문단이 mismatch 경로로 떨어지고,
    // 그 문단의 linesegarray 가 통째로 빠진다.
    from_char_count
        .max(from_offsets)
        .saturating_sub(para.field_ranges.len() as u32)
        .saturating_sub(para.orphan_field_ends.len() as u32)
        .saturating_sub(para.title_marks.len() as u32) as usize
}

/// HWPX 인라인 슬롯(U+FFFC 오브젝트 위치)을 점유하는 컨트롤인지 판정한다.
///
/// 이 목록은 두 곳에서 쓰인다: `render_runs`(이 파일, mismatch-분기 위치 축)와
/// `roundtrip::diff_documents`(포맷 비종속 IR 비교 — `export-hwpx` **뿐 아니라**
/// `convert`(HWP5 대상) `--verify` 도 재사용한다).
///
/// **[#4388] `Control::Hyperlink`/`Control::Unknown` 은 의도적으로 여기 등록하지
/// 않는다.** 둘 다 HWP3/HWP5 PARA_TEXT 상에서는 Picture/Table 과 동형인 U+FFFC
/// 오브젝트 슬롯(위치 점유)이라 처음엔 등록했으나, 등록하면 HWP5 직렬화기가 이미
/// (이슈 범위 밖으로) Hyperlink 의 ctrl_id 를 0 으로 고정해 버려 CTRL_HEADER 자체를
/// 안 쓰는 기존 HWP5 경로 손실까지 `diff_documents` 가 새로 "검출"해,
/// `bookmarks_survive_saving_to_hwp5`(tests/issue_hwp3_bookmark_native.rs) 같이
/// 하이퍼링크와 무관한 기존 회귀 테스트가 hwp3-sample16.hwp 에서 깨졌다(#4388 후속
/// 보고, 실측 재현: s0 문단 27/32/196/657 이 전부 Hyperlink). "조용히 버리지
/// 말라"는 경고 축이지 비교 축이 아니다 — 경고는 `render_control_slot` catch-all과
/// `render_runs` 의 mismatch-분기 제외 지점(`warn_if_unrepresentable_in_hwpx` 호출)
/// 두 곳에서 낸다. 회귀 가드: `roundtrip.rs` 의
/// `issue4388_diff_documents_hyperlink_not_compared_as_control`/
/// `issue4388_diff_documents_unknown_not_compared_as_control`.
/// [#4677] `render_runs` 의 **위치 축** 전용 슬롯 판정 — 책갈피를 포함한다.
///
/// 책갈피는 HWP5 PARA_TEXT 에서 8 유닛 확장 제어문자 자리를 차지한다(한컴 원본 바이트
/// `16 00 6d 6b 6f 62 … 16 00`). 종전엔 zero-width 로 보고 슬롯 축에서 빼는 대신
/// `emit_inorder_bookmarks` 로 따로 끼워 넣었는데, 그러면 HWPX 파서가 자리를 잡는 순간
/// 슬롯 수가 어긋나 위치 추정이 통째로 무너진다(x2x 산출물 6,995자 → 416자).
///
/// [`is_hwpx_inline_slot`] 자체를 넓히지 않는 이유는 그 헬퍼가 `roundtrip::diff_documents`
/// 의 **비교 축**과 공유되기 때문이다 — 비교 축 변경은 #4388 처럼 무관한 회귀를 만든다.
/// 여기서 필요한 것은 위치 축 하나뿐이다.
fn occupies_hwpx_slot_axis(control: &Control) -> bool {
    is_hwpx_inline_slot(control)
        || matches!(
            control,
            // Hyperlink은 HWP3의 8-unit object slot이고 HWPX에서는 fieldBegin으로 승격한다.
            // `is_hwpx_inline_slot`에는 넣지 않는다. 해당 헬퍼는 포맷 비종속 diff에도 공유되어
            // HWP5의 기존 Hyperlink 표현 범위까지 바꾸기 때문이다.
            // 숨은 설명은 HWPX 네이티브 표현(`<hp:hiddenComment>`)이 있고 방출 arm 도
            // 있으나, 이 필터에서 빠지면 mismatch 경로에서 **조용히 버려진다**.
            // HWP3 는 char_count 에 8유닛 슬롯을 배정하지 않아(06397 문단 0.0:
            // `cc=2, text_len=1, controls=3`) 늘 mismatch 경로로 오므로, 여기 없으면
            // HWP3 문서의 숨은 설명이 통째로 사라진다(06397 유지율 2.2%).
            Control::Bookmark(_)
                | Control::Hyperlink(_)
                | Control::HiddenComment(_)
                | Control::IndexMark(_)
        )
}

/// 이 컨트롤이 슬롯 자리에 **실제 XML 을 남기는가**.
///
/// `render_control_slot` 은 표현할 방법이 없는 컨트롤(`Unknown`)을 경고만
/// 내고 버린다. 그런데 위치 슬롯 수가 `controls.len()` 과 같으면 축 정합 경로로 들어가
/// **버려진 컨트롤도 슬롯으로 세어 놓고** `axis_faithful=true` 를 주장한다. 그러면 방출본은
/// 컨트롤 하나당 8유닛씩 짧은데 lineseg 는 원본 좌표 그대로 나가고, 한글 2022 는 범위를
/// 넘는 `textpos` 를 만나면 **파일 자체를 열지 못한다**(본문 폐기보다 강한 실패).
///
/// 실측(06926 section3 문단 347): 찾아보기 표식(`idxm`) 3개가 여기서 사라져 축이 366→342 로
/// 줄었고, 원본에서 유효하던 `textpos=348` 이 범위 밖이 되어 산출물이 열리지 않았다.
fn emits_hwpx_slot_xml(control: &Control) -> bool {
    // `Hyperlink` 는 devel 이 방출 arm 을 갖췄으므로(`render_control_slot`) 여기서
    // 빼면 안 된다 — 빼면 HWP3 변환본의 축이 근거 없이 무너진 것으로 판정돼
    // 저장 lineseg 가 통째로 억제된다(#3739 암호 변환본 계약 파손).
    !matches!(control, Control::Unknown(_))
}

pub(crate) fn is_hwpx_inline_slot(control: &Control) -> bool {
    matches!(
        control,
        Control::Table(_)
            | Control::Shape(_)
            | Control::Picture(_)
            | Control::CharOverlap(_)
            | Control::Ruby(_)
            | Control::Equation(_)
            | Control::Field(_)
            | Control::Form(_)
            | Control::Footnote(_)
            | Control::Endnote(_)
            | Control::PageHide(_)
            | Control::PageNumberPos(_)
            | Control::NewNumber(_)
            | Control::Header(_)
            | Control::Footer(_)
            | Control::AutoNumber(_)
            | Control::PageNumCtrl(_)
    )
}

fn flush_text_fragment(
    out: &mut String,
    text_buf: &mut String,
    tab_extended: &[[u16; 7]],
    cursor: &mut InlineCursor<'_>,
) {
    if !text_buf.is_empty() {
        out.push_str(&render_hp_t_content(text_buf, tab_extended, cursor));
        text_buf.clear();
    }
}

/// 문단 끝에 남은 제목 차례 표시를 빈 `<hp:t>` 조각으로 방출한다.
///
/// 조각 경계에서 flush 를 건너뛰는 이유는 표시가 **다음** run 소속이기 때문인데,
/// 다음 run 이 없는 문단 말미에서는 그 규칙이 그대로 유실이 된다.
fn flush_trailing_title_marks(
    out: &mut String,
    tab_extended: &[[u16; 7]],
    cursor: &mut InlineCursor<'_>,
) {
    if cursor.mark_idx < cursor.title_marks.len() {
        out.push_str(&render_hp_t_content("", tab_extended, cursor));
    }
}

fn render_control_slot(out: &mut String, control: &Control, ctx: &mut SerializeContext) {
    match control {
        // [#4677] 책갈피는 위치 슬롯이다 — 슬롯 축에 들어온 이상 여기서 제자리에 방출한다
        // (종전 `emit_inorder_bookmarks` 의 역할을 대체).
        Control::Bookmark(bm) => match writer_to_string(|w| write_bookmark(w, bm)) {
            Ok(xml) => {
                out.push_str("<hp:ctrl>");
                out.push_str(&xml);
                out.push_str("</hp:ctrl>");
            }
            Err(e) => eprintln!("[hwpx] Bookmark 직렬화 실패: {e}"),
        },
        Control::Hyperlink(link) => {
            let field_id = ctx.next_generated_hyperlink_id();
            match writer_to_string(|w| write_hyperlink_begin(w, link, field_id)) {
                Ok(xml) => {
                    out.push_str("<hp:ctrl>");
                    out.push_str(&xml);
                    out.push_str("</hp:ctrl>");
                }
                Err(e) => eprintln!("[hwpx] Hyperlink 직렬화 실패: {e}"),
            }
        }
        // 찾아보기 표식 — 한컴 실측(06926, 23건)은 `secondKey` 가 비면 아예 쓰지 않는다.
        Control::IndexMark(im) => {
            out.push_str("<hp:ctrl><hp:indexmark><hp:firstKey>");
            out.push_str(&xml_escape(&im.first_key));
            out.push_str("</hp:firstKey>");
            if !im.second_key.is_empty() {
                out.push_str("<hp:secondKey>");
                out.push_str(&xml_escape(&im.second_key));
                out.push_str("</hp:secondKey>");
            }
            out.push_str("</hp:indexmark></hp:ctrl>");
        }
        Control::Equation(eq) => {
            out.push_str(&render_equation(eq));
        }
        Control::Table(tbl) => match writer_to_string(|w| table::write_table(w, tbl, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Table 직렬화 실패: {e}"),
        },
        Control::Picture(pic) => match writer_to_string(|w| picture::write_picture(w, pic, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Picture 직렬화 실패: {e}"),
        },
        Control::Shape(shape) => {
            out.push_str(&render_shape(shape, ctx));
        }
        Control::Footnote(note) => {
            out.push_str(&render_footnote(note, ctx));
        }
        Control::Endnote(note) => {
            out.push_str(&render_endnote(note, ctx));
        }
        Control::Field(f) => {
            // fieldBegin은 <hp:ctrl>...</hp:ctrl>로 감싸야 함 (Table/Picture와 달리)
            out.push_str("<hp:ctrl>");
            let generated_params = generated_field_parameters(f);
            let has_params = f.raw_parameters_xml.is_some() || generated_params.is_some();
            let has_memo = f.field_type == crate::model::control::FieldType::Memo
                && !f.memo_paragraphs.is_empty();
            if has_params || has_memo {
                // [#1391] 자식(parameters / memo subList)이 있으면 start/end 태그.
                out.push_str(&super::field::field_begin_open_tag(f));
                out.push('>');
                if let Some(params) = f
                    .raw_parameters_xml
                    .as_deref()
                    .or(generated_params.as_deref())
                {
                    out.push_str(params);
                }
                if has_memo {
                    out.push_str(&render_sub_list_open(f.memo_text_direction.as_deref()));
                    let mut vert_cursor: u32 = 0;
                    for para in &f.memo_paragraphs {
                        ctx.para_shape_ids.reference(para.para_shape_id);
                        let sid = ctx.effective_style_id(para.style_id);
                        ctx.style_ids.reference(sid as u16);
                        let (runs, linesegs, advance) =
                            render_paragraph_parts(para, vert_cursor, ctx);
                        vert_cursor = advance;
                        let pid = ctx.next_para_id();
                        out.push_str(&render_hp_p_open(para, pid, sid));
                        out.push_str(&runs);
                        out.push_str(&linesegs);
                        out.push_str("</hp:p>");
                    }
                    out.push_str("</hp:subList>");
                }
                out.push_str("</hp:fieldBegin>");
            } else {
                match writer_to_string(|w| write_field_begin(w, f)) {
                    Ok(xml) => out.push_str(&xml),
                    Err(e) => eprintln!("[hwpx] Field 직렬화 실패: {e}"),
                }
            }
            out.push_str("</hp:ctrl>");
        }
        Control::PageHide(ph) => out.push_str(&render_page_hiding(ph)),
        Control::PageNumberPos(pn) => out.push_str(&render_page_num(pn)),
        Control::PageNumCtrl(pnc) => out.push_str(&format!(
            r#"<hp:ctrl><hp:pageNumCtrl pageStartsOn="{}"/></hp:ctrl>"#,
            pnc.page_starts_on.as_hwpx()
        )),
        Control::NewNumber(nn) => out.push_str(&render_new_num(nn)),
        Control::Header(h) => out.push_str(&render_header(h, ctx)),
        Control::Footer(f) => out.push_str(&render_footer(f, ctx)),
        Control::AutoNumber(an) => out.push_str(&render_autonum(an)),
        Control::Form(form) => match writer_to_string(|w| super::form::write_form(w, form)) {
            // 폼은 <hp:run> 직접 자식 (Table/Picture와 동일, <hp:ctrl> 비포장)
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Form 직렬화 실패: {e}"),
        },
        Control::CharOverlap(co) => out.push_str(&render_compose(co)),
        // [Task #1587] 덧말(Ruby) 인라인 방출. is_hwpx_inline_slot 에 등록돼 슬롯 위치는
        // 자동이나 종전 방출 arm 부재로 드롭됐다. parse_dutmal 의 역매핑.
        Control::Ruby(r) => out.push_str(&render_dutmal(r)),
        // 숨은 설명 — 화면에 안 보여도 파일에는 문단 리스트로 존재한다. 방출 arm 이
        // 없던 탓에 저장할 때마다 통째로 사라졌다(자세한 근거는 아래 catch-all 주석).
        Control::HiddenComment(hc) => out.push_str(&render_hidden_comment(hc, ctx)),
        // [Task #1379/#1584] 인라인 colPr 방출.
        // - subList(depth>0): 전부 인라인 방출(원본 XML 인라인 존재).
        // - 본문(depth 0): 첫 문단의 첫 ColumnDef 1개는 섹션 템플릿 colPr 앵커가 이미
        //   방출했으므로(중복 방지) consume-once 플래그로 XML 만 건너뛴다. 슬롯 자체는
        //   상위(render_runs)에서 유지하므로 char-offset 위치 정합은 보존된다.
        Control::ColumnDef(cd) => {
            if ctx.sub_list_depth == 0 && ctx.body_coldef_template_pending {
                ctx.body_coldef_template_pending = false;
            } else {
                out.push_str(&render_col_pr_ctrl(cd));
            }
        }
        // [#4388] Unknown은 HWPX 로 옮길 대응 표현이 없어 여기서도 드롭된다.
        // Hyperlink은 위 arm에서 HWPX fieldBegin으로 승격한다. Unknown 경고는
        // `warn_if_unrepresentable_in_hwpx` 하나로 통일한다(mismatch-분기 제외 지점,
        // 위쪽 934행 부근과 동일 호출). 이 함수 아래 catch-all 이 처리한다.
        //
        // [#4388 census] catch-all `_`(warn 호출 포함)에 실제로 도달하는 나머지
        // `Control` 변형은 세 가지 — 새 변형을 추가할 때 이 목록을 갱신할 것:
        //   - `Unknown`: 위 참조. 조용히 버리지 않도록 경고.
        //   - `SectionDef`: 의도적 무해 no-op. 위치 슬롯 축(hidden 슬롯 정합, 이 함수
        //     966행 근처 주석)만 소비하고 XML 은 별도 경로(`<hp:secPr>` 섹션 템플릿)가
        //     방출한다 — 여기서 다시 방출하면 중복이 된다. warn 헬퍼도 이 변형은
        //     무시한다.
        //   - `HiddenComment`: 이제 위 `render_hidden_comment` arm 이 방출한다.
        //     종전 주석은 "폭이 0이라 이 함수에 거의 도달하지 않는다"고 봤지만 그건
        //     **HWPX 파서 기준**이었다. HWP5 에서 `tcmt` 는 8유닛 확장 컨트롤이라
        //     h2x 경로에서는 정상적으로 슬롯에 들어와 여기 도달한다 — 실측(08361
        //     문단 0.0: `cc=33, text_len=0, controls=4` = 4×8+1)이 이를 보인다.
        //     방출 arm 이 없던 탓에 10k 스윕에서 숨은 설명이 통째로 사라졌고
        //     (`tcmt` CTRL_DIFF 13경로), 내용이 본문의 대부분인 문서는 96.9% 를
        //     잃었다(06397). x2x 축(HWPX 파서가 위치 마커를 push 하지 않아 폭 0)은
        //     여전히 별도 작업이 필요하다.
        c => warn_if_unrepresentable_in_hwpx(c),
    }
}

/// [#4388] HWPX 로 옮길 대응 표현이 없어 드롭되는 컨트롤(Hyperlink/Unknown)을
/// **조용히 버리지 않도록** 경고만 남긴다. 두 호출 지점(이 함수 위쪽
/// `render_runs` 의 mismatch-분기 제외 시점, `render_control_slot` 의
/// catch-all)에서 공유한다 — 한 문단의 한 컨트롤은 두 분기 중 하나로만
/// 소비되므로 중복 경고는 없다.
///
/// **`is_hwpx_inline_slot` 에는 등록하지 않는다.** 그 목록은
/// `roundtrip::diff_documents` 의 IR 비교(포맷 비종속 — `export-hwpx` 뿐 아니라
/// `convert`(HWP5 대상) `--verify` 도 재사용)에도 쓰이는데, 등록해봤다면 HWP5
/// 직렬화기가 이슈 범위 밖에서 이미 Hyperlink 의 ctrl_id 를 0 으로 고정해 버려
/// CTRL_HEADER 자체를 쓰지 않는 기존 HWP5 손실까지 새로 "검출"되어
/// `tests/issue_hwp3_bookmark_native.rs::bookmarks_survive_saving_to_hwp5`
/// 처럼 하이퍼링크와 무관한 기존 회귀 테스트가 깨진다(#4388 후속 보고, 실측
/// 재현: hwp3-sample16.hwp 문단 27/32/196/657 이 전부 Hyperlink). "조용히
/// 버리지 말라"는 경고 축과 "IR 비교에 넣는다"는 검증 축은 별개다.
fn warn_if_unrepresentable_in_hwpx(control: &Control) {
    match control {
        Control::Hyperlink(hl) => {
            eprintln!(
                "[hwpx] 경고: Hyperlink 컨트롤을 HWPX로 저장할 수 없어 드롭됩니다 \
                 (url={:?}, text={:?}) — HWPX는 하이퍼링크를 Field(HYPERLINK)로 표현하며 \
                 이 변환은 아직 지원되지 않습니다.",
                hl.url, hl.text
            );
        }
        Control::Unknown(u) => {
            eprintln!(
                "[hwpx] 경고: 인식할 수 없는 컨트롤(ctrl_id=0x{:08X})을 HWPX로 저장할 수 \
                 없어 드롭됩니다 — 원본 컨트롤의 세부 데이터가 보존되지 않아 이 변환은 \
                 지원되지 않습니다.",
                u.ctrl_id
            );
        }
        _ => {}
    }
}

/// 덧말(Ruby) `<hp:dutmal>` 직렬화 (#1587). `parse_dutmal` 의 역매핑.
/// 속성 순서는 한컴 실측(posType szRatio option styleIDRef align)을 따른다.
fn render_dutmal(r: &Ruby) -> String {
    let pos_type = match r.pos_type {
        1 => "BOTTOM",
        _ => "TOP",
    };
    let align = match r.align {
        1 => "RIGHT",
        2 => "CENTER",
        _ => "LEFT",
    };
    format!(
        r#"<hp:dutmal posType="{}" szRatio="{}" option="{}" styleIDRef="{}" align="{}"><hp:mainText>{}</hp:mainText><hp:subText>{}</hp:subText></hp:dutmal>"#,
        pos_type,
        r.sz_ratio,
        r.option,
        r.style_id_ref,
        align,
        xml_escape(&r.main_text),
        xml_escape(&r.ruby_text),
    )
}

/// `raw_parameters_xml` 이 없을 때(HWPX 원문 밖 경로 — HWP5 왕복 또는 API 로 새로
/// 만든 필드) `<hp:parameters>` 를 다시 조립한다.
///
/// [#4396] `field.parameters` 트리(HWPX 파서 또는 HWP5 CTRL_DATA 확장 아이템에서 채워짐)
/// 가 있으면 그 트리를 그대로 재조립 — `Prop`/`Direction`/`Path`/`Category` 등이
/// `Command` 하나로 축소되던 손실을 여기서 막는다. 트리가 없으면(순수 API 생성 필드
/// 등, 파싱 이력이 전혀 없는 경우) 예전처럼 `Command` 하나만 담은 최소 형태로 합성한다.
fn generated_field_parameters(field: &Field) -> Option<String> {
    if field.raw_parameters_xml.is_some() {
        return None;
    }
    if !field.parameters.is_empty() {
        return Some(field.parameters.render_xml("parameters"));
    }
    if field.command.is_empty() {
        return None;
    }
    Some(format!(
        r#"<hp:parameters cnt="1" name=""><hp:stringParam name="Command">{}</hp:stringParam></hp:parameters>"#,
        xml_escape(&field.command),
    ))
}

/// 셀·글상자 subList 인라인 `<hp:ctrl><hp:colPr .../></hp:ctrl>` (#1379 3단계).
/// `parse_col_pr` / `parse_col_line` / `parse_col_sz`(parser/hwpx/section.rs)의 역매핑.
fn render_col_pr_ctrl(cd: &ColumnDef) -> String {
    let col_type = match cd.column_type {
        ColumnType::Distribute => "BalancedNewspaper",
        ColumnType::Parallel => "Parallel",
        ColumnType::Normal => "NEWSPAPER",
    };
    let layout = match cd.direction {
        ColumnDirection::RightToLeft => "RIGHT",
        ColumnDirection::LeftToRight => "LEFT",
    };
    let mut out = format!(
        r#"<hp:ctrl><hp:colPr id="" type="{}" layout="{}" colCount="{}" sameSz="{}" sameGap="{}""#,
        col_type, layout, cd.column_count, cd.same_width as u8, cd.spacing,
    );
    // [#4387] sameSz="false"(단 너비 개별 지정)일 때 <hp:colSz>(단별 너비·간격,
    // ColumnDefType 스키마상 colLine 다음 자식)를 방출한다. 미방출 시 HWPX
    // 왕복에서 불균등 단 너비가 사라지고 렌더러(page_layout.rs
    // calculate_column_areas)가 균등 분할로 저하한다.
    let mut col_sz_xml = String::new();
    if !cd.same_width {
        for (i, w) in cd.widths.iter().enumerate() {
            let gap = cd.gaps.get(i).copied().unwrap_or(0);
            col_sz_xml.push_str(&format!(r#"<hp:colSz width="{}" gap="{}"/>"#, w, gap));
        }
    }
    if cd.separator_type != 0 || !col_sz_xml.is_empty() {
        out.push('>');
        if cd.separator_type != 0 {
            out.push_str(&format!(
                r#"<hp:colLine type="{}" width="{} mm" color="{}"/>"#,
                col_line_type_str(cd.separator_type),
                line_width_mm(cd.separator_width),
                super::shape::color_to_hex(cd.separator_color),
            ));
        }
        out.push_str(&col_sz_xml);
        out.push_str("</hp:colPr></hp:ctrl>");
    } else {
        out.push_str("/></hp:ctrl>");
    }
    out
}

/// `parse_hwpx_line_type` 역매핑 (colLine type).
fn col_line_type_str(t: u8) -> &'static str {
    match t {
        2 => "DASH",
        3 => "DOT",
        4 => "DASH_DOT",
        5 => "DASH_DOT_DOT",
        6 => "LONG_DASH",
        7 => "CIRCLE",
        _ => "SOLID",
    }
}

/// HWP 선 굵기 인덱스 → mm 수치 문자열 (`parse_hwpx_line_width` 역매핑).
fn line_width_mm(w: u8) -> &'static str {
    match w {
        0 => "0.1",
        1 => "0.12",
        2 => "0.15",
        3 => "0.2",
        4 => "0.25",
        5 => "0.3",
        6 => "0.4",
        7 => "0.5",
        8 => "0.6",
        9 => "0.7",
        10 => "1.0",
        11 => "1.5",
        12 => "2.0",
        13 => "3.0",
        14 => "4.0",
        _ => "5.0",
    }
}

/// `<hp:compose>` 글자겹침(CharOverlap) — `<hp:run>` 직접 자식 (<hp:ctrl> 비포장).
/// `parse_compose`(parser/hwpx/section.rs)의 역매핑. charPr 목록은 미설정(u32::MAX)
/// 항목 포함 원본 그대로 방출한다 (ID 참조 등록 비대상).
fn render_compose(co: &CharOverlap) -> String {
    let circle_type = match co.border_type {
        1 => "SHAPE_CIRCLE",
        2 => "SHAPE_REVERSAL_CIRCLE",
        3 => "SHAPE_RECTANGLE",
        4 => "SHAPE_REVERSAL_RECTANGLE",
        5 => "SHAPE_TRIANGLE",
        6 => "SHAPE_REVERSAL_TIRANGLE",
        _ => "CHAR",
    };
    let compose_type = if co.expansion == 1 {
        "OVERLAP"
    } else {
        "SPREAD"
    };
    let text: String = co.chars.iter().collect();
    let mut out = format!(
        r#"<hp:compose circleType="{}" charSz="{}" composeType="{}" charPrCnt="{}" composeText="{}">"#,
        circle_type,
        co.inner_char_size,
        compose_type,
        co.char_shape_ids.len(),
        xml_escape(&text),
    );
    for id in &co.char_shape_ids {
        out.push_str(&format!(r#"<hp:charPr prIDRef="{}"/>"#, id));
    }
    out.push_str("</hp:compose>");
    out
}

/// 장식 문자(userChar/prefixChar/suffixChar)용 속성값. '\0'(미설정)은 빈 문자열.
fn ctrl_char_attr(c: char) -> String {
    if c == '\0' {
        String::new()
    } else {
        xml_escape(&c.to_string())
    }
}

/// `<hp:ctrl><hp:autoNum num=".." numType=".."><hp:autoNumFormat .../></hp:autoNum></hp:ctrl>`
/// 자동 번호(AutoNumber) 컨트롤. format은 pageNum formatType과 동일한 코드→문자열 매핑.
fn render_autonum(an: &AutoNumber) -> String {
    format!(
        concat!(
            r#"<hp:ctrl><hp:autoNum num="{num}" numType="{nt}">"#,
            r#"<hp:autoNumFormat type="{ty}" userChar="{u}" prefixChar="{p}" "#,
            r#"suffixChar="{s}" supscript="{sup}"/></hp:autoNum></hp:ctrl>"#
        ),
        num = an.number,
        nt = auto_number_type_to_str(an.number_type),
        // [#2957] <hp:autoNumFormat type> 은 <hp:pageNum formatType> 과 달리 원 문자
        // 형식을 "CIRCLED_DIGIT"로 표기한다(각주/미주 경로에서 실측 검증됨, #2742).
        ty = if an.format == 1 {
            "CIRCLED_DIGIT"
        } else {
            page_num_format_to_str(an.format)
        },
        u = ctrl_char_attr(an.user_symbol),
        p = ctrl_char_attr(an.prefix_char),
        s = ctrl_char_attr(an.suffix_char),
        sup = an.superscript as u8,
    )
}

/// 머리말/꼬리말 적용 범위 → HWPX `applyPageType`. `parse_apply_page_type`의 역매핑.
fn apply_page_type_to_str(a: HeaderFooterApply) -> &'static str {
    match a {
        HeaderFooterApply::Both => "BOTH",
        HeaderFooterApply::Even => "EVEN",
        HeaderFooterApply::Odd => "ODD",
    }
}

/// `<hp:ctrl><hp:{header|footer} applyPageType=".."><hp:subList ...>문단들</hp:subList>...`
/// 머리말/꼬리말은 중첩 문단(subList)을 가진다 — render_note_sublist와 동일한 문단 직렬화
/// 경로(render_paragraph_parts)를 쓰되, subList 텍스트 영역 속성은 IR 보존값을 사용한다.
fn render_header_footer(
    tag: &str,
    h: HeaderFooterFields<'_>,
    ctx: &mut SerializeContext,
) -> String {
    let mut out = format!(
        concat!(
            r#"<hp:ctrl><hp:{tag} id="{id}" applyPageType="{apply}">"#,
            r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" "#,
            r#"linkListIDRef="0" linkListNextIDRef="0" textWidth="{tw}" textHeight="{th}" "#,
            r#"hasTextRef="{tr}" hasNumRef="{nr}">"#
        ),
        tag = tag,
        id = h.id,
        apply = apply_page_type_to_str(h.apply_to),
        tw = h.text_width,
        th = h.text_height,
        tr = h.text_ref,
        nr = h.num_ref,
    );
    let mut vert_cursor: u32 = 0;
    for p in h.paragraphs.iter() {
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let pid = ctx.next_para_id();
        let sid = ctx.effective_style_id(p.style_id);
        out.push_str(&render_hp_p_open(p, pid, sid));
        out.push_str(&runs);
        out.push_str(&linesegs);
        out.push_str("</hp:p>");
    }
    out.push_str(&format!("</hp:subList></hp:{tag}></hp:ctrl>", tag = tag));
    out
}

/// render_header_footer 공통 인자 묶음 (Header/Footer가 동일 필드를 가짐).
struct HeaderFooterFields<'a> {
    id: u32,
    apply_to: HeaderFooterApply,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
    paragraphs: &'a [Paragraph],
}

fn hwpx_header_footer_id(raw_ctrl_extra: &[u8]) -> u32 {
    raw_ctrl_extra
        .get(..4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(0)
}

fn render_header(h: &Header, ctx: &mut SerializeContext) -> String {
    render_header_footer(
        "header",
        HeaderFooterFields {
            id: hwpx_header_footer_id(&h.raw_ctrl_extra),
            apply_to: h.apply_to,
            text_width: h.text_width,
            text_height: h.text_height,
            text_ref: h.text_ref,
            num_ref: h.num_ref,
            paragraphs: &h.paragraphs,
        },
        ctx,
    )
}

fn render_footer(f: &Footer, ctx: &mut SerializeContext) -> String {
    render_header_footer(
        "footer",
        HeaderFooterFields {
            id: hwpx_header_footer_id(&f.raw_ctrl_extra),
            apply_to: f.apply_to,
            text_width: f.text_width,
            text_height: f.text_height,
            text_ref: f.text_ref,
            num_ref: f.num_ref,
            paragraphs: &f.paragraphs,
        },
        ctx,
    )
}

/// `<hp:ctrl><hp:pageHiding .../></hp:ctrl>` — 감추기(PageHide) 컨트롤.
/// `parse_page_hiding_attrs`의 역매핑. bool → "0"/"1" (한컴 정합).
fn render_page_hiding(ph: &PageHide) -> String {
    format!(
        concat!(
            r#"<hp:ctrl><hp:pageHiding hideHeader="{}" hideFooter="{}" "#,
            r#"hideMasterPage="{}" hideBorder="{}" hideFill="{}" hidePageNum="{}"/></hp:ctrl>"#
        ),
        ph.hide_header as u8,
        ph.hide_footer as u8,
        ph.hide_master_page as u8,
        ph.hide_border as u8,
        ph.hide_fill as u8,
        ph.hide_page_num as u8,
    )
}

/// 쪽 번호 위치 코드(표 150) → HWPX `pos` 문자열. `parse_page_num_attrs`의 역매핑.
fn page_num_pos_to_str(pos: u8) -> &'static str {
    match pos {
        0 => "NONE",
        1 => "TOP_LEFT",
        2 => "TOP_CENTER",
        3 => "TOP_RIGHT",
        4 => "BOTTOM_LEFT",
        5 => "BOTTOM_CENTER",
        6 => "BOTTOM_RIGHT",
        7 => "OUTSIDE_TOP",
        8 => "OUTSIDE_BOTTOM",
        9 => "INSIDE_TOP",
        10 => "INSIDE_BOTTOM",
        _ => "BOTTOM_CENTER",
    }
}

/// 번호 형식 코드(표 134) → HWPX `formatType` 문자열. `parse_page_num_attrs`의 역매핑.
fn page_num_format_to_str(fmt: u8) -> &'static str {
    match fmt {
        0 => "DIGIT",
        // [#XXXX] OWPML Core 스키마 NumberType1(<hp:pageNum formatType>)의 실제 값은
        // "CIRCLED_DIGIT"이다 (Core XML schema.xml 12행). "CIRCLE_DIGIT"은 오탈자.
        1 => "CIRCLED_DIGIT",
        2 => "ROMAN_CAPITAL",
        3 => "ROMAN_SMALL",
        4 => "LATIN_CAPITAL",
        5 => "LATIN_SMALL",
        6 => "HANGUL",
        7 => "HANJA",
        _ => "DIGIT",
    }
}

/// `<hp:ctrl><hp:pageNum .../></hp:ctrl>` — 쪽 번호 위치(PageNumberPos) 컨트롤.
fn render_page_num(pn: &PageNumberPos) -> String {
    // dash_char 기본값은 '-' (모델: 항상 '-'); '\0'이면 '-'로 폴백.
    let side = if pn.dash_char == '\0' {
        '-'
    } else {
        pn.dash_char
    };
    format!(
        r#"<hp:ctrl><hp:pageNum pos="{}" formatType="{}" sideChar="{}"/></hp:ctrl>"#,
        page_num_pos_to_str(pn.position),
        page_num_format_to_str(pn.format),
        xml_escape(&side.to_string()),
    )
}

/// 번호 종류 → HWPX `numType` 문자열. `parse_num_type`의 역매핑.
///
/// [#1387] Picture 는 "PICTURE" — 한컴 실물 표기. 종전 "FIGURE" 는 한컴 생산 파일에
/// 비실재(samples/hwpx 전수 0건)하고 한컴에디터가 그림 번호로 인식하지 못해 캡션
/// 번호가 미출력됐다 (한컴 판정 실증). 파서는 FIGURE/PICTURE 양쪽 수용 유지.
fn auto_number_type_to_str(t: AutoNumberType) -> &'static str {
    match t {
        AutoNumberType::Page => "PAGE",
        AutoNumberType::Footnote => "FOOTNOTE",
        AutoNumberType::Endnote => "ENDNOTE",
        AutoNumberType::Picture => "PICTURE",
        AutoNumberType::Table => "TABLE",
        AutoNumberType::Equation => "EQUATION",
        AutoNumberType::TotalPage => "TOTAL_PAGE",
    }
}

/// `<hp:ctrl><hp:newNum .../></hp:ctrl>` — 새 번호 지정(NewNumber) 컨트롤.
fn render_new_num(nn: &NewNumber) -> String {
    format!(
        r#"<hp:ctrl><hp:newNum num="{}" numType="{}"/></hp:ctrl>"#,
        nn.number,
        auto_number_type_to_str(nn.number_type),
    )
}

fn writer_to_string<F>(f: F) -> Result<String, SerializeError>
where
    F: FnOnce(&mut Writer<Vec<u8>>) -> Result<(), SerializeError>,
{
    let mut writer = Writer::new(Vec::new());
    f(&mut writer)?;
    let bytes = writer.into_inner();
    String::from_utf8(bytes)
        .map_err(|e| SerializeError::XmlError(format!("invalid UTF-8 from XML writer: {e}")))
}

fn render_shape(shape: &ShapeObject, ctx: &mut SerializeContext) -> String {
    // Rectangle: Writer-based serializer (drawText 포함)
    if let ShapeObject::Rectangle(r) = shape {
        return match writer_to_string(|w| super::shape::write_rect(w, r, ctx)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Rectangle 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    // Line: Writer-based serializer
    if let ShapeObject::Line(l) = shape {
        return match writer_to_string(|w| super::shape::write_line(w, l, ctx)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Line 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    if let ShapeObject::Group(g) = shape {
        let mut xml = match writer_to_string(|w| {
            super::shape::write_container_open(w, &g.common, &g.shape_attr)
        }) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Group 직렬화 실패: {e}");
                String::new()
            }
        };
        for child in &g.children {
            xml.push_str(&render_shape(child, ctx));
        }
        // 캡션 (#1403) 은 자식 도형 뒤에 방출 — 한컴 실물(aift.hwpx) 순서
        // 설명(#1392)은 캡션 직후 (write_container_close 내부)
        match writer_to_string(|w| {
            super::shape::write_container_close(w, g.caption.as_ref(), &g.common, ctx)
        }) {
            Ok(close) => xml.push_str(&close),
            Err(e) => eprintln!("[hwpx] Shape::Group 닫기 실패: {e}"),
        }
        return xml;
    }
    const NO_PTS: &[crate::model::Point] = &[];
    let (tag, c, caption, drawing, points): (
        _,
        _,
        _,
        Option<&crate::model::shape::DrawingObjAttr>,
        &[crate::model::Point],
    ) = match shape {
        ShapeObject::Rectangle(_) | ShapeObject::Line(_) => unreachable!(),
        ShapeObject::Ellipse(e) => (
            "ellipse",
            &e.common,
            &e.drawing.caption,
            Some(&e.drawing),
            NO_PTS,
        ),
        ShapeObject::Arc(a) => (
            "arc",
            &a.common,
            &a.drawing.caption,
            Some(&a.drawing),
            NO_PTS,
        ),
        ShapeObject::Polygon(p) => (
            "polygon",
            &p.common,
            &p.drawing.caption,
            Some(&p.drawing),
            &p.points,
        ),
        // [#4676] curve 의 점은 `<hc:pt>` 가 아니라 `<hp:seg>` 체인으로 나간다(geom_tail).
        ShapeObject::Curve(cv) => (
            "curve",
            &cv.common,
            &cv.drawing.caption,
            Some(&cv.drawing),
            NO_PTS,
        ),
        ShapeObject::Group(_) => unreachable!(),
        ShapeObject::Picture(pic) => {
            return match writer_to_string(|w| picture::write_picture(w, pic, ctx)) {
                Ok(xml) => xml,
                Err(e) => {
                    eprintln!("[hwpx] Shape::Picture 직렬화 실패: {e}");
                    String::new()
                }
            };
        }
        ShapeObject::Chart(ch) => ("chart", &ch.common, &ch.caption, None, NO_PTS),
        ShapeObject::Ole(o) => {
            // [#3546] hp:chart 출신(chart_id_ref 표식)은 hp:ole 이 아니라 원형
            // hp:chart(+switch) 구조로 재방출한다.
            return match writer_to_string(|w| super::shape::write_ole_or_chart(w, o, ctx)) {
                Ok(xml) => xml,
                Err(e) => {
                    eprintln!("[hwpx] Shape::Ole 직렬화 실패: {e}");
                    String::new()
                }
            };
        }
    };
    // [Task #1598] ellipse / arc 전용 지오메트리(center/축/시작끝점) — 미방출 시 한글이
    // 타원/호를 다르게 렌더 → 누적 레이아웃 변동 → 페이지 붕괴(#1589 잔여). hc:pt(polygon/
    // curve) 와 상호배타이므로 동일 위치(shadow 직후, sz 직전)에 방출.
    let hc = |t: &str, p: &crate::model::Point| format!(r#"<hc:{t} x="{}" y="{}"/>"#, p.x, p.y);
    let geom_tail = match shape {
        ShapeObject::Ellipse(e) => format!(
            "{}{}{}{}{}{}{}",
            hc("center", &e.center),
            hc("ax1", &e.axis1),
            hc("ax2", &e.axis2),
            hc("start1", &e.start1),
            hc("end1", &e.end1),
            hc("start2", &e.start2),
            hc("end2", &e.end2),
        ),
        ShapeObject::Arc(a) => format!(
            "{}{}{}",
            hc("center", &a.center),
            hc("ax1", &a.axis1),
            hc("ax2", &a.axis2),
        ),
        // [#4676] curve 는 점을 `<hp:seg>` 체인으로 방출한다 — `<hc:pt>` 나열은 한글이
        // 열다 죽는다(RPC 0x800706BE). 한컴 원본 실측: hp:curve 는 seg 만 쓰고 hc:pt 는
        // 한 번도 쓰지 않는다. seg 는 이웃한 두 점을 잇는 구간이므로 점 N 개 → seg N-1 개.
        ShapeObject::Curve(cv) => curve_segs_xml(&cv.points, &cv.segment_types),
        _ => String::new(),
    };
    // [#4388] `<hp:arc>` 전용 `type` 속성(NORMAL/PIE/CHORD) — OWPML `CArcType` 계약.
    // 종전엔 이 속성 자체가 방출되지 않아 `arc_type` 이 항상 0(NORMAL)으로 저장됐다.
    let extra_attrs = match shape {
        ShapeObject::Arc(a) => format!(r#" type="{}""#, arc_type_hwpx_str(a.arc_type)),
        _ => String::new(),
    };
    render_common_shape_xml(
        tag,
        c,
        caption,
        drawing,
        points,
        &geom_tail,
        &extra_attrs,
        ctx,
    )
}

/// [#4676] `CurveShape` 의 점 목록을 OWPML `<hp:seg>` 체인으로 방출한다.
///
/// 한글은 `<hp:curve>` 안의 `<hc:pt>` 나열을 만나면 여는 도중 죽는다(COM RPC 0x800706BE,
/// 10k 오라클 스윕에서 크래시 산출물 다수의 공통 원인). 한컴 원본은 언제나 seg 를 쓴다:
///
/// ```xml
/// <hp:seg type="CURVE" x1="0" y1="1680" x2="10440" y2="0"/>
/// <hp:seg type="LINE"  x1="10440" y1="0" x2="20940" y2="1800"/>
/// ```
///
/// `segment_types[i]` 는 HWP5의 구간 종류(0: 직선, 1: 곡선)다. HWPX의 `hp:seg type`은
/// HWP5 `1`이 기대하는 베지어 제어점 두 개를 담지 않으므로 파서는 이 필드에 옮기지 않는다.
/// 비어 있으면 한글 호환을 위해 CURVE로 방출한다.
fn curve_segs_xml(points: &[crate::model::Point], segment_types: &[u8]) -> String {
    points
        .windows(2)
        .enumerate()
        .map(|(i, w)| {
            let kind = match segment_types.get(i) {
                Some(0) => "LINE",
                _ => "CURVE",
            };
            format!(
                r#"<hp:seg type="{}" x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                kind, w[0].x, w[0].y, w[1].x, w[1].y
            )
        })
        .collect()
}

/// [#4388] `ArcShape.arc_type` (0: Arc, 1: CircularSector, 2: Bow) →
/// `<hp:arc>` 의 `type` 속성값. `parse_arc_type_attr` 의 역매핑.
fn arc_type_hwpx_str(arc_type: u8) -> &'static str {
    match arc_type {
        1 => "PIE",
        2 => "CHORD",
        _ => "NORMAL",
    }
}

fn render_common_shape_xml(
    tag: &str,
    c: &CommonObjAttr,
    caption: &Option<crate::model::shape::Caption>,
    drawing: Option<&crate::model::shape::DrawingObjAttr>,
    points: &[crate::model::Point],
    geom_tail: &str,
    // [#4388] 태그별 부가 속성(현재는 `<hp:arc type="...">` 전용) — 없으면 "".
    extra_attrs: &str,
    ctx: &mut SerializeContext,
) -> String {
    // 도형 좌표계 블록(offset/orgSz/curSz/flip/rotationInfo/renderingInfo) — 누락 시
    // 회전/뒤집힘이 소실되어 bbox 가 전치되는 등 렌더가 어긋난다(#1501 동류, polygon 등).
    let shape_block = drawing
        .map(|d| {
            writer_to_string(|w| super::shape::write_shape_component_block(w, &d.shape_attr))
                .unwrap_or_default()
        })
        .unwrap_or_default();
    // [#1596] 지오메트리(lineShape/fillBrush/shadow/꼭짓점) — 종전 드롭으로 도형 형상 소실 →
    // 페이지 붕괴(#1589 잔여). write_rect 와 동형 순서(shape_block 직후, sz 직전).
    let geometry = drawing
        .map(|d| {
            let ls = writer_to_string(|w| super::shape::write_line_shape(w, &d.border_line))
                .unwrap_or_default();
            let fb = writer_to_string(|w| super::shape::write_fill_brush(w, &d.fill, ctx))
                .unwrap_or_default();
            let sh = writer_to_string(|w| super::shape::write_shadow(w, d)).unwrap_or_default();
            // [Issue #1944] drawText(도형 내 글상자 문단) — rect 경로(shape.rs write_rect)는
            // shadow 직후 방출하나 legacy 공용 경로(ellipse/arc/polygon/curve)는 누락해
            // 도형 안 텍스트가 저장 시 소실됐다(순서도 마름모 라벨 등). OWPML 순서
            // (shadow → drawText → hc:pt) 대로 shadow 와 points 사이에 방출한다.
            let dt = d
                .text_box
                .as_ref()
                .filter(|tb| !tb.paragraphs.is_empty())
                .map(|tb| {
                    writer_to_string(|w| super::shape::write_draw_text(w, tb, ctx))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let pts: String = points
                .iter()
                .map(|p| format!(r#"<hc:pt x="{}" y="{}"/>"#, p.x, p.y))
                .collect();
            // [#1598] ellipse/arc 전용 지오메트리(center/축/시작끝점)는 hc:pt 와 상호배타.
            format!("{ls}{fb}{sh}{dt}{pts}{geom_tail}")
        })
        .unwrap_or_default();
    // 태그 부수 속성 — numberingType/dropcapstyle/href/groupLevel/instid (rect/line 동형).
    let group_level = drawing.map(|d| d.shape_attr.group_level).unwrap_or(0);
    let instid = drawing
        .map(|d| d.inst_id)
        .filter(|&i| i != 0)
        .unwrap_or(c.instance_id);
    let mut out = format!(
        concat!(
            r#"<hp:{tag} id="{id}" zOrder="{zo}" numberingType="{nt}" textWrap="{tw}" textFlow="{tf}" lock="{lock}" dropcapstyle="None" href="" groupLevel="{gl}" instid="{iid}"{extra}>"#,
            "{block}",
            "{geometry}",
            r#"<hp:sz width="{w}" height="{h}" widthRelTo="{wrt}" heightRelTo="{hrt}" protect="{prot}"/>"#,
            r#"<hp:pos treatAsChar="{tac}" affectLSpacing="0" flowWithText="{fwt}" allowOverlap="{ao}" holdAnchorAndSO="{hold}" vertRelTo="{vr}" vertAlign="{va}" horzRelTo="{hr}" horzAlign="{ha}" vertOffset="{vo}" horzOffset="{ho}"/>"#,
            r#"<hp:outMargin left="{ml}" right="{mr}" top="{mt}" bottom="{mb}"/>"#,
        ),
        tag = tag,
        extra = extra_attrs,
        block = shape_block,
        geometry = geometry,
        id = c.instance_id,
        zo = c.z_order,
        nt = super::shape::numbering_type_str(c.numbering_type),
        // [#2840] lock(개체 잠금) — IR 보존 값 방출 (종전 "0" 하드코딩).
        lock = if c.locked { "1" } else { "0" },
        gl = group_level,
        iid = instid,
        tw = text_wrap_to_hwpx(c.text_wrap),
        // [#2790] textFlow(글 흐름) — 종전 "BOTH_SIDES" 리터럴은 파서
        // (parse_object_element_attrs:2962)가 IR 에 적재한 값을 저장에서만 버리는 순손실이었다.
        // write_rect/write_line(shape.rs)·render_equation 은 이미 IR 기반이다.
        tf = text_flow_to_hwpx(c.text_flow),
        tac = if c.treat_as_char { "1" } else { "0" },
        fwt = if c.flow_with_text { "1" } else { "0" },
        ao = if c.allow_overlap { "1" } else { "0" },
        hold = if c.prevent_page_break != 0 { "1" } else { "0" },
        w = c.width,
        h = c.height,
        // [#2726] 종전 widthRelTo/heightRelTo 는 "ABSOLUTE" 리터럴, protect 는 아예
        // 미방출이었다. 파서(parse_object_layout_child:2909)가 이미 3값을 IR 에 적재하므로
        // 저장에서만 버려지는 순손실이었다. #2697(표)·#2712(rect/line/container/pic) 와 동형.
        wrt = size_criterion_str(c.width_criterion),
        hrt = height_criterion_str(c.height_criterion),
        prot = if c.size_protect { "1" } else { "0" },
        vr = vert_rel_to_hwpx(c.vert_rel_to),
        va = vert_align_to_hwpx(c.vert_align),
        hr = horz_rel_to_hwpx(c.horz_rel_to),
        ha = horz_align_to_hwpx(c.horz_align),
        vo = c.vertical_offset,
        ho = c.horizontal_offset,
        ml = c.margin.left,
        mr = c.margin.right,
        mt = c.margin.top,
        mb = c.margin.bottom,
    );
    // 캡션 (#1403) — HWP5 파서는 모든 도형의 캡션을 적재하므로(parser/control/shape.rs)
    // legacy 경로(ellipse/arc/polygon/curve/chart/ole)도 방출해야 소실되지 않는다.
    if let Some(cap) = caption {
        match writer_to_string(|w| super::table::write_caption(w, cap, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Shape({tag}) 캡션 직렬화 실패: {e}"),
        }
    }
    // 설명 (#1451) — caption 직후 (OWPML AbstractShapeObjectType: outMargin→caption→shapeComment).
    // picture.rs:104 선례와 동일 순서. legacy 경로(ellipse/arc/polygon/curve/chart/ole) 보존.
    // 빈 description 미방출은 write_shape_comment 내부 가드로 보장된다.
    match writer_to_string(|w| super::shape::write_shape_comment(w, c)) {
        Ok(xml) => out.push_str(&xml),
        Err(e) => eprintln!("[hwpx] Shape({tag}) shapeComment 직렬화 실패: {e}"),
    }
    out.push_str(&format!("</hp:{tag}>"));
    out
}

/// [#2716] `<hp:footNote>` / `<hp:endNote>` 의 IR 보존 속성 묶음.
///
/// 종전에는 `number` 스칼라 하나만 `render_note_sublist` 로 넘겨 나머지 4개 필드가
/// 구조적으로 방출 불가였다. 같은 파일의 `render_header_footer` 가 쓰는
/// `HeaderFooterFields` 와 동일한 묶음 전달 패턴이다.
struct NoteAttrs {
    number: u16,
    /// `before_decoration_letter` — 0 이면 속성 생략(한컴 계약).
    prefix_char: u16,
    /// `after_decoration_letter` — 항상 방출.
    suffix_char: u16,
    /// HWP5 `numberShape` — 0 이면 속성 생략(한컴 계약).
    number_shape: u32,
    /// HWP5 `instanceId` — 항상 방출.
    inst_id: u32,
}

/// [#2716] 각주/미주 ctrl 속성 문자열.
///
/// 한컴 저장본 실측 계약(samples 16파일 · note 828개, 3-09월_교육_통합_2023 의 HWP5/HWPX
/// 쌍 46개 필드 단위 전수 대조 일치):
/// - `number` / `suffixChar` / `instId` 는 항상 존재(828/828)
/// - `prefixChar` 는 `before_decoration_letter != 0` 일 때만 존재(598/828)
/// - `flag`(= HWP5 `numberShape`) 는 `number_shape != 0` 일 때만 존재(27/828)
/// - 속성 순서는 `flag → number → prefixChar → suffixChar → instId`
///
/// `suffixChar` 를 생략하면 파서(`parse_ctrl_footnote`)가 기본값 `0x0029` `)` 를 넣어
/// 닫는 장식이 없는(0) 각주가 오염된다 — HWP5 직렬화기가 `serializer/control.rs` 에서
/// 같은 이유로 고친 문제이므로 0 이어도 항상 방출한다.
fn render_note_attrs(attrs: &NoteAttrs) -> String {
    let mut out = String::new();
    if attrs.number_shape != 0 {
        out.push_str(&format!(r#" flag="{}""#, attrs.number_shape));
    }
    out.push_str(&format!(r#" number="{}""#, attrs.number));
    if attrs.prefix_char != 0 {
        out.push_str(&format!(r#" prefixChar="{}""#, attrs.prefix_char));
    }
    out.push_str(&format!(
        r#" suffixChar="{}" instId="{}""#,
        attrs.suffix_char, attrs.inst_id
    ));
    out
}

/// [#2726] 너비 기준 → HWPX `widthRelTo`. 파서 `parse_size_criterion(_, true)` 의 정확한 역.
///
/// `table.rs:147` 이 `#2697` 로 정립한 관례와 **동일 의미**다. 해당 사본이 private 이라
/// 이 모듈에서 도달할 수 없어 부득이 복제했다. 공용 위치 1벌 통합은 잔여다(이슈 7장).
fn size_criterion_str(c: SizeCriterion) -> &'static str {
    match c {
        SizeCriterion::Paper => "PAPER",
        SizeCriterion::Page => "PAGE",
        SizeCriterion::Column => "COLUMN",
        SizeCriterion::Para => "PARA",
        SizeCriterion::Absolute => "ABSOLUTE",
    }
}

/// [#2726] 높이 기준 → HWPX `heightRelTo`. 파서는 높이를
/// `parse_size_criterion(_, allow_column_para = false)` 로 읽으므로(`parser/hwpx/section.rs:1860`)
/// 치역이 `{PAPER, PAGE, ABSOLUTE}` 3값뿐이다. 방출도 같은 3값으로 접어야 왕복이 정확한
/// 역이 된다 — `COLUMN`/`PARA` 를 내면 되읽기에서 `Absolute` 로 접혀 비-멱등이 된다.
/// HWP5 측 `height_criterion_to_bits`(`common_obj_attr_writer.rs:160`)도 동일하게 접는다.
fn height_criterion_str(c: SizeCriterion) -> &'static str {
    match c {
        SizeCriterion::Paper => "PAPER",
        SizeCriterion::Page => "PAGE",
        SizeCriterion::Column | SizeCriterion::Para | SizeCriterion::Absolute => "ABSOLUTE",
    }
}

fn render_note_sublist(
    tag: &str,
    attrs: NoteAttrs,
    paragraphs: &[Paragraph],
    ctx: &mut SerializeContext,
) -> String {
    let mut out = format!(
        r#"<hp:ctrl><hp:{tag}{attrs}><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">"#,
        tag = tag,
        attrs = render_note_attrs(&attrs),
    );
    let mut vert_cursor: u32 = 0;
    for p in paragraphs.iter() {
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let pid = ctx.next_para_id();
        let sid = ctx.effective_style_id(p.style_id);
        out.push_str(&render_hp_p_open(p, pid, sid));
        out.push_str(&runs);
        out.push_str(&linesegs);
        out.push_str("</hp:p>");
    }
    out.push_str(&format!("</hp:subList></hp:{tag}></hp:ctrl>", tag = tag));
    out
}

/// 숨은 설명(`<hp:hiddenComment>`) — 스키마상 `subList` 하나만 갖는다
/// (ParaList XML schema.xml:217-224). 각주와 달리 속성이 없다.
fn render_hidden_comment(
    hc: &crate::model::control::HiddenComment,
    ctx: &mut SerializeContext,
) -> String {
    let mut out = String::from("<hp:ctrl><hp:hiddenComment>");
    out.push_str(&render_sub_list_open(None));
    let mut vert_cursor: u32 = 0;
    for p in hc.paragraphs.iter() {
        ctx.para_shape_ids.reference(p.para_shape_id);
        let sid = ctx.effective_style_id(p.style_id);
        ctx.style_ids.reference(sid as u16);
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let pid = ctx.next_para_id();
        out.push_str(&render_hp_p_open(p, pid, sid));
        out.push_str(&runs);
        out.push_str(&linesegs);
        out.push_str("</hp:p>");
    }
    out.push_str("</hp:subList></hp:hiddenComment></hp:ctrl>");
    out
}

fn render_footnote(note: &Footnote, ctx: &mut SerializeContext) -> String {
    // [#2716] IR 이 보존한 장식 문자/번호 모양/고유 ID 를 모두 방출한다. 종전엔 number 만
    // 써서 저장 왕복마다 앞 장식('문')이 사라지고 뒤 장식이 ')' 로 변조됐다.
    render_note_sublist(
        "footNote",
        NoteAttrs {
            number: note.number,
            prefix_char: note.before_decoration_letter,
            suffix_char: note.after_decoration_letter,
            number_shape: note.number_shape,
            inst_id: note.instance_id,
        },
        &note.paragraphs,
        ctx,
    )
}

fn render_endnote(note: &Endnote, ctx: &mut SerializeContext) -> String {
    // [#2716] Footnote 와 동일 계약.
    render_note_sublist(
        "endNote",
        NoteAttrs {
            number: note.number,
            prefix_char: note.before_decoration_letter,
            suffix_char: note.after_decoration_letter,
            number_shape: note.number_shape,
            inst_id: note.instance_id,
        },
        &note.paragraphs,
        ctx,
    )
}

fn render_equation(eq: &Equation) -> String {
    let c = &eq.common;
    let id = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let version = xml_escape(&eq.version_info);
    let baseline = eq.baseline.to_string();
    let text_color = color_ref_to_hwpx(eq.color);
    let base_unit = eq.font_size.to_string();
    let font = xml_escape(&eq.font_name);
    let script = xml_escape(&eq.script);
    let width = c.width.to_string();
    let height = c.height.to_string();
    let treat = if c.treat_as_char { "1" } else { "0" };
    let flow_with_text = if c.flow_with_text { "1" } else { "0" };
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    let margin_left = c.margin.left.to_string();
    let margin_right = c.margin.right.to_string();
    let margin_top = c.margin.top.to_string();
    let margin_bottom = c.margin.bottom.to_string();

    // 설명 (#1392) — outMargin 직후, 빈 description 은 미방출
    let shape_comment = if c.description.is_empty() {
        String::new()
    } else {
        format!(
            "<hp:shapeComment>{}</hp:shapeComment>",
            xml_escape(&c.description)
        )
    };

    // [#1594] holdAnchorAndSO 는 IR(prevent_page_break)을 보존(종전 "0" 하드코딩 제거).
    let hold = if c.prevent_page_break != 0 { "1" } else { "0" };

    // [#2727] lineMode(수식이 차지하는 범위) 를 EQEDIT attribute bit0 에서 방출한다.
    // 종전엔 속성 자체를 내보내지 않아 LINE 설정이 왕복마다 CHAR 로 되돌아갔다.
    // 한컴 저장본과 동일하게 baseUnit 과 font 사이에 위치시킨다.
    let line_mode = if eq.attr & EQUATION_LINE_MODE_BIT != 0 {
        "LINE"
    } else {
        "CHAR"
    };

    // [#2840] 개체 잠금(lock) — 종전 하드코딩 "0" 제거, IR(common.locked) 값을 방출.
    let lock = if c.locked { "1" } else { "0" };

    // [#2778] 크기 기준(widthRelTo/heightRelTo) — 종전 "ABSOLUTE" 리터럴은 파서
    // (parse_object_layout_child:3023)가 IR 에 적재한 단/열/쪽 상대 기준을 저장에서만
    // 버렸다. #2697(표)·#2712(그림/도형)·#2726(공용 도형)과 동형이다. 높이는 파서가
    // `parse_size_criterion(_, false)` 로 읽어 치역이 3값이므로 같은 접기 함수를 쓴다.
    let width_rel_to = size_criterion_str(c.width_criterion);
    let height_rel_to = height_criterion_str(c.height_criterion);

    // [#2782] 개체 겹침 허용(allowOverlap) — 종전 하드코딩 "0" 제거, IR 값을 방출.
    // 같은 요소의 treatAsChar·flowWithText 는 이미 IR 기반이었다.
    let allow_overlap = if c.allow_overlap { "1" } else { "0" };

    format!(
        r#"<hp:equation id="{id}" zOrder="{z_order}" numberingType="EQUATION" textWrap="{}" textFlow="{}" lock="{lock}" dropcapstyle="None" version="{version}" baseLine="{baseline}" textColor="{text_color}" baseUnit="{base_unit}" lineMode="{line_mode}" font="{font}"><hp:script>{script}</hp:script><hp:sz width="{width}" widthRelTo="{width_rel_to}" height="{height}" heightRelTo="{height_rel_to}"/><hp:pos treatAsChar="{treat}" affectLSpacing="0" flowWithText="{flow_with_text}" allowOverlap="{allow_overlap}" holdAnchorAndSO="{hold}" vertRelTo="{}" horzRelTo="{}" vertAlign="{}" horzAlign="{}" vertOffset="{vert_offset}" horzOffset="{horz_offset}"/><hp:outMargin left="{margin_left}" right="{margin_right}" top="{margin_top}" bottom="{margin_bottom}"/>{shape_comment}</hp:equation>"#,
        text_wrap_to_hwpx(c.text_wrap),
        text_flow_to_hwpx(c.text_flow),
        vert_rel_to_hwpx(c.vert_rel_to),
        horz_rel_to_hwpx(c.horz_rel_to),
        vert_align_to_hwpx(c.vert_align),
        horz_align_to_hwpx(c.horz_align),
    )
}

fn char_utf16_width(c: char) -> u32 {
    if c == '\t' {
        8
    } else if (c as u32) > 0xFFFF {
        2
    } else {
        1
    }
}

fn color_ref_to_hwpx(color: u32) -> String {
    if color == 0xFFFFFFFF {
        return "none".to_string();
    }

    let a = (color >> 24) & 0xFF;
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    if a == 0 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
    }
}

fn text_wrap_to_hwpx(wrap: TextWrap) -> &'static str {
    match wrap {
        TextWrap::Square => "SQUARE",
        TextWrap::Tight => "TIGHT",
        TextWrap::Through => "THROUGH",
        TextWrap::TopAndBottom => "TOP_AND_BOTTOM",
        TextWrap::BehindText => "BEHIND_TEXT",
        TextWrap::InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn text_flow_to_hwpx(flow: crate::model::shape::TextFlow) -> &'static str {
    use crate::model::shape::TextFlow;
    match flow {
        TextFlow::BothSides => "BOTH_SIDES",
        TextFlow::LeftOnly => "LEFT_ONLY",
        TextFlow::RightOnly => "RIGHT_ONLY",
        TextFlow::LargestOnly => "LARGEST_ONLY",
    }
}

fn vert_rel_to_hwpx(rel: VertRelTo) -> &'static str {
    match rel {
        VertRelTo::Paper => "PAPER",
        VertRelTo::Page => "PAGE",
        VertRelTo::Para => "PARA",
    }
}

fn horz_rel_to_hwpx(rel: HorzRelTo) -> &'static str {
    match rel {
        HorzRelTo::Paper => "PAPER",
        HorzRelTo::Page => "PAGE",
        HorzRelTo::Column => "COLUMN",
        HorzRelTo::Para => "PARA",
    }
}

fn vert_align_to_hwpx(align: VertAlign) -> &'static str {
    match align {
        VertAlign::Top => "TOP",
        VertAlign::Center => "CENTER",
        VertAlign::Bottom => "BOTTOM",
        VertAlign::Inside => "INSIDE",
        VertAlign::Outside => "OUTSIDE",
    }
}

fn horz_align_to_hwpx(align: HorzAlign) -> &'static str {
    match align {
        HorzAlign::Left => "LEFT",
        HorzAlign::Center => "CENTER",
        HorzAlign::Right => "RIGHT",
        HorzAlign::Inside => "INSIDE",
        HorzAlign::Outside => "OUTSIDE",
    }
}

/// IR의 `line_segs` 를 그대로 XML로 직렬화 (9개 필드 전부 IR 값 사용).
///
/// rhwp 는 자신의 문서에서 비표준 lineseg 를 **새로 생산하지 않는다**.
/// 원본 한컴 파일의 lineseg 값이 파서에 의해 `Paragraph.line_segs` 에 담겼다면,
/// 저장 시 그 값을 훼손 없이 보존한다.
fn render_lineseg_array_from_ir(segs: &[LineSeg]) -> String {
    let mut out = String::new();
    for seg in segs {
        out.push_str(&render_one_lineseg(seg, seg.text_start));
    }
    out
}

/// `<hp:lineseg>` 한 줄 — `textpos` 만 호출부가 정하고 나머지 8필드는 IR 값 그대로.
fn render_one_lineseg(seg: &LineSeg, text_start: u32) -> String {
    format!(
        r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="{}" textheight="{}" baseline="{}" spacing="{}" horzpos="{}" horzsize="{}" flags="{}"/>"#,
        text_start,
        seg.vertical_pos,
        seg.line_height,
        seg.text_height,
        seg.baseline_distance,
        seg.line_spacing,
        seg.column_start,
        seg.segment_width,
        seg.tag,
    )
}

/// IR 기반 다음 문단의 vert_start 계산 — 마지막 lineseg 의 vpos + lh 사용.
fn next_vert_cursor_from_ir(segs: &[LineSeg], vert_start: u32) -> u32 {
    if segs.len() == 1 && segs[0].is_missing_lineseg_placeholder() {
        return vert_start;
    }

    if let Some(last) = segs.last() {
        // vertical_pos 는 섹션 시작 기준 절대값일 수도, 문단 기준 상대값일 수도 있음.
        // 현재 rhwp 는 섹션 절대값이므로 그대로 + lh 로 다음 커서 산출.
        let next = (last.vertical_pos as i64) + (last.line_height.max(0) as i64);
        if next > vert_start as i64 {
            next as u32
        } else {
            vert_start + VERT_STEP
        }
    } else {
        vert_start + VERT_STEP
    }
}

fn flush_buf(t_xml: &mut String, buf: &mut String) {
    if !buf.is_empty() {
        t_xml.push_str(&xml_escape(buf));
        buf.clear();
    }
}

/// 템플릿의 첫 `<hp:linesegarray>...</hp:linesegarray>` **요소 전체**를
/// `new_element` 로 치환한다. `new_element` 가 빈 문자열이면 요소가 제거된다
/// (#1380 — line_segs 빈 문단의 linesegarray 방출 생략).
fn replace_first_linesegs(xml: &str, new_element: &str) -> String {
    let open = xml
        .find(LINESEG_SLOT_OPEN)
        .expect("template has linesegarray");
    let close_rel = xml[open..]
        .find(LINESEG_SLOT_CLOSE)
        .expect("template has closing linesegarray");
    let elem_end = open + close_rel + LINESEG_SLOT_CLOSE.len();
    let mut out = String::with_capacity(xml.len() + new_element.len());
    out.push_str(&xml[..open]);
    out.push_str(new_element);
    out.push_str(&xml[elem_end..]);
    out
}

/// [#1166] 템플릿 pagePr 의 고정 용지 속성(landscape/width/height)을 IR page_def
/// 값으로 치환한다. 종전엔 템플릿 하드코딩값(landscape="WIDELY" width=59528
/// height=84186)이 그대로 출력되어 HWPX 저장 시 가로/세로 + 용지 크기가 손실됐다.
///
/// OWPML landscape: WIDELY=세로(landscape=false), NARROWLY=가로(landscape=true).
/// width/height 는 짧은변/긴변 그대로 (HWP 바이너리 동일 규약).
///
/// [#1388] 확장: gutterType(제본 방향) + `<hp:margin>` 여백 7필드를 IR 값으로 치환.
/// 종전엔 템플릿 고정 여백(left/right=8504 등)이 그대로 출력되어 원본 여백이
/// 변형됐다 (samples/hwpx 전수 51/74 섹션 영향, 본문 +56.7px 시프트·페이지 수 변동).
/// gutterType ↔ binding 매핑은 parser/hwpx/section.rs `parse_page_pr` 의 역매핑.
fn replace_page_pr(xml: &str, page_def: &crate::model::page::PageDef) -> String {
    // 템플릿의 pagePr 여는 태그(고정 문자열) → IR 기반으로 교체.
    const TEMPLATE_PAGE_PR: &str =
        r#"<hp:pagePr landscape="WIDELY" width="59528" height="84186" gutterType="LEFT_ONLY">"#;
    let landscape = if page_def.landscape {
        "NARROWLY"
    } else {
        "WIDELY"
    };
    let gutter_type = match page_def.binding {
        crate::model::page::BindingMethod::SingleSided => "LEFT_ONLY",
        crate::model::page::BindingMethod::DuplexSided => "LEFT_RIGHT",
        crate::model::page::BindingMethod::TopFlip => "TOP_BOTTOM",
    };
    let new_page_pr = format!(
        r#"<hp:pagePr landscape="{}" width="{}" height="{}" gutterType="{}">"#,
        landscape, page_def.width, page_def.height, gutter_type,
    );
    let out = if xml.contains(TEMPLATE_PAGE_PR) {
        xml.replacen(TEMPLATE_PAGE_PR, &new_page_pr, 1)
    } else {
        // 템플릿이 변경됐거나 이미 치환된 경우 — 원본 유지(회귀 방지).
        xml.to_string()
    };

    // 템플릿의 hp:margin(고정 문자열) → IR 여백 7필드로 교체 (#1388).
    // write_section 은 콘텐츠 삽입 전에 호출하므로 템플릿 내 유일 1회 등장이 보장된다.
    const TEMPLATE_PAGE_MARGIN: &str = r#"<hp:margin header="4252" footer="4252" gutter="0" left="8504" right="8504" top="5668" bottom="4252"/>"#;
    let new_margin = format!(
        r#"<hp:margin header="{}" footer="{}" gutter="{}" left="{}" right="{}" top="{}" bottom="{}"/>"#,
        page_def.margin_header,
        page_def.margin_footer,
        page_def.margin_gutter,
        page_def.margin_left,
        page_def.margin_right,
        page_def.margin_top,
        page_def.margin_bottom,
    );
    if out.contains(TEMPLATE_PAGE_MARGIN) {
        out.replacen(TEMPLATE_PAGE_MARGIN, &new_margin, 1)
    } else {
        // 템플릿이 변경됐거나 이미 치환된 경우 — 원본 유지(회귀 방지).
        out
    }
}

/// 템플릿의 3개 `pageBorderFill`(BOTH/EVEN/ODD, 하드코딩 borderFillIDRef="1") 을 IR 값으로
/// 치환한다. 누락 시 문서의 실제 쪽 테두리가 소실된다. 파서는 `type` 속성 값을 기준으로
/// `page_border_fill`(BOTH) / `extra_page_border_fills`(EVEN/ODD) 슬롯을 배정한다(#2885).
///
/// `extra_page_border_fills` 에 실제 EVEN/ODD 데이터가 없는 경우(절대다수 — 실제 한컴
/// 문서는 `pageBorderFill` 을 1개(BOTH)만 갖는다) `page_border_fill` 값을 그대로 복제해
/// EVEN/ODD 자리를 채우면, 원본에 없던 요소가 왕복 후 생겨나고 그 값(=BOTH 복제본)이
/// 재파싱 시 `extra_page_border_fills` 로 다시 흡수되어 존재하지 않던 필드가 왕복마다
/// 늘어난다(#2896 CI 발견 — IR 필드 스윕 baseline 발산). 대신 그 자리는 템플릿에서
/// 통째로 제거해 원본 문서 구조(단일 BOTH)를 보존한다.
fn replace_page_border_fill(xml: &str, sec_def: &crate::model::document::SectionDef) -> String {
    let entries: [(&str, Option<&crate::model::page::PageBorderFill>); 3] = [
        ("BOTH", Some(&sec_def.page_border_fill)),
        ("EVEN", sec_def.extra_page_border_fills.first()),
        ("ODD", sec_def.extra_page_border_fills.get(1)),
    ];
    let mut out = xml.to_string();
    for (ty, pbf) in entries {
        // 템플릿 고정 문자열 (empty_section0.xml 와 정확히 일치해야 함).
        let template = format!(
            r#"<hp:pageBorderFill type="{ty}" borderFillIDRef="1" textBorder="PAPER" headerInside="0" footerInside="0" fillArea="PAPER"><hp:offset left="1417" right="1417" top="1417" bottom="1417"/></hp:pageBorderFill>"#
        );
        if out.contains(&template) {
            let replacement = match pbf {
                Some(pbf) => render_page_border_fill(ty, pbf),
                // IR 에 이 슬롯의 실데이터가 없음 — BOTH 를 복제해 채우는 대신
                // 요소 자체를 제거한다(원본 없던 EVEN/ODD 요소를 만들어내지 않음).
                None => String::new(),
            };
            out = out.replacen(&template, &replacement, 1);
        }
        // 미일치 시 원본 유지(회귀 방지) — replace_page_pr 패턴과 동형.
    }
    out
}

/// 단일 `pageBorderFill` 요소를 IR 에서 재구성한다. attr 비트 → textBorder/fillArea/
/// headerInside/footerInside (parser `page_border_fill_attr` 의 역).
fn render_page_border_fill(ty: &str, pbf: &crate::model::page::PageBorderFill) -> String {
    let text_border = if pbf.attr & 0x0000_0001 != 0 {
        "PAPER"
    } else {
        "CONTENT"
    };
    let fill_area = if pbf.attr & 0x0000_0008 != 0 {
        "PAGE"
    } else if pbf.attr & 0x0000_0010 != 0 {
        "BORDER"
    } else {
        "PAPER"
    };
    let header_inside = if pbf.attr & 0x0000_0002 != 0 {
        "1"
    } else {
        "0"
    };
    let footer_inside = if pbf.attr & 0x0000_0004 != 0 {
        "1"
    } else {
        "0"
    };
    format!(
        r#"<hp:pageBorderFill type="{ty}" borderFillIDRef="{}" textBorder="{}" headerInside="{}" footerInside="{}" fillArea="{}"><hp:offset left="{}" right="{}" top="{}" bottom="{}"/></hp:pageBorderFill>"#,
        pbf.border_fill_id,
        text_border,
        header_inside,
        footer_inside,
        fill_area,
        pbf.spacing_left,
        pbf.spacing_right,
        pbf.spacing_top,
        pbf.spacing_bottom,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{CharShapeRef, Paragraph};

    /// [#XXXX] `<hp:pageNum formatType="...">`의 원문자(circled digit) 값은 OWPML Core
    /// 스키마 NumberType1 표기인 "CIRCLED_DIGIT"이어야 한다. 종전엔 "CIRCLE_DIGIT"(D 없음)
    /// 오탈자로 방출돼 한컴이 값을 인식하지 못했다.
    #[test]
    fn page_num_circled_digit_format_reflects_spec_spelling() {
        use crate::model::control::PageNumberPos;
        let mut pn = PageNumberPos::default();
        pn.format = 1; // circled digit
        let xml = render_page_num(&pn);
        assert!(
            xml.contains(r#"formatType="CIRCLED_DIGIT""#),
            "pageNum formatType 이 스펙 철자(CIRCLED_DIGIT)여야 함: {xml}"
        );
        assert!(
            !xml.contains(r#"formatType="CIRCLE_DIGIT""#),
            "CIRCLE_DIGIT 오탈자 잔존 금지: {xml}"
        );
    }

    /// 쪽 번호 시작 쪽 컨트롤(`pgct`)은 `<hp:ctrl><hp:pageNumCtrl>` 로 나가야 한다.
    ///
    /// 구역 속성의 `pageStartsOn`(`<hp:secPr>`)과는 다른 자리다 — 이쪽은 문단 위
    /// 8유닛 슬롯을 차지하는 컨트롤이라, 방출하지 않으면 축이 그만큼 짧아진다.
    #[test]
    fn page_num_ctrl_is_emitted_as_a_ctrl_element() {
        let mut ctx = SerializeContext::default();
        for (want, text) in [
            (PageStartsOn::Both, "BOTH"),
            (PageStartsOn::Even, "EVEN"),
            (PageStartsOn::Odd, "ODD"),
        ] {
            let mut out = String::new();
            render_control_slot(
                &mut out,
                &Control::PageNumCtrl(PageNumCtrl {
                    page_starts_on: want,
                }),
                &mut ctx,
            );
            assert_eq!(
                out,
                format!(r#"<hp:ctrl><hp:pageNumCtrl pageStartsOn="{text}"/></hp:ctrl>"#)
            );
        }
    }

    /// 제목 차례 표시는 `<hp:t>` 안 인라인 요소로 되살아나야 한다 — 스키마상
    /// `<hp:ctrl>` 이 아니라 `<hp:t>` 의 자식이다(ParaList XML schema.xml:238).
    #[test]
    fn title_mark_is_emitted_inside_hp_t() {
        let mut cursor = InlineCursor {
            title_marks: &[TitleMark {
                char_idx: 0,
                ignore: true,
            }],
            ..Default::default()
        };
        let xml = render_hp_t_content("가나", &[], &mut cursor);
        assert_eq!(xml, "<hp:t><hp:titleMark ignore=\"1\"/>가나</hp:t>");
    }

    /// `ignore="0"`(`Mign`)도 구별해 낸다 — 한글 2022 양방향 실측(06699).
    #[test]
    fn title_mark_ignore_off_round_trips() {
        let mut cursor = InlineCursor {
            title_marks: &[TitleMark {
                char_idx: 1,
                ignore: false,
            }],
            ..Default::default()
        };
        let xml = render_hp_t_content("가나", &[], &mut cursor);
        assert_eq!(xml, "<hp:t>가<hp:titleMark ignore=\"0\"/>나</hp:t>");
    }

    /// 표시가 주장하는 8유닛 슬롯을 슬롯 수에서 빼지 않으면 문단이 mismatch 경로로
    /// 떨어져 `<hp:linesegarray>` 가 통째로 빠진다 — F-절단군의 근인이다.
    #[test]
    fn title_mark_slot_is_not_counted_as_a_control_slot() {
        let mut para = Paragraph {
            text: "가나".to_string(),
            char_offsets: vec![8, 9],
            // 표시 8 + 글자 2 + 끝 마커 1
            char_count: 11,
            title_marks: vec![TitleMark {
                char_idx: 0,
                ignore: true,
            }],
            ..Default::default()
        };
        assert_eq!(
            inferred_control_slot_count(&para),
            0,
            "표시 슬롯은 controls[] 에 대응이 없으므로 차감돼야 한다"
        );

        para.title_marks.clear();
        assert_eq!(
            inferred_control_slot_count(&para),
            1,
            "표시를 모르면 같은 문단이 컨트롤 슬롯 1개를 주장한다(종전 동작)"
        );
    }

    #[test]
    fn equation_text_flow_reflects_ir() {
        use crate::model::control::Equation;
        use crate::model::shape::TextFlow;
        // 수식의 textFlow 가 IR(common.text_flow)에서 방출돼야 한다.
        // 종전엔 "BOTH_SIDES" 하드코딩으로 왕복 시 유실됐다(textWrap 은 이미 IR 구동).
        let mut eq = Equation::default();
        eq.common.text_flow = TextFlow::LeftOnly;
        let xml = render_equation(&eq);
        assert!(
            xml.contains(r#"textFlow="LEFT_ONLY""#),
            "수식 textFlow 이 IR 값이어야 함: {xml}"
        );
        assert!(
            !xml.contains(r#"textFlow="BOTH_SIDES""#),
            "BOTH_SIDES 하드코딩 잔존 금지"
        );
    }

    /// [Issue #2727] 수식의 lineMode(수식이 차지하는 범위)가 IR(EQEDIT attribute bit0)에서
    /// 방출돼야 한다. 종전엔 속성 자체를 내보내지 않아 LINE 설정이 왕복마다 CHAR 로 돌아갔다.
    /// 한컴 저장본은 값이 기본값(CHAR)이어도 예외 없이 baseUnit 과 font 사이에 기록한다.
    #[test]
    fn equation_line_mode_reflects_ir() {
        use crate::model::control::{Equation, EQUATION_LINE_MODE_BIT};

        let char_xml = render_equation(&Equation::default());
        assert!(
            char_xml.contains(r#"lineMode="CHAR""#),
            "기본값도 lineMode 속성을 방출해야 함(한컴 저장본 정합): {char_xml}"
        );

        let eq = Equation {
            attr: EQUATION_LINE_MODE_BIT,
            font_size: 1000,
            ..Default::default()
        };
        let line_xml = render_equation(&eq);
        assert!(
            line_xml.contains(r#"baseUnit="1000" lineMode="LINE" font="""#),
            "lineMode 는 IR 값으로, 한컴과 같은 baseUnit·font 사이 자리에 와야 함: {line_xml}"
        );
        assert!(
            !line_xml.contains(r#"lineMode="CHAR""#),
            "CHAR 하드코딩 잔존 금지"
        );
    }

    /// [Issue #2840] 수식의 lock(개체 잠금)이 IR(common.locked)에서 방출돼야 한다.
    /// 종전엔 파서가 lock 속성을 읽지 않아 하드코딩 "0"으로 왕복마다 잠금이 풀렸다.
    #[test]
    fn equation_lock_reflects_ir() {
        use crate::model::control::Equation;

        let mut eq = Equation::default();
        eq.common.locked = true;
        let xml = render_equation(&eq);
        assert!(
            xml.contains(r#"lock="1""#),
            "locked=true 면 lock=\"1\" 을 방출해야 함(하드코딩 \"0\" 잔존 금지): {xml}"
        );
    }

    /// [Issue #2778] 수식의 크기 기준(`hp:sz` widthRelTo/heightRelTo)이 IR
    /// (common.width_criterion/height_criterion)에서 방출돼야 한다. 종전엔 "ABSOLUTE"
    /// 하드코딩이라 단/열/쪽 상대 크기 기준이 왕복마다 소실됐다.
    #[test]
    fn equation_size_criterion_reflects_ir() {
        use crate::model::control::Equation;
        use crate::model::shape::SizeCriterion;

        let mut eq = Equation::default();
        eq.common.width_criterion = SizeCriterion::Column;
        eq.common.height_criterion = SizeCriterion::Page;
        let xml = render_equation(&eq);
        assert!(
            xml.contains(r#"widthRelTo="COLUMN""#),
            "수식 widthRelTo 이 IR 값이어야 함: {xml}"
        );
        assert!(
            xml.contains(r#"heightRelTo="PAGE""#),
            "수식 heightRelTo 이 IR 값이어야 함: {xml}"
        );

        // 파서는 높이를 `parse_size_criterion(_, allow_column_para = false)` 로 읽어
        // 치역이 {PAPER, PAGE, ABSOLUTE} 3값뿐이므로, 방출도 같은 3값으로 접어야 멱등이다.
        let mut folded = Equation::default();
        folded.common.height_criterion = SizeCriterion::Column;
        assert!(
            render_equation(&folded).contains(r#"heightRelTo="ABSOLUTE""#),
            "heightRelTo 는 COLUMN/PARA 를 ABSOLUTE 로 접어야 왕복이 멱등"
        );
    }

    /// [Issue #2782] 수식의 allowOverlap(개체 겹침 허용)이 IR(common.allow_overlap)에서
    /// 방출돼야 한다. 종전 하드코딩 "0" 은 겹침 허용 설정을 왕복마다 껐다.
    #[test]
    fn equation_allow_overlap_reflects_ir() {
        use crate::model::control::Equation;

        let mut eq = Equation::default();
        eq.common.allow_overlap = true;
        let xml = render_equation(&eq);
        assert!(
            xml.contains(r#"allowOverlap="1""#),
            "allow_overlap=true 면 allowOverlap=\"1\" 을 방출해야 함: {xml}"
        );
        assert!(
            render_equation(&Equation::default()).contains(r#"allowOverlap="0""#),
            "기본값(false)은 종전대로 allowOverlap=\"0\" 이어야 함(회귀 방지)"
        );
    }

    /// [Issue #3543] 수식(`hp:equation`)은 스키마 비허용 `instid` 속성을 방출하지
    /// 않아야 한다. instid 는 도형 컴포넌트 공통(KS X 6101:2024 표 209) 소관이고
    /// equation 요소 정의(표 207·샘플 114)에는 없다 — 한컴 저장본도 수식에는
    /// 방출하지 않는다. 파서는 수식의 instid 를 버리므로 제거해도 왕복 불변.
    #[test]
    fn equation_omits_instid() {
        use crate::model::control::Equation;

        let xml = render_equation(&Equation::default());
        assert!(
            !xml.contains("instid"),
            "수식은 스키마 비허용 instid 를 방출하면 안 됨(KS X 6101 표 207): {xml}"
        );
        assert!(xml.contains(r#" id=""#), "id 속성은 유지돼야 함: {xml}");
    }

    /// [Issue #2790] legacy 공용 도형 경로(ellipse/arc/polygon/curve/chart/ole)의 textFlow
    /// 가 IR(common.text_flow)에서 방출돼야 한다. 종전 "BOTH_SIDES" 하드코딩으로 한쪽
    /// 배치(LEFT_ONLY 등) 설정이 왕복마다 소실됐다.
    #[test]
    fn common_shape_text_flow_reflects_ir() {
        use crate::model::shape::{CommonObjAttr, TextFlow};

        let mut c = CommonObjAttr::default();
        c.text_flow = TextFlow::LargestOnly;
        let mut ctx = SerializeContext::default();
        let xml = render_common_shape_xml("ellipse", &c, &None, None, &[], "", "", &mut ctx);
        assert!(
            xml.contains(r#"textFlow="LARGEST_ONLY""#),
            "공용 도형 textFlow 이 IR 값이어야 함: {xml}"
        );
        assert!(
            !xml.contains(r#"textFlow="BOTH_SIDES""#),
            "BOTH_SIDES 하드코딩 잔존 금지"
        );

        // 기본값(BothSides)은 종전 출력과 동일해야 한다(회귀 방지).
        let default_xml = render_common_shape_xml(
            "ellipse",
            &CommonObjAttr::default(),
            &None,
            None,
            &[],
            "",
            "",
            &mut ctx,
        );
        assert!(
            default_xml.contains(r#"textFlow="BOTH_SIDES""#),
            "기본값은 BOTH_SIDES 를 유지해야 함: {default_xml}"
        );
    }

    /// [Issue #1944] legacy 공용 도형 경로(polygon/ellipse/arc/curve)가 도형 내
    /// 글상자(drawText) 문단을 방출해야 한다 — 종전 누락으로 도형 안 텍스트 소실.
    #[test]
    fn common_shape_emits_draw_text_for_legacy_shapes() {
        use crate::model::shape::{CommonObjAttr, DrawingObjAttr, TextBox};

        let mut para = Paragraph::default();
        para.text = "마름모라벨".to_string();

        let drawing = DrawingObjAttr {
            text_box: Some(TextBox {
                paragraphs: vec![para],
                ..Default::default()
            }),
            ..Default::default()
        };
        let c = CommonObjAttr::default();
        let mut ctx = SerializeContext::default();

        let xml =
            render_common_shape_xml("polygon", &c, &None, Some(&drawing), &[], "", "", &mut ctx);
        assert!(
            xml.contains("<hp:drawText"),
            "polygon 도형이 drawText 를 방출해야 함: {xml}"
        );
        assert!(
            xml.contains("마름모라벨"),
            "글상자 문단 텍스트가 보존되어야 함"
        );
        // 빈 글상자는 미방출 (rect 경로와 동일 계약).
        let empty = DrawingObjAttr {
            text_box: Some(TextBox::default()),
            ..Default::default()
        };
        let xml_empty =
            render_common_shape_xml("polygon", &c, &None, Some(&empty), &[], "", "", &mut ctx);
        assert!(
            !xml_empty.contains("<hp:drawText"),
            "빈 글상자는 drawText 를 방출하지 않아야 함"
        );
    }

    /// [#2726] 공용 도형 경로(ellipse/arc/polygon/curve/chart)의 `hp:sz` 가 IR 의
    /// 크기 기준·크기 보호를 보존해야 한다. 종전엔 `widthRelTo`/`heightRelTo` 가
    /// `"ABSOLUTE"` 리터럴이고 `protect` 는 아예 미방출이었다.
    #[test]
    fn issue2726_common_shape_sz_preserves_criteria_and_protect() {
        use crate::model::shape::{CommonObjAttr, DrawingObjAttr};

        let c = CommonObjAttr {
            width: 4000,
            height: 3000,
            width_criterion: SizeCriterion::Column,
            height_criterion: SizeCriterion::Page,
            size_protect: true,
            ..Default::default()
        };
        let drawing = DrawingObjAttr::default();
        let mut ctx = SerializeContext::default();

        for tag in ["ellipse", "arc", "polygon", "curve", "chart"] {
            let xml =
                render_common_shape_xml(tag, &c, &None, Some(&drawing), &[], "", "", &mut ctx);
            assert!(
                xml.contains(r#"widthRelTo="COLUMN""#),
                "{tag}: 너비 기준 COLUMN 이 보존되어야 함: {xml}"
            );
            assert!(
                xml.contains(r#"heightRelTo="PAGE""#),
                "{tag}: 높이 기준 PAGE 가 보존되어야 함: {xml}"
            );
            assert!(
                xml.contains(r#"protect="1""#),
                "{tag}: 크기 보호가 보존되어야 함: {xml}"
            );
        }
    }

    /// [#2726] `protect` 는 값이 0 이어도 **속성 자체가** 방출되어야 한다.
    /// `samples/hwpx` 실제 한글 파일 60개의 `hp:sz` 1583개가 **전부** `protect` 를 갖는다
    /// (1583/1583). 종전 미방출은 그중 `hp:polygon` 150개(8파일)에서 한컴 원본 대비
    /// 구조 이탈을 만들었다.
    #[test]
    fn issue2726_common_shape_sz_always_emits_protect_attribute() {
        use crate::model::shape::{CommonObjAttr, DrawingObjAttr};

        let c = CommonObjAttr {
            size_protect: false,
            ..Default::default()
        };
        let drawing = DrawingObjAttr::default();
        let mut ctx = SerializeContext::default();

        let xml =
            render_common_shape_xml("polygon", &c, &None, Some(&drawing), &[], "", "", &mut ctx);
        assert!(
            xml.contains(r#"protect="0""#),
            "size_protect=false 여도 protect=\"0\" 속성이 방출되어야 함: {xml}"
        );
    }

    /// [#2726] `heightRelTo` 는 파서(`parse_size_criterion(_, allow_column_para=false)`)의
    /// **정확한 역**이어야 한다. 파서 치역이 `{PAPER, PAGE, ABSOLUTE}` 3값뿐이므로
    /// 방출도 절대 `COLUMN`/`PARA` 를 내면 안 된다 — 내면 되읽기에서 `Absolute` 로 접혀
    /// 왕복이 비-멱등이 된다. 5값 전수 대조로 못 박는다.
    #[test]
    fn issue2726_height_criterion_never_emits_column_or_para() {
        use crate::model::shape::{CommonObjAttr, DrawingObjAttr};

        let cases = [
            (SizeCriterion::Paper, "PAPER"),
            (SizeCriterion::Page, "PAGE"),
            (SizeCriterion::Column, "ABSOLUTE"),
            (SizeCriterion::Para, "ABSOLUTE"),
            (SizeCriterion::Absolute, "ABSOLUTE"),
        ];
        let drawing = DrawingObjAttr::default();
        let mut ctx = SerializeContext::default();

        for (criterion, expected) in cases {
            assert_eq!(
                height_criterion_str(criterion),
                expected,
                "높이 기준 {criterion:?} 는 {expected} 로 방출되어야 함"
            );

            let c = CommonObjAttr {
                height_criterion: criterion,
                ..Default::default()
            };
            let xml = render_common_shape_xml(
                "polygon",
                &c,
                &None,
                Some(&drawing),
                &[],
                "",
                "",
                &mut ctx,
            );
            assert!(
                xml.contains(&format!(r#"heightRelTo="{expected}""#)),
                "{criterion:?} → heightRelTo=\"{expected}\" 이어야 함: {xml}"
            );
            assert!(
                !xml.contains(r#"heightRelTo="COLUMN""#) && !xml.contains(r#"heightRelTo="PARA""#),
                "heightRelTo 는 COLUMN/PARA 를 방출하면 안 됨: {xml}"
            );
        }
    }

    /// [#2726] `widthRelTo` 는 5값 전부를 그대로 방출한다 — 파서
    /// `parse_size_criterion(_, allow_column_para=true)` 의 정확한 역.
    #[test]
    fn issue2726_width_criterion_emits_all_five_values() {
        let cases = [
            (SizeCriterion::Paper, "PAPER"),
            (SizeCriterion::Page, "PAGE"),
            (SizeCriterion::Column, "COLUMN"),
            (SizeCriterion::Para, "PARA"),
            (SizeCriterion::Absolute, "ABSOLUTE"),
        ];
        for (criterion, expected) in cases {
            assert_eq!(
                size_criterion_str(criterion),
                expected,
                "너비 기준 {criterion:?} 는 {expected} 로 방출되어야 함"
            );
        }
    }

    /// [Task #1627] empty-text(객체-only) 문단에서 bookmark 는 문단 시작으로 끌려가지 않고
    /// para.controls 순서대로 in-order 방출되어야 한다(원본 컨트롤 순서 보존).
    #[test]
    fn task1627_empty_para_bookmark_serialized_after_preceding_table() {
        use crate::model::control::{Bookmark, Control};

        let mut para = Paragraph::default();
        // empty text, controls 순서 = [Table, Bookmark]
        para.controls = vec![
            Control::Table(Box::default()),
            Control::Bookmark(Bookmark {
                name: "BM_AFTER_TBL".to_string(),
            }),
        ];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        let tbl_pos = xml.find("<hp:tbl");
        let bm_pos = xml.find("BM_AFTER_TBL");
        assert!(
            tbl_pos.is_some(),
            "table 직렬화 필요: {}",
            &xml[..400.min(xml.len())]
        );
        assert!(bm_pos.is_some(), "bookmark 직렬화 필요");
        assert!(
            tbl_pos < bm_pos,
            "bookmark 가 table 뒤(원본 순서)에 와야 함 — 문단 시작 강제 회귀 (#1627)"
        );
    }

    fn make_doc_with_paragraph(para: Paragraph) -> (Document, Section) {
        let mut section = Section::default();
        section.paragraphs.push(para);
        let mut doc = Document::default();
        doc.sections.push(section.clone());
        (doc, section)
    }

    #[test]
    fn footnote_endnote_beneath_text_reflects_ir() {
        // beneathText(본문 아래 바로 이어 출력)가 IR 에서 방출돼야 한다.
        // 종전엔 템플릿 "0" 고정이라 저장할 때마다 이 설정이 꺼졌다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.footnote_shape.print_inline_after_text = true;
        section.section_def.endnote_shape.print_inline_after_text = true;
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:placement place="EACH_COLUMN" beneathText="1"/>"#),
            "각주 beneathText 가 IR 값이어야 함"
        );
        assert!(
            xml.contains(r#"<hp:placement place="END_OF_DOCUMENT" beneathText="1"/>"#),
            "미주 beneathText 가 IR 값이어야 함"
        );
        assert!(
            !xml.contains(r#"beneathText="0""#),
            "템플릿 기본 beneathText=0 잔존 금지"
        );
    }

    /// [#2779] placement 의 `place`(배치 방법)가 템플릿 상수가 아니라 IR 값으로
    /// 방출돼야 한다. 각주/미주는 같은 코드 공간(attr bits 8-9)을 쓰지만 OWPML 토큰이
    /// 서로 달라, 컨텍스트 키 역매핑이 필요하다:
    ///   각주 1=MERGED_COLUMN(통단) / 2=RIGHT_MOST_COLUMN, 미주 1=END_OF_SECTION.
    /// 재파싱까지 확인해 rhwp 자기 왕복이 무손실인지 본다.
    #[test]
    fn issue2779_note_place_reflects_ir() {
        use crate::model::footnote::FootnotePlacement;
        use crate::parser::hwpx::section::parse_hwpx_section;

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.footnote_shape.placement = FootnotePlacement::BelowText;
        section.section_def.endnote_shape.placement = FootnotePlacement::BelowText;
        doc.sections = vec![section.clone()];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:placement place="MERGED_COLUMN" beneathText="0"/>"#),
            "각주 통단 배치가 IR 값으로 방출돼야 함"
        );
        assert!(
            xml.contains(r#"<hp:placement place="END_OF_SECTION" beneathText="0"/>"#),
            "미주 구역끝 배치가 IR 값으로 방출돼야 함"
        );
        assert!(
            !xml.contains(r#"place="EACH_COLUMN""#),
            "템플릿 상수 EACH_COLUMN 잔존 금지"
        );
        assert!(
            !xml.contains(r#"place="END_OF_DOCUMENT""#),
            "템플릿 상수 END_OF_DOCUMENT 잔존 금지"
        );

        let reparsed = parse_hwpx_section(&xml).unwrap();
        assert_eq!(
            reparsed.section_def.footnote_shape.placement,
            FootnotePlacement::BelowText,
            "각주 배치가 재파싱에서 보존돼야 함"
        );
        assert_eq!(
            reparsed.section_def.endnote_shape.placement,
            FootnotePlacement::BelowText,
            "미주 배치가 재파싱에서 보존돼야 함"
        );
    }

    /// #4403: `tab_extended` 항목이 없던 "암묵적 기본 탭"은 HWPX 라운드트립(직렬화→재파싱)
    /// 후에도 `tab_extended` 가 비어 있어야 한다. 예전에는 폴백으로 고정 상수
    /// `width="4000"` 를 방출해 재파싱 시 `tab_extended` 항목이 새로 생겼다 — 렌더러는
    /// 그 항목이 있으면 폭을 "실제 계산된 값"으로 신뢰해(`total + width`) 문단의 진짜
    /// `TabDef`(예: 목차의 우측 정렬 쪽번호 탭)를 무시하고 커서 위치와 무관한 고정 거리만
    /// 전진시킨다(실측: `samples/SO-SUEOP.hwp` 자기 라운드트립 목차 페이지 최대 변위 470px).
    /// 지금은 "데이터 없음" 마커(`width="0"`)를 방출하고, 파서가 그 마커를 인식해
    /// `tab_extended` 항목을 만들지 않는다 — 렌더러가 실제 `TabDef` 기준으로 탭 정지를
    /// 다시 계산하게 한다.
    #[test]
    fn issue4403_implicit_tab_stays_empty_after_hwpx_roundtrip() {
        use crate::parser::hwpx::section::parse_hwpx_section;

        let mut para = Paragraph::default();
        para.text = "I.소설의 이해\t3".to_string();
        assert!(
            para.tab_extended.is_empty(),
            "전제: 원본은 tab_extended 데이터가 없는 암묵적 기본 탭"
        );
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:tab width="0" leader="0" type="1"/>"#),
            "데이터 없음 탭은 width=0 마커로 방출돼야 함: {}",
            &xml[..600.min(xml.len())]
        );

        let reparsed = parse_hwpx_section(&xml).unwrap();
        let reparsed_para = &reparsed.paragraphs[0];
        assert_eq!(reparsed_para.text, "I.소설의 이해\t3");
        assert!(
            reparsed_para.tab_extended.is_empty(),
            "width=0 마커 재파싱 후 tab_extended 는 비어 있어야 함(원본과 동일): {:?}",
            reparsed_para.tab_extended
        );
    }

    /// #4675: 고정폭 빈칸(U+2007)은 리터럴 문자가 아니라 `<hp:fwSpace/>` 요소로
    /// 직렬화돼야 한다. 리터럴 방출은 한글의 텍스트 추출·재조판 결과를 원본과 다르게
    /// 만든다(10k 스윕 TEXT_MISMATCH 의 76%). 왕복(직렬화→재파싱) 후 IR 텍스트는
    /// 동일해야 한다 — 파서가 요소를 같은 코드포인트로 되돌린다.
    ///
    /// 묶음 빈칸(U+00A0)은 **리터럴로 남긴다**. 한컴 원본은 U+2007 을 항상 요소로
    /// 쓰지만(실측 hwpx 300건: 요소 530 · 리터럴 0), U+00A0 은 요소 15 · 리터럴 9 로
    /// 섞여 쓴다 — IR 이 두 표기를 구분하지 못하는데 요소로 강제하면 리터럴이던 원본의
    /// 추출 텍스트에서 NBSP 가 사라진다(한글 2022 오라클 실측 23건).
    #[test]
    fn issue4675_fixed_width_space_serializes_as_element() {
        use crate::parser::hwpx::section::parse_hwpx_section;

        let mut para = Paragraph::default();
        para.text = "보\u{2007}도\u{2007}자\u{2007}료\u{00A0}끝".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert_eq!(
            xml.matches("<hp:fwSpace/>").count(),
            3,
            "U+2007 은 전부 fwSpace 요소로 방출돼야 함: {}",
            &xml[..800.min(xml.len())]
        );
        assert!(
            !xml.contains('\u{2007}'),
            "리터럴 U+2007 이 XML 에 남으면 안 됨"
        );
        assert!(
            xml.contains('\u{00A0}') && !xml.contains("<hp:nbSpace/>"),
            "U+00A0 은 리터럴 유지 — 요소로 바꾸면 한글 추출에서 사라진다"
        );

        let reparsed = parse_hwpx_section(&xml).unwrap();
        assert_eq!(
            reparsed.paragraphs[0].text, "보\u{2007}도\u{2007}자\u{2007}료\u{00A0}끝",
            "요소 왕복 후 IR 텍스트 불변"
        );
    }

    /// #4676: `<hp:curve>` 의 점은 `<hc:pt>` 나열이 아니라 `<hp:seg>` 체인으로 나가야 한다.
    /// `<hc:pt>` 로 저장하면 한글이 파일을 여는 도중 프로세스째 죽는다(COM RPC 0x800706BE).
    /// 한컴 원본 실측: `hp:curve` 는 seg 만 쓰고 `hc:pt` 는 한 번도 쓰지 않는다.
    /// HWP5 유래 구간 종류(LINE/CURVE)는 IR의 `segment_types`에서 나온다.
    #[test]
    fn issue4676_curve_emits_seg_chain_not_pts() {
        use crate::model::shape::{CommonObjAttr, CurveShape, DrawingObjAttr};
        use crate::model::Point;

        let curve = CurveShape {
            common: CommonObjAttr::default(),
            drawing: DrawingObjAttr::default(),
            points: vec![
                Point { x: 0, y: 100 },
                Point { x: 500, y: 0 },
                Point { x: 900, y: 250 },
            ],
            segment_types: vec![1, 0],
        };
        let mut ctx = SerializeContext::default();
        let xml = render_shape(&ShapeObject::Curve(curve), &mut ctx);

        assert!(
            !xml.contains("<hc:pt "),
            "curve 는 hc:pt 를 방출하면 안 된다(한글 크래시): {xml}"
        );
        assert!(
            xml.contains(r#"<hp:seg type="CURVE" x1="0" y1="100" x2="500" y2="0"/>"#),
            "첫 구간은 곡선: {xml}"
        );
        assert!(
            xml.contains(r#"<hp:seg type="LINE" x1="500" y1="0" x2="900" y2="250"/>"#),
            "둘째 구간은 직선(segment_types 보존): {xml}"
        );
        // 점 N 개 → 구간 N-1 개
        assert_eq!(xml.matches("<hp:seg ").count(), 2, "{xml}");
    }

    /// [#4676] HWPX `hp:seg type="CURVE"`는 HWP5의 cubic Bezier 구간 타입이 아니다.
    /// XML → IR → XML 경계에서 `segment_types=1`로 오매핑하면 renderer가 제어점 둘과
    /// 끝점을 한 구간으로 소비한다. HWPX 점 체인은 빈 HWP5 구간 타입으로 유지하면서
    /// 한글 호환 `hp:seg` 출력만 보존해야 한다.
    #[test]
    fn issue4676_hwpx_curve_chain_never_becomes_hwp5_bezier_segments() {
        use crate::parser::hwpx::section::parse_hwpx_section;

        let source = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:curve id="0" zOrder="0" numberingType="NONE" textWrap="TOP_AND_BOTTOM"
                textFlow="BOTH_SIDES" lock="0" href="" groupLevel="0" instid="1">
        <hp:offset x="0" y="0"/>
        <hp:orgSz width="100" height="100"/>
        <hp:curSz width="100" height="100"/>
        <hp:lineShape color="#000000" width="113" style="SOLID"/>
        <hp:seg type="CURVE" x1="0" y1="0" x2="10" y2="20"/>
        <hp:seg type="CURVE" x1="10" y1="20" x2="40" y2="30"/>
        <hp:seg type="CURVE" x1="40" y1="30" x2="90" y2="80"/>
      </hp:curve>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(source).expect("HWPX curve 파싱");
        let mut doc = Document::default();
        doc.sections.push(section.clone());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap())
            .expect("section XML");

        assert_eq!(
            xml.matches("<hp:seg ").count(),
            3,
            "점 체인 길이 보존: {xml}"
        );
        assert!(
            !xml.contains("<hc:pt "),
            "curve에는 hc:pt를 다시 쓰면 안 됨: {xml}"
        );

        let reparsed = parse_hwpx_section(&xml).expect("재파싱");
        let curve = reparsed.paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Shape(shape) => match shape.as_ref() {
                    ShapeObject::Curve(curve) => Some(curve),
                    _ => None,
                },
                _ => None,
            })
            .expect("curve shape");
        assert_eq!(curve.points.len(), 4, "첫 점과 세 segment 끝점이 남아야 함");
        assert!(
            curve.segment_types.is_empty(),
            "HWPX CURVE를 HWP5 Bezier 타입으로 재도입하면 안 됨: {:?}",
            curve.segment_types
        );
    }

    /// [#2779] 각주 코드 2(가장 오른쪽 단)는 각주 전용 토큰이 있으나, 미주에는 스키마상
    /// 대응 토큰이 없어 기본값 END_OF_DOCUMENT 로 강등한다(주석 참조).
    #[test]
    fn issue2779_note_place_right_column_footnote_only() {
        use crate::model::footnote::FootnotePlacement;

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.footnote_shape.placement = FootnotePlacement::RightColumn;
        section.section_def.endnote_shape.placement = FootnotePlacement::RightColumn;
        doc.sections = vec![section.clone()];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:placement place="RIGHT_MOST_COLUMN" beneathText="0"/>"#),
            "각주 오른쪽단 배치가 IR 값으로 방출돼야 함"
        );
        assert!(
            xml.contains(r#"<hp:placement place="END_OF_DOCUMENT" beneathText="0"/>"#),
            "미주는 스키마 밖 코드 2 를 기본값으로 강등해야 함"
        );
    }

    /// [#2779] 기본 SectionDef(placement=EachColumn, beneathText=0)의 출력은 템플릿과
    /// 동일해야 한다 — 무변경 가드.
    #[test]
    fn issue2779_note_place_keeps_template_defaults() {
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:placement place="EACH_COLUMN" beneathText="0"/>"#),
            "각주 기본 배치는 템플릿과 동일해야 함"
        );
        assert!(
            xml.contains(r#"<hp:placement place="END_OF_DOCUMENT" beneathText="0"/>"#),
            "미주 기본 배치는 템플릿과 동일해야 함"
        );
    }

    #[test]
    fn issue1984_footnote_shape_reflects_ir() {
        // [#1984] footNotePr 의 noteLine/noteSpacing 이 템플릿 기본값(aboveLine=850,
        // betweenNotes=283 등)이 아니라 IR FootnoteShape 값으로 방출돼야 한다. 미치환 시
        // 각주 zone 높이가 달라져 각주 있는 페이지의 표 분할·페이지 수가 갈린다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        let fs = &mut section.section_def.footnote_shape;
        fs.separator_margin_top = 1417; // aboveLine
        fs.note_spacing = 850; // belowLine
        fs.raw_unknown = 566; // betweenNotes
        fs.separator_line_width = 9; // 0.7 mm
        fs.separator_length = -2;
        fs.separator_line_type = 1; // SOLID
        fs.separator_color = 0x99_99_99; // #999999
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(
                r#"<hp:noteSpacing betweenNotes="566" belowLine="850" aboveLine="1417"/>"#
            ),
            "각주 noteSpacing 이 IR 값이어야 함: {}",
            &xml[..xml
                .find("footNote")
                .map(|i| i + 300)
                .unwrap_or(600)
                .min(xml.len())]
        );
        assert!(
            xml.contains(
                r##"<hp:noteLine length="-2" type="SOLID" width="0.7 mm" color="#999999"/>"##
            ),
            "각주 noteLine 이 IR 값이어야 함"
        );
        assert!(
            !xml.contains(r#"betweenNotes="283""#),
            "템플릿 기본 betweenNotes=283 잔존 금지"
        );
    }

    #[test]
    fn footnote_endnote_numbering_and_start_reflect_ir() {
        // 템플릿 기본(type="CONTINUOUS" newNum="1")이 아니라 IR FootnoteShape 의
        // 번호 종류·시작 번호로 방출돼야 한다(각주/미주 각각). fn/en 템플릿 문자열이
        // 동일하므로 위치 기반 치환이 정확히 각 슬롯을 채우는지도 함께 검증한다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.footnote_shape.numbering =
            crate::model::footnote::FootnoteNumbering::RestartPage;
        section.section_def.footnote_shape.start_number = 3;
        section.section_def.endnote_shape.numbering =
            crate::model::footnote::FootnoteNumbering::RestartSection;
        section.section_def.endnote_shape.start_number = 5;
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:numbering type="RESTART_PAGE" newNum="3"/>"#),
            "각주 numbering 이 IR 값이어야 함"
        );
        assert!(
            xml.contains(r#"<hp:numbering type="RESTART_SECTION" newNum="5"/>"#),
            "미주 numbering 이 IR 값이어야 함"
        );
        assert!(
            !xml.contains(r#"<hp:numbering type="CONTINUOUS" newNum="1"/>"#),
            "템플릿 기본 numbering 잔존 금지"
        );
    }

    #[test]
    fn issue2742_auto_num_format_reflects_ir() {
        // [#2742] footNotePr/endNotePr 의 autoNumFormat 5속성(type·userChar·prefixChar·
        // suffixChar·supscript)이 템플릿 상수가 아니라 IR FootnoteShape 값으로 방출돼야
        // 한다. 미치환 시 구역 각주/미주 모양이 저장마다 한컴 기본값으로 리셋돼, 실측
        // 코퍼스에서 미주 장식문자 「문…）」(17파일)와 위첨자 설정(1파일)이 사라졌다.
        // fn/en 템플릿 문자열이 동일하므로 위치 기반 치환이 각 슬롯을 정확히 채우는지도
        // 함께 검증한다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        let fs = &mut section.section_def.footnote_shape;
        fs.number_format = crate::model::footnote::NumberFormat::CircledDigit;
        fs.number_code_superscript = true;
        let es = &mut section.section_def.endnote_shape;
        es.number_format = crate::model::footnote::NumberFormat::UserChar;
        es.user_char = '★';
        es.prefix_char = '문';
        es.suffix_char = '）'; // U+FF09 전각 — 한컴 실물 표기
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(
                r#"<hp:autoNumFormat type="CIRCLED_DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="1"/>"#
            ),
            "각주 autoNumFormat 이 IR 값이어야 함: {}",
            &xml[..xml
                .find("footNotePr")
                .map(|i| i + 200)
                .unwrap_or(600)
                .min(xml.len())]
        );
        assert!(
            xml.contains(
                r#"<hp:autoNumFormat type="USER_CHAR" userChar="★" prefixChar="문" suffixChar="）" supscript="0"/>"#
            ),
            "미주 autoNumFormat 이 IR 값이어야 함(장식문자 보존)"
        );
        assert!(
            !xml.contains(TEMPLATE_AUTO_NUM_FORMAT),
            "템플릿 기본 autoNumFormat 잔존 금지"
        );
    }

    #[test]
    fn issue2742_auto_num_format_keeps_template_when_ir_unset() {
        // 파싱을 거치지 않은 기본 SectionDef(number_format=Digit, 장식문자 '\0',
        // supscript=false)에서는 템플릿과 바이트 동일해야 한다 — '\0' = 미지정 규약.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        assert_eq!(
            section.section_def.endnote_shape.suffix_char, '\0',
            "전제: 기본 장식문자는 '\\0'"
        );
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert_eq!(
            xml.matches(TEMPLATE_AUTO_NUM_FORMAT).count(),
            2,
            "IR 미설정 시 각주/미주 모두 템플릿 문자열 유지: {xml:.900}"
        );
    }

    #[test]
    fn hp_p_attrs_reflect_para_shape_id_and_style_id() {
        let mut para = Paragraph::default();
        para.para_shape_id = 7;
        para.style_id = 3;
        para.text = "hi".to_string();
        let (mut doc, section) = make_doc_with_paragraph(para);
        // style_id=3 이 등록되도록 스타일 4개 확보 (미등록 강등 #1933 회피).
        doc.doc_info.styles = vec![crate::model::style::Style::default(); 4];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"paraPrIDRef="7""#),
            "<hp:p> must reflect para_shape_id=7: {}",
            &xml[..200.min(xml.len())]
        );
        assert!(
            xml.contains(r#"styleIDRef="3""#),
            "<hp:p> must reflect style_id=3"
        );
    }

    /// [Issue #1933] 스타일 목록 밖 styleIDRef 는 기본(0)으로 강등되어 직렬화가
    /// 하드 실패하지 않는다 (한글 정합 — 열리는데 저장 불가 해소).
    #[test]
    fn out_of_range_style_id_downgraded_to_default() {
        let mut para = Paragraph::default();
        para.style_id = 96; // 스타일 목록(4개, idx 0..3) 밖
        para.text = "hi".to_string();
        let (mut doc, section) = make_doc_with_paragraph(para);
        doc.doc_info.styles = vec![crate::model::style::Style::default(); 4];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        // 참조 정합 단언 — 강등 없으면 styleIDRef:[96] 미등록으로 하드 실패한다.
        ctx.assert_all_refs_resolved()
            .expect("#1933: 미등록 styleIDRef 강등으로 참조 정합해야 함");
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"styleIDRef="0""#),
            "미등록 style_id=96 은 0 으로 강등되어야 함"
        );
        assert!(
            !xml.contains(r#"styleIDRef="96""#),
            "미등록 style_id 는 방출되지 않아야 함"
        );
    }

    #[test]
    fn secpr_emits_ir_text_direction_vertical() {
        // 파서는 textDirection="VERTICAL" → text_direction=1 로 읽지만 직렬화기가 재출력하지
        // 않아 세로쓰기 구역이 .hwpx 저장 시 HORIZONTAL 로 유실됐다. 1 이면 VERTICAL 방출.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, _) = make_doc_with_paragraph(para);
        doc.sections[0].section_def.text_direction = 1;
        let section = doc.sections[0].clone();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"textDirection="VERTICAL""#),
            "세로쓰기 구역은 textDirection=VERTICAL 로 방출돼야 함: {xml:.400}"
        );
        assert!(
            !xml.contains(r#"textDirection="HORIZONTAL""#),
            "secPr 의 HORIZONTAL 이 남아있으면 안 됨: {xml:.400}"
        );
    }

    #[test]
    fn secpr_keeps_horizontal_when_text_direction_unset() {
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, _) = make_doc_with_paragraph(para);
        let section = doc.sections[0].clone();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"textDirection="HORIZONTAL""#),
            "기본(text_direction=0) 은 HORIZONTAL 유지: {xml:.400}"
        );
    }

    #[test]
    fn secpr_emits_ir_tab_stop_grid_and_start_num() {
        // 템플릿 secPr 의 grid / startNum / tabStop 은 [#1987] 의 spaceColumns·
        // outlineShapeIDRef 치환에서 빠져 있어, IR 값과 무관하게 늘 템플릿 상수로
        // 나갔다. 열었다 저장하면 구역 그리드·시작 번호·기본 탭 폭이 사라진다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, _) = make_doc_with_paragraph(para);
        {
            let sd = &mut doc.sections[0].section_def;
            sd.default_tab_spacing = 4000;
            sd.line_grid = 1200;
            sd.char_grid = 900;
            sd.page_num = 47;
            sd.picture_num = 3;
            sd.table_num = 5;
            sd.equation_num = 7;
        }
        let section = doc.sections[0].clone();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"tabStop="4000""#),
            "기본 탭 폭이 IR 값이어야 함: {xml:.600}"
        );
        assert!(
            xml.contains(r#"<hp:grid lineGrid="1200" charGrid="900" wonggojiFormat="0"/>"#),
            "구역 그리드가 IR 값이어야 함: {xml:.600}"
        );
        assert!(
            xml.contains(
                r#"<hp:startNum pageStartsOn="BOTH" page="47" pic="3" tbl="5" equation="7"/>"#
            ),
            "시작 번호가 IR 값이어야 함: {xml:.600}"
        );
    }

    #[test]
    fn secpr_keeps_template_tab_stop_when_ir_unset() {
        // SectionDef 는 derive(Default) 라 파싱을 거치지 않은 문서에서 0 이다.
        // 0 을 그대로 내보내면 탭 폭이 0 이 되어 지금보다 나빠지므로 템플릿 기본값을 유지한다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        assert_eq!(
            section.section_def.default_tab_spacing, 0,
            "전제: 기본값은 0"
        );
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"tabStop="8000""#),
            "IR 미설정 시 템플릿 상수 유지: {xml:.600}"
        );
    }

    #[test]
    fn secpr_emits_tab_stop_val_and_unit() {
        // [Finding 14] 원본 secPr 의 tabStopVal="4000" tabStopUnit="HWPUNIT" (한컴
        // 기본 탭 폭 상수) 이 직렬화에서 누락되지 않아야 한다. 순서는
        // tabStop → tabStopVal → tabStopUnit → outlineShapeIDRef.
        // [#1987] outlineShapeIDRef 는 이제 IR 값 치환 대상 — 기본 SectionDef 는
        // outline_numbering_id=0 이므로 "0" 이 방출된다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(
                r#"tabStop="8000" tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="0""#
            ),
            "secPr 에 tabStopVal/tabStopUnit 이 정확 순서로 있어야 함: {}",
            &xml[..600.min(xml.len())]
        );
    }

    #[test]
    fn issue1987_secpr_scalars_reflect_ir() {
        // [#1987] secPr 의 spaceColumns/outlineShapeIDRef 가 템플릿 하드코딩(1134/1)이
        // 아니라 IR 값으로 방출돼야 한다. 미치환 시 멀티구역 문서 후반부 렌더 붕괴.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.column_spacing = 1130;
        section.section_def.outline_numbering_id = 0;
        doc.sections = vec![section.clone()];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"spaceColumns="1130""#),
            "spaceColumns=1130 방출: {}",
            &xml[..400]
        );
        assert!(
            xml.contains(r#"outlineShapeIDRef="0""#),
            "outlineShapeIDRef=0 방출"
        );
        assert!(
            !xml.contains(r#"spaceColumns="1134""#),
            "템플릿 1134 잔존 금지"
        );
    }

    /// [#2779] secPr@memoShapeIDRef 가 템플릿 상수 "0" 이 아니라 IR 값으로 방출되고
    /// 재파싱까지 살아남아야 한다. 종전엔 3계층(모델/파서/직렬화기) 모두 결손이라
    /// 메모 모양 참조가 저장마다 0 으로 리셋됐다(실측 14 secPr/9 파일).
    #[test]
    fn issue2779_secpr_memo_shape_id_reflects_ir() {
        use crate::parser::hwpx::section::parse_hwpx_section;

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (mut doc, mut section) = make_doc_with_paragraph(para);
        section.section_def.memo_shape_id = 3;
        doc.sections = vec![section.clone()];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"memoShapeIDRef="3""#),
            "memoShapeIDRef 가 IR 값이어야 함: {}",
            &xml[..600.min(xml.len())]
        );
        assert!(
            !xml.contains(r#"memoShapeIDRef="0""#),
            "템플릿 상수 memoShapeIDRef=0 잔존 금지"
        );

        let reparsed = parse_hwpx_section(&xml).unwrap();
        assert_eq!(
            reparsed.section_def.memo_shape_id, 3,
            "memoShapeIDRef 가 재파싱에서 보존돼야 함"
        );
    }

    /// [#2779] 기본 SectionDef(memo_shape_id=0)의 출력은 템플릿과 동일해야 한다 —
    /// 무변경 가드.
    #[test]
    fn issue2779_secpr_memo_shape_id_keeps_template_default() {
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"outlineShapeIDRef="0" memoShapeIDRef="0""#),
            "기본 문서는 템플릿 그대로여야 함: {}",
            &xml[..600.min(xml.len())]
        );
    }

    #[test]
    fn section_root_declares_hwpunitchar_namespace() {
        // [Finding 15] hs:sec 루트의 xmlns:hwpunitchar 선언(원본에 존재, 미사용
        // 이지만 무손실 위해 보존)이 누락되지 않아야 한다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar""#),
            "hs:sec 에 xmlns:hwpunitchar 선언이 있어야 함"
        );
    }

    #[test]
    fn task1407_body_col_pr_reflects_ir_column_def() {
        // [#1407] 본문 첫 문단 IR 에 2단 ColumnDef 가 있으면 템플릿 하드코딩
        // colPr(colCount=1)이 IR 값(colCount=2)으로 치환돼야 한다. 미치환 시
        // 2단→1단 손실로 페이지 넘침(143E RT 1→2).
        use crate::model::page::{ColumnDef, ColumnType};
        let mut cd = ColumnDef::default();
        cd.column_type = ColumnType::Normal;
        cd.column_count = 2;
        cd.same_width = true;
        cd.spacing = 2268;

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.controls.push(Control::ColumnDef(cd));

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"colCount="2""#) && xml.contains(r#"sameGap="2268""#),
            "본문 colPr 이 IR 2단 정의로 치환돼야 함: {}",
            &xml[..900.min(xml.len())]
        );
        assert!(
            !xml.contains(r#"colCount="1""#),
            "하드코딩 colCount=1 이 남으면 안 됨 (회귀 가드): {}",
            &xml[..900.min(xml.len())]
        );
    }

    #[test]
    fn task1407_single_column_doc_unaffected() {
        // ColumnDef IR 이 없는 문단(단일 단)은 템플릿 colCount=1 유지 — 회귀 없음.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"colCount="1""#),
            "ColumnDef 없으면 템플릿 colCount=1 유지"
        );
    }

    #[test]
    fn hp_run_reflects_first_char_shape_id() {
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 42,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="42"><hp:t>hello</hp:t>"#),
            "first run must use char_shape_id 42, xml excerpt around <hp:t>: {:?}",
            xml.find("<hp:t>")
                .map(|i| &xml[i.saturating_sub(50)..(i + 50).min(xml.len())])
        );
    }

    // ---------- #1382: autoNum placeholder 슬롯 ----------

    /// autoNum IR 규약 문단: 텍스트 "가·placeholder·나", offsets [0,1,9].
    fn autonum_para() -> Paragraph {
        let mut para = Paragraph::default();
        para.text = "가 나".to_string(); // 가(0) + placeholder 공백(1) + 나(9)
        para.char_offsets = vec![0, 1, 9];
        para.char_count = 11; // offsets 축 총 10 + 끝 마커 1
        para.controls
            .push(crate::model::control::Control::AutoNumber(
                crate::model::control::AutoNumber {
                    number_type: crate::model::control::AutoNumberType::Footnote,
                    ..Default::default()
                },
            ));
        para
    }

    #[test]
    fn task1382_inference_counts_autonum_placeholder_slot() {
        // placeholder 가 8유닛 중 1유닛을 점유해 잉여가 7로 측정되는 패턴 —
        // autoNum 보정으로 슬롯 1개로 집계되어야 한다 (종전 0 → mismatch 경로).
        assert_eq!(inferred_control_slot_count(&autonum_para()), 1);
    }

    #[test]
    fn task1382_autonum_slot_emitted_at_placeholder() {
        // ctrl 이 placeholder 원위치(mid-text)에 방출되고 placeholder 는 텍스트로
        // 내보내지 않는다 (한컴 원본 XML 동형).
        let (doc, section) = make_doc_with_paragraph(autonum_para());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains("<hp:t>가</hp:t><hp:ctrl><hp:autoNum"),
            "ctrl 은 placeholder 위치(가 직후)에 방출: {}",
            xml
        );
        assert!(
            xml.contains("</hp:ctrl><hp:t>나</hp:t>"),
            "placeholder 공백은 텍스트로 미방출, 후속 텍스트만 이어짐: {}",
            xml
        );
        assert!(
            !xml.contains("<hp:t>가 나</hp:t>"),
            "종전 결함(슬롯 끝 방출 + placeholder 텍스트 이중 방출) 재발 금지"
        );
    }

    #[test]
    fn task1382_synthetic_autonum_without_placeholder_keeps_legacy_path() {
        // 합성 IR(placeholder/offset 없는 편집기 생성 문단)은 보정 후에도 추론 0 —
        // 기존 mismatch 경로(끝 방출) 유지로 회귀 없음.
        let mut para = Paragraph::default();
        para.text = "가나".to_string();
        para.char_offsets = vec![0, 1];
        para.char_count = 3;
        para.controls
            .push(crate::model::control::Control::AutoNumber(
                crate::model::control::AutoNumber::default(),
            ));
        assert_eq!(inferred_control_slot_count(&para), 0);
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains("<hp:t>가나</hp:t><hp:ctrl><hp:autoNum"),
            "합성 IR 은 텍스트 후 끝 방출(기존 동작): {}",
            xml
        );
    }

    // ---------- #1388: replace_page_pr — secPr 페이지 여백 원본 보존 ----------

    #[test]
    fn task1388_page_margin_reflects_page_def() {
        let mut para = Paragraph::default();
        para.text = "m".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        let pd = &mut section.section_def.page_def;
        pd.width = 59527;
        pd.height = 84189;
        pd.margin_header = 4251;
        pd.margin_footer = 4251;
        pd.margin_gutter = 0;
        pd.margin_left = 7086;
        pd.margin_right = 14173;
        pd.margin_top = 4251;
        pd.margin_bottom = 4251;
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(
                r#"<hp:margin header="4251" footer="4251" gutter="0" left="7086" right="14173" top="4251" bottom="4251"/>"#
            ),
            "hp:margin must reflect IR PageDef 7 fields (온새미로 sec0 실측값)"
        );
        assert!(
            !xml.contains(r#"left="8504""#),
            "template margin value must not survive"
        );
        // #1166 동적화 회귀 없음 — width/height 동반 검증.
        assert!(xml.contains(r#"width="59527" height="84189""#));
    }

    #[test]
    fn task1388_gutter_type_reflects_binding() {
        use crate::model::page::BindingMethod;
        for (binding, expected) in [
            (BindingMethod::SingleSided, r#"gutterType="LEFT_ONLY""#),
            (BindingMethod::DuplexSided, r#"gutterType="LEFT_RIGHT""#),
            (BindingMethod::TopFlip, r#"gutterType="TOP_BOTTOM""#),
        ] {
            let mut para = Paragraph::default();
            para.text = "g".to_string();
            let (doc, mut section) = make_doc_with_paragraph(para);
            section.section_def.page_def.binding = binding;
            let mut ctx = SerializeContext::collect_from_document(&doc);
            let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
            let xml = std::str::from_utf8(&bytes).unwrap();
            assert!(
                xml.contains(expected),
                "binding={:?} must emit {}",
                binding,
                expected
            );
        }
    }

    #[test]
    fn task1388_template_mismatch_keeps_original() {
        // 템플릿 anchor 가 없는 입력 — 원본 유지(silent no-op, 회귀 방지 정책).
        let mut pd = crate::model::page::PageDef::default();
        pd.margin_left = 1234;
        let xml = r#"<hp:pagePr landscape="WIDELY" width="1" height="2" gutterType="LEFT_ONLY"><hp:margin header="0" footer="0" gutter="0" left="9" right="9" top="9" bottom="9"/></hp:pagePr>"#;
        assert_eq!(replace_page_pr(xml, &pd), xml);
    }

    #[test]
    fn task1388_template_anchor_present_in_template() {
        // 템플릿 변경 시 silent no-op 으로 빠지지 않도록 anchor 존재를 직접 보장한다.
        let pd = crate::model::page::PageDef {
            margin_header: 1,
            margin_footer: 2,
            margin_gutter: 3,
            margin_left: 4,
            margin_right: 5,
            margin_top: 6,
            margin_bottom: 7,
            ..Default::default()
        };
        let out = replace_page_pr(EMPTY_SECTION_XML, &pd);
        assert!(
            out.contains(
                r#"<hp:margin header="1" footer="2" gutter="3" left="4" right="5" top="6" bottom="7"/>"#
            ),
            "EMPTY_SECTION_XML 의 margin anchor 치환이 실패 — 템플릿이 변경됐는지 확인"
        );
    }

    #[test]
    fn page_break_paragraph_emits_attr() {
        let mut para = Paragraph::default();
        para.text = "p1".to_string();
        para.column_type = crate::model::paragraph::ColumnBreakType::Page;
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"pageBreak="1""#),
            "pageBreak must be 1 for Page column_type"
        );
        assert!(xml.contains(r#"columnBreak="0""#));
    }

    #[test]
    fn default_paragraph_keeps_zero_attrs() {
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(xml.contains(r#"paraPrIDRef="0""#));
        assert!(xml.contains(r#"styleIDRef="0""#));
        // char_shapes 가 비어있으면 fallback 0
        assert!(xml.contains(r#"<hp:run charPrIDRef="0">"#));
    }

    #[test]
    fn additional_paragraphs_use_their_own_char_shape() {
        let mut p1 = Paragraph::default();
        p1.text = "first".to_string();
        p1.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 5,
        });
        let mut p2 = Paragraph::default();
        p2.text = "second".to_string();
        p2.para_shape_id = 2;
        p2.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 6,
        });
        let mut section = Section::default();
        section.paragraphs.push(p1);
        section.paragraphs.push(p2);
        let mut doc = Document::default();
        doc.sections.push(section.clone());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // 두 번째 문단: paraPrIDRef=2, charPrIDRef=6
        assert!(xml.contains(r#"paraPrIDRef="2""#));
        assert!(
            xml.matches(r#"charPrIDRef="6""#).count() >= 1,
            "second paragraph must emit charPrIDRef=6"
        );
    }

    // ---------- #177 Stage 2: IR 기반 lineseg 출력 ----------

    use crate::model::paragraph::LineSeg;

    #[test]
    fn task177_lineseg_reflects_ir_values() {
        // IR에 담긴 lineseg 값이 XML 속성에 그대로 반영되는지 확인.
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.line_segs.push(LineSeg {
            text_start: 0,
            vertical_pos: 5000,
            line_height: 1200,
            text_height: 1100,
            baseline_distance: 900,
            line_spacing: 700,
            column_start: 100,
            segment_width: 50000,
            tag: 999,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(xml.contains(r#"<hp:lineseg textpos="0" vertpos="5000" vertsize="1200" textheight="1100" baseline="900" spacing="700" horzpos="100" horzsize="50000" flags="999"/>"#),
            "lineseg must reflect IR values exactly, got XML: {}",
            &xml[xml.find("<hp:lineseg").unwrap_or(0)..(xml.find("<hp:lineseg").unwrap_or(0) + 200).min(xml.len())]);
    }

    #[test]
    fn task177_multiple_linesegs_preserved_in_order() {
        let mut para = Paragraph::default();
        para.text = "three\nlines\nhere".to_string();
        for (i, (tp, vp, lh)) in [(0u32, 0i32, 1000), (6, 1500, 1200), (12, 3100, 1100)]
            .iter()
            .enumerate()
        {
            let _ = i;
            para.line_segs.push(LineSeg {
                text_start: *tp,
                vertical_pos: *vp,
                line_height: *lh,
                text_height: *lh,
                baseline_distance: 850,
                line_spacing: 600,
                column_start: 0,
                segment_width: 42520,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            });
        }
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // 3개 lineseg 모두 출력되고 각각의 vertsize 값이 IR 값과 일치
        assert_eq!(xml.matches("<hp:lineseg ").count(), 3);
        assert!(xml.contains(r#"textpos="0" vertpos="0" vertsize="1000""#));
        assert!(xml.contains(r#"textpos="6" vertpos="1500" vertsize="1200""#));
        assert!(xml.contains(r#"textpos="12" vertpos="3100" vertsize="1100""#));
    }

    #[test]
    fn text_positions_unshifted_detects_leading_and_mid_controls() {
        // mismatch 경로는 텍스트를 0 부터 연속으로 다시 쓴다. 원본에서도 연속이었으면
        // 좌표가 그대로라 lineseg 를 버릴 이유가 없다.
        let mut para = Paragraph::default();
        para.text = "abcde".to_string();

        // 컨트롤이 전부 텍스트 뒤 — 위치 안 밀림
        para.char_offsets = vec![0, 1, 2, 3, 4];
        assert!(text_positions_unshifted(&para));

        // 앞에 8유닛 슬롯 — 전부 밀림
        para.char_offsets = vec![8, 9, 10, 11, 12];
        assert!(!text_positions_unshifted(&para));

        // 중간에 슬롯 — 뒤쪽만 밀림
        para.char_offsets = vec![0, 1, 2, 11, 12];
        assert!(!text_positions_unshifted(&para));

        // char_offsets 없는 합성 IR 은 종전 동작 유지
        para.char_offsets = vec![];
        assert!(text_positions_unshifted(&para));
    }

    #[test]
    fn task1380_linesegarray_omitted_when_ir_empty() {
        // IR 의 line_segs 가 비어있으면 linesegarray 요소 자체를 방출 생략 (#1380).
        // 종전 fallback(vertsize=1000 합성)은 원본 무 → RT 유 비대칭을 만들었다.
        let mut para = Paragraph::default();
        para.text = "a\nb".to_string(); // 텍스트가 있어도 IR 에 lineseg 없으면 생략
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            !xml.contains("<hp:linesegarray"),
            "empty line_segs must omit linesegarray entirely: {}",
            xml
        );
        assert!(!xml.contains("<hp:lineseg "));
    }

    #[test]
    fn task1380_empty_section_omits_linesegarray() {
        // 문단이 없는 섹션(비파싱 IR)도 템플릿의 linesegarray 가 제거되어야 함 (#1380).
        let doc = crate::model::document::Document::default();
        let section = crate::model::document::Section::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(!xml.contains("<hp:linesegarray"));
    }

    #[test]
    fn task177_ir_lineseg_takes_precedence_over_text() {
        // text 의 \n 개수가 2개(lineseg 3개 기대)이지만 IR의 line_segs 는 1개만 있음.
        // IR 기반 출력이 우선 — 1개만 출력돼야 함.
        let mut para = Paragraph::default();
        para.text = "a\nb\nc".to_string(); // 3줄
        para.line_segs.push(LineSeg {
            text_start: 0,
            vertical_pos: 0,
            line_height: 2000, // IR 값
            text_height: 2000,
            baseline_distance: 1700,
            line_spacing: 300,
            column_start: 0,
            segment_width: 40000,
            tag: 0,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // IR 에 1개만 있으므로 lineseg 도 1개만 출력 (rhwp 는 원본 보존)
        assert_eq!(xml.matches("<hp:lineseg ").count(), 1);
        assert!(
            xml.contains(r#"vertsize="2000""#),
            "IR value 2000 must be used, not fallback 1000"
        );
    }

    // ---------- #1556: 고아(다단락) fieldEnd 방출 ----------

    #[test]
    fn task1556_orphan_field_end_emitted_at_text_end() {
        // 다단락 필드의 end 문단(para 0.16 동형): 텍스트 "끝." 뒤에 고아 fieldEnd 8유닛.
        use crate::model::paragraph::{CharShapeRef, OrphanFieldEnd};
        let mut para = Paragraph::default();
        para.text = "끝.".to_string();
        para.char_offsets = vec![0, 1];
        para.char_count = 11; // 텍스트 2 + fieldEnd 8 + 끝마커 1
        para.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 3,
            },
            CharShapeRef {
                start_pos: 10,
                char_shape_id: 30,
            },
        ];
        para.orphan_field_ends = vec![OrphanFieldEnd {
            char_idx: 2,
            begin_id_ref: 1_878_228_493,
            field_id: 627_272_811,
            begin_ctrl_id: 0,
        }];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="1878228493" fieldid="627272811"/>"#),
            "고아 fieldEnd 가 attrs 와 함께 방출되어야 함: {xml}"
        );
        // 텍스트가 fieldEnd 보다 앞에 나온다 (run 말미 fieldEnd 패턴).
        let t_pos = xml.find("끝.").expect("텍스트");
        let fe_pos = xml.find("<hp:fieldEnd").expect("fieldEnd");
        assert!(t_pos < fe_pos, "텍스트가 fieldEnd 앞: {xml}");
    }

    #[test]
    fn task1556_multipara_field_parse_serialize_parse_roundtrip() {
        // 합성 다단락 필드(begin=문단0, end=문단1) → parse → serialize → re-parse.
        // end 문단의 char_count/text/char_offsets/orphan 이 보존되어야 한다 (IR diff=0).
        use crate::parser::hwpx::section::parse_hwpx_section;
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="1878228493" type="CLICK_HERE" name="본문" fieldid="627272811"/></hp:ctrl><hp:t>본문</hp:t></hp:run>
  </hp:p>
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="3"><hp:t>끝.</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="1878228493" fieldid="627272811"/></hp:ctrl></hp:run>
    <hp:run charPrIDRef="30"><hp:t/></hp:run>
  </hp:p>
</hs:sec>"#;
        let sec1 = parse_hwpx_section(xml).unwrap();
        let mut doc = Document::default();
        doc.sections.push(sec1.clone());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&sec1, &doc, 0, &mut ctx).unwrap();
        let xml2 = String::from_utf8(bytes).unwrap();
        let sec2 = parse_hwpx_section(&xml2).unwrap();

        // 두 번째 문단(고아 fieldEnd 보유) IR 보존.
        let a = &sec1.paragraphs[1];
        let b = &sec2.paragraphs[1];
        assert_eq!(b.text, a.text, "text 보존");
        assert_eq!(
            b.char_count, a.char_count,
            "char_count 보존 (8유닛 소실 없음)"
        );
        assert_eq!(b.char_offsets, a.char_offsets, "char_offsets 보존");
        assert_eq!(
            b.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            a.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            "char_shape 경계 보존"
        );
        assert_eq!(b.orphan_field_ends.len(), 1, "고아 fieldEnd 재파싱 보존");
        assert_eq!(b.orphan_field_ends[0].begin_id_ref, 1_878_228_493);
    }

    #[test]
    fn task1556_orphan_field_end_zero_fieldid_omits_attr() {
        use crate::model::paragraph::OrphanFieldEnd;
        let mut para = Paragraph::default();
        para.text = "a".to_string();
        para.char_offsets = vec![0];
        para.char_count = 10; // 1 + 8 + 1
        para.orphan_field_ends = vec![OrphanFieldEnd {
            char_idx: 1,
            begin_id_ref: 42,
            field_id: 0,
            begin_ctrl_id: 0,
        }];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="42"/>"#),
            "field_id 0 이면 fieldid 속성 생략: {xml}"
        );
    }

    /// [bookmark-hyperlink] 같은 문단 내 짝(matched) HYPERLINK fieldBegin/fieldEnd 라운드트립 —
    /// fieldEnd 자신의 fieldid(=100)가 fieldBegin 의 id(=42, beginIDRef 로 사용)와 다를 때,
    /// 과거엔 emit_field_end 가 write_field_end(f.field_id) 만 호출해 fieldid 속성을 항상
    /// 누락시켰다(고아 fieldEnd 경로만 보존해 비대칭). fr.end_field_id 를 IR 에 보존하고
    /// 되돌려 쓰면 fieldid="100" 이 살아남아야 한다.
    #[test]
    fn bookmark_hyperlink_matched_field_end_preserves_own_fieldid() {
        let mut f = Field::default();
        f.field_type = FieldType::Hyperlink;
        f.field_id = 42;
        let mut para = Paragraph::default();
        para.text = "링크".to_string();
        para.char_count = 2 + 8 + 8 + 1; // text(2) + FIELD_BEGIN(8) + FIELD_END(8) + para_end(1)
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 2,
            control_idx: 0,
            end_field_id: 100,
            inner_slot_count: 0,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="42" fieldid="100"/>"#),
            "matched fieldEnd 는 beginIDRef(=fieldBegin id)와 별개로 자신의 fieldid 를 보존해야 함: {xml}"
        );
    }

    // ---------- #1289: Bookmark / Field dispatcher 연결 ----------

    use crate::model::control::{Bookmark, Control, Field, FieldType};
    use crate::model::paragraph::FieldRange;

    #[test]
    fn task1289_bookmark_emits_ctrl_wrapper() {
        // Bookmark는 슬롯 시스템이 위치를 추적할 수 없으므로 문단 시작에 배치한다.
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 6; // "hello"(5) + para_end(1)
        para.controls.push(Control::Bookmark(Bookmark {
            name: "test_bm".to_string(),
        }));
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:ctrl><hp:bookmark name="test_bm"/></hp:ctrl>"#),
            "bookmark must be wrapped in <hp:ctrl>: {}",
            &xml[..300.min(xml.len())]
        );
        assert!(xml.contains("hello"), "text must still be present");
    }

    #[test]
    fn task1391_memo_field_emits_parameters_and_sublist() {
        // MEMO 필드: parameters verbatim + subList 본문 방출, start/end 태그.
        let mut f = Field::default();
        f.field_type = FieldType::Memo;
        f.field_id = 7;
        f.raw_parameters_xml =
            Some(r#"<hp:parameters cnt="1" name=""><hp:stringParam name="ID">memo1</hp:stringParam></hp:parameters>"#.to_string());
        let mut memo_para = Paragraph::default();
        memo_para.text = "메모 본문".to_string();
        f.memo_paragraphs.push(memo_para);

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.char_count = 18; // fieldBegin(8) + x(1) + fieldEnd(8) + end(1)
        para.char_offsets = vec![8];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:stringParam name="ID">memo1</hp:stringParam>"#),
            "parameters verbatim 방출: {xml}"
        );
        assert!(xml.contains("<hp:t>메모 본문</hp:t>"), "메모 본문 방출");
        assert!(
            xml.contains("</hp:fieldBegin>"),
            "자식 있으면 start/end 태그 (자기닫힘 금지)"
        );
        // 순서: parameters → subList
        let pp = xml.find("<hp:parameters").unwrap();
        let sl = xml.find("<hp:subList").unwrap();
        assert!(pp < sl, "parameters 가 subList 보다 먼저");
    }

    #[test]
    fn memo_vertical_text_direction_roundtrips() {
        // [#task-m100] 세로쓰기 MEMO subList 는 파싱 시 textDirection="VERTICAL" 을
        // 보존해야 하며, 재직렬화 시 하드코딩된 "HORIZONTAL" 로 뒤집히면 안 된다.
        let mut f = Field::default();
        f.field_type = FieldType::Memo;
        f.field_id = 9;
        f.memo_text_direction = Some("VERTICAL".to_string());
        let mut memo_para = Paragraph::default();
        memo_para.text = "메모".to_string();
        f.memo_paragraphs.push(memo_para);

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.char_count = 18;
        para.char_offsets = vec![8];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 0,
            end_field_id: 0,
            inner_slot_count: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:subList id="" textDirection="VERTICAL""#),
            "세로쓰기 메모 subList 의 textDirection 보존: {xml}"
        );
    }

    #[test]
    fn task1391_field_without_params_keeps_empty_tag() {
        // parameters/memo 없는 필드는 기존 empty_tag 자기닫힘 유지 (회귀 방지).
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 5;
        let mut para = Paragraph::default();
        para.text = "y".to_string();
        para.char_count = 18;
        para.char_offsets = vec![8];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 0,
            ..Default::default()
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            !xml.contains("</hp:fieldBegin>"),
            "자식 없으면 자기닫힘 유지: {xml}"
        );
    }

    #[test]
    fn task1289_field_begin_end_roundtrip() {
        // HWPX 파서가 생성하는 구조 시뮬레이션:
        // fieldBegin(8 cu) + "hello"(5 cu) + fieldEnd(8 cu) + para_end(1 cu) = 22
        // para.text 에는 "hello"만 있고 char_offsets 가 +8 오프셋으로 시작한다.
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 99;

        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 22;
        para.char_offsets = vec![8, 9, 10, 11, 12];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 5,
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldBegin id="99" type="CLICK_HERE""#),
            "fieldBegin must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldEnd beginIDRef="99"/></hp:ctrl>"#),
            "fieldEnd must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(xml.contains("hello"), "field text must be present");

        // 순서 검증: fieldBegin < "hello" < fieldEnd
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let hello_pos = xml.find("hello").expect("hello");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        assert!(begin_pos < hello_pos, "fieldBegin must precede text");
        assert!(hello_pos < end_pos, "text must precede fieldEnd");
    }

    #[test]
    fn task1289_field_end_at_para_boundary() {
        // end_char_idx == text.len() 인 경우: 루프 내 감지 불가 → 루프 후 처리
        let mut f = Field::default();
        f.field_type = FieldType::Date;
        f.field_id = 7;

        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_count = 20; // fieldBegin(8) + "abc"(3) + fieldEnd(8) + para_end(1)
        para.char_offsets = vec![8, 9, 10];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3, // == text.len() → 루프 후 처리 경로
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="7"/>"#),
            "fieldEnd must be emitted even when end_char_idx == text.len(): {}",
            &xml[..400.min(xml.len())]
        );
    }

    // ---------- #1298: 0-length field range fieldBegin/fieldEnd 인터리빙 ----------

    #[test]
    fn task1298_zero_length_field_at_para_start() {
        // 0-length 필드 at position 0 (start=0, end=0):
        // HWP stream: fieldBegin(8cu) fieldEnd(8cu) "hello"(5cu) para_end(1cu) = 22cu
        // char_offsets: [16, 17, 18, 19, 20] (fieldBegin+fieldEnd 갭 16 이후 텍스트)
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 55;

        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 22;
        para.char_offsets = vec![16, 17, 18, 19, 20];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 0, // 0-length
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldBegin id="55""#),
            "fieldBegin must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldEnd beginIDRef="55"/></hp:ctrl>"#),
            "fieldEnd must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(xml.contains("hello"), "text must still be present");

        // 순서 검증: fieldBegin < fieldEnd < "hello"
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let hello_pos = xml.find("hello").expect("hello");
        assert!(begin_pos < end_pos, "fieldBegin must precede fieldEnd");
        assert!(
            end_pos < hello_pos,
            "fieldEnd must precede text for 0-length field"
        );
    }

    #[test]
    fn task1298_zero_length_field_mid_text() {
        // 0-length 필드 at position 3 (start=3, end=3), text="ABCDE":
        // HWP stream: A B C fieldBegin(8cu) fieldEnd(8cu) D E para_end
        // char_offsets: [0,1,2, 19,20] (D 앞에 16cu 갭)
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 77;

        let mut para = Paragraph::default();
        para.text = "ABCDE".to_string();
        para.char_count = 5 + 8 + 8 + 1; // text + fieldBegin + fieldEnd + para_end
        para.char_offsets = vec![0, 1, 2, 19, 20];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 3,
            end_char_idx: 3, // 0-length mid-text
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains("ABCDE") || (xml.contains("ABC") && xml.contains("DE")),
            "all text must be present: {}",
            &xml[..500.min(xml.len())]
        );

        // 순서 검증: "ABC" < fieldBegin < fieldEnd < "DE"
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        // ABC는 fieldBegin 앞에
        let abc_pos = xml.find('A').expect("A");
        // DE는 fieldEnd 뒤에 (fieldEnd 태그 닫힘 이후)
        let field_end_close =
            xml.find("fieldEnd").unwrap() + xml[xml.find("fieldEnd").unwrap()..].find('>').unwrap();
        let de_pos = xml[field_end_close..]
            .find('D')
            .map(|p| p + field_end_close)
            .expect("D after fieldEnd");

        assert!(abc_pos < begin_pos, "ABC must precede fieldBegin");
        assert!(begin_pos < end_pos, "fieldBegin must precede fieldEnd");
        assert!(end_pos < de_pos, "fieldEnd must precede DE");
    }

    // ---------- #1321: 빈 문단(text == "")의 0-length field 순서 ----------

    #[test]
    fn task1321_zero_length_field_in_empty_paragraph() {
        // 빈 문단(text="")에 0-length 필드:
        // HWP stream: fieldBegin(8cu) + fieldEnd(8cu) + para_end(1cu) = 17cu
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 99;

        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 17;
        para.char_offsets = vec![];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 0,
            control_idx: 0,
            ..Default::default()
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:fieldBegin id="99""#),
            "fieldBegin must be emitted: {}",
            &xml[..400.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="99"/>"#),
            "fieldEnd must be emitted: {}",
            &xml[..400.min(xml.len())]
        );

        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        assert!(
            begin_pos < end_pos,
            "빈 문단에서도 fieldBegin이 fieldEnd보다 앞에 와야 한다: {}",
            &xml[..400.min(xml.len())]
        );
    }

    // ---------- #1407: newNum(텍스트 끝 슬롯)이 fieldEnd 갭을 가로채지 않음 ----------

    #[test]
    fn task1407_field_end_not_stolen_by_newnum_slot() {
        // 143E 문단 0.14 모델: 하이퍼링크 필드가 "ABC"를 래핑하고 그 뒤 " DEF" 텍스트,
        // newNum 은 텍스트 끝. char_offsets 는 head(secPr) 16유닛 뒤로 시작:
        // fieldBegin(8) "ABC"(3) fieldEnd(8) " DEF"(4) newNum(8) end(1).
        // 핵심: fieldEnd(텍스트 중간 갭)가 후속 슬롯(newNum)에 가로채이면 " DEF"가
        // +8 밀린다. render_runs 방출 XML 의 컨트롤·텍스트 순서로 봉인.
        let mut f = Field::default();
        f.field_type = FieldType::Hyperlink;
        f.field_id = 42;

        let mut nn = NewNumber::default();
        nn.number = 2;
        nn.number_type = AutoNumberType::Page;

        let mut para = Paragraph::default();
        para.text = "ABC DEF".to_string();
        para.char_count = 8 + 3 + 8 + 4 + 8 + 1;
        para.char_offsets = vec![8, 9, 10, 19, 20, 21, 22];
        para.controls.push(Control::Field(f));
        para.controls.push(Control::NewNumber(nn));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3, // "ABC" 래핑
            control_idx: 0,
            ..Default::default()
        });

        let xml = runs_of(&para);

        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let newnum_pos = xml.find("newNum").expect("newNum");
        // " DEF" 텍스트(<hp:t> DEF</hp:t>) 위치 — fieldEnd 닫힘 뒤의 'DEF'.
        let after_end = end_pos + xml[end_pos..].find('>').unwrap();
        let def_pos = xml[after_end..]
            .find("DEF")
            .map(|p| p + after_end)
            .expect("DEF");

        assert!(begin_pos < end_pos, "fieldBegin이 fieldEnd보다 앞: {xml}");
        assert!(
            end_pos < def_pos,
            "fieldEnd 가 ' DEF' 텍스트보다 앞 (갭이 가로채이지 않음): {xml}"
        );
        assert!(
            def_pos < newnum_pos,
            "newNum 은 텍스트 끝 — ' DEF' 뒤에 와야 한다 (#1407 회귀 가드): {xml}"
        );
    }

    // ---------- #1378: char_shapes 경계 기준 다중 run 분할 ----------

    fn cs(start_pos: u32, char_shape_id: u32) -> CharShapeRef {
        CharShapeRef {
            start_pos,
            char_shape_id,
        }
    }

    fn runs_of(para: &Paragraph) -> String {
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        render_runs(para, &mut ctx).0
    }

    /// [#4778] 위치 축이 무너진 문단(파서 미수용 8유닛 슬롯 — 차례표지 0x0008 등)의
    /// 저장 lineseg 는 방출하지 않는다. textpos 사다리와 어긋난 lineseg 를 한글 2022 가
    /// 만나면 그 문단부터 문서 끝까지 본문을 폐기한다(성년후견 h2x -112,075자 실측,
    /// lineseg 억제만으로 전량 회복).
    #[test]
    fn issue4778_broken_position_axis_suppresses_stored_linesegs() {
        use crate::model::paragraph::LineSeg;
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);

        // 성년후견 #156 동형: 텍스트 앞에 8유닛 슬롯 갭(차례표지 자리)이 있는데
        // controls 는 비어 있다 — char_count/char_offsets 만 슬롯을 주장한다.
        let mut broken = Paragraph::default();
        broken.text = "다. 표제".to_string();
        broken.char_offsets = (0..broken.text.chars().count() as u32)
            .map(|i| 8 + i)
            .collect();
        broken.char_count = 8 + broken.text.chars().count() as u32 + 1;
        broken.line_segs = vec![LineSeg {
            line_height: 1100,
            ..Default::default()
        }];
        let (_, linesegs, _) = render_paragraph_parts(&broken, 0, &mut ctx);
        assert!(
            linesegs.is_empty(),
            "축 붕괴 문단은 저장 lineseg 를 방출하면 안 된다(한글 본문 폐기 트리거): {linesegs}"
        );

        // 대조군: 축이 온전한 문단은 종전대로 저장 lineseg 를 방출한다.
        let mut intact = Paragraph::default();
        intact.text = "본문".to_string();
        intact.char_offsets = (0..intact.text.chars().count() as u32).collect();
        intact.char_count = intact.text.chars().count() as u32 + 1;
        intact.line_segs = vec![LineSeg {
            line_height: 1100,
            ..Default::default()
        }];
        let (_, linesegs, _) = render_paragraph_parts(&intact, 0, &mut ctx);
        assert!(
            linesegs.contains("<hp:lineseg"),
            "온전한 문단의 저장 lineseg 보존이 깨졌다"
        );
    }

    #[test]
    fn task1378_two_run_split_mid_text() {
        // 경계가 텍스트 중간 — 텍스트 분할 (경계 케이스 2)
        let mut para = Paragraph::default();
        para.text = "abcdef".to_string();
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        assert_eq!(
            runs_of(&para),
            r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t>def</hp:t></hp:run>"#
        );
    }

    #[test]
    fn task1378_boundary_at_slot_position_slot_in_new_run() {
        // 경계와 컨트롤 슬롯이 같은 위치 — 경계 먼저 cut, 컨트롤은 새 run 소속 (경계 케이스 1)
        // 스트림: "ab"(0..2) + equation 슬롯(2..10) + "cd"(10..12), 경계 pos=2
        let mut para = Paragraph::default();
        para.text = "abcd".to_string();
        para.char_offsets = vec![0, 1, 10, 11];
        para.char_count = 13; // 4 + 8(슬롯) + 1
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(2, 2)];
        let xml = runs_of(&para);
        assert!(
            xml.starts_with(r#"<hp:run charPrIDRef="1"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="2"><hp:equation"#),
            "슬롯 위치의 경계는 슬롯보다 먼저 적용돼야 한다: {}",
            &xml[..200.min(xml.len())]
        );
        let run2 = xml.find(r#"<hp:run charPrIDRef="2">"#).unwrap();
        let cd = xml.find("<hp:t>cd</hp:t>").expect("cd 텍스트");
        assert!(run2 < cd, "cd 는 새 run 소속이어야 한다");
    }

    #[test]
    fn task1378_boundary_between_slot_and_text() {
        // 슬롯 이후 텍스트 중간 경계 — 슬롯은 이전 run, 분할은 텍스트에서
        // 스트림: equation 슬롯(0..8) + "ab"(8..10) + "cd"(10..12), 경계 pos=10
        let mut para = Paragraph::default();
        para.text = "abcd".to_string();
        para.char_offsets = vec![8, 9, 10, 11];
        para.char_count = 13;
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(10, 2)];
        let xml = runs_of(&para);
        let run1_end = xml.find("</hp:run>").unwrap();
        let eq = xml.find("<hp:equation").expect("equation");
        let ab = xml.find("<hp:t>ab</hp:t>").expect("ab");
        assert!(
            eq < run1_end && ab < run1_end,
            "equation 과 ab 는 첫 run 소속"
        );
        assert!(
            xml.ends_with(r#"<hp:run charPrIDRef="2"><hp:t>cd</hp:t></hp:run>"#),
            "cd 만 새 run 으로 분할: {}",
            xml
        );
    }

    #[test]
    fn issue_3739_consecutive_same_id_boundary_preserved() {
        // 같은 ID여도 start_pos는 HWP PARA_CHAR_SHAPE 의 보존 대상이다.
        let mut para = Paragraph::default();
        para.text = "abcdef".to_string();
        para.char_shapes = vec![cs(0, 5), cs(2, 5), cs(4, 6)];
        let xml = runs_of(&para);
        assert_eq!(
            xml,
            r#"<hp:run charPrIDRef="5"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="5"><hp:t>cd</hp:t></hp:run><hp:run charPrIDRef="6"><hp:t>ef</hp:t></hp:run>"#,
            "동일 id 경계 (2,5)도 별도 run으로 출력해야 한다"
        );
    }

    #[test]
    fn task1378_tab_and_linebreak_in_split_runs() {
        // 탭(8 유닛)/lineBreak 를 포함한 run 분할 (경계 케이스 4)
        // 위치: a=0, \t=1..9, b=9, \n=10, c=11 — 경계 pos=9
        let mut para = Paragraph::default();
        para.text = "a\tb\nc".to_string();
        para.char_shapes = vec![cs(0, 1), cs(9, 2)];
        let xml = runs_of(&para);
        assert_eq!(
            xml,
            concat!(
                r#"<hp:run charPrIDRef="1"><hp:t>a<hp:tab width="0" leader="0" type="1"/></hp:t></hp:run>"#,
                r#"<hp:run charPrIDRef="2"><hp:t>b<hp:lineBreak/>c</hp:t></hp:run>"#
            )
        );
    }

    #[test]
    fn task1378_empty_paragraph_single_run_id_zero() {
        // [#1592 갱신] 완전 빈 문단(text="", char_shapes=[], 컨트롤 없음)은 run 을 방출하지
        // 않는다. char_shapes=[] 는 "원본에 <hp:run> 없음"을 의미하므로(빈 run 이 있었다면
        // 파서가 [(0,0)] 을 산출), run 을 추가하면 재파싱 시 spurious (0,0) 가 생긴다(#1592).
        // 종전 #1378 은 빈 run(id 0)을 방출했으나, 이는 run 없던 빈 문단에 entry 를 가공했다.
        let para = Paragraph::default();
        assert_eq!(runs_of(&para), "");
    }

    #[test]
    fn task1378_field_end_stays_in_previous_run() {
        // 필드 begin/end 와 경계 교차 (경계 케이스 6)
        // 스트림: fieldBegin(0..8) + "abc"(8..11) + fieldEnd(11..19) + "de"(19..21)
        // 경계 pos=19 — fieldEnd 는 이전 run, "de" 는 새 run
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 11;
        let mut para = Paragraph::default();
        para.text = "abcde".to_string();
        para.char_offsets = vec![8, 9, 10, 19, 20];
        para.char_count = 22;
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3,
            control_idx: 0,
            ..Default::default()
        });
        para.char_shapes = vec![cs(0, 1), cs(19, 2)];
        let xml = runs_of(&para);
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let run2 = xml
            .find(r#"<hp:run charPrIDRef="2">"#)
            .expect("두 번째 run");
        assert!(
            end_pos < run2,
            "fieldEnd 는 이전 run 소속이어야 한다: {}",
            xml
        );
        assert!(
            xml.ends_with(r#"<hp:run charPrIDRef="2"><hp:t>de</hp:t></hp:run>"#),
            "de 는 새 run 소속이어야 한다: {}",
            xml
        );
    }

    #[test]
    fn task1378_trailing_boundary_emits_empty_run() {
        // 콘텐츠 끝 이후 경계 — 빈 run(<hp:t></hp:t>)으로 entry 보존 (규칙 5)
        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        assert_eq!(
            runs_of(&para),
            r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t></hp:t></hp:run>"#
        );
    }

    #[test]
    fn task1378_trailing_slot_in_new_run() {
        // 문단 끝 슬롯 위치의 경계 — trailing 슬롯도 새 run 소속 (규칙 1)
        // 스트림: "abc"(0..3) + equation 슬롯(3..11), 경계 pos=3
        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_offsets = vec![0, 1, 2];
        para.char_count = 12; // 3 + 8(슬롯) + 1
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        let xml = runs_of(&para);
        assert!(
            xml.starts_with(
                r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:equation"#
            ),
            "trailing 슬롯은 경계 적용 후 새 run 에 들어가야 한다: {}",
            &xml[..200.min(xml.len())]
        );
    }

    #[test]
    fn task1378_section_first_paragraph_secpr_run_id_follows_first_cs() {
        // 섹션 첫 문단 — secPr run id 와 텍스트 run id 의 dedup 상호작용 (경계 케이스 7)
        let mut para = Paragraph::default();
        para.text = "hi".to_string();
        para.char_shapes = vec![cs(0, 7)];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="7"><hp:secPr "#),
            "secPr run id 는 첫 텍스트 run id 와 일치해야 한다 (재파싱 시 (0,0) 오염 방지)"
        );
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="7"><hp:t>hi</hp:t></hp:run>"#),
            "텍스트 run 은 완전한 run 시퀀스로 치환돼야 한다"
        );
        assert!(
            !xml.contains(r#"<hp:run charPrIDRef="0">"#),
            "템플릿의 charPrIDRef=0 run 이 남아있으면 안 된다: {}",
            xml
        );
    }

    #[test]
    fn task1378_serialize_parse_roundtrip_preserves_char_shapes() {
        // serialize → parse 왕복 후 본문 char_shapes 시퀀스 보존.
        // 파서 정합 IR 로 구성: 섹션 첫 문단은 secPr(8)+colPr(8) 가 위치 축을 16 만큼
        // 선점하므로 char_offsets 가 16 부터 시작하고, 첫 entry 는 secPr run 시작(0)에
        // 기록된다 (stage1 양상 ② 메커니즘).
        use crate::model::style::CharShape;

        // 첫 문단: "abcdef", 경계 1개 — 원본 파스 결과는 [(0,1),(19,2)]
        let mut p0 = Paragraph::default();
        p0.text = "abcdef".to_string();
        p0.char_offsets = vec![16, 17, 18, 19, 20, 21];
        p0.char_count = 23;
        p0.char_shapes = vec![cs(0, 1), cs(19, 2)];

        // 추가 문단: 위치 축이 0 부터 — [(0,1),(3,2)]
        let mut p1 = Paragraph::default();
        p1.text = "abcdef".to_string();
        p1.char_offsets = vec![0, 1, 2, 3, 4, 5];
        p1.char_count = 7;
        p1.char_shapes = vec![cs(0, 1), cs(3, 2)];

        let mut section = Section::default();
        section.paragraphs.push(p0);
        section.paragraphs.push(p1);
        let mut doc = Document::default();
        doc.doc_info.char_shapes = vec![
            CharShape::default(),
            CharShape::default(),
            CharShape::default(),
        ];
        doc.sections.push(section);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let shapes_of = |i: usize| -> Vec<(u32, u32)> {
            doc2.sections[0].paragraphs[i]
                .char_shapes
                .iter()
                .map(|r| (r.start_pos, r.char_shape_id))
                .collect()
        };
        assert_eq!(shapes_of(0), vec![(0, 1), (19, 2)], "섹션 첫 문단");
        assert_eq!(shapes_of(1), vec![(0, 1), (3, 2)], "추가 문단");
    }

    // ---------- #1584: 본문 인라인 ColumnDef 드롭 회귀 가드 ----------

    #[test]
    fn task1584_body_first_para_two_columndefs_roundtrip() {
        // 본문 첫 문단에 ColumnDef 2개(섹션 단 정의 + 인라인 단 정의).
        // 섹션 템플릿은 첫 ColumnDef 1개만 흡수하고, 2번째 인라인 ColumnDef 는
        // 본문 인라인 슬롯에서 제외되어 드롭된다(controls 6→5 양상).
        // 수정 전: reparse 후 ColumnDef 1개만 → RED. 수정 후: 2개 보존 → GREEN.
        let mut p0 = Paragraph::default();
        p0.controls.push(Control::ColumnDef(ColumnDef::default()));
        p0.controls.push(Control::ColumnDef(ColumnDef::default()));

        let mut section = Section::default();
        section.paragraphs.push(p0);
        let mut doc = Document::default();
        doc.sections.push(section);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let coldef_count = doc2.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|c| matches!(c, Control::ColumnDef(_)))
            .count();
        assert_eq!(
            coldef_count, 2,
            "본문 첫 문단의 ColumnDef 2개가 roundtrip 후 모두 보존돼야 한다 (템플릿1 + 인라인1): {coldef_count}"
        );
    }

    #[test]
    fn page_starts_on_odd_survives_hwpx_roundtrip() {
        // pageStartsOn(홀수 시작)이 HWPX 왕복에서 보존돼야 한다. 종전엔 파서 미독 +
        // serializer 의 pageStartsOn="BOTH" 고정으로 page_num_type 이 0 으로 유실됐다.
        let mut section = Section::default();
        section.section_def.page_num_type = 1; // 홀수 시작
        section.section_def.page_num = 1;
        section.paragraphs.push(Paragraph::default());
        let mut doc = Document::default();
        doc.sections.push(section);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        assert_eq!(
            doc2.sections[0].section_def.page_num_type, 1,
            "홀수 쪽 시작(pageStartsOn=ODD)이 왕복에서 보존돼야 함"
        );
    }

    // ---------- #1596: generic-shape 지오메트리 직렬화 ----------

    #[test]
    fn task1596_polygon_geometry_serialized() {
        // [#1596] polygon 의 꼭짓점(hc:pt)·테두리(lineShape)·그림자(shadow)가 방출돼야 한다.
        // render_common_shape_xml 이 종전 이들을 드롭 → 도형 형상 소실 → 페이지 붕괴(#1589 잔여).
        use crate::model::shape::PolygonShape;
        use crate::model::Point;
        let mut poly = PolygonShape::default();
        poly.points = vec![
            Point { x: 0, y: 0 },
            Point { x: 100, y: 0 },
            Point { x: 100, y: 100 },
        ];
        poly.drawing.border_line.width = 50;
        poly.drawing.shadow_type = 1;
        let mut ctx = SerializeContext::collect_from_document(&Document::default());
        let xml = render_shape(&ShapeObject::Polygon(poly), &mut ctx);
        assert!(xml.contains("<hc:pt "), "폴리곤 꼭짓점(hc:pt) 방출: {xml}");
        assert!(xml.contains("<hp:lineShape"), "lineShape 방출: {xml}");
        assert!(xml.contains("<hp:shadow"), "shadow 방출: {xml}");
    }
}
