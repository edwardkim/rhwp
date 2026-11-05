//! 관계형·고정형 회귀 — [#8539] 저장 줄의 쪽·칸 소속과 독립 PDF의 전체 3쪽을 확인한다.
//! 주석 갱신: 2026-11-05T22:34:19+09:00.
//! 직전 파일 변경: 2026-11-03T21:23:59+09:00, `1543ae62fe07137da39bbaa88aa82884cc084bf2`.
//! 2026-11-05 실측: PDFium 96dpi, 색상 경계 보정 전 2px raw 실루엣
//! (`threshold_boundary_color_support_v1`), 같은 HWP·독립 Hwp 2020 PDF의 전 3쪽.
//! 1쪽 Native 100.00000% / fresh WASM Canvas 100.00000%,
//! 2쪽 Native 100.00000% / fresh WASM Canvas 100.00000%,
//! 3쪽 Native 100.00000% / fresh WASM Canvas 99.99607%; 전쪽 최저도 각각 100.00000% / 99.99607%.
//! 근거: `mydocs/pr/assets/pr8542/visual-validation.md`의 전쪽 TSV·입력/빌드/글꼴 출처와 직접 판독.
//! 관계형은 줄의 소유·순서·보존·행의 비겹침, 고정형은 독립 PDF의 전체 쪽수다. 절대 px 핀은 없다.
//!
//! 쪽을 넘는 RowBreak 표 칸은 저장 재시작이 아니라 남은 공간까지 줄을 싣는다.
//!
//! 입력 `samples/issue8539/nhis-dependent-requirements.hwp`(HWP 5.0.3.4, 18행×3열 RowBreak 표)는
//! 셀 줄에 vpos 0 재시작을 저장했다. 9행은 가운데·오른쪽 칸 셋째 줄, 16행은 왼쪽 칸 `p[4]` 첫 줄과
//! 오른쪽 칸 열째 줄이다. 같은 곳에 공개된 한/글 2020 Print PDF
//! (`nhis-dependent-requirements-2020.pdf`)는 두 경계 모두 저장 재시작보다 한 줄을 더 앞 쪽에 싣는다.
//!
//! | 경계 | 행 | 칸별 앞 쪽 줄 수 (한/글 2020) | 수정 전 rhwp |
//! |---|---|---|---|
//! | 1→2쪽 | 9 「5. 손ㆍ외손」 | 2 / 3 / 3 | 2 / 2 / 2 |
//! | 2→3쪽 | 16 「10. 형제ㆍ자매」 | 10 / 8 / 10 | 9 / 8 / 9 |
//!
//! 원인은 두 겹이다. ① 저장 재시작 흡수 판정이 다음 줄을 끝 줄간격까지 세어, 용량 컷으로는 드는
//! 줄을 재시작에서 잘랐다. ② 앞 조각이 저장 프레임 중간에서 잘린 이어받은 조각도 뒤 행의 저장
//! 프레임 끝을 이 쪽의 끝으로 인증했다. 칸 줄 수의 합은 원본 저장 줄(LineSeg) 수다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue8539/nhis-dependent-requirements.hwp";

/// 한 쪽에 그려진 칸 글줄 — `(행, 열)` 별로 위에서 아래 순서. 공백은 비교에서 뺀다.
#[derive(Default)]
struct PageCells {
    lines: BTreeMap<(u64, u64), Vec<(f64, f64, String)>>,
}

impl PageCells {
    fn texts(&self, row: u64, col: u64) -> Vec<&str> {
        self.lines
            .get(&(row, col))
            .map(|lines| lines.iter().map(|(_, _, text)| text.as_str()).collect())
            .unwrap_or_default()
    }

    fn count(&self, row: u64, col: u64) -> usize {
        self.texts(row, col).len()
    }

    /// 행의 가장 위·아래 글줄 범위 `(위, 아래)`.
    fn row_span(&self, row: u64) -> Option<(f64, f64)> {
        self.lines
            .iter()
            .filter(|((r, _), _)| *r == row)
            .flat_map(|(_, lines)| lines.iter())
            .fold(None, |span, (top, bottom, _)| match span {
                None => Some((*top, *bottom)),
                Some((t, b)) => Some((t.min(*top), b.max(*bottom))),
            })
    }
}

fn pages() -> Vec<PageCells> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");

    fn line_text(node: &Value, out: &mut String) {
        if node["type"] == "TextRun" {
            out.push_str(node["text"].as_str().unwrap_or_default());
        }
        for child in node["children"].as_array().into_iter().flatten() {
            line_text(child, out);
        }
    }
    fn walk(node: &Value, cell: Option<(u64, u64)>, page: &mut PageCells) {
        let cell = if node["type"] == "Cell" {
            Some((node["row"].as_u64().unwrap(), node["col"].as_u64().unwrap()))
        } else {
            cell
        };
        if node["type"] == "TextLine" {
            if let Some(key) = cell {
                let mut text = String::new();
                line_text(node, &mut text);
                let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                if !text.is_empty() {
                    let top = node["bbox"]["y"].as_f64().unwrap();
                    let bottom = top + node["bbox"]["h"].as_f64().unwrap();
                    page.lines.entry(key).or_default().push((top, bottom, text));
                }
            }
            return;
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, cell, page);
        }
    }

    (0..doc.page_count())
        .map(|index| {
            let tree: Value =
                serde_json::from_str(&doc.get_page_render_tree(index).unwrap()).unwrap();
            let mut page = PageCells::default();
            walk(&tree, None, &mut page);
            for lines in page.lines.values_mut() {
                lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            }
            page
        })
        .collect()
}

#[test]
fn row_split_lines_follow_hancom_page_boundaries() {
    let pages = pages();
    assert_eq!(pages.len(), 3, "한/글 2020 PDF 는 3쪽이다");

    let row9: Vec<_> = (0..3).map(|col| pages[0].count(9, col)).collect();
    assert_eq!(
        row9,
        vec![2, 3, 3],
        "9행은 1쪽에 칸별 2/3/3 줄을 싣는다(한/글 2020) — 수정 전 rhwp 는 저장 재시작에서 2/2/2"
    );
    assert_eq!(pages[0].texts(9, 1).last(), Some(&"수또는소득이없는경우"));
    assert_eq!(pages[0].texts(9, 2).last(), Some(&"경우부양인정.다만,이"));
    assert_eq!(pages[1].texts(9, 1).first(), Some(&"부양인정"));
    assert_eq!(pages[1].texts(9, 2).first(), Some(&"혼ㆍ사별한경우자녀인"));

    let row16: Vec<_> = (0..3).map(|col| pages[1].count(16, col)).collect();
    assert_eq!(
        row16,
        vec![10, 8, 10],
        "16행은 2쪽에 칸별 10/8/10 줄을 싣는다(한/글 2020) — 수정 전 rhwp 는 9/8/9"
    );
    assert_eq!(pages[1].texts(16, 0).last(), Some(&"라.「국가유공자"));
    assert_eq!(pages[1].texts(16, 2).last(), Some(&"있어도보수또는소득이"));
    assert_eq!(pages[2].texts(16, 0).first(), Some(&"등예우및지원"));
    assert_eq!(pages[2].texts(16, 2).first(), Some(&"없는경우부양인정"));
}

#[test]
fn split_rows_keep_every_stored_line_once() {
    let pages = pages();
    // 원본 셀의 저장 줄(LineSeg) 수: 9행 2/4/7, 16행 36/8/11.
    for (row, stored) in [(9, [2usize, 4, 7]), (16, [36, 8, 11])] {
        for (col, &expected) in stored.iter().enumerate() {
            let col = col as u64;
            let drawn: Vec<&str> = pages.iter().flat_map(|page| page.texts(row, col)).collect();
            assert_eq!(
                drawn.len(),
                expected,
                "{row}행 {col}열의 저장 줄은 모든 쪽을 합쳐 한 번씩 그린다"
            );
            for pair in pages.windows(2) {
                if let (Some(last), Some(first)) = (
                    pair[0].texts(row, col).last(),
                    pair[1].texts(row, col).first(),
                ) {
                    assert_ne!(last, first, "{row}행 {col}열의 쪽 경계 줄이 겹쳐 그려졌다");
                }
            }
        }
    }
}

#[test]
fn continued_row_stays_above_next_row() {
    let pages = pages();
    // 2쪽 첫머리의 9행 남은 줄은 10행(「6. 직계비속의 배우자」) 글자 위에 겹치지 않는다(#8198 ①).
    let (_, continued_bottom) = pages[1].row_span(9).expect("2쪽에 9행 남은 줄이 있다");
    let (next_top, _) = pages[1].row_span(10).expect("2쪽에 10행이 있다");
    assert!(
        continued_bottom <= next_top,
        "9행 남은 줄 아래({continued_bottom:.1})는 10행 글줄 위({next_top:.1})를 넘지 않는다"
    );
}
