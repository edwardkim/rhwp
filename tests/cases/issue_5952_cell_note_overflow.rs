//! [#5952] 행정업무운영편람 61쪽 유의사항 상자 줄이 셀 오른쪽을 넘어
//! 사이드바 "공문서"와 겹치지 않는다.
//!
//! 저장 LINE_SEG 는 2줄(horzsize=37560HU ≈ 500.8px)인데, rhwp 는 일부 문단을
//! 1줄로 그려 maxx≈680 까지 보낸다. 상자 오른쪽은 ≈600, 사이드바는 ≈671.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// `-p` 는 0 기준. 한글 인쇄 쪽번호 61 은 rhwp 69쪽(-p 68).
const PAGE_ARG: &str = "68";
/// 유의사항 상자 오른쪽(~600)과 사이드바(~671) 사이.
const BOX_RIGHT_LIMIT: f64 = 640.0;

fn render_page_svg() -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5952_{}_{}", std::process::id(), nth));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            PAGE_ARG,
            "-o",
            out.to_str().expect("출력 경로"),
        ])
        .output()
        .expect("rhwp export-svg 실행");
    assert!(
        done.status.success(),
        "export-svg 실패: {}",
        String::from_utf8_lossy(&done.stderr)
    );
    let svg = std::fs::read_dir(&out)
        .expect("출력 디렉토리")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .expect("SVG 산출물");
    let text = std::fs::read_to_string(svg).expect("SVG 읽기");
    let _ = std::fs::remove_dir_all(&out);
    text
}

fn glyph_x(svg: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(open_end) = chunk.find('>') else {
            continue;
        };
        let (attrs, rest) = chunk.split_at(open_end);
        let Some(close) = rest[1..].find("</text>") else {
            continue;
        };
        let body = rest[1..close + 1].to_string();
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))?;
            let tail = &attrs[at + key.len() + 2..];
            tail[..tail.find('"')?].parse().ok()
        };
        let x = pick("x").or_else(|| {
            let at = attrs.find("translate(")?;
            let tail = &attrs[at + 10..];
            tail.split([',', ' ', ')']).next()?.parse().ok()
        });
        let y = pick("y").or_else(|| {
            let at = attrs.find("translate(")?;
            let tail = &attrs[at + 10..];
            let mut parts = tail.split([',', ' ', ')']);
            parts.next()?;
            parts.next()?.parse().ok()
        });
        if let (Some(x), Some(y)) = (x, y) {
            if !body.trim().is_empty() {
                out.push((x, y, body));
            }
        }
    }
    out
}

fn text_rows(svg: &str) -> Vec<(f64, String)> {
    let mut rows: std::collections::BTreeMap<i32, Vec<(f64, String)>> =
        std::collections::BTreeMap::new();
    for (x, y, text) in glyph_x(svg) {
        if (70.0..=690.0).contains(&x) {
            rows.entry((y * 10.0).round() as i32)
                .or_default()
                .push((x, text));
        }
    }
    rows.into_iter()
        .map(|(key, mut cells)| {
            cells.sort_by(|a, b| a.0.total_cmp(&b.0));
            (
                key as f64 / 10.0,
                cells.into_iter().map(|(_, text)| text).collect::<String>(),
            )
        })
        .collect()
}

#[test]
fn note_box_lines_do_not_overlap_the_gongmunseo_sidebar() {
    let svg = render_page_svg();
    let glyphs = glyph_x(&svg);
    assert!(
        glyphs
            .iter()
            .any(|(_, _, t)| t.contains("기") || t.contains("안")),
        "69쪽에 유의사항 상자 글자가 없다"
    );

    let mut overflow = Vec::new();
    for (x, y, ch) in &glyphs {
        if *y < 150.0 || *y > 420.0 {
            continue;
        }
        if *x > BOX_RIGHT_LIMIT && *x < 670.0 {
            overflow.push((*x, *y, ch.clone()));
        }
    }
    assert!(
        overflow.is_empty(),
        "유의사항 상자 글자가 셀 오른쪽(>{BOX_RIGHT_LIMIT})으로 넘어 사이드바와 겹친다: {:?}",
        overflow.iter().take(12).collect::<Vec<_>>()
    );
}

#[test]
fn note_box_bottom_stays_separate_from_following_body_heading() {
    let svg = render_page_svg();
    let rows = text_rows(&svg);
    let note_bottom_y = rows
        .iter()
        .filter_map(|(y, row)| {
            ((200.0..500.0).contains(y)
                && (row.contains('※')
                    || row.contains("붙임")
                    || row.contains("문서보안")
                    || row.contains("개인정보")
                    || row.contains("전자기록")))
            .then_some(*y)
        })
        .reduce(f64::max)
        .expect("유의사항 상자 본문 줄");
    let following_heading_y = rows
        .iter()
        .find_map(|(y, row)| {
            (*y > note_bottom_y && row.contains("4)") && row.contains("문서")).then_some(*y)
        })
        .expect("상자 아래의 '4) 문서의 끝 표시' 본문 줄");

    assert!(
        following_heading_y - note_bottom_y >= 45.0,
        "유의사항 상자 하단이 다음 본문 줄과 분리되어야 함: note_bottom_y={note_bottom_y:.1}, following_heading_y={following_heading_y:.1}"
    );
}

/// [#5952] 저장 2줄이 composed 1줄로 접혀 셀 내폭을 넘치면 fresh 재래핑한다.
/// source-side `#[cfg(test)]` 가 아니라 integration 으로 둔다 (RustUnitTier).
#[test]
fn collapsed_stored_two_lines_rewrap_when_over_cell() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = format!("※{}", "가".repeat(39));
    let n = text.chars().count();
    let collapsed = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    };
    let stored_two = Paragraph {
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: 20,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..collapsed.clone()
    };
    let mut composed = compose_paragraph(&collapsed);
    assert_eq!(composed.lines.len(), 1, "1-ls 문단은 1줄로 시작한다");
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines.len() > 1,
        "유의사항 bullet 저장 2줄이 셀을 넘치면 재래핑돼야 함 (#5952)"
    );
}

/// 끝의 빈 LINE_SEG는 저장 행이 접힌 결함이 아니므로 재래핑하지 않는다.
#[test]
fn trailing_empty_stored_line_seg_is_preserved() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = "가".repeat(40);
    let n = text.chars().count();
    let stored_two = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: n as u32,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut composed = compose_paragraph(&stored_two);
    assert_eq!(
        composed.lines.len(),
        2,
        "빈 둘째 저장 행도 composed에 남는다"
    );
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines[1].runs.is_empty(),
        "끝의 빈 저장 행은 fresh 재래핑으로 채우지 않아야 함 (#5952)"
    );
}
