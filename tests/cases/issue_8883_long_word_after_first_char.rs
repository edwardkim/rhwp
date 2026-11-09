//! 관계형·고정형 — [#8883] 저장 줄 없는 글자 단위 문단의 긴 낱말과 줄머리 금칙 묶음.
//! 주석 갱신: 2026-11-10 09:40 KST (새 파일)
//!
//! 입력: 공개 결재문서 36421178(한컴오피스 2020 저장) 원본. `pi=7` 은 `나.` + `*` 245개, `pi=8` 은
//! `다` + `*` 189개이며 저장 줄(`linesegarray`)이 없어 다시 조판한다. 두 문단은 내어쓰기 2434HU 의
//! 글자 단위 문단이다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8883/36421178-long-word-first-char-2020.pdf`(2쪽).
//! - 고정형: `pi=7` 줄 길이 1·78·78·78·12, `pi=8` 줄 길이 1·78·78·33. 첫 글자만 첫 줄에 남고 낱말은 다음 줄부터 나뉜다.
//!
//! 같은 PDF 경로로 `pi=7` 문장만 바꾼 변형 18종을 출력해 줄머리 금칙 경계를 정했다.
//! - 관계형: 뒤 낱말이 한 줄을 넘으면 금칙 묶음은 뒤 낱말로 간다(`나` / `.)***…`).
//! - 관계형: 뒤 낱말이 한 줄 안이면 금칙 묶음은 앞 글자와 함께 다음 줄로 간다(`카.a`·`카.)***`·`카.)`).
//!
//! 측정(2026-11-10, head a224b037fa, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 문서 face 굴림 공급):
//! - Native: 2쪽 / 2쪽, p1 100.00, p2 100.00, 전쪽 최저 100.00 (base b5064fae1e 는 p1 64.30)
//! - fresh WASM: 2쪽 / 2쪽, p1 100.00, p2 100.00, 전쪽 최저 100.00
//! - 증거: `mydocs/pr/assets/issue_8883/`
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8883/36421178-long-word-first-char.hwpx";
const SOURCE_TEXT_LEN: usize = 245;
const HANGUL_41: &str =
    "가나다라마바사아자차가나다라마바사아자차가나다라마바사아자차가나다라마바사아자차카";

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// `pi=7` 글만 바꾼 HWPX. 저장 줄이 없는 문단이므로 조판 입력은 글자열뿐이다.
fn with_para7_text(text: &str) -> Vec<u8> {
    let source = format!("나.{}", "*".repeat(SOURCE_TEXT_LEN));
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert_eq!(xml.matches(source.as_str()).count(), 1, "원본 문단");
            data = xml.replace(source.as_str(), text).into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn walk(node: &RenderNode, para: usize, rows: &mut BTreeMap<i64, Vec<(f64, String)>>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(para) && run.cell_context.is_none() && !run.text.is_empty() {
            rows.entry((node.bbox.y * 2.0).round() as i64)
                .or_default()
                .push((node.bbox.x, run.text.clone()));
        }
    }
    for child in &node.children {
        walk(child, para, rows);
    }
}

fn lines(bytes: &[u8], para: usize) -> Vec<String> {
    let core = DocumentCore::from_bytes(bytes).expect("open");
    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut rows = BTreeMap::new();
    walk(&page.root, para, &mut rows);
    rows.into_values()
        .map(|mut row| {
            row.sort_by(|a, b| a.0.total_cmp(&b.0));
            row.into_iter().map(|(_, text)| text).collect::<String>()
        })
        .collect()
}

fn lengths(lines: &[String]) -> Vec<usize> {
    lines.iter().map(|line| line.chars().count()).collect()
}

#[test]
fn long_word_leaves_first_char_alone_and_refits_to_hanging_width() {
    let bytes = sample_bytes();
    let para7 = lines(&bytes, 7);
    assert_eq!(para7[0], "나", "첫 줄에는 첫 글자만 남는다: {para7:?}");
    assert!(
        para7[1].starts_with(".*"),
        "줄머리 `.` 은 긴 낱말 쪽: {para7:?}"
    );
    assert_eq!(
        lengths(&para7),
        [1, 78, 78, 78, 12],
        "내어쓰기 폭으로 다시 채운다"
    );
    let para8 = lines(&bytes, 8);
    assert_eq!(para8[0], "다");
    assert_eq!(lengths(&para8), [1, 78, 78, 33]);
}

#[test]
fn forbidden_tail_joins_word_longer_than_a_line() {
    for head in [".)", ".,", "..", ".:", ".\""] {
        let text = format!("나{}{}", head.replace('"', "&quot;"), "*".repeat(243));
        let rows = lines(&with_para7_text(&text), 7);
        assert_eq!(rows[0], "나", "`{head}` 뒤 낱말이 한 줄을 넘는다: {rows:?}");
        assert!(
            rows[1].starts_with(head),
            "`{head}` 는 다음 줄 첫머리: {rows:?}"
        );
    }
}

#[test]
fn forbidden_tail_stays_with_previous_char_before_word_within_a_line() {
    for tail in [
        ".a 뒤글자",
        ".) 뒤글자",
        ".)*** 뒤글자",
        ". 뒤글자",
        "., 뒤글자",
        ")))) 뒤글자",
    ] {
        let rows = lines(&with_para7_text(&format!("{HANGUL_41}{tail}")), 7);
        assert_eq!(rows.len(), 2, "`{tail}`: {rows:?}");
        assert_eq!(
            rows[0].chars().count(),
            40,
            "`{tail}` 앞 글자도 다음 줄로: {rows:?}"
        );
        assert_eq!(rows[1], format!("카{tail}"), "`{tail}`: {rows:?}");
    }
}
