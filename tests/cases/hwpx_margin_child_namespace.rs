//! HWPX `<hh:margin>` 의 값 요소(intent/left/right/prev/next)는 core 네임스페이스
//! (`hc:`)일 때만 한/글이 읽는다.
//!
//! `samples/hwpx/hwpx-02.hwpx` 는 옛 rhwp 가 저장한 파일로 여백 자식을 `<hh:intent>` 처럼
//! head 네임스페이스로 썼다. 문단 29(`* ‘24년 전기장비·전자부품 …`)의 문단 모양은
//! `<hh:intent value="-5184">` 내어쓰기를 적는다.
//!
//! 독립 근거 — 한/글 변환 서비스(engine 2020)로 같은 파일과 변형을 PDF 로 냈다:
//!
//! | 입력 | 2쪽 `39.2억 달러로…` 줄 x |
//! |---|---|
//! | 원본(`hh:`) · 저장소 정본 `pdf/hwpx/hwpx-02-2022.pdf` | 113.44px = 본문 왼쪽(내어쓰기 없음) |
//! | 여백 자식 접두어만 `hc:` 로 바꾼 변형 | 148.00px = 본문 + 34.56(5184 HU 의 절반) |
//!
//! 반례: 접두어가 `hc:` 이면 같은 값이 그대로 적용된다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/hwpx-02.hwpx";
const PARA: usize = 29;

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// header.xml 여백 자식의 `hh:` 접두어를 `hc:` 로 바꾼 패키지.
fn with_core_prefixed_margins() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("entry");
        let name = entry.name().expect("entry name").to_string();
        let method = entry.compression();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/header.xml" {
            let mut text = String::from_utf8(data).expect("utf8 header");
            for tag in ["intent", "left", "right", "prev", "next"] {
                text = text.replace(&format!("<hh:{tag} "), &format!("<hc:{tag} "));
            }
            data = text.into_bytes();
        }
        let options = zip::write::SimpleFileOptions::default().compression_method(method);
        out.start_file(name, options).expect("start file");
        out.write_all(&data).expect("write entry");
    }
    out.finish().expect("finish").into_inner()
}

fn para_indent(bytes: &[u8]) -> i32 {
    let doc = rhwp::parser::hwpx::parse_hwpx(bytes).expect("parse hwpx");
    let para = &doc.sections[0].paragraphs[PARA];
    assert!(
        para.text.contains("전기장비"),
        "전제: 문단 {PARA} 는 `* ‘24년 전기장비…` 각주성 문단이어야 한다"
    );
    doc.doc_info.para_shapes[para.para_shape_id as usize].indent
}

#[test]
fn head_namespace_margin_children_are_not_read() {
    let header = {
        let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
        let mut text = String::new();
        archive
            .by_name("Contents/header.xml")
            .expect("header")
            .read_to_string(&mut text)
            .expect("read header");
        text
    };
    assert!(
        header.contains(r#"<hh:intent unit="HWPUNIT" value="-5184"/>"#),
        "전제: 원본은 head 네임스페이스로 내어쓰기 -5184 를 적는다"
    );
    assert_eq!(
        para_indent(&sample_bytes()),
        0,
        "한/글은 `<hh:intent>` 를 읽지 않는다 — 정본 2쪽 둘째 줄은 본문 왼쪽(113.44px)"
    );
}

#[test]
fn core_namespace_margin_children_are_read() {
    assert_eq!(
        para_indent(&with_core_prefixed_margins()),
        -5184,
        "`<hc:intent>` 로 바꾸면 한/글이 내어 쓴다(둘째 줄 148.00px)"
    );
}
