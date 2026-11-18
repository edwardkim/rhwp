//! 관계형·고정형 — [#8996] 편집하지 않은 HWP 를 HWPX 로 내보내면 표 높이(sz)를 원본 그대로 쓰고, 다시 열어도 쪽 수가 같다.
//! 주석 갱신: 2026-11-12 19:40 KST (새 파일)
//!
//! 입력: `samples/80168_regulatory_analysis.hwp`(저장 줄 없는 칸 문단·옛 rhwp 줄 표지 칸 문단을 가진 표가 많다).
//! 기준: 한컴 engine 2020 — 같은 HWP 를 HWPX 로 저장하면 표 184개의 높이가 모두 원본과 같다.
//! Print PDF(PrintMethod 0)는 원본 HWP `pdf/issue8996/80168-hwp-2020.pdf`, rhwp 가 내보낸 HWPX
//! `pdf/issue8996/80168-rhwp-hwpx-2020.pdf` 모두 157쪽이다.
//! - 고정형: 쪽 수 157.
//! - 관계형: 내보낸 뒤 다시 연 표 높이 = 원본 표 높이(184개), 쪽별 글 = 원본 쪽별 글.
//!
//! 측정(2026-11-12, 코드 head e6fc6637b3, base 80b1c354c2, `scripts/visual_sweep.py --silhouette-only
//! --dpi 96`, raw = `silhouette_raw_match_percent`, `RHWP_FONT_PATH` 글꼴 공급, `--compat 2022`):
//! - 내보낸 HWPX Native 157쪽 / 157쪽: 표가 나뉘는 13쪽 99.82·14쪽 99.80. fresh WASM 157쪽 / 157쪽: 13쪽 100.00·14쪽 100.00.
//! - 두 경로 전쪽 최저: Native 83.88, WASM 84.02 — 80쪽(줄 나눔 한 글자 차)·112쪽 88.03/88.05 두 쪽이
//!   90% 미만이다. 원본 HWP 를 devel 로 그려도 같은 값인 기존 미달이라 #9002 로 분리했다.
//! - 증거: `mydocs/pr/assets/issue_8996/`
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

const SAMPLE: &str = "samples/80168_regulatory_analysis.hwp";

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("open")
}

/// 본문·칸 안 모든 표의 (문단 경로, 저장 높이)를 깊이 우선 순서로 모은다.
fn table_heights(paragraphs: &[Paragraph], path: &str, out: &mut Vec<(String, u32)>) {
    for (index, paragraph) in paragraphs.iter().enumerate() {
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                let here = format!("{path}{index}");
                out.push((here.clone(), table.common.height));
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    table_heights(&cell.paragraphs, &format!("{here}>c{cell_index}>"), out);
                }
            }
        }
    }
}

fn heights(core: &DocumentCore) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for section in &core.document().sections {
        table_heights(&section.paragraphs, "", &mut out);
    }
    out
}

/// 원본은 저장 줄이 없는 칸 문단과 옛 rhwp 줄 표지(bit 31) 칸 문단을 가진 표가 많다. 한컴 2020 이
/// 같은 HWP 를 HWPX 로 저장한 결과는 표 184개의 높이가 모두 원본과 같다. rhwp 도 편집하지 않았으면
/// 원본 높이를 써야 한다 — 종전에는 23개 표를 rhwp 측정 높이로 키웠다(문단 97 표 20204 → 24523 HU).
#[test]
fn unedited_export_keeps_every_stored_table_height() {
    let bytes = std::fs::read(SAMPLE).expect("sample");
    let original = open(&bytes);
    let before = heights(&original);
    assert_eq!(before.len(), 184, "본문·칸 표 184개");
    let exported = original.export_hwpx_native().expect("export hwpx");
    let reopened = open(&exported);
    let after = heights(&reopened);
    let changed: Vec<_> = before.iter().zip(&after).filter(|(a, b)| a != b).collect();
    assert_eq!(after.len(), before.len(), "표 수 보존");
    assert!(
        changed.is_empty(),
        "편집 없는 저장의 표 높이 변경: {changed:?}"
    );
}

/// 한컴 engine 2020 Print 는 원본 HWP·rhwp 가 내보낸 HWPX 모두 157쪽이다
/// (`pdf/issue8996/80168-hwp-2020.pdf`, `pdf/issue8996/80168-rhwp-hwpx-2020.pdf`). 다시 연 HWPX 도
/// 같은 쪽 수와 쪽별 글이어야 한다 — 종전에는 문단 97 표가 키운 선언 높이로 통째로 다음 쪽에
/// 넘어가 159쪽이 됐다.
#[test]
fn reopened_hwpx_keeps_page_count_and_page_texts() {
    let bytes = std::fs::read(SAMPLE).expect("sample");
    let original = open(&bytes);
    assert_eq!(original.page_count(), 157, "원본 한컴 Print 157쪽");
    let reopened = open(&original.export_hwpx_native().expect("export hwpx"));
    assert_eq!(reopened.page_count(), 157, "다시 연 HWPX 한컴 Print 157쪽");
    for page in 0..157usize {
        let a: String = original
            .page_text(page)
            .expect("원본 쪽 글")
            .split_whitespace()
            .collect();
        let b: String = reopened
            .page_text(page)
            .expect("HWPX 쪽 글")
            .split_whitespace()
            .collect();
        assert_eq!(a, b, "{}쪽 글 소속", page + 1);
    }
}
