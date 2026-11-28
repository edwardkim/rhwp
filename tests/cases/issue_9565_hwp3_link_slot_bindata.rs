//! #9565: 연결 그림만 있는 HWP3 문서를 HWP5 로 저장하면 빈 `BinData/BIN0000.` 스트림이 생기던 문제.
//!
//! HWP3 파서는 그림 번호마다 연결 레코드 자리(`link_slot` — id 0 의 빈 항목)를 채우고 문서 안에 담긴
//! 그림이 제 자리를 덮는다. 저장기는 연결 자리를 건너뛰어야 하는데, 그 판정이 HWP5 연결 레코드(저장 id 0)만
//! 보아 HWP3 연결 레코드(저장 id = 그림 번호)의 자리를 실제 항목으로 써 `BIN0000.`(id 0·확장자 없음·빈
//! deflate)을 만들었다. 다시 열면 어느 레코드도 가리키지 않아 사라졌지만, 첫 저장본에만 사양 밖의 빈
//! 스트림이 남았다. 연결 그림 정보(레코드·경로)는 그대로 남아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Cursor;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;

/// 그림이 모두 연결(Link) 그림인 HWP3 문서.
const SAMPLE_HWP3_LINKED: &str = "samples/hwp3-sample10.hwp";

fn open_sample() -> DocumentCore {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_HWP3_LINKED))
        .expect("HWP3 샘플");
    DocumentCore::from_bytes(&bytes).expect("HWP3 열기")
}

/// HWP5 저장본의 `/BinData/` 스트림 이름들
fn bin_data_streams(hwp: &[u8]) -> Vec<String> {
    let cfb = cfb::CompoundFile::open(Cursor::new(hwp)).expect("CFB 열기");
    let mut names: Vec<String> = cfb
        .walk()
        .filter(|e| e.is_stream() && e.path().parent() == Some(Path::new("/BinData")))
        .map(|e| e.name().to_string())
        .collect();
    names.sort();
    names
}

/// HWPX 저장본의 `BinData/` 항목 이름들
fn hwpx_bin_data_entries(hwpx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(hwpx)).expect("ZIP 열기");
    let mut names = Vec::new();
    for i in 0..zip.len() {
        let entry = zip.by_index(i).expect("ZIP 엔트리");
        let name = entry.name().expect("ZIP 엔트리 이름").into_owned();
        if name.starts_with("BinData/") {
            names.push(name);
        }
    }
    names.sort();
    names
}

/// 연결 레코드의 원본 경로 — 저장 전후에 그대로여야 한다
fn link_paths(core: &DocumentCore) -> Vec<String> {
    let mut paths: Vec<String> = core
        .document()
        .doc_info
        .bin_data_list
        .iter()
        .filter(|bd| bd.data_type == BinDataType::Link)
        .map(|bd| bd.abs_path.clone().unwrap_or_default())
        .collect();
    paths.sort();
    paths
}

#[test]
fn issue_9565_hwp3_linked_pictures_save_without_empty_bindata_stream() {
    let core = open_sample();
    let links = link_paths(&core);
    assert!(
        !links.is_empty(),
        "샘플에는 연결 그림이 있어야 시험이 뜻이 있다"
    );
    assert!(
        core.document()
            .doc_info
            .bin_data_list
            .iter()
            .all(|bd| bd.data_type == BinDataType::Link),
        "샘플의 그림은 모두 연결 그림이다"
    );

    let hwp = core.export_hwp_native().expect("HWP5 저장");
    let streams = bin_data_streams(&hwp);
    assert!(
        streams.iter().all(|name| !name.starts_with("BIN0000")),
        "연결 자리를 BIN0000 스트림으로 쓰면 안 된다: {streams:?}"
    );
    assert!(
        streams.is_empty(),
        "연결 그림만 있는 문서의 저장본에는 BinData 스트림이 없다: {streams:?}"
    );

    // 다시 열어도 연결 그림 정보는 그대로(저장 → 다시 열기 → 재저장 → 다시 열기)
    let reopened = DocumentCore::from_bytes(&hwp).expect("저장본 다시 열기");
    assert_eq!(link_paths(&reopened), links, "연결 레코드 경로 보존");
    let resaved = reopened.export_hwp_native().expect("재저장");
    assert!(bin_data_streams(&resaved).is_empty());
    let again = DocumentCore::from_bytes(&resaved).expect("재저장본 다시 열기");
    assert_eq!(
        link_paths(&again),
        links,
        "재저장 뒤에도 연결 레코드 경로 보존"
    );
}

#[test]
fn issue_9565_hwp3_linked_pictures_edit_then_save_keeps_links() {
    // 편집 → 저장 → 다시 열기 → 재저장 → 다시 열기
    let mut core = open_sample();
    let links = link_paths(&core);
    core.insert_text_native(0, 0, 0, "연결 그림 시험 ")
        .expect("본문 편집");
    let hwp = core.export_hwp_native().expect("편집본 저장");
    assert!(
        bin_data_streams(&hwp).is_empty(),
        "편집 뒤 저장본에도 빈 BinData 스트림이 없다"
    );
    let reopened = DocumentCore::from_bytes(&hwp).expect("편집본 다시 열기");
    assert_eq!(link_paths(&reopened), links);
    let again = DocumentCore::from_bytes(&reopened.export_hwp_native().expect("재저장"))
        .expect("다시 열기");
    assert_eq!(link_paths(&again), links);
}

#[test]
fn issue_9565_hwp3_linked_pictures_hwpx_save_has_no_empty_bindata_item() {
    // HWPX 저장도 같은 판정을 쓴다 — 연결 자리를 ZIP 항목으로 쓰지 않고, 다시 열면 연결 정보가 그대로다.
    let core = open_sample();
    let links = link_paths(&core);
    let hwpx = core.export_hwpx_native().expect("HWPX 저장");
    let entries = hwpx_bin_data_entries(&hwpx);
    assert!(
        entries.is_empty(),
        "연결 그림만 있는 문서의 HWPX 에는 BinData 항목이 없다: {entries:?}"
    );
    let reopened = DocumentCore::from_bytes(&hwpx).expect("HWPX 다시 열기");
    assert_eq!(
        link_paths(&reopened),
        links,
        "HWPX 저장 뒤 연결 레코드 경로 보존"
    );
}

#[test]
fn issue_9565_hwp3_linked_pictures_hwpx_package_check_agrees() {
    // HWPX 패키지 점검도 같은 판정으로 BinData 기대 개수를 센다 — 저장기와 어긋나면 BinData 수 불일치를 보고한다.
    let core = open_sample();
    let hwpx = core.export_hwpx_native().expect("HWPX 저장");
    let report = rhwp::serializer::hwpx::package_check::check_package(&hwpx, core.document());
    assert!(
        !report.problems.iter().any(|p| p.contains("BinData")),
        "패키지 점검의 BinData 기대값이 저장기와 같아야 한다: {}",
        report.summary()
    );
}
