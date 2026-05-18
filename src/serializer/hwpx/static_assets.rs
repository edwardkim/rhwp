//! HWPX 빈 문서에 필요한 정적 보일러플레이트 파일
//!
//! 한컴2020 레퍼런스(ref_empty.hwpx) 기반 정적 템플릿.
//! Stage 2+에서 IR 기반 동적 생성으로 점진 교체한다.

/// version.xml — 한컴 레퍼런스와 동일 형식
pub const VERSION_XML: &str = include_str!("templates/version.xml");

/// META-INF/container.xml — OCF 루트 엔트리 (3개 rootfile)
pub const META_INF_CONTAINER_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#,
    r#"<ocf:container xmlns:ocf="urn:oasis:names:tc:opendocument:xmlns:container""#,
    r#" xmlns:hpf="http://www.hancom.co.kr/schema/2011/hpf">"#,
    r#"<ocf:rootfiles>"#,
    r#"<ocf:rootfile full-path="Contents/content.hpf" media-type="application/hwpml-package+xml"/>"#,
    r#"<ocf:rootfile full-path="Preview/PrvText.txt" media-type="text/plain"/>"#,
    r#"<ocf:rootfile full-path="META-INF/container.rdf" media-type="application/rdf+xml"/>"#,
    r#"</ocf:rootfiles>"#,
    r#"</ocf:container>"#,
);

/// META-INF/container.rdf — 패키지 내 파일 역할 RDF 선언
pub const META_INF_CONTAINER_RDF: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#,
    r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#,
    r#"<rdf:Description rdf:about="">"#,
    r#"<ns0:hasPart xmlns:ns0="http://www.hancom.co.kr/hwpml/2016/meta/pkg#" rdf:resource="Contents/header.xml"/>"#,
    r#"</rdf:Description>"#,
    r#"<rdf:Description rdf:about="Contents/header.xml">"#,
    r#"<rdf:type rdf:resource="http://www.hancom.co.kr/hwpml/2016/meta/pkg#HeaderFile"/>"#,
    r#"</rdf:Description>"#,
    r#"<rdf:Description rdf:about="">"#,
    r#"<ns0:hasPart xmlns:ns0="http://www.hancom.co.kr/hwpml/2016/meta/pkg#" rdf:resource="Contents/section0.xml"/>"#,
    r#"</rdf:Description>"#,
    r#"<rdf:Description rdf:about="Contents/section0.xml">"#,
    r#"<rdf:type rdf:resource="http://www.hancom.co.kr/hwpml/2016/meta/pkg#SectionFile"/>"#,
    r#"</rdf:Description>"#,
    r#"<rdf:Description rdf:about="">"#,
    r#"<rdf:type rdf:resource="http://www.hancom.co.kr/hwpml/2016/meta/pkg#Document"/>"#,
    r#"</rdf:Description>"#,
    r#"</rdf:RDF>"#,
);

/// META-INF/manifest.xml — 빈 ODF manifest
pub const META_INF_MANIFEST_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#,
    r#"<odf:manifest xmlns:odf="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"/>"#,
);

/// settings.xml — 한컴 레퍼런스와 동일 형식
pub const SETTINGS_XML: &str = include_str!("templates/settings.xml");

/// Contents/content.hpf — OPF manifest 한컴 레퍼런스 기반 (metadata 일반화)
pub const EMPTY_CONTENT_HPF: &str = include_str!("templates/empty_content.hpf");

/// Preview/PrvText.txt — 빈 문서 미리보기 텍스트
pub const PRV_TEXT: &[u8] = b"\r\n";

/// Preview/PrvImage.png — 1x1 투명 PNG (한컴 호환 최소 썸네일)
pub const PRV_IMAGE_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xB5, 0x1C, 0x0C,
    0x02, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x64, 0x60, 0x00, 0x00,
    0x00, 0x05, 0x00, 0x01, 0x6F, 0x68, 0x67, 0xBC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
    0xAE, 0x42, 0x60, 0x82,
];
