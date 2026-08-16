//! Issue #4896: 열거 밖 필드 종류가 `CROSSREF` 로 굳어 필드 정체성이 사라지는 결함.
//!
//! 한글 2022 오라클 10k 전수(s13)에서 원본의 교정부호 필드(`%%*d`)가 rhwp 저장본에서
//! 상호참조(`%xrf`)로 바뀌었다 — 27경로. 사슬은 두 마디였다:
//!
//! 1. HWPX 파서가 모르는 `type` 값을 `FieldType::Unknown` 으로만 떨어뜨려 원문을 버린다.
//! 2. HWPX 직렬화기가 `Unknown` 을 본문 보존용 `CROSSREF` 로 굳힌다(#4776).
//!
//! ②의 판단(열거 밖 값을 새로 지어내면 한글이 본문을 버린다)은 유효하다. 고칠 곳은 ①이다 —
//! **원본이 준 값은 한글이 이미 받아들인 값**이므로 그대로 되돌려주면 본문도 정체성도 산다.
//! 10k 코퍼스 실측: 한컴 hwpx 원본이 쓰는 열거 밖 값은 `PROOFREADING_MARKS_DELETE` 하나뿐이고
//! (12회/7문서), 같은 문서에서 한글이 세는 컨트롤 `%%*d` 와 개수까지 일치한다.
//!
//! 표본은 저장소에 새 이진 파일을 넣지 않으려고 **시험 시점에 합성**한다(#3707 선례).

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/누름틀-2024.hwpx";
const EXOTIC: &str = "PROOFREADING_MARKS_DELETE";

/// 표본의 `CLICK_HERE` 필드 하나를 열거 밖 종류로 바꾼 hwpx 를 만든다.
fn synth_exotic_field_hwpx() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut out = Vec::new();
    {
        let mut zout = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let opts: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut replaced = false;
        for i in 0..zin.len() {
            let mut f = zin.by_index(i).expect("zip 항목");
            let name = f.name().to_string();
            let mut data = Vec::new();
            f.read_to_end(&mut data).expect("항목 읽기");
            if !replaced && name.starts_with("Contents/section") && name.ends_with(".xml") {
                let s = String::from_utf8_lossy(&data).into_owned();
                if let Some(pos) = s.find(r#"type="CLICK_HERE""#) {
                    let mut edited = s.clone();
                    edited.replace_range(
                        pos..pos + r#"type="CLICK_HERE""#.len(),
                        &format!(r#"type="{EXOTIC}""#),
                    );
                    data = edited.into_bytes();
                    replaced = true;
                }
            }
            zout.start_file(name, opts).expect("항목 쓰기");
            zout.write_all(&data).expect("바이트 쓰기");
        }
        assert!(replaced, "표본에 CLICK_HERE 필드가 있어야 합성이 성립한다");
        zout.finish().expect("zip 마무리");
    }
    out
}

fn section_xml(bytes: &[u8]) -> String {
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut out = String::new();
    for i in 0..zin.len() {
        let mut f = zin.by_index(i).expect("zip 항목");
        let name = f.name().to_string();
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            let mut s = String::new();
            f.read_to_string(&mut s).expect("section xml");
            out.push_str(&s);
        }
    }
    out
}

#[test]
fn hwpx_roundtrip_keeps_out_of_enum_field_type() {
    let synth = synth_exotic_field_hwpx();
    let core = DocumentCore::from_bytes(&synth).expect("합성 hwpx 파싱");

    let saved = core.export_hwpx_native().expect("hwpx 저장");
    let xml = section_xml(&saved);

    assert!(
        xml.contains(&format!(r#"type="{EXOTIC}""#)),
        "원본이 준 필드 종류가 그대로 되돌아가야 한다"
    );
    assert_eq!(
        xml.matches(r#"type="CROSSREF""#).count(),
        section_xml(&synth).matches(r#"type="CROSSREF""#).count(),
        "모르는 종류를 상호참조로 굳히면 안 된다"
    );
}

#[test]
fn hwpx_to_hwp5_keeps_field_ctrl_id() {
    let synth = synth_exotic_field_hwpx();
    let mut core = DocumentCore::from_bytes(&synth).expect("합성 hwpx 파싱");

    // HWP5 는 ctrl_id 자체가 종류다 — 0 으로 저장하면 한글이 무효 컨트롤로 보고 필드를 잃는다.
    let saved = core.export_hwp_with_adapter().expect("hwp 저장");
    let reloaded = DocumentCore::from_bytes(&saved).expect("hwp 재파싱");

    let ctrl_ids = collect_field_ctrl_ids(&reloaded);
    assert!(
        ctrl_ids.contains(&u32::from_be_bytes(*b"%%*d")),
        "교정부호 필드의 ctrl_id 가 보존되어야 한다: {ctrl_ids:?}"
    );
}

fn collect_field_ctrl_ids(core: &DocumentCore) -> Vec<u32> {
    use rhwp::model::control::Control;
    let mut ids = Vec::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Field(f) = ctrl {
                    ids.push(f.ctrl_id);
                }
            }
        }
    }
    ids
}
