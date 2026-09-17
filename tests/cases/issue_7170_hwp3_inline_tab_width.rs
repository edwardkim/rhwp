//! [Issue #7170] HWP3→HWP5 저장에서 차례의 탭 채움(점선)이 사라진다.
//!
//! HWP3 은 인라인 탭을 8바이트로 저장하고 그 안에 **탭 폭과 점끌기 여부**를 담는다
//! (스펙 `한글문서파일구조3.0.md` §10.5 표 39).
//!
//! ```text
//!   offset 0: hchar(=9)   여는 코드
//!   offset 2: hunit       탭 폭
//!   offset 4: word        점끌기 여부
//!   offset 6: hchar(=9)   닫는 코드
//! ```
//!
//! HWP5 도 같은 정보를 본문 안 8코드유닛 탭 확장에 싣는다. 종전 파서는 그 6바이트를
//! 읽고 **버려서**, HWP3 에서 온 문단이 저장본에서 탭 폭 0 · 채움 없음이 됐다. 한글은
//! 문단 `TabDef` 가 아니라 이 인라인 값으로 점선을 그리므로 차례가 빈칸이 된다.
//!
//! # 기대값의 출처
//!
//! 정본은 한글 자신의 HWP5 변환본이다. 표본 264쪽
//! `1170000-200500003_D0150004-1-001`(탭 112개)에서 우리 저장본의 탭 확장 112개가
//! 정본과 **바이트 단위로 전부 같아진다**(폭·`ext[2]` 모두 112/112 일치).
//!
//! 정본 확장의 형상은 `[폭_lo, 폭_hi, 0x0003, 0, 0, 0, 0x0009]` 이고, 폭 112개가
//! 모두 4의 배수여서 HWP3 hunit 에 4를 곱한 값과 맞는다.
//!
//! 한글이 그린 PDF 의 점(U+00B7) 개수로도 확인된다.
//!
//! ```text
//!   한글 정본   264쪽  248,006자  점 9,189
//!   수정 전     273쪽  239,649자  점     0
//!   수정 후     273쪽  248,839자  점 9,189
//! ```
//!
//! # 이 시험이 잠그는 것
//!
//! 저장소 HWP3 표본에서 (1) 인라인 탭 확장이 실리는지, (2) 폭이 0 이 아닌지,
//! (3) 채움이 문서가 말한 대로 갈리는지를 본다. **반례**로 같은 문서 안에 점끌기를
//! 끈 탭이 함께 있어야 한다 — 전부 점선으로 칠하는 구현은 이 시험을 통과하지 못한다.
//!
//! # 잠그지 않는 것
//!
//! `ext[2]` 고바이트(탭 종류)는 정본이 0(미지정)을 쓰므로 건드리지 않는다. 우리
//! 렌더러가 인라인 탭 채움으로 점선을 그리지 않는 **읽기 쪽** 축도 범위 밖이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::paragraph::Paragraph;
use std::path::Path;

fn load(rel: &str) -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    rhwp::parser::parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱")
}

/// 문서의 모든 문단을 훑어 인라인 탭 확장을 모은다(표 칸·주석 안까지).
fn collect_tabs(paragraphs: &[Paragraph], out: &mut Vec<[u16; 7]>) {
    use rhwp::model::control::Control;
    for para in paragraphs {
        out.extend(para.tab_extended.iter().copied());
        for control in &para.controls {
            match control {
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_tabs(&cell.paragraphs, out);
                    }
                }
                Control::Footnote(note) => collect_tabs(&note.paragraphs, out),
                Control::Endnote(note) => collect_tabs(&note.paragraphs, out),
                Control::Header(hf) => collect_tabs(&hf.paragraphs, out),
                Control::Footer(hf) => collect_tabs(&hf.paragraphs, out),
                _ => {}
            }
        }
    }
}

fn tabs_of(rel: &str) -> Vec<[u16; 7]> {
    let doc = load(rel);
    let mut out = Vec::new();
    for section in &doc.sections {
        collect_tabs(&section.paragraphs, &mut out);
    }
    out
}

/// HWP3 인라인 탭이 폭을 싣는다 — 종전에는 `tab_extended` 가 비어 저장본이 폭 0 이었다.
#[test]
fn hwp3_inline_tab_carries_its_width() {
    for rel in [
        "samples/hwp3-sample10.hwp",
        "samples/hwp3-sample16.hwp",
        "samples/SO-SUEOP.hwp",
    ] {
        let tabs = tabs_of(rel);
        assert!(
            !tabs.is_empty(),
            "{rel}: 인라인 탭 확장이 하나도 없다 — 저장본에서 탭 폭과 채움을 통째로 잃는다"
        );
        let zero_width = tabs
            .iter()
            .filter(|ext| ((ext[0] as u32) | ((ext[1] as u32) << 16)) == 0)
            .count();
        assert_eq!(
            zero_width,
            0,
            "{rel}: 탭 {}개 중 {zero_width}개가 폭 0 이다 — 한글이 점선을 그리지 않는다",
            tabs.len()
        );
        assert!(
            tabs.iter().all(|ext| ext[6] == 0x0009),
            "{rel}: 탭 확장의 닫는 마커가 0x0009 가 아니다"
        );
    }
}

/// 채움은 문서가 말한 대로 갈린다 — 점끌기를 끈 탭까지 점선으로 칠하지 않는다.
#[test]
fn hwp3_inline_tab_fill_follows_the_document() {
    let tabs = tabs_of("samples/hwp3-sample10.hwp");
    let dotted = tabs.iter().filter(|ext| ext[2] & 0xFF == 3).count();
    let plain = tabs.iter().filter(|ext| ext[2] & 0xFF == 0).count();
    assert!(
        dotted > 0,
        "점끌기를 켠 탭이 하나도 없다 — 표본 전제가 깨졌거나 채움을 잃었다"
    );
    assert!(
        plain > 0,
        "점끌기를 끈 탭이 하나도 없다 — 전부 점선으로 칠하는 구현은 이 반례를 못 지킨다"
    );
    assert_eq!(
        dotted + plain,
        tabs.len(),
        "채움 종류가 0 과 3 말고 다른 값으로 나왔다 — HWP3 인라인 탭은 점끌기 on/off 뿐이다"
    );
    assert!(
        tabs.iter().all(|ext| ext[2] >> 8 == 0),
        "탭 종류(ext[2] 고바이트)는 정본이 0(미지정)을 쓴다 — 여기서 채우지 않는다"
    );
}

/// 탭 확장 개수가 본문의 `\t` 개수와 맞는다 — 직렬화가 `\t` 순번으로 확장을 꺼낸다.
#[test]
fn hwp3_inline_tab_count_matches_the_tab_characters() {
    for rel in ["samples/hwp3-sample16.hwp", "samples/SO-SUEOP.hwp"] {
        let doc = load(rel);
        let mut checked = 0usize;
        for section in &doc.sections {
            for para in &section.paragraphs {
                if para.tab_extended.is_empty() {
                    continue;
                }
                assert_eq!(
                    para.tab_extended.len(),
                    para.text.matches('\t').count(),
                    "{rel}: 탭 확장 개수가 본문 '\\t' 개수와 다르다 — 직렬화가 엉뚱한 확장을 쓴다"
                );
                checked += 1;
            }
        }
        assert!(
            checked > 0,
            "{rel}: 탭이 있는 문단을 하나도 못 찾았다 — 대상 0건은 통과 증거가 아니다"
        );
    }
}
