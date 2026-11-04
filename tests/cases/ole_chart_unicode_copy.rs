//! 제목의 정본은 자료 IR과 표현 파서에서 같아야 한다.
//! 원본 HWPX의 ANSI 사본만 바꾼 독립 한컴 Print는 원래 UTF-16 제목을 보존했다.

use std::io::{Cursor, Read};
use std::path::Path;

use rhwp::ole_chart::{parse_ole_chart_contents, render_ole_chart_svg_fragment_with_contents};

fn source_contents() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/143E433F503322BD33.hwpx");
    let mut zip = zip::ZipArchive::new(Cursor::new(std::fs::read(path).expect("원본 HWPX")))
        .expect("원본 ZIP");
    let mut ole = Vec::new();
    zip.by_name("BinData/ole3.ole")
        .expect("원본 차트 OLE")
        .read_to_end(&mut ole)
        .expect("OLE 바이트");
    let mut compound = cfb::CompoundFile::open(Cursor::new(&ole[4..])).expect("차트 CFB");
    let mut contents = Vec::new();
    compound
        .open_stream("Contents")
        .expect("차트 Contents")
        .read_to_end(&mut contents)
        .expect("Contents 바이트");
    contents
}

#[test]
fn chart_ir_and_painted_title_use_the_unicode_copy() {
    let original = source_contents();
    let mut changed = original.clone();
    let (old, _, old_errors) = encoding_rs::EUC_KR.encode("연금　재정　전망");
    let (new, _, new_errors) = encoding_rs::EUC_KR.encode("기금　재정　전망");
    assert!(!old_errors && !new_errors);
    assert_eq!(old.len(), new.len());
    let positions: Vec<_> = original
        .windows(old.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == old.as_ref()).then_some(index))
        .collect();
    let [position] = positions.as_slice() else {
        panic!("원본 제목의 ANSI 사본은 한 개여야 한다");
    };
    changed[*position..*position + old.len()].copy_from_slice(&new);

    let before = parse_ole_chart_contents(&original).expect("원본 차트 IR");
    let after = parse_ole_chart_contents(&changed).expect("ANSI 사본만 바꾼 차트 IR");
    assert_eq!(before.title.as_deref(), Some("연금 재정 전망"));
    assert_eq!(
        after, before,
        "독립 Print가 읽지 않는 ANSI 사본 변경으로 자료·제목이 바뀌지 않는다"
    );
    // 물리 배치를 고정하지 않고 같은 외곽 안에서 정본 선택과 표현의 일관성을 검사한다.
    let bounds = [0.0, 0.0, 300.0, 300.0];
    let original_svg = render_ole_chart_svg_fragment_with_contents(&before, &original, bounds, 1);
    let changed_svg = render_ole_chart_svg_fragment_with_contents(&after, &changed, bounds, 1);
    assert!(original_svg.contains("연금　재정　전망"));
    assert_eq!(
        changed_svg, original_svg,
        "표현 파서도 같은 UTF-16 정본을 사용하며 캐시·자료를 바꾸지 않는다"
    );
}
