//! [Issue #4882] 정책연구용역사업 중간진도보고서 HWPX 왕복 쪽수 보존.
//!
//! 원본 HWP5 각주 subList lineseg 는 후속 줄 vertpos=0 이 저장값이다.
//! 재파싱이 이를 1172/2344 로 쌓으면 --verify 가 5건 IR 차이를 내고
//! pages(원본)=215, pages(export-hwpx→reimport)=223 이 된다.
//!
//! 이 시험은 그 5경로 vpos 와 쪽수 등식을 고정한다. #4056 #5128 은 건드리지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Section, HWP5_ORIGIN_HWPX_MARKER_PATH};
use rhwp::model::paragraph::Paragraph;
use rhwp::parse_document;
use rhwp::parser::hwpx::{parse_hwpx, section::parse_hwpx_section};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";
const EXPECTED_PAGES: u32 = 215;

/// #4882 기계 판정이 남긴 5개 IR 경로. 원본 후속 줄 vertpos=0.
const PINNED_NOTE_PATHS: &[&str] = &[
    "section[0] paragraph[421]/ctrl[0]fn.p[0]",
    "section[0] paragraph[728]/ctrl[0]tbl.cell[3].p[0]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1372]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1832]/ctrl[0]tbl.cell[0].p[3]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1865]/ctrl[0]fn.p[0]",
];

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn paragraph_at<'a>(paragraphs: &'a [Paragraph], index: usize) -> &'a Paragraph {
    paragraphs
        .get(index)
        .unwrap_or_else(|| panic!("paragraph[{index}] 없음"))
}

fn footnote_para_vpos(paragraphs: &[Paragraph], path: &str) -> Vec<i32> {
    // 좁은 경로만 해석한다. #4882 5경로는 fn / tbl.cell.p / fn.p 뿐이다.
    if let Some(rest) = path.strip_prefix("section[0] paragraph[") {
        let (idx_s, rest) = rest.split_once(']').expect("paragraph index");
        let para = paragraph_at(paragraphs, idx_s.parse().unwrap());
        return footnote_para_vpos_from(&para.controls, rest);
    }
    panic!("지원하지 않는 경로: {path}");
}

fn footnote_para_vpos_from(controls: &[Control], path: &str) -> Vec<i32> {
    if let Some(rest) = path.strip_prefix("/ctrl[") {
        let (idx_s, rest) = rest.split_once(']').expect("ctrl index");
        let idx: usize = idx_s.parse().unwrap();
        let ctrl = controls
            .get(idx)
            .unwrap_or_else(|| panic!("ctrl[{idx}] 없음"));
        return match (ctrl, rest) {
            (Control::Footnote(n), rest) if rest.starts_with("fn.p[") => {
                let rest = rest.strip_prefix("fn.p[").unwrap();
                let (idx_s, _) = rest.split_once(']').expect("fn.p index");
                let pidx: usize = idx_s.parse().unwrap();
                n.paragraphs[pidx]
                    .line_segs
                    .iter()
                    .map(|s| s.vertical_pos)
                    .collect()
            }
            (Control::Table(t), rest) if rest.starts_with("tbl.cell[") => {
                let rest = rest.strip_prefix("tbl.cell[").unwrap();
                let (idx_s, rest) = rest.split_once(']').expect("cell index");
                let cidx: usize = idx_s.parse().unwrap();
                let rest = rest.strip_prefix(".p[").expect(".p[");
                let (idx_s, rest) = rest.split_once(']').expect("cell p index");
                let pidx: usize = idx_s.parse().unwrap();
                footnote_para_vpos_from(&t.cells[cidx].paragraphs[pidx].controls, rest)
            }
            _ => panic!("경로 해석 실패: {rest}"),
        };
    }
    panic!("ctrl 경로가 아니다: {path}");
}

fn collect_zero_tail_notes(paragraphs: &[Paragraph], out: &mut usize) {
    for p in paragraphs {
        for c in &p.controls {
            match c {
                Control::Footnote(n) => {
                    for np in &n.paragraphs {
                        if np.line_segs.len() > 1
                            && np.line_segs.iter().all(|s| s.vertical_pos == 0)
                        {
                            *out += 1;
                        }
                    }
                    collect_zero_tail_notes(&n.paragraphs, out);
                }
                Control::Endnote(n) => {
                    collect_zero_tail_notes(&n.paragraphs, out);
                }
                Control::Table(t) => {
                    for cell in &t.cells {
                        collect_zero_tail_notes(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn note_xml(first_vpos: i32, rest_vpos: &[i32]) -> String {
    let mut segs = format!(
        r#"<hp:lineseg textpos="0" vertpos="{first_vpos}" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="38276" flags="393216"/>"#
    );
    for (i, vpos) in rest_vpos.iter().enumerate() {
        let textpos = (i + 1) * 20;
        segs.push_str(&format!(
            r#"<hp:lineseg textpos="{textpos}" vertpos="{vpos}" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="38276" flags="393216"/>"#
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:footNote number="1" suffixChar="41" instId="421">
          <hp:subList>
            <hp:p paraPrIDRef="0" styleIDRef="0">
              <hp:run charPrIDRef="0"><hp:t>정책연구 각주</hp:t></hp:run>
              <hp:linesegarray>{segs}</hp:linesegarray>
            </hp:p>
          </hp:subList>
        </hp:footNote>
      </hp:ctrl>
    </hp:run>
  </hp:p>
</hs:sec>"#
    )
}

fn note_vpos(section: &Section) -> Vec<i32> {
    section.paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Footnote(n) => Some(
                n.paragraphs[0]
                    .line_segs
                    .iter()
                    .map(|s| s.vertical_pos)
                    .collect(),
            ),
            _ => None,
        })
        .expect("footnote")
}

#[test]
fn issue4882_all_zero_note_vpos_is_preserved_without_hwp5_marker() {
    let section = parse_hwpx_section(&note_xml(0, &[0])).unwrap();
    assert_eq!(
        note_vpos(&section),
        vec![0, 0],
        "전 줄 vpos=0 각주는 합성하지 않는다 (#4882)"
    );
    let section = parse_hwpx_section(&note_xml(0, &[0, 0])).unwrap();
    assert_eq!(
        note_vpos(&section),
        vec![0, 0, 0],
        "3줄 전 vpos=0 도 그대로다 (issue 경로 [2].vertpos=0)"
    );
}

#[test]
fn issue4882_hangul_hwpx_artifact_still_normalizes_trailing_zero() {
    let section = parse_hwpx_section(&note_xml(2344, &[0])).unwrap();
    assert_eq!(
        note_vpos(&section),
        vec![2344, 3516],
        "첫 줄 vpos>0 + 후속 0 은 task 1692 대로 3516=2344+1172 복원"
    );
}

#[test]
fn issue4882_table_cell_footnote_keeps_all_zero_vpos() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:tbl rowCnt="1" colCnt="1">
        <hp:tr>
          <hp:tc>
            <hp:subList>
              <hp:p paraPrIDRef="0" styleIDRef="0">
                <hp:run charPrIDRef="0">
                  <hp:ctrl>
                    <hp:footNote number="2" suffixChar="41" instId="728">
                      <hp:subList>
                        <hp:p paraPrIDRef="0" styleIDRef="0">
                          <hp:run charPrIDRef="0"><hp:t>표 안 각주</hp:t></hp:run>
                          <hp:linesegarray>
                            <hp:lineseg textpos="0" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="393216"/>
                            <hp:lineseg textpos="20" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="393216"/>
                            <hp:lineseg textpos="40" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="393216"/>
                          </hp:linesegarray>
                        </hp:p>
                      </hp:subList>
                    </hp:footNote>
                  </hp:ctrl>
                </hp:run>
              </hp:p>
            </hp:subList>
          </hp:tc>
        </hp:tr>
      </hp:tbl>
    </hp:run>
  </hp:p>
</hs:sec>"#;
    let section = parse_hwpx_section(xml).unwrap();
    let vpos = section.paragraphs[0].controls.iter().find_map(|c| match c {
        Control::Table(t) => t.cells[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|c| match c {
                Control::Footnote(n) => Some(
                    n.paragraphs[0]
                        .line_segs
                        .iter()
                        .map(|s| s.vertical_pos)
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            }),
        _ => None,
    });
    assert_eq!(
        vpos.as_deref(),
        Some([0, 0, 0].as_slice()),
        "표 셀 각주 전 줄 vpos=0 보존"
    );
}

#[test]
fn issue4882_pinned_footnote_vpos_are_all_zero_on_original() {
    let path = sample_path();
    if !path.is_file() {
        return;
    }
    let bytes = std::fs::read(&path).unwrap();
    let original = parse_document(&bytes).expect("parse hwp");
    for pin in PINNED_NOTE_PATHS {
        let vpos = footnote_para_vpos(&original.sections[0].paragraphs, pin);
        assert!(
            vpos.len() > 1,
            "{pin}: 후속 줄이 있어야 한다 (got {vpos:?})"
        );
        assert!(
            vpos.iter().skip(1).all(|&v| v == 0),
            "{pin}: 후속 줄 vertpos=0 전제 (got {vpos:?})"
        );
    }
}

#[test]
fn issue4882_hwpx_export_keeps_hwp5_origin_marker() {
    let path = sample_path();
    if !path.is_file() {
        return;
    }
    let bytes = std::fs::read(&path).unwrap();
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export");
    let parsed = parse_hwpx(&hwpx).expect("parse exported");
    assert!(
        parsed
            .hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH)
            .is_some(),
        "HWP5-origin 마커가 있어야 각주 vpos 계약을 재파싱이 따른다"
    );
}

#[test]
fn issue4882_note_zero_vpos_survives_hwpx_roundtrip() {
    let path = sample_path();
    if !path.is_file() {
        return;
    }
    let bytes = std::fs::read(&path).unwrap();
    let original = parse_document(&bytes).expect("parse hwp");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export");
    let roundtripped = parse_document(&hwpx).expect("reparse");

    for pin in PINNED_NOTE_PATHS {
        let before = footnote_para_vpos(&original.sections[0].paragraphs, pin);
        let after = footnote_para_vpos(&roundtripped.sections[0].paragraphs, pin);
        assert_eq!(before, after, "{pin}: 각주 lineseg vertpos 왕복 보존");
        assert!(
            after.iter().skip(1).all(|&v| v == 0),
            "{pin}: 재파싱이 후속 줄을 쌓으면 안 된다 (got {after:?})"
        );
    }

    let mut orig_zero = 0usize;
    let mut rt_zero = 0usize;
    collect_zero_tail_notes(&original.sections[0].paragraphs, &mut orig_zero);
    collect_zero_tail_notes(&roundtripped.sections[0].paragraphs, &mut rt_zero);
    assert_eq!(
        orig_zero, rt_zero,
        "전 줄 vpos=0 각주 개수가 왕복에서 줄면 합성 회귀"
    );
    assert!(
        orig_zero >= 5,
        "표본 전제: 전 줄 vpos=0 각주 ≥5 (got {orig_zero})"
    );
}

#[test]
fn issue4882_export_hwpx_reimport_keeps_215_pages() {
    let path = sample_path();
    if !path.is_file() {
        return;
    }
    let bytes = std::fs::read(&path).unwrap();
    let source = DocumentCore::from_bytes(&bytes).expect("open");
    let before = source.page_count();
    assert_eq!(before, EXPECTED_PAGES, "원본 쪽수 전제");

    let exported = source.export_hwpx_native().expect("export hwpx");
    let reparsed = DocumentCore::from_bytes(&exported).expect("reparse");
    let after = reparsed.page_count();
    assert_eq!(
        after, before,
        "pages(원본)==pages(export-hwpx→reimport) (#4882)"
    );
}
