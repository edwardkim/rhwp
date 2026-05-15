//! Issue #884 RED: CharShape start_pos 해석 결함
//!
//! `samples/table-in-tbox.hwp` 의 글상자 안 표 셀 r=0,c=0 paragraph (" 충남중부권지사장")
//! 에 char_shape (start_pos=0 id=14 HY헤드라인M 26pt), (start_pos=9 id=20 HY수평선B 16pt)
//! 가 정의되어 있다.
//!
//! 9 visible chars (인덱스 0~8) 에 대해:
//! - 해석 A (현재 코드, u16 stream): start_pos=9 가 char_offsets[1]=9 와 일치하여
//!   id=20 (HY수평선B) 가 visible[1]("충") 부터 적용. 잘못된 결과.
//! - 해석 B (visible char idx): start_pos=9 가 text 길이 9 보다 ≥ 이므로 미적용,
//!   전체 id=14 (HY헤드라인M) 적용. 한컴 PDF 정합 정답.
//!
//! 이슈 본문 (Task #696 B-2-Z 실험):
//!   table-in-tbox.hwp 푸터 정상 (HY헤드라인M 26pt #14, 이전 HY수평선B 16pt #20)
//!
//! 본 테스트는 결함이 존재함을 가드 — fix 후 assert 반전 필요.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

#[test]
fn issue_884_chungnam_jisajang_should_be_hy_headlinem() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/table-in-tbox.hwp");
    let bytes = fs::read(&hwp_path).expect("read table-in-tbox.hwp");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let svg = doc
        .render_page_svg(0)
        .expect("render page 0");

    // 결함 검증: 'Shape.TextBox > Table > cell[0]' paragraph 의 "충" 글자의 font-family.
    // 정답 (B 해석): font-family="HY헤드라인M..."
    // 현재 (A 해석): font-family="HY수평선B..."

    // 모든 "충" element 를 찾고 그 중 결함 위치 (HY수평선B 또는 HY헤드라인M 사용) 식별
    let mut search_from = 0;
    let mut uses_hy_supb = false;
    let mut uses_hy_headlinem = false;
    while let Some(idx) = svg[search_from..].find(">충<") {
        let abs_idx = search_from + idx;
        let element_start = svg[..abs_idx].rfind("<text").expect("<text> 시작 못 찾음");
        let element = &svg[element_start..abs_idx + 5];
        if element.contains("font-family=\"HY수평선B") {
            uses_hy_supb = true;
            eprintln!("결함 '충' element (HY수평선B): {}", &element[..element.len().min(200)]);
        }
        if element.contains("font-family=\"HY헤드라인M") {
            uses_hy_headlinem = true;
            eprintln!("정답 '충' element (HY헤드라인M): {}", &element[..element.len().min(200)]);
        }
        search_from = abs_idx + 5;
    }

    // Fix (해석 B) 적용 후 GREEN 가드: HY헤드라인M 적용 + HY수평선B 미사용.
    assert!(
        uses_hy_headlinem && !uses_hy_supb,
        "Issue #884 회귀: '충' 글자에 HY헤드라인M 가 적용되지 않음. \
         (uses_hy_supb={} uses_hy_headlinem={}). \
         해석 B (start_pos as visible char idx) 가 부분적으로 회귀했을 가능성.",
        uses_hy_supb, uses_hy_headlinem
    );

    eprintln!("\nIssue #884 GREEN: '충' 글자 HY헤드라인M 정합 (해석 B 적용 후).");
}

#[test]
fn issue_884_diagnostic_dump() {
    use rhwp::model::control::Control;
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/table-in-tbox.hwp");
    let bytes = fs::read(&hwp_path).expect("read");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let document = doc.document();

    // Locate the failing paragraph and dump its raw data
    fn find(
        paragraphs: &[rhwp::model::paragraph::Paragraph],
        document: &rhwp::model::document::Document,
    ) -> bool {
        for p in paragraphs {
            if p.text.contains("충남중부권지사장") {
                eprintln!("결함 paragraph 발견: text={:?}", p.text);
                eprintln!("  char_offsets: {:?}", p.char_offsets);
                eprintln!("  char_shapes:");
                for cs in &p.char_shapes {
                    if let Some(s) = document.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                        let name = document.doc_info.font_faces.get(0)
                            .and_then(|fonts| fonts.get(s.font_ids[0] as usize))
                            .map(|f| f.name.clone()).unwrap_or_default();
                        eprintln!("    start_pos={} id={} → {:?} {:.1}pt bold={}",
                            cs.start_pos, cs.char_shape_id, name,
                            s.base_size as f64 / 100.0, s.bold);
                    }
                }
                // 해석 A vs B
                eprintln!("  해석 A (u16): start_pos=9 → visible[1]=충 (id=20 HY수평선B 잘못 적용)");
                eprintln!("  해석 B (vis): start_pos=9 → out of range (id=14 HY헤드라인M 전체 유지, 정답)");
                return true;
            }
            for ctrl in &p.controls {
                let found = match ctrl {
                    Control::Table(t) => t.cells.iter().any(|c| find(&c.paragraphs, document)),
                    Control::Shape(s) => s.drawing().and_then(|d| d.text_box.as_ref())
                        .map(|tb| find(&tb.paragraphs, document)).unwrap_or(false),
                    _ => false,
                };
                if found { return true; }
            }
        }
        false
    }
    let found = document.sections.iter().any(|s| find(&s.paragraphs, document));
    assert!(found, "결함 paragraph (table-in-tbox 충남중부권지사장) 미발견 — 샘플 변경?");
}
