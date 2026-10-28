//! HWPX ZIP 컨테이너의 결정론 — 공개 export 경로 판.
//!
//! [#5969] `SimpleFileOptions::default()` 가 "version made by" 호스트 바이트를 빌드
//! 호스트에서 가져와(리눅스 0x03 / Windows 0x00) 같은 문서의 컨테이너 해시가
//! 플랫폼마다 갈렸다. writer 는 UNIX(0x03)·고정 mtime 으로 못 박는다.
//! 종전 src 쪽 white-box 시험 4개를 실제 `export_hwpx()` 산출물 검사로 옮긴 판이다
//! (const 동일성 검사는 실바이트 검사에 흡수).

use std::fs;
use std::path::Path;

/// ZIP 중앙 디렉터리 레코드 서명. 뒤이어 spec 버전 1바이트 + "version made by" 호스트 바이트.
const CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const HOST_SYSTEM_OFFSET: usize = 5;
/// UNIX 고정값. 커밋된 판정 자산이 리눅스 생성이라 이 값이어야 해시가 유지된다.
const EXPECTED_HOST_BYTE: u8 = 3;
/// ZIP 로컬 파일 헤더 서명. mod time/date 가 +10..+14 에 온다.
const LOCAL_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];

fn exported_sample() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/143E433F503322BD33.hwpx");
    let bytes = fs::read(&path).expect("read sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse sample");
    doc.export_hwpx().expect("export_hwpx")
}

fn signature_offsets(bytes: &[u8], sig: [u8; 4]) -> Vec<usize> {
    bytes
        .windows(sig.len())
        .enumerate()
        .filter(|(_, w)| *w == sig)
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn export_pins_central_directory_host_byte() {
    // [#5969] 실제 export 산출물의 모든 중앙 디렉터리 레코드가 UNIX(0x03) 여야 한다.
    let bytes = exported_sample();
    let offsets = signature_offsets(&bytes, CENTRAL_DIRECTORY_SIGNATURE);
    assert!(
        offsets.len() >= 3,
        "mimetype·version.xml·content 최소 3개 레코드: {}",
        offsets.len()
    );
    for off in offsets {
        assert_eq!(
            bytes[off + HOST_SYSTEM_OFFSET],
            EXPECTED_HOST_BYTE,
            "중앙 디렉터리 +{HOST_SYSTEM_OFFSET} 호스트 바이트가 UNIX(0x03) 여야 한다"
        );
    }
}

#[test]
fn repeated_export_produces_identical_bytes() {
    // mtime 고정 + 호스트 바이트 고정이 함께 있어야 성립한다.
    assert_eq!(exported_sample(), exported_sample());
}

#[test]
fn stored_and_deflated_entries_share_deterministic_fields() {
    // 실아카이브에는 Stored(mimetype)와 Deflated(xml) 엔트리가 공존한다.
    // 한쪽 경로만 결정론 설정을 받는 회귀를 막는다 — 모든 로컬 헤더의
    // mod time/date(+10..+14)가 동일해야 한다.
    let bytes = exported_sample();
    let offsets = signature_offsets(&bytes, LOCAL_FILE_SIGNATURE);
    assert!(offsets.len() >= 3, "로컬 헤더 최소 3개: {}", offsets.len());
    let stamp = |off: usize| -> [u8; 4] { bytes[off + 10..off + 14].try_into().unwrap() };
    let first = stamp(offsets[0]);
    for &off in &offsets[1..] {
        assert_eq!(
            stamp(off),
            first,
            "압축 방식이 달라도 mod time/date 는 고정 스탬프여야 한다"
        );
    }
}
