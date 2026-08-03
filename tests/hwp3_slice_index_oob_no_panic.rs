//! HWP3 파서가 문서 파생 길이 필드로 슬라이스를 시작할 때 경계를 넘지 않는다.
//!
//! `info_block_length`(doc_info, u16)는 **문서가 정하는 값**이다. 짧은/손상된 `.hwp`
//! 에서 본문 시작 오프셋 `30 + docinfo(128) + summary(1008) + info_block_length` 가
//! 파일 끝을 넘으면, 종전 `&data[start..]` 는 `range start index N out of range for
//! slice of length M` 로 **패닉**했다(mod.rs:3489). WMF(#3875)와 같은 부류의 DoS —
//! 신뢰 경계 밖 `.hwp` 로 `parse_hwp3` 를 부르는 라이브러리·MCP 소비자를 죽인다.
//!
//! 형제 사이트(`block.data[32..]`, `>= 24` 가드뿐)도 같은 커밋에서 함께 막았다.

fn sample() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/hwp3-pagedef-1915.hwp"
    ))
    .expect("HWP3 샘플")
}

/// 유효 파일은 여전히 정상 파싱된다(가드가 정상 경로를 막지 않음 — 양성 대조).
#[test]
fn valid_hwp3_still_parses() {
    let data = sample();
    let r = std::panic::catch_unwind(|| rhwp::parser::hwp3::parse_hwp3(&data));
    assert!(r.is_ok(), "유효 HWP3 가 패닉했습니다");
    assert!(
        matches!(r, Ok(Ok(_))),
        "유효 HWP3 파싱이 실패했습니다(가드가 정상 경로를 막음)"
    );
}

/// `info_block_length` 를 0xFFFF 로 키우면 본문 시작이 파일 끝을 넘는다 → 패닉 금지.
#[test]
fn oversized_info_block_length_does_not_panic() {
    let mut data = sample();
    // info_block_length: doc_info off 126 = 파일 off 156 (u16 LE). encrypted(off 126)는 0 유지.
    data[156] = 0xFF;
    data[157] = 0xFF;
    let r = std::panic::catch_unwind(|| {
        let _ = rhwp::parser::hwp3::parse_hwp3(&data);
    });
    assert!(
        r.is_ok(),
        "info_block_length 가 파일 범위를 넘을 때 패닉했습니다 (DoS)"
    );
}
