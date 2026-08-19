//! [#5553] 합성 쪽나눔(파서가 자연 쪽 경계에서 승격한 column_type=Page)은 조판
//! 힌트일 뿐 문서 내용이 아니므로 HWPX `pageBreak` 로 저장되지 않아야 한다.
//! 저장하면 한글 재조판의 자연 경계와 이중 작용해 빈 쪽이 생긴다
//! (07615: 합성 138건이 264쪽 → 329쪽 부풀림, 전량 중화 시 264쪽 복원).
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API(`serialize_hwpx`) 경로의 이 통합 테스트로 옮겼다.

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{ColumnBreakType, Paragraph};
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::hwpx::serialize_hwpx;
use rhwp::wasm_api::HwpDocument;

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

// HWP3의 자연 쪽 경계도 HWP5 저장에서 실제 쪽나눔으로 되살아나면 안 된다.
// 이 픽스처는 잘못된 방출이 첫 문단에 SectionDef를 추가해 표 제어문 오프셋을 밀던
// 실제 회귀(#5610 CI)를 고정한다.
#[test]
fn hwp3_synthesized_page_break_keeps_table_caption_roundtrip_lossless() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join("hwp3-table-caption.hwp");
    let source = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("fixture 읽기 실패 ({}): {error}", fixture.display()));
    let mut document = HwpDocument::from_bytes(&source).expect("HWP3 파싱 실패");
    assert!(
        document
            .document()
            .sections
            .iter()
            .flat_map(|section| &section.paragraphs)
            .any(|paragraph| paragraph.page_break_synthesized),
        "전제: HWP3 자연 쪽 경계가 합성 Page 힌트로 파싱되어야 함"
    );

    document
        .convert_to_editable_native()
        .expect("HWP3 편집 가능 변환 실패");
    let serialized = document
        .export_hwp_with_adapter()
        .expect("HWP5 직렬화 실패");
    let reparsed = HwpDocument::from_bytes(&serialized).expect("HWP5 재파싱 실패");
    let difference = diff_documents(document.document(), reparsed.document());
    assert!(
        difference.differences.is_empty(),
        "합성 쪽나눔을 HWP5에 기록해 표 캡션 문단 축이 변했습니다: {difference:?}"
    );
}
