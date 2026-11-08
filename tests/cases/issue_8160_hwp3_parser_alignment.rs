//! #8160: HWP3 책갈피·날짜 제어의 레코드 정렬과 본문 보존. 이전 Task #877.
use rhwp::parser::hwp3::parse_hwp3;
use std::{fs::File, io::Read};
#[test]
fn issue_8160_hwp3_load_preserves_record_alignment() {
    // [Task #877] hwp3-sample16.hwp panic 회귀 + paragraph alignment 정합.
    // Stage 1: WASM RawVec overflow panic → graceful Err (가드 도입)
    // Stage 2: ch=6 책갈피 / ch=7 날짜형식 / ch=8 날짜코드 record size 정합
    //          (한글문서파일구조3.0 §10.2/§10.3/§10.4 참고)
    //
    // 본 sample16 은 표지 picture(ch=11) + 책갈피(ch=6) 가 다수 포함된 64쪽 RFP 문서.
    // ch=6 가 8 byte (current) 가 아닌 spec 의 42 byte 로 처리되지 않으면 paragraph
    // stream alignment 가 어긋나 28737 페이지로 폭주 인식됨.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16.hwp");
    let mut data = Vec::new();
    File::open(path).unwrap().read_to_end(&mut data).unwrap();
    let doc = parse_hwp3(&data).expect("sample16 parse failed");
    // 정상 alignment 시 한컴 HWP5 변환본과 동일한 1058 paragraphs 인식.
    // 누락/오인 alignment 시 77 (Stage 1 only) 또는 더 적은 수 인식됨.
    let total_paras: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
    assert!(
        total_paras >= 1000,
        "sample16 paragraph count too low ({}); ch=6/7/8 alignment 회귀 의심",
        total_paras
    );
}
