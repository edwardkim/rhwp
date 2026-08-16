//! [Issue #3557] HWPX 왕복에서 IR 로 모델링되지 않는 패키지 실체(OLE 바이너리·
//! 임베드 폰트·스크립트)가 소실되지 않아야 한다 — `--verify`(IR 대조)로는 잡히지
//! 않는 손실이라 패키지(ZIP) 수준 계약으로 고정한다.
//!
//! - OLE 바이너리·임베드 폰트는 `image{N}` 명명 불변식(#1891)으로 **개명**되지만
//!   내용 바이트는 보존된다(개명은 의도 설계 — #4669 이슈 본문도 결함 아님으로
//!   확인). 이 테스트는 바이트 보존을 계약으로 잰다.
//! - Scripts/* 는 경로·바이트 원문 그대로, content.hpf 의 opf:item/spine 참조까지
//!   보존된다(이 PR 의 수정 지점).

use std::io::Read;

use rhwp::document_core::DocumentCore;

fn roundtrip(rel: &str) -> (Vec<u8>, Vec<u8>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let original = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let doc = DocumentCore::from_bytes(&original).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    let out = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export {rel}: {e:?}"));
    (original, out)
}

fn zip_entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("ZIP 열기");
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    let mut out = Vec::new();
    for name in names {
        let mut data = Vec::new();
        zip.by_name(&name)
            .expect("엔트리")
            .read_to_end(&mut data)
            .expect("읽기");
        out.push((name, data));
    }
    out
}

fn bin_data_payload_set(entries: &[(String, Vec<u8>)], ext: &str) -> Vec<Vec<u8>> {
    let mut v: Vec<Vec<u8>> = entries
        .iter()
        .filter(|(n, _)| {
            n.starts_with("BinData/") && n.to_ascii_lowercase().ends_with(&ext.to_ascii_lowercase())
        })
        .map(|(_, d)| d.clone())
        .collect();
    v.sort();
    v
}

#[test]
fn ole_binary_bytes_survive_roundtrip() {
    let (original, out) = roundtrip("samples/한셀OLE.hwpx");
    let a = bin_data_payload_set(&zip_entries(&original), ".ole");
    let b = bin_data_payload_set(&zip_entries(&out), ".ole");
    assert!(!a.is_empty(), "샘플 전제: OLE 바이너리가 있어야 한다");
    assert_eq!(a, b, "OLE 바이너리 바이트가 왕복에서 보존돼야 한다 (#3557)");
}

#[test]
fn embedded_font_bytes_survive_roundtrip() {
    let (original, out) = roundtrip("samples/render-p35-font-native-bitmap.hwpx");
    let a = bin_data_payload_set(&zip_entries(&original), ".ttf");
    let b = bin_data_payload_set(&zip_entries(&out), ".ttf");
    assert!(!a.is_empty(), "샘플 전제: 임베드 폰트가 있어야 한다");
    assert_eq!(a, b, "임베드 폰트 바이트가 왕복에서 보존돼야 한다 (#3557)");
}

#[test]
fn scripts_survive_roundtrip_with_manifest_refs() {
    let (original, out) = roundtrip("samples/표-텍스트.hwpx");
    let orig_entries = zip_entries(&original);
    let out_entries = zip_entries(&out);
    let orig_scripts: Vec<&(String, Vec<u8>)> = orig_entries
        .iter()
        .filter(|(n, _)| n.starts_with("Scripts/"))
        .collect();
    assert!(!orig_scripts.is_empty(), "샘플 전제: Scripts 파트");
    for (name, data) in &orig_scripts {
        let found = out_entries.iter().find(|(n, _)| n == name);
        let Some((_, out_data)) = found else {
            panic!("Scripts 엔트리 소실: {name} (#3557)");
        };
        assert_eq!(out_data, data, "Scripts 바이트 원문 보존: {name}");
    }
    let hpf = out_entries
        .iter()
        .find(|(n, _)| n == "Contents/content.hpf")
        .map(|(_, d)| String::from_utf8_lossy(d).into_owned())
        .expect("content.hpf");
    assert!(
        hpf.contains(r#"href="Scripts/headerScripts.js""#),
        "content.hpf 매니페스트에 스크립트 항목이 보존돼야 한다: {hpf}"
    );
    let spine = hpf.split("<opf:spine>").nth(1).expect("spine");
    assert!(
        spine.contains(r#"idref="headersc""#),
        "spine 에 스크립트 itemref 가 보존돼야 한다: {spine}"
    );
}
