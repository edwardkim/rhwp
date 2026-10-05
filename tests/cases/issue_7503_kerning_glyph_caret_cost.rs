//! [#7503] 등록 글꼴로 커닝한 글자를 커닝 전 폭으로, 커닝한 자리에 그린다.
//! 캐럿·클릭 위치는 그린 자리를 따르고, 입력마다 등록 글꼴을 다시 해시하지 않는다.
//!
//! 글꼴은 저장소의 합성 글꼴이다(`pos A V -80`, `pos T o -40`, 1000 em, advance 600).
//! 26pt(34.67px)에서 `A`의 advance는 24.47px이고 `AV` 커닝은 -2.77px이다.
//! 같은 문서를 글꼴 등록 없이 그린 결과를 커닝 전 기준으로 쓴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const KERNING_FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const TEXT: &str = "AVTo";

/// 글자모양을 모두 커닝·26pt로 바꾼 새 문서.
fn kerning_blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("새 문서 템플릿");
    let mut document = core.document().clone();
    for char_shape in &mut document.doc_info.char_shapes {
        char_shape.raw_data = None;
        char_shape.kerning = true;
        char_shape.base_size = 2_600;
    }
    core.set_document(document);
    core
}

/// 모든 글자모양의 영문 slot(언어 1)에 글꼴을 등록한다.
fn register_all(core: &mut DocumentCore, font: &[u8]) {
    let count = core.document().doc_info.char_shapes.len() as u32;
    for char_shape_id in 0..count {
        core.register_exact_font_source_native(char_shape_id, 1, font, 0)
            .expect("글꼴 등록");
    }
}

/// 본문 문단 0, 표 셀(문단 1의 표), 머리말, 각주에 `AVTo`를 넣는다.
fn fixture(register: bool) -> (DocumentCore, usize) {
    let mut core = kerning_blank();
    core.insert_text_native(0, 0, 0, TEXT).expect("본문 입력");
    core.split_paragraph_native(0, 0, TEXT.len(), None)
        .expect("문단 나누기");
    core.create_table_native(0, 1, 0, 1, 1).expect("1×1 표");
    let table_ctrl = core.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .expect("표 컨트롤");
    core.insert_text_in_cell_native(0, 1, table_ctrl, 0, 0, 0, TEXT)
        .expect("셀 입력");
    core.create_header_footer_native(0, true, 0)
        .expect("머리말");
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, TEXT)
        .expect("머리말 입력");
    core.insert_footnote_native(0, 0, TEXT.len()).expect("각주");
    let footnote_ctrl = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Footnote(_)))
        .expect("각주 컨트롤");
    core.insert_text_in_footnote_native(0, 0, footnote_ctrl, 0, 0, TEXT)
        .expect("각주 입력");
    if register {
        register_all(&mut core, KERNING_FONT);
    }
    (core, table_ctrl)
}

#[derive(Debug, Clone, PartialEq)]
struct SvgGlyph {
    ch: char,
    x: f64,
    y: f64,
    text_length: Option<f64>,
}

fn svg_attr(tag: &str, name: &str) -> Option<f64> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let end = start + tag[start..].find('"')?;
    tag[start..end].parse().ok()
}

/// SVG의 한 글자 `<text>`를 기준선별로 모아 `AVTo` 줄만 돌려준다.
fn svg_lines(svg: &str) -> Vec<Vec<SvgGlyph>> {
    let mut glyphs = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let tag = format!(" {}", &chunk[..tag_end]);
        let Some(content_end) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        let content = &chunk[tag_end + 1..tag_end + 1 + content_end];
        let mut chars = content.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            continue;
        };
        if !TEXT.contains(ch) {
            continue;
        }
        let (Some(x), Some(y)) = (svg_attr(&tag, "x"), svg_attr(&tag, "y")) else {
            continue;
        };
        glyphs.push(SvgGlyph {
            ch,
            x,
            y,
            text_length: svg_attr(&tag, "textLength"),
        });
    }
    let mut lines: Vec<Vec<SvgGlyph>> = Vec::new();
    for glyph in glyphs {
        match lines
            .iter_mut()
            .find(|line| (line[0].y - glyph.y).abs() < 1e-6)
        {
            Some(line) => line.push(glyph),
            None => lines.push(vec![glyph]),
        }
    }
    for line in &mut lines {
        line.sort_by(|a, b| a.x.total_cmp(&b.x));
    }
    lines.retain(|line| line.iter().map(|glyph| glyph.ch).eq(TEXT.chars()));
    lines.sort_by(|a, b| a[0].y.total_cmp(&b[0].y));
    lines
}

#[test]
fn issue_7503_kerned_glyphs_keep_their_natural_width() {
    let (plain, _) = fixture(false);
    let (kerned, _) = fixture(true);
    let plain_lines = svg_lines(&plain.render_page_svg_native(0).expect("커닝 전 SVG"));
    let kerned_lines = svg_lines(&kerned.render_page_svg_native(0).expect("커닝 SVG"));
    assert_eq!(plain_lines.len(), 4, "본문·셀·머리말·각주: {plain_lines:?}");
    assert_eq!(kerned_lines.len(), plain_lines.len());

    let mut kerned_line_count = 0;
    for (plain_line, kerned_line) in plain_lines.iter().zip(&kerned_lines) {
        // 커닝은 자리만 옮긴다. 글리프 폭은 커닝 전 advance 그대로다.
        for (plain_glyph, kerned_glyph) in plain_line.iter().zip(kerned_line) {
            assert!(plain_glyph.text_length.is_some());
            assert_eq!(
                kerned_glyph.text_length, plain_glyph.text_length,
                "'{}'가 커닝만큼 눌렸다: {kerned_line:?}",
                kerned_glyph.ch
            );
        }
        if kerned_line[1].x < plain_line[1].x - 1.0 {
            kerned_line_count += 1;
            // `V`는 `A` 뒤 커닝한 자리, `o`는 `T` 뒤 커닝한 자리에 놓인다.
            assert!(kerned_line[3].x - kerned_line[2].x < plain_line[3].x - plain_line[2].x - 0.5);
        }
    }
    assert!(
        kerned_line_count >= 3,
        "본문·셀·머리말은 커닝해야 한다: {kerned_lines:?}"
    );
}

#[test]
fn unverified_dedicated_font_preserves_rendered_positions_and_widths() {
    // 동일 glyph·advance·GPOS를 두고 전용 테이블이 추가된 source의 지원 경계를 검사한다.
    // name은 선택 테이블이다. 그 디렉터리 항목만 바꿔 실제 글꼴 bytes를 Git에 넣지 않는다.
    let mut font = KERNING_FONT.to_vec();
    let count = u16::from_be_bytes([font[4], font[5]]) as usize;
    let mut records: Vec<_> = font[12..12 + count * 16]
        .chunks_exact(16)
        .map(|record| record.to_vec())
        .collect();
    let name = records
        .iter_mut()
        .find(|record| &record[..4] == b"name")
        .expect("합성 source의 name 테이블");
    name[..4].copy_from_slice(b"HJCT");
    records.sort_by(|left, right| left[..4].cmp(&right[..4]));
    for (target, record) in font[12..12 + count * 16].chunks_exact_mut(16).zip(&records) {
        target.copy_from_slice(record);
    }

    let (mut core, _) = fixture(false);
    let before = svg_lines(&core.render_page_svg_native(0).expect("기본 위치 SVG"));
    assert_eq!(before.len(), 4, "본문·셀·머리말·각주");
    register_all(&mut core, &font);
    let after = svg_lines(&core.render_page_svg_native(0).expect("전용 source SVG"));
    assert_eq!(
        after, before,
        "지원 계약을 확인하지 못한 source로 기본 위치·자연 폭을 바꾸면 안 된다"
    );
}

#[test]
fn issue_7503_middle_dot_stays_centred_in_its_drawn_advance() {
    use rhwp::renderer::svg::SvgRenderer;
    use rhwp::renderer::{Renderer, TextStyle};

    // `·`는 글리프 대신 원으로 그린다. 원은 커닝 전 폭이 아니라 그린 칸 가운데에 둔다(Skia와 같다).
    let style = TextStyle {
        font_family: "함초롬바탕".to_string(),
        font_size: 20.0,
        kerning: true,
        ..TextStyle::default()
    };
    let circle_cx = |positions: Option<&[f64]>| {
        let mut svg = SvgRenderer::new();
        svg.draw_text_positioned("\u{00B7}V", 0.0, 20.0, &style, positions);
        let output = svg.output();
        svg_attr(&output[output.find("<circle").expect("`·` 원")..], "cx").expect("cx")
    };
    let natural_cx = circle_cx(None);
    assert!(natural_cx > 2.0, "`·` 칸 폭: {natural_cx}");
    // 다음 글자를 2px 당긴 커닝 자리
    let kerned_advance = natural_cx * 2.0 - 2.0;
    let kerned_cx = circle_cx(Some(&[0.0, kerned_advance, kerned_advance + 20.0]));
    assert!(
        (kerned_cx - kerned_advance / 2.0).abs() <= 1e-3,
        "`·` 원 {kerned_cx} ≠ 그린 칸 가운데 {}",
        kerned_advance / 2.0
    );
}

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).expect("JSON")
}

/// 캐럿 x(0.1 반올림)가 그린 글자 원점과 같은지 본다.
fn assert_caret_at(place: &str, offset: usize, caret: &str, origin: f64) {
    let x = json(caret)["x"].as_f64().expect("캐럿 x");
    assert!(
        (x - origin).abs() <= 0.051,
        "{place} offset {offset}: 캐럿 {x} ≠ 그린 원점 {origin}"
    );
}

#[test]
fn issue_7503_caret_and_hit_test_follow_rendered_origins() {
    let (core, table_ctrl) = fixture(true);
    let lines = svg_lines(&core.render_page_svg_native(0).expect("SVG"));
    let [header, body, cell, footnote] = lines.as_slice() else {
        panic!("본문·셀·머리말·각주 줄: {lines:?}");
    };
    // 커닝이 걸린 줄에서 검사해야 의미가 있다.
    for line in [header, body, cell] {
        assert!(line[1].x - line[0].x < 24.0, "커닝 안 됨: {line:?}");
    }

    for offset in 0..TEXT.len() {
        assert_caret_at(
            "본문",
            offset,
            &core
                .get_cursor_rect_native(0, 0, offset)
                .expect("본문 캐럿"),
            body[offset].x,
        );
        assert_caret_at(
            "셀",
            offset,
            &core
                .get_cursor_rect_in_cell_native(0, 1, table_ctrl, 0, 0, offset)
                .expect("셀 캐럿"),
            cell[offset].x,
        );
        assert_caret_at(
            "머리말",
            offset,
            &core
                .get_cursor_rect_in_header_footer_native(0, true, 0, 0, offset, 0)
                .expect("머리말 캐럿"),
            header[offset].x,
        );
        assert_caret_at(
            "각주",
            offset,
            &core
                .get_cursor_rect_in_footnote_native(0, 0, 0, offset)
                .expect("각주 캐럿"),
            footnote[offset].x,
        );
    }

    // 클릭 전환점은 그린 원점 사이의 중점이다. 커닝 전 중점과는 0.7~1.4px 다르다.
    // 머리말·각주 hit-test는 커닝이 없어도 한 글자 뒤를 돌려줘 여기서 보지 않는다.
    for boundary in [1usize, 3] {
        for (side, expected) in [(-0.3, boundary - 1), (0.3, boundary)] {
            let at = |line: &[SvgGlyph]| {
                let x = (line[boundary - 1].x + line[boundary].x) / 2.0 + side;
                json(
                    &core
                        .hit_test_native(0, x, line[0].y - 10.0)
                        .expect("hit-test"),
                )
            };
            let body_hit = at(body);
            assert_eq!(body_hit["paragraphIndex"], 0);
            assert_eq!(body_hit["charOffset"], expected, "본문 {body_hit}");
            let cell_hit = at(cell);
            assert_eq!(cell_hit["parentParaIndex"], 1);
            assert_eq!(cell_hit["charOffset"], expected, "셀 {cell_hit}");
        }
    }

    // 쪽 텍스트 배치의 charX도 그린 원점이고, 끝값은 run 폭이다.
    let layout = json(&core.get_page_text_layout_native(0).expect("쪽 텍스트 배치"));
    let runs: Vec<&serde_json::Value> = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| {
            run["text"]
                .as_str()
                .is_some_and(|text| text.starts_with(TEXT))
        })
        .collect();
    assert_eq!(runs.len(), 4);
    for run in runs {
        let x = run["x"].as_f64().expect("x");
        let char_x: Vec<f64> = run["charX"]
            .as_array()
            .expect("charX")
            .iter()
            .map(|value| value.as_f64().expect("charX 값"))
            .collect();
        let (y, height) = (run["y"].as_f64().expect("y"), run["h"].as_f64().expect("h"));
        let line = lines
            .iter()
            .find(|line| {
                (line[0].x - x).abs() <= 0.051 && line[0].y > y && line[0].y <= y + height + 0.1
            })
            .expect("같은 줄");
        for (index, glyph) in line.iter().enumerate() {
            assert!((x + char_x[index] - glyph.x).abs() <= 0.11, "{run}");
        }
        let width = run["w"].as_f64().expect("w");
        let text_end = run["text"].as_str().unwrap().chars().count();
        assert!((char_x[text_end] - width).abs() <= 0.051, "{run}");
    }
}

/// 셀에 `A`만 둔 문서에서 `A` 뒤에 `V`를 넣고 빠른 경로의 deltaX와 실제 캐럿 이동을 돌려준다.
fn cell_typing_delta(register: bool) -> (Option<f64>, f64) {
    let mut core = kerning_blank();
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let ctrl = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .expect("표 컨트롤");
    core.insert_text_in_cell_native(0, 0, ctrl, 0, 0, 0, "A")
        .expect("셀 입력");
    if register {
        register_all(&mut core, KERNING_FONT);
    }
    let caret_x = |core: &DocumentCore, offset: usize| {
        json(
            &core
                .get_cursor_rect_in_cell_native(0, 0, ctrl, 0, 0, offset)
                .expect("셀 캐럿"),
        )["x"]
            .as_f64()
            .expect("캐럿 x")
    };
    let before = caret_x(&core, 1);
    // 빠른 경로는 캐시한 쪽 트리를 고친다. 먼저 쪽 트리를 만든다.
    core.hit_test_native(0, 0.0, 0.0).expect("쪽 트리");
    let result = json(
        &core
            .insert_text_in_cell_native_deferred_pagination(0, 0, ctrl, 0, 0, 1, "V")
            .expect("지연 입력"),
    );
    core.flush_deferred_pagination();
    let after = caret_x(&core, 2);
    (
        result["focusedCursorGeometry"]["deltaX"].as_f64(),
        after - before,
    )
}

#[test]
fn issue_7503_cell_typing_caret_delta_follows_kerned_origin() {
    // 커닝이 없으면 빠른 경로를 그대로 쓴다.
    let (plain_delta, plain_move) = cell_typing_delta(false);
    let plain_delta = plain_delta.expect("커닝 없는 셀은 빠른 경로를 쓴다");
    assert!(
        (plain_delta - plain_move).abs() <= 0.11,
        "{plain_delta} vs {plain_move}"
    );

    // 커닝하면 `V`는 2.77px 앞에 놓인다. 빠른 경로가 답한다면 그 이동과 같아야 한다.
    let (kerned_delta, kerned_move) = cell_typing_delta(true);
    assert!(kerned_move < plain_move - 2.0, "커닝 안 됨: {kerned_move}");
    if let Some(delta) = kerned_delta {
        assert!(
            (delta - kerned_move).abs() <= 0.11,
            "deltaX {delta} ≠ 실제 캐럿 이동 {kerned_move}"
        );
    }
}

/// 커닝 문단에서 입력·쪽 렌더 트리·캐럿 조회를 한 번 하는 시간(ms)의 최솟값.
///
/// 공개 API로는 해시 횟수를 셀 수 없고, 제품 소스에 `#[cfg(test)]` 계수기를 둘 수 없다.
/// 그래서 같은 프로세스에서 잰 SHA-256 한 번과 비교해 장비 속도에 기대지 않는다.
fn fastest_typing_cycle_ms(font: &[u8]) -> f64 {
    let mut core = kerning_blank();
    core.insert_text_native(0, 0, 0, &"AVTo ".repeat(8))
        .expect("본문 입력");
    register_all(&mut core, font);
    core.build_page_render_tree(0).expect("쪽 트리");
    (0..5)
        .map(|_| {
            let len = core.document().sections[0].paragraphs[0]
                .text
                .chars()
                .count();
            let start = std::time::Instant::now();
            core.insert_text_native(0, 0, len, "A").expect("입력");
            core.build_page_render_tree(0).expect("쪽 트리");
            core.get_cursor_rect_native(0, 0, len + 1).expect("캐럿");
            start.elapsed().as_secs_f64() * 1_000.0
        })
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn issue_7503_registered_source_is_not_rehashed_per_layout() {
    use sha2::{Digest, Sha256};

    // 뒤에 0을 붙여 크기만 키운 같은 face다. 표는 offset으로 읽는다.
    let mut large = KERNING_FONT.to_vec();
    large.resize(8 * 1024 * 1024, 0);
    let one_hash_ms = (0..3)
        .map(|_| {
            let start = std::time::Instant::now();
            std::hint::black_box(Sha256::digest(std::hint::black_box(&large)));
            start.elapsed().as_secs_f64() * 1_000.0
        })
        .fold(f64::INFINITY, f64::min);

    let small_ms = fastest_typing_cycle_ms(KERNING_FONT);
    let large_ms = fastest_typing_cycle_ms(&large);
    assert!(
        large_ms - small_ms < one_hash_ms / 2.0,
        "입력 한 번에 등록 글꼴을 다시 해시한다: 1.2KB {small_ms:.2}ms, 8MB {large_ms:.2}ms, \
         8MB SHA-256 한 번 {one_hash_ms:.2}ms"
    );
}
