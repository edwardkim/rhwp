//! HWP3 표 격자를 셀 왼쪽·위쪽 변으로 세운다.
//!
//! 오른쪽·아래쪽 변까지 경계로 넣고 40HU 허용치로 묶으면, 셀 높이 합이 틀보다 짧은 칸이 가짜
//! 행을 만들고 1단위 어긋난 변이 다른 열과 합쳐진다. 기대값은 같은 문서의 한컴 변환본
//! (`samples/hwp3-sample16-hwp5.hwpx`, `pdf/issue7642-hwp3-realign-controls-20261012/
//! hwp3-table-cell-order-hancom-2020.hwpx`)의 rowCnt·colCnt 다.
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

fn tables(name: &str) -> Vec<(u16, u16, String)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let raw = std::fs::read(&path).expect("read sample");
    let doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    fn walk(paragraphs: &[Paragraph], out: &mut Vec<(u16, u16, String)>) {
        for p in paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    let text = t
                        .cells
                        .iter()
                        .flat_map(|cell| cell.paragraphs.iter().map(|q| q.text.as_str()))
                        .collect::<Vec<_>>()
                        .join("|");
                    out.push((t.row_count, t.col_count, text));
                    for cell in &t.cells {
                        walk(&cell.paragraphs, out);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for sec in &doc.sections {
        walk(&sec.paragraphs, &mut out);
    }
    out
}

fn grid_of(all: &[(u16, u16, String)], key: &str) -> (u16, u16) {
    let found: Vec<_> = all.iter().filter(|t| t.2.contains(key)).collect();
    assert_eq!(found.len(), 1, "'{key}' 를 담은 표가 하나여야 한다");
    (found[0].0, found[0].1)
}

#[test]
fn sample16_tables_match_the_hancom_grid() {
    let all = tables("hwp3-sample16.hwp");
    assert_eq!(grid_of(&all, "접 수 번 호"), (15, 5));
    assert_eq!(grid_of(&all, "사업책임자(주계약자)"), (23, 17));
    assert_eq!(grid_of(&all, "투입인력 개인별 경력사항 확인서"), (15, 17));
}

#[test]
fn cell_order_table_matches_the_hancom_grid() {
    let all = tables("hwp3-table-cell-order.hwp");
    let (rows, cols) = all
        .iter()
        .map(|t| (t.0, t.1))
        .max_by_key(|g| g.0 as u32 * g.1 as u32)
        .expect("표");
    assert_eq!((rows, cols), (29, 12));
}
