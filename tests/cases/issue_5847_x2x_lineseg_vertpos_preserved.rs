//! [Issue #5847] HWPX 재저장이 원본 lineseg `vertpos`(쪽-상대)를 구역 누적
//! 좌표로 덮어써 한글 2022 가 본문을 폐기한다 — 08818: 81쪽 → 5쪽.
//!
//! 근인: `reflow_zero_height_paragraphs` 가 구역 안에 캐시 없는 문단이 하나라도
//! 있으면 **원본 캐시 보유 문단까지** `running_vpos` 누적값으로 덮어쓰고, HWPX
//! 직렬화기가 그 내부 좌표를 그대로 방출했다. 합성(bit31) 줄도 함께 방출돼
//! 원본에 없던 linesegarray 가 생겼다.
//!
//! 수정: (1) 덮어쓰기 전 원본 vertpos 를 문단별 스냅샷(`source_line_seg_
//! vertical_pos`)으로 보존하고 HWPX 직렬화기가 그 값으로 되돌려 낸다 —
//! 렌더 좌표는 불변. (2) 줄 전체가 reflow 합성(bit31)인 문단은 원본과 같게
//! linesegarray 방출을 생략한다(#1380 계약).
//!
//! 픽스처는 원본 HWPX(156682086, 02319) 구역0 문단 24..36 절단 + BinData
//! 스텁(94KB) — 캐시 없는 문단 2개가 구역 재계산을 발화시키는 최소 재현.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5847/x2x_lineseg_vertpos_cumulative.hwpx";

fn section0_xml(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip open");
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .expect("section0.xml")
        .read_to_string(&mut xml)
        .expect("read section0.xml");
    xml
}

fn lineseg_vertpos(xml: &str) -> Vec<i64> {
    xml.split("<hp:lineseg ")
        .skip(1)
        .filter_map(|chunk| {
            let head = &chunk[..chunk.find("/>")?];
            let v = head.split_once("vertpos=\"")?.1;
            v[..v.find('"')?].parse().ok()
        })
        .collect()
}

#[test]
fn issue_5847_x2x_export_preserves_source_lineseg_vertpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let src_bytes = std::fs::read(path).expect("read sample");
    let core = DocumentCore::from_bytes(&src_bytes).expect("open");
    let out_bytes = core.export_hwpx_native().expect("export hwpx");

    let src_xml = section0_xml(&src_bytes);
    let out_xml = section0_xml(&out_bytes);

    // 원본 캐시 보유 문단의 vertpos 는 값 그대로 (쪽-상대 좌표 보존).
    let src_vp = lineseg_vertpos(&src_xml);
    let out_vp = lineseg_vertpos(&out_xml);
    assert_eq!(
        src_vp, out_vp,
        "재저장 vertpos 가 원본과 달라졌다 — 구역 누적 좌표 오염(#5847) 재발"
    );
    // 결함 시 캐시 없는 문단에 합성 줄이 방출돼 개수가 늘고 누적 좌표가 실린다.
    let src_lsa = src_xml.matches("<hp:linesegarray").count();
    let out_lsa = out_xml.matches("<hp:linesegarray").count();
    assert_eq!(
        src_lsa, out_lsa,
        "원본에 linesegarray 가 없던 문단은 재저장에서도 생략되어야 한다(#1380)"
    );
    // 합성 줄 표식(bit31)이 파일로 새면 안 된다.
    for chunk in out_xml.split("flags=\"").skip(1) {
        let flags: u64 = chunk[..chunk.find('"').unwrap()].parse().unwrap_or(0);
        assert_eq!(
            flags & (1 << 31),
            0,
            "합성 lineseg 표식(bit31)이 파일에 방출됐다: flags={flags}"
        );
    }
}
