//! [#5553] 합성 쪽나눔(파서가 자연 쪽 경계에서 승격한 column_type=Page)은 조판
//! 힌트일 뿐 문서 내용이 아니므로 HWPX `pageBreak` 로 저장되지 않아야 한다.
//! 저장하면 한글 재조판의 자연 경계와 이중 작용해 빈 쪽이 생긴다
//! (07615: 합성 138건이 264쪽 → 329쪽 부풀림, 전량 중화 시 264쪽 복원).
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API(`serialize_hwpx`) 경로의 이 통합 테스트로 옮겼다.

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{ColumnBreakType, Paragraph};
use rhwp::serializer::hwpx::serialize_hwpx;

fn doc_with_paragraph(para: Paragraph) -> Document {
    let mut section = Section::default();
    section.paragraphs.push(para);
    let mut doc = Document::default();
    doc.sections.push(section);
    doc
}

fn section0_xml(doc: &Document) -> String {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기 실패");
    let mut file = zip
        .by_name("Contents/section0.xml")
        .expect("section0.xml 없음");
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml).expect("section0.xml 읽기 실패");
    xml
}

// 합성 표시가 없는(명시적) Page 나눔은 종전대로 pageBreak="1" 로 저장된다.
#[test]
fn explicit_page_break_is_serialized() {
    let mut para = Paragraph::default();
    para.text = "p1".to_string();
    para.column_type = ColumnBreakType::Page;
    let xml = section0_xml(&doc_with_paragraph(para));
    assert!(
        xml.contains(r#"pageBreak="1""#),
        "명시적 Page 나눔은 pageBreak=1 이어야 함"
    );
}

// 합성 표시(page_break_synthesized)가 켜진 Page 나눔은 pageBreak="0" 으로 나간다.
#[test]
fn synthesized_page_break_is_not_serialized() {
    let mut para = Paragraph::default();
    para.text = "p1".to_string();
    para.column_type = ColumnBreakType::Page;
    para.page_break_synthesized = true;
    let xml = section0_xml(&doc_with_paragraph(para));
    assert!(
        xml.contains(r#"pageBreak="0""#) && !xml.contains(r#"pageBreak="1""#),
        "합성 쪽나눔은 pageBreak=0 으로 나가야 함"
    );
}
