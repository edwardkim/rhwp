//! Issue #884 / #915: CharShape `start_pos` 해석 회귀 가드.
//!
//! `samples/table-in-tbox.hwp` page 1 footer (글상자 안 표 셀) paragraph
//! " 충남중부권지사장" 의 char_shape:
//!   - (start_pos=0, id=14 HY헤드라인M 26pt)
//!   - (start_pos=9, id=20 HY수평선B 16pt)
//! char_offsets = [8,9,…,16] — 선두 인라인 그림이 stream offset 0~7 을 점유.
//!
//! `start_pos` 는 paragraph 텍스트의 UTF-16 stream offset 이다 (해석 A):
//!   - start_pos=0 → offset 0 이상 첫 가시문자 = visible[0](" ") → id=14
//!   - start_pos=9 → offset 9 이상 첫 가시문자 = visible[1]("충") → id=20
//!   ⇒ "충남중부권지사장" = id=20 HY수평선B 16pt.
//!
//! 이력: #884 가 이 footer 를 "26pt HY헤드라인M" 으로 오진(HY수평선B 굵은
//! 글꼴 + 로고 옆 배치로 인한 육안 착시)하고 `start_pos` 를 visible char
//! index 로 해석(해석 B)하도록 composer 를 바꿨다. 해석 B 는 인라인 제어자가
//! 문단 *중간* 에 있는 문단에서 `start_pos` 가 char_offsets gap 만큼 부풀려져
//! char_shape 를 통째 누락시켜 #915(table-in-tbox p2 "충남중부권지사" 1pt
//! 렌더)를 유발했다. 한컴2024 글자모양 패널 확인 결과 footer 는 실제로
//! HY수평선B 16.0pt — 해석 A 가 정답이며 #915 에서 해석 A 로 복원했다.
//!
//! 본 테스트는 footer "충" 이 HY수평선B(해석 A)로 렌더됨을 가드한다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

#[test]
fn issue_884_chungnam_jisajang_uses_hy_supb_per_stream_offset() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/table-in-tbox.hwp");
    let bytes = fs::read(&hwp_path).expect("read table-in-tbox.hwp");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let svg = doc.render_page_svg(0).expect("render page 0");

    // footer "충남중부권지사장" 의 "충" font-family 검증.
    // 해석 A 정답: char_shape start_pos=9 (stream offset) → visible[1]="충" →
    // id=20 HY수평선B. 해석 B 회귀 시: footer 전체가 id=14 HY헤드라인M.
    let mut search_from = 0;
    let mut uses_hy_supb = false;
    let mut uses_hy_headlinem = false;
    while let Some(idx) = svg[search_from..].find(">충<") {
        let abs_idx = search_from + idx;
        let element_start = svg[..abs_idx].rfind("<text").expect("<text> 시작 못 찾음");
        let element = &svg[element_start..abs_idx + 5];
        if element.contains("font-family=\"HY수평선B") {
            uses_hy_supb = true;
        }
        if element.contains("font-family=\"HY헤드라인M") {
            uses_hy_headlinem = true;
        }
        search_from = abs_idx + 5;
    }

    assert!(
        uses_hy_supb && !uses_hy_headlinem,
        "Issue #884/#915 회귀: footer '충' 은 HY수평선B (해석 A — start_pos=9 \
         는 UTF-16 stream offset → visible[1]) 로 렌더되어야 함. \
         (uses_hy_supb={uses_hy_supb} uses_hy_headlinem={uses_hy_headlinem}). \
         HY헤드라인M 가 잡히면 composer 가 해석 B(#884 오진)로 회귀한 것."
    );
}

#[test]
fn issue_884_diagnostic_dump() {
    use rhwp::model::control::Control;
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/table-in-tbox.hwp");
    let bytes = fs::read(&hwp_path).expect("read");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let document = doc.document();

    // 결함 paragraph 의 raw 데이터를 덤프한다 (회귀 진단 보조).
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
                        let name = document
                            .doc_info
                            .font_faces
                            .first()
                            .and_then(|fonts| fonts.get(s.font_ids[0] as usize))
                            .map(|f| f.name.clone())
                            .unwrap_or_default();
                        eprintln!(
                            "    start_pos={} id={} → {:?} {:.1}pt bold={}",
                            cs.start_pos,
                            cs.char_shape_id,
                            name,
                            s.base_size as f64 / 100.0,
                            s.bold
                        );
                    }
                }
                eprintln!(
                    "  해석 A (stream offset, 정답): start_pos=9 → char_offsets 에서 \
                     9 이상 첫 항목 = visible[1]=충 → id=20 HY수평선B 16pt 적용"
                );
                return true;
            }
            for ctrl in &p.controls {
                let found = match ctrl {
                    Control::Table(t) => t.cells.iter().any(|c| find(&c.paragraphs, document)),
                    Control::Shape(s) => s
                        .drawing()
                        .and_then(|d| d.text_box.as_ref())
                        .map(|tb| find(&tb.paragraphs, document))
                        .unwrap_or(false),
                    _ => false,
                };
                if found {
                    return true;
                }
            }
        }
        false
    }
    let found = document
        .sections
        .iter()
        .any(|s| find(&s.paragraphs, document));
    assert!(
        found,
        "결함 paragraph (table-in-tbox 충남중부권지사장) 미발견 — 샘플 변경?"
    );
}
