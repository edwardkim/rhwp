//! 관계형·고정형 — [#8950] 긴 ASCII 낱말 앞 줄머리 금칙 묶음의 앞 글자는 다음 줄에 홀로 남는다.
//! 주석 갱신: 2026-11-11 23:25 KST (새 파일)
//!
//! 입력: 공개 결재문서 36446447(한컴오피스 2020 저장) 원본 `pi=11` 은 낱말 단위 문단의 `다만,` + `*`
//! 127개이며 저장 줄(`linesegarray`)이 없어 다시 조판한다. 변형은 같은 문단 글만 바꾼다
//! (`samples/issue8950/36446447-variants.hwpx`). 글자 단위 변형은 #8883 원본 36421178 `pi=7` 의 글을
//! 바꾼다(`samples/issue8950/36421178-variants.hwpx`).
//! 기준: 한컴 engine 2020(11.0.0.9136) Print PDF, PrintMethod 0 —
//! `pdf/issue8950/36446447-forbidden-head-2020.pdf`(2쪽), `36446447-variants-2020.pdf`,
//! `36421178-variants-2020.pdf`.
//! - 고정형: 원본 `pi=11` 은 `다` / `만` / `,***…` / `***…` 네 줄이다.
//! - 관계형: 금칙 문자 앞 글자가 줄 첫 글자가 아니면 그 글자도 다음 줄로 가서 홀로 남고 금칙 문자는
//!   긴 낱말 첫머리다(`가나다` / `만` / `,***…`, 글자 단위 `가나다라` / `나` / `.***…`).
//! - 관계형: 금칙 문자가 없거나 뒤 낱말이 한 줄 안이면 앞 글자는 앞 줄에 남는다.
//!
//! 측정(2026-11-11, 코드 head d899d34157, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, `RHWP_FONT_PATH` 글꼴 공급, WASM 은 같은 폴더를 `--font-path` 로):
//! - 36446447 원본: Native 2쪽 / 2쪽, p1 100.00, p2 100.00 (base 4ab860651b p1 75.95) · fresh WASM p1 100.00,
//!   p2 100.00 · 두 경로 전쪽 최저 100.00
//! - 36446447 변형: Native 3쪽 / 3쪽 전쪽 100.00 (base 최저 37.83) · fresh WASM 최저 99.99
//! - 36421178 변형: Native 2쪽 / 2쪽 전쪽 100.00 (base 최저 48.69) · fresh WASM 전쪽 100.00
//! - 증거: `mydocs/pr/assets/issue_8950/`
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8950/36446447-forbidden-head-before-long-run.hwpx";
const PARA: usize = 11;
const SAMPLE_8883: &str = "samples/issue8883/36421178-long-word-first-char.hwpx";
const PARA_8883: usize = 7;

fn read(rel: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// 저장 줄이 없는 문단 하나의 글만 바꾼 HWPX.
fn with_text(bytes: &[u8], source: &str, text: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert_eq!(xml.matches(source).count(), 1, "원본 문단");
            data = xml.replace(source, text).into_bytes();
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

/// 문단의 줄 글(양쪽 배분 공백 제외).
fn lines(bytes: &[u8], para: usize) -> Vec<String> {
    let core = DocumentCore::from_bytes(bytes).expect("open");
    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut rows = BTreeMap::new();
    walk(&page.root, para, &mut rows);
    rows.into_values()
        .map(|mut row| {
            row.sort_by(|a, b| a.0.total_cmp(&b.0));
            row.into_iter()
                .map(|(_, text)| text)
                .collect::<String>()
                .replace(' ', "")
        })
        .collect()
}

fn word_unit(text: &str) -> Vec<String> {
    let source = format!("다만,{}", "*".repeat(127));
    lines(&with_text(&read(SAMPLE), &source, text), PARA)
}

fn char_unit(text: &str) -> Vec<String> {
    let source = format!("나.{}", "*".repeat(245));
    lines(&with_text(&read(SAMPLE_8883), &source, text), PARA_8883)
}

fn heads(rows: &[String], n: usize) -> Vec<String> {
    rows.iter()
        .take(n)
        .map(|row| row.chars().take(2).collect())
        .collect()
}

#[test]
fn source_paragraph_leaves_each_head_char_alone() {
    let rows = lines(&read(SAMPLE), PARA);
    assert_eq!(rows.len(), 4, "한컴 4줄: {rows:?}");
    assert_eq!(heads(&rows, 3), ["다", "만", ",*"], "{rows:?}");
    let stars: usize = rows.iter().map(|row| row.matches('*').count()).sum();
    assert_eq!(stars, 127, "긴 낱말 보존: {rows:?}");
}

#[test]
fn word_unit_head_char_moves_with_forbidden_tail() {
    let stars = "*".repeat(127);
    for (text, expected) in [
        (format!("가나다만,{stars}"), ["가나다", "만", ",*"]),
        (format!("가 다만,{stars}"), ["가다", "만", ",*"]),
        (format!("다만.{stars}"), ["다", "만", ".*"]),
        (format!("다만,{}", "a".repeat(127)), ["다", "만", ",a"]),
        (format!("다만,{}", "1".repeat(127)), ["다", "만", ",1"]),
    ] {
        let rows = word_unit(&text);
        let got: Vec<String> = rows
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, row)| {
                if i < 2 {
                    row.clone()
                } else {
                    row.chars().take(2).collect()
                }
            })
            .collect();
        assert_eq!(got, expected, "`{text:.8}`: {rows:?}");
    }
}

#[test]
fn word_unit_head_char_stays_without_forbidden_tail_or_long_run() {
    let stars = "*".repeat(127);
    let rows = word_unit(&format!("다만{stars}"));
    assert_eq!(rows[0], "다만", "금칙 문자 없음: {rows:?}");
    let rows = word_unit(&format!("다,{stars}"));
    assert_eq!(
        heads(&rows, 2),
        ["다", ",*"],
        "앞 글자가 줄 첫 글자: {rows:?}"
    );
    for text in [
        format!("다만,{}", "*".repeat(40)),
        format!("다만,{} 뒤글자", "*".repeat(60)),
    ] {
        let rows = word_unit(&text);
        assert_eq!(rows.len(), 1, "뒤 낱말이 한 줄 안: {rows:?}");
    }
}

#[test]
fn char_unit_head_char_moves_with_forbidden_tail() {
    let stars = "*".repeat(245);
    for (text, tail) in [
        (format!("가나다라 나.{stars}"), ".*"),
        (format!("가나다라나.{stars}"), ".*"),
        (format!("가나다라 나,{stars}"), ",*"),
    ] {
        let rows = char_unit(&text);
        assert_eq!(rows[0], "가나다라", "`{text:.8}`: {rows:?}");
        assert_eq!(rows[1], "나", "`{text:.8}`: {rows:?}");
        assert!(rows[2].starts_with(tail), "`{text:.8}`: {rows:?}");
    }
    let rows = char_unit(&format!("가나다라나{stars}"));
    assert_eq!(rows[0], "가나다라나", "금칙 문자 없음: {rows:?}");
}
