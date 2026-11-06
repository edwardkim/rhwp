//! 관계형 회귀: 원고지 원본 격자 간격·글리프 폭·셀 내 위치의 비율 및 포함 관계. HWPUNIT 피치를 환산한 계약은 임의 절대 좌표 핀이 아니다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! hancom-mac-200-b5.hwp (8629-hancom-mac-200-b5-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! hancom-mac-200-b5.hwp (8629-hancom-mac-200-b5-hwp-2024): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형·고정형(독립 Print 전체 쪽수, 줄마다 칸 소속).
//! 주석 갱신: 2026-11-05T15:04:00+09:00 (새 파일).
//! 2026-11-05 측정 raw(96dpi, pypdfium2, 2px 실루엣 threshold_boundary_color_support_v1, compat 2024):
//! hancom-mac-200-b5.hwp 1쪽 Native 100.00000% / fresh WASM 100.00000%(미대응 0픽셀), 전1쪽 최저 같음.
//! 수정 전 devel 6e1de4062: Native 77.50756% / fresh WASM 77.59633%.
//! 근거: mydocs/pr/assets/issue_8628_wonggoji/{native,wasm,base}/ 의 TSV와 review·overlay PNG.
//!
//! 원고지 정서법(HWP5 구역 flags bit 22, HWPX `wonggojiFormat`)과 줄·글자 격자가 켜진 구역은
//! 글자를 칸에 하나씩 놓는다. 입력은 rhwp로 만든 200자 원고지 서식(B5 가로, 칸 2863HU·줄 3968HU,
//! 바탕쪽 칸 표)에 한/글 12.30 Mac에서 글을 쳐 저장한 HWP이고, 기대값은 같은 문서의 한/글 Print
//! PDF에서 읽은 줄마다의 칸 소속이다(글자 왼쪽 x를 본문 왼쪽에서 칸 폭으로 나눈 칸 번호).
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;

const FIXTURE: &str = "tests/fixtures/wonggoji/hancom-mac-200-b5.hwp";
/// HWP5 구역 정의 flags의 원고지 정서법 비트.
const WONGGOJI_FORMAT: u32 = 1 << 22;
/// 한/글 Print PDF(hancom-mac-200-b5.pdf)의 줄마다 칸 소속. `|`가 칸 경계, 빈 칸은 공백·들여쓰기다.
/// 소문자·숫자는 두 자가 한 칸이고, 넷째 줄은 끝 칸 뒤로 `E`를 매달고 `FG`를 넘긴다.
const HANCOM_CELLS: [&str; 6] = [
    "|우|리||집||강|아|지|는||20|24|년||봄|에||태|어",
    "났|다|.||이|름|은||H|ap|py|이|고|,||아|주||귀|엽",
    "다|!||하|루|에||산|책|을||세||번||한|다|.",
    "|\"|안|녕|?|\"||하|고||인|사|했|다|.||A|B|C|D|E|FG",
    "12|34|56|78|90||끝|,||그|리|고||마|지|막||줄|입|니",
    "다|.",
];

fn fixture() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("원고지 fixture")).expect("열기")
}

fn attr(tag: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    tag[start..].split('"').next()?.parse().ok()
}

/// 쪽 SVG에서 한 글자씩 그린 text 요소의 (글자, x, 기준선 y, 글자 크기).
fn glyph_runs(svg: &str) -> Vec<(char, f64, f64, f64)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let (Some(open), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        let text = rest[open + 1..close]
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        let mut chars = text.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            let tag = &rest[..open];
            if let (Some(x), Some(y), Some(size)) =
                (attr(tag, "x"), attr(tag, "y"), attr(tag, "font-size"))
            {
                out.push((ch, x, y, size));
            }
        }
        rest = &rest[close + 7..];
    }
    out
}

/// 쪽 SVG에서 한 글자씩 그린 text 요소의 (글자, x, 기준선 y).
fn glyphs(svg: &str) -> Vec<(char, f64, f64)> {
    glyph_runs(svg)
        .into_iter()
        .map(|(ch, x, y, _)| (ch, x, y))
        .collect()
}

/// 본문 영역(위 여백 아래)에 그린 글자. 바탕쪽 쪽 번호(No. 1)는 위 여백에 걸쳐 빠진다.
fn body_glyphs(core: &DocumentCore) -> Vec<(char, f64, f64)> {
    let top = f64::from(core.document().sections[0].section_def.page_def.margin_top) / 75.0;
    let svg = core.render_page_svg_native(0).expect("1쪽 SVG");
    glyph_runs(&svg)
        .into_iter()
        // 글자 상자 위(기준선 위 0.85em)가 본문 위 여백 안에 있는 글자만 본문이다.
        .filter(|(ch, _, y, size)| !ch.is_whitespace() && y - size * 0.85 >= top)
        .map(|(ch, x, y, _)| (ch, x, y))
        .collect()
}

/// 본문 글줄을 위에서부터 칸 소속 문자열로 바꾼다. 공백은 그리지 않으므로 빈 칸으로 남는다.
fn cell_lines(core: &DocumentCore) -> Vec<String> {
    let sd = &core.document().sections[0].section_def;
    let px = |hu: i32| f64::from(hu) / 75.0;
    let origin = px(sd.page_def.margin_left as i32 + sd.page_def.margin_gutter as i32);
    let cell = px(i32::from(sd.char_grid));
    let mut rows: Vec<(f64, Vec<String>)> = Vec::new();
    for (ch, x, y) in body_glyphs(core) {
        let k = ((x - origin) / cell + 0.02).floor();
        assert!(k >= 0.0, "본문 왼쪽 밖 글자 {ch} x={x}");
        let k = k as usize;
        let row = match rows.iter_mut().find(|(row_y, _)| (row_y - y).abs() < 0.5) {
            Some(row) => row,
            None => {
                rows.push((y, Vec::new()));
                rows.last_mut().unwrap()
            }
        };
        if row.1.len() <= k {
            row.1.resize(k + 1, String::new());
        }
        row.1[k].push(ch);
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.into_iter().map(|(_, cells)| cells.join("|")).collect()
}

/// 글줄 기준선 사이가 모두 줄 격자 한 칸이다.
fn assert_line_pitch(core: &DocumentCore) {
    let sd = &core.document().sections[0].section_def;
    let pitch = f64::from(sd.line_grid) / 75.0;
    let mut ys: Vec<f64> = body_glyphs(core).into_iter().map(|(_, _, y)| y).collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert_eq!(ys.len(), HANCOM_CELLS.len(), "글줄 수");
    for pair in ys.windows(2) {
        assert!(
            (pair[1] - pair[0] - pitch).abs() < 0.05,
            "줄 간격 {} ≠ 줄 격자 {pitch}",
            pair[1] - pair[0]
        );
    }
}

#[test]
fn hancom_manuscript_lays_glyphs_on_cells_like_hancom_print() {
    let core = fixture();
    let sd = &core.document().sections[0].section_def;
    assert_ne!(
        sd.flags & WONGGOJI_FORMAT,
        0,
        "한/글 저장본의 원고지 정서법"
    );
    assert_eq!((sd.line_grid, sd.char_grid), (3968, 2863));
    assert_eq!(core.page_count(), 1, "독립 Print 전체 쪽수");
    assert_eq!(cell_lines(&core), HANCOM_CELLS);
    assert_line_pitch(&core);
}

/// 원고지 서식처럼 저장 줄(linesegarray) 없이 연 HWPX도 한/글이 저장한 줄과 같은 칸 수로 나눈다.
#[test]
fn unlined_manuscript_paragraphs_break_like_hancom_stored_lines() {
    let mut core = fixture();
    let starts = |core: &DocumentCore| -> Vec<Vec<u32>> {
        core.document().sections[0]
            .paragraphs
            .iter()
            .map(|para| para.line_segs.iter().map(|line| line.text_start).collect())
            .collect()
    };
    let hancom = starts(&core);
    for para in core.document_mut().sections[0].paragraphs.iter_mut() {
        para.line_segs.clear();
    }
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX")).unwrap();
    assert_eq!(starts(&reopened), hancom, "한/글 저장 줄의 시작");
    assert_eq!(cell_lines(&reopened), HANCOM_CELLS);
    assert_line_pitch(&reopened);
}

/// 원고지 정서법은 HWPX에 `wonggojiFormat`으로 오가고 구역 정의 API에 격자 키로 나온다.
#[test]
fn wonggoji_format_survives_hwpx_and_section_def_api() {
    let core = fixture();
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX")).unwrap();
    let sd = &reopened.document().sections[0].section_def;
    assert_ne!(sd.flags & WONGGOJI_FORMAT, 0, "HWPX 왕복 뒤 원고지 정서법");
    let json = reopened.get_section_def_native(0).unwrap();
    for key in [
        "\"lineGrid\":3968",
        "\"charGrid\":2863",
        "\"wonggojiFormat\":true",
    ] {
        assert!(json.contains(key), "{key}: {json}");
    }
}

fn manuscript_blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.set_section_def_native(
        0,
        r#"{"lineGrid":3968,"charGrid":2863,"wonggojiFormat":true}"#,
    )
    .unwrap();
    core
}

/// 한 글자씩 친 글은 칸 수로 줄이 나뉘고, 글자는 칸 폭·줄 격자 간격으로 그려지며,
/// HWP·HWPX 저장 뒤에도 정서법·격자와 줄이 남는다. A4 본문 42520HU는 14칸이다.
#[test]
fn typed_text_breaks_by_cells_and_survives_both_formats() {
    let mut core = manuscript_blank();
    let text: Vec<char> = "가나다라마바사아자차카타파하거너더러머버서어저처커"
        .chars()
        .collect();
    // 쪽 안 빠른 입력 경로도 원고지 구역에서는 쪽 나눔을 미루지 않아야 칸에 그린다.
    for (offset, ch) in text.iter().enumerate() {
        let result = core
            .replace_body_text_local_native(0, 0, offset, 0, &ch.to_string())
            .unwrap();
        assert!(
            result.contains("\"documentPaginationPending\":false"),
            "{result}"
        );
    }
    let lines = &core.document().sections[0].paragraphs[0].line_segs;
    assert_eq!(lines.len(), 2, "25자 = 14 + 11칸");
    assert_eq!(lines[1].vertical_pos - lines[0].vertical_pos, 3968);

    core.replace_body_text_local_native(0, 0, text.len(), 0, "2024")
        .unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    // 칸으로 나눈 런이라 글리프 폭 맞춤이 칸 사이만큼 글리프를 늘이지 않는다.
    for fit in svg.split("textLength=\"").skip(1) {
        let width: f64 = fit.split('"').next().unwrap().parse().unwrap();
        assert!(width < 2863.0 / 75.0, "글리프 폭 {width}");
    }
    let at = |ch: char| {
        glyphs(&svg)
            .into_iter()
            .find(|g| g.0 == ch)
            .map(|g| (g.1, g.2))
            .unwrap()
    };
    let (ga, na, geo) = (at('가'), at('나'), at('거'));
    assert!(
        (na.0 - ga.0 - 2863.0 / 75.0).abs() < 0.05,
        "칸 폭: {ga:?} {na:?}"
    );
    // 둘째 줄 첫 글자는 첫 줄 첫 글자와 같은 칸, 줄 격자 한 칸 아래
    assert!((geo.0 - ga.0).abs() < 0.05 && (geo.1 - ga.1 - 3968.0 / 75.0).abs() < 0.05);

    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let sd = &reopened.document().sections[0].section_def;
        assert_ne!(sd.flags & WONGGOJI_FORMAT, 0);
        assert_eq!((sd.line_grid, sd.char_grid), (3968, 2863));
        assert_eq!(
            reopened.document().sections[0].paragraphs[0]
                .line_segs
                .len(),
            2
        );
    }
}

/// 나눠 만든 문단과 저장 줄 없이 연 문단(HWPX)도 칸 수로 나눈 줄 그대로 그린다.
/// 글자 본래 폭의 프레임에서 다시 짜면 한 줄로 합쳐져 칸 밖으로 넘친다.
#[test]
fn split_and_unlined_paragraphs_draw_their_cell_lines() {
    let mut blank = manuscript_blank();
    blank.document_mut().sections[0].paragraphs[0]
        .line_segs
        .clear();
    let mut core = DocumentCore::from_bytes(&blank.export_hwpx_native().unwrap()).unwrap();
    // 14칸 줄: 한글 10자, 공백, Tauri 세 칸, 매단 공백 뒤 2026은 둘째 줄이다.
    core.insert_text_native(0, 0, 0, "가").unwrap();
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    core.insert_text_native(0, 1, 0, "나다라마바사아자차카 Tauri 2026 ok.")
        .unwrap();
    let pitch = 3968.0 / 75.0;
    let check = |core: &DocumentCore| {
        let svg = core.render_page_svg_native(0).unwrap();
        let at = |ch: char| {
            glyphs(&svg)
                .into_iter()
                .find(|g| g.0 == ch)
                .map(|g| (g.1, g.2))
                .unwrap()
        };
        let (ga, na, two) = (at('가'), at('나'), at('2'));
        assert!(
            (na.1 - ga.1 - pitch).abs() < 0.05,
            "둘째 문단: {ga:?} {na:?}"
        );
        assert!(
            (two.1 - na.1 - pitch).abs() < 0.05 && (two.0 - na.0).abs() < 2863.0 / 75.0,
            "둘째 문단 둘째 줄: {na:?} {two:?}"
        );
    };
    check(&core);
    check(&DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap());
    check(&DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap());
}

/// 글을 쓴 뒤 격자를 켜도, 서식을 바꾸거나 머리말을 넣어도 문단이 칸 수로 나뉘고 세로 위치가
/// 겹치지 않는다.
#[test]
fn turning_the_grid_on_formatting_and_header_keep_cell_lines() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let line = "가나다라마바사아자차카타파하거너더러머버서어저처커";
    core.insert_text_native(0, 0, 0, line).unwrap();
    core.split_paragraph_native(0, 0, 25, None).unwrap();
    core.insert_text_native(0, 1, 0, line).unwrap();
    core.split_paragraph_native(0, 1, 25, None).unwrap();
    core.insert_text_native(0, 2, 0, line).unwrap();
    core.set_section_def_native(
        0,
        r#"{"lineGrid":3968,"charGrid":2863,"wonggojiFormat":true}"#,
    )
    .unwrap();
    let check = |core: &DocumentCore, what: &str| {
        let paragraphs = &core.document().sections[0].paragraphs;
        let mut end = paragraphs[0].line_segs[0].vertical_pos;
        for (index, para) in paragraphs.iter().enumerate() {
            assert_eq!(para.line_segs.len(), 2, "{what}: 문단 {index} 줄 수");
            assert_eq!(
                para.line_segs[0].vertical_pos, end,
                "{what}: 문단 {index} 시작"
            );
            assert_eq!(
                para.line_segs[1].text_start, para.char_offsets[14],
                "{what}"
            );
            end = para.line_segs[1].vertical_pos + 3968;
        }
    };
    check(&core, "격자 켜기");
    core.apply_char_format_native(0, 1, 0, 5, r#"{"bold":true}"#)
        .unwrap();
    check(&core, "진하게");
    core.create_header_footer_native(0, true, 0).unwrap();
    check(&core, "머리말");
}
