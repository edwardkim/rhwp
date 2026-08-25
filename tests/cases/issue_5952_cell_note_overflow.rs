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
