//! WMF POLYLINE/POLYGON 의 음수 점 개수는 패닉이 아니라 `Err` 로 끝난다.
//!
//! `META_POLYLINE`·`META_POLYGON` 레코드의 `NumberOfPoints` 는 **부호 있는 16비트**다
//! (MS-WMF §2.3.3.14/§2.3.3.15). 손상되거나 악의적인 WMF 가 음수를 담을 수 있는데,
//! 파서가 그 값을 `as usize` 로 넓히면 usize::MAX 근처가 된다 — `-1i16 as usize` 는
//! 64비트에서 **18446744073709551615** 이고, `Vec::with_capacity` 는 이 요청에
//! capacity overflow 로 **패닉**한다.
//!
//! 같은 결함을 Region 은 이미 막고 있었다
//! (`src/wmf/parser/objects/graphics/region.rs:96`, `if scan_count < 0 { Err }`, #3004).
//! 그런데 구조가 같은 POLYLINE·POLYGON 두 곳에는 그 가드가 없었다 — 한 곳을 고칠 때
//! 같은 모양을 전수로 훑지 않으면 이렇게 남는다.
//!
//! WMF 는 HWP 문서에 그림으로 **임베드**되므로 이 경로는 신뢰 경계 밖 입력을 받는다.
//! 패닉은 라이브러리 소비자(WASM 모듈 포함)를 통째로 죽이므로 DoS 다 —
//! 파싱 실패(`Err`)로 끝나는 것이 언제나 낫다.
//!
//! 참고: 이 하니스는 `fuzz/fuzz_targets/parse_wmf.rs` 의 사정거리 안에 있다.
//! 퍼저가 돌았다면 잡았을 결함이고, 그래서 코퍼스 재생만으로도 재발을 검출할 수 있다.

/// WMF 레코드 하나를 감싼 최소 placeable-less 파일을 만든다.
///
/// 헤더는 `META_HEADER`(type=1, headersize=9 words) 형태를 갖추고, 그 뒤에 대상 레코드
/// 하나와 `META_EOF` 를 붙인다. 파서가 레코드 순회에 도달하는 것이 목적이라
/// 크기 필드는 실제 바이트 수와 맞춘다.
fn synth_wmf_with_poly_record(record_function: u16, number_of_points: i16) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut push_u16 = |v: u16, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
    let mut push_u32 = |v: u32, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());

    // ── META_HEADER ──
    push_u16(1, &mut out); // Type: 1 = memory metafile
    push_u16(9, &mut out); // HeaderSize: 9 words
    push_u16(0x0300, &mut out); // Version: 3.0
    push_u32(0, &mut out); // SizeLow (검증하지 않는 파서를 위해 0)
    push_u32(0, &mut out); // SizeHigh 자리 — 실제로는 u16 2개지만 합이 같다
    push_u16(0, &mut out); // NumberOfObjects
    push_u32(0, &mut out); // MaxRecord
    push_u16(0, &mut out); // NumberOfMembers

    // ── 대상 레코드: size(u32 words) + function(u16) + NumberOfPoints(i16) ──
    // 점 데이터는 일부러 넣지 않는다. 가드가 없으면 with_capacity 에서 먼저 죽고,
    // 가드가 있으면 그 전에 Err 로 끝나므로 어느 쪽이든 점 데이터에 도달하지 않는다.
    push_u32(4, &mut out); // RecordSize: 4 words
    push_u16(record_function, &mut out);
    out.extend_from_slice(&number_of_points.to_le_bytes());

    // ── META_EOF ──
    push_u32(3, &mut out);
    push_u16(0x0000, &mut out);

    out
}

const META_POLYGON: u16 = 0x0324;
const META_POLYLINE: u16 = 0x0325;

/// 파싱을 시도하되 **패닉하지 않는 것**만 검사한다.
///
/// 성공이든 `Err` 든 상관없다 — 이 계약이 막는 것은 프로세스가 죽는 것이다.
/// (합성 파일이 다른 이유로 먼저 거부될 수도 있으므로 결과값을 단언하지 않는다.)
fn assert_no_panic(label: &str, bytes: &[u8]) {
    let result = std::panic::catch_unwind(|| {
        let _ =
            rhwp::wmf::converter::WMFConverter::new(bytes, rhwp::wmf::converter::SVGPlayer::new())
                .run();
    });
    assert!(
        result.is_ok(),
        "{label}: 손상된 WMF 가 패닉했습니다. \
         NumberOfPoints 는 i16 이므로 음수를 `as usize` 로 넓히기 전에 걸러야 합니다 \
         (region.rs:96 과 같은 방식)."
    );
}

#[test]
fn polyline_with_negative_point_count_does_not_panic() {
    for n in [-1i16, -2, i16::MIN] {
        let bytes = synth_wmf_with_poly_record(META_POLYLINE, n);
        assert_no_panic(&format!("META_POLYLINE NumberOfPoints={n}"), &bytes);
    }
}

#[test]
fn polygon_with_negative_point_count_does_not_panic() {
    for n in [-1i16, -2, i16::MIN] {
        let bytes = synth_wmf_with_poly_record(META_POLYGON, n);
        assert_no_panic(&format!("META_POLYGON NumberOfPoints={n}"), &bytes);
    }
}

/// 양성 대조 — 음수가 아닌 값에서는 이 가드가 걸리지 않는다.
///
/// 이 대조가 없으면 "전부 Err 로 만들어 버리는" 수정도 위 두 시험을 통과한다.
#[test]
fn non_negative_point_count_is_not_rejected_by_the_negative_guard() {
    for n in [0i16, 1, 3] {
        let bytes = synth_wmf_with_poly_record(META_POLYLINE, n);
        let result = std::panic::catch_unwind(|| {
            rhwp::wmf::converter::WMFConverter::new(
                bytes.as_slice(),
                rhwp::wmf::converter::SVGPlayer::new(),
            )
            .run()
        });
        assert!(
            result.is_ok(),
            "NumberOfPoints={n}: 음수가 아닌데 패닉했습니다"
        );
        // 파싱이 Err 로 끝날 수는 있다(합성 파일이 최소형이라 점 데이터가 없다).
        // 다만 그 사유가 "must not be negative" 여서는 안 된다.
        if let Ok(Err(e)) = result {
            let msg = format!("{e:?}");
            assert!(
                !msg.contains("must not be negative"),
                "NumberOfPoints={n} 인데 음수 가드가 걸렸습니다: {msg}"
            );
        }
    }
}
