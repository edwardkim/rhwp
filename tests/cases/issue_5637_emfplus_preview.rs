//! [Issue #5637] EMF+ 전용 OLE 미리보기를 그리지 못한다.
//!
//! 일부 생산자의 OlePres000 EMF 는 GDI+ 레코드를 `EMR_COMMENT`("EMF+" 시그니처)에
//! 내장하는 이중 스트림인데, 코멘트 레코드의 Size 필드가 실데이터보다 작게 적혀 있어
//! 코멘트 뒤에서 레코드 프레이밍이 임의 바이트 위에 얹힌다. 종전 파서는 그 지점에서
//! 통째로 실패해 미리보기가 placeholder 로 떨어졌다.
//!
//! 수정: EMF+ 코멘트를 본 스트림에 한해 구조 파단 지점부터 다음 그럴듯한 레코드
//! 연쇄로 재동기한다. 그런 스트림 뒤쪽에는 온전한 GDI 폴백(EMR_STRETCHDIBITS)이
//! 이어지므로 실제 미리보기 비트맵이 표준 경로로 그려진다.
//!
//! 계약: `samples/issue5637/2817919_emfplus_ole_preview.hwpx`(관세청, HWPX 코퍼스
//! 실물)의 BinData OLE 미리보기가 EMR_STRETCHDIBITS 를 포함해 파싱되고, standalone
//! SVG 로 `<image>` 가 방출되어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

const SAMPLE: &str = "samples/issue5637/2817919_emfplus_ole_preview.hwpx";

fn load_preview_emf() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::open(Path::new(repo_root).join(SAMPLE))
        .unwrap_or_else(|e| panic!("open {SAMPLE}: {e}"));
    let mut zip = zip::ZipArchive::new(file).expect("zip archive");
    let mut bytes = Vec::new();
    zip.by_name("BinData/ole1.ole")
        .expect("BinData/ole1.ole")
        .read_to_end(&mut bytes)
        .expect("read ole stream");
    // HWPX BinData .ole 는 u32 LE 길이 접두사 + CFB (#2263)
    assert_eq!(&bytes[4..8], &[0xD0, 0xCF, 0x11, 0xE0], "CFB magic");
    let container =
        rhwp::parser::ole_container::parse_ole_container(&bytes[4..]).expect("ole container");
    container.preview_emf.expect("preview_emf")
}

#[test]
fn emfplus_dual_stream_preview_parses_with_bitmap() {
    let emf = load_preview_emf();
    let records = rhwp::emf::parse_emf(&emf)
        .unwrap_or_else(|e| panic!("EMF+ 이중 스트림 재동기 파싱이 실패했다 (#5637 회귀): {e}"));
    assert!(
        records
            .iter()
            .any(|r| matches!(r, rhwp::emf::Record::StretchDIBits(_))),
        "GDI 폴백 비트맵(EMR_STRETCHDIBITS)을 회수해야 한다: {}개 레코드",
        records.len()
    );
    assert!(
        records.iter().any(|r| matches!(r, rhwp::emf::Record::Eof)),
        "재동기 후 EOF 레코드까지 도달해야 한다"
    );
}

#[test]
fn emfplus_dual_stream_preview_converts_to_svg_image() {
    let emf = load_preview_emf();
    let svg = rhwp::emf::convert_to_standalone_svg(&emf)
        .expect("standalone SVG 변환이 성공해야 한다 (#5637)");
    let svg = String::from_utf8(svg).expect("svg utf-8");
    assert!(
        svg.contains("<image "),
        "미리보기 비트맵 <image> 가 방출되어야 한다: {}",
        &svg[..svg.len().min(200)]
    );
}
