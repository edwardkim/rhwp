//! WMF 텍스트 레코드의 음수/오버플로 문자열 길이는 패닉이 아니라 `Err` 로 끝난다.
//!
//! `META_TEXTOUT`(MS-WMF §2.3.5.6)·`META_EXTTEXTOUT`(§2.3.5.2) 의 `StringLength` 는
//! **부호 있는 16비트**다. 손상되거나 악의적인 WMF 가 음수를 담을 수 있는데, 파서가
//! 그 값을 `as usize` 로 넓혀 `read_variable`(내부 `vec![0u8; len]`)에 넘기면
//! `-1i16 as usize` = 18446744073709551615 → **capacity overflow 패닉**이다.
//!
//! #3875 가 `META_POLYLINE`/`META_POLYGON` 의 `NumberOfPoints`(같은 i16 음수 결함)에
//! 넣은 가드와 정확히 같은 클래스다 — 그 PR 의 테스트가 "한 곳을 고칠 때 같은 모양을
//! 전수로 훑지 않으면 이렇게 남는다"고 적었고, 텍스트 레코드가 바로 그 남은 사각이었다.
//!
//! WMF 는 HWP 문서에 그림으로 임베드되므로 이 경로는 신뢰 경계 밖 입력을 받는다.
//! 패닉은 라이브러리·MCP 서버·WASM 소비자를 통째로 죽이므로 DoS 다.

fn synth_wmf(record_function: u16, body: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let push_u16 = |v: u16, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
    let push_u32 = |v: u32, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());

    // ── META_HEADER (type=1, 9 words) ──
    push_u16(1, &mut out);
    push_u16(9, &mut out);
    push_u16(0x0300, &mut out);
    push_u32(0, &mut out);
    push_u32(0, &mut out);
    push_u16(0, &mut out);
    push_u32(0, &mut out);
    push_u16(0, &mut out);

    // ── 대상 레코드: size(u32 words) + function(u16) + body ──
    let record_words = 3 + (body.len() / 2) as u32;
    push_u32(record_words, &mut out);
    push_u16(record_function, &mut out);
    out.extend_from_slice(body);

    // ── META_EOF ──
    push_u32(3, &mut out);
    push_u16(0x0000, &mut out);
    out
}

type RunOut = std::thread::Result<Result<Vec<u8>, rhwp::wmf::converter::ConvertError>>;

fn run_result(bytes: &[u8]) -> RunOut {
    std::panic::catch_unwind(|| {
        rhwp::wmf::converter::WMFConverter::new(bytes, rhwp::wmf::converter::SVGPlayer::new()).run()
    })
}

const META_TEXTOUT: u16 = 0x0521;
const META_EXTTEXTOUT: u16 = 0x0a32;

fn textout_body(string_length: i16) -> Vec<u8> {
    string_length.to_le_bytes().to_vec()
}

fn exttextout_body(string_length: i16) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&0i16.to_le_bytes()); // Y
    body.extend_from_slice(&0i16.to_le_bytes()); // X
    body.extend_from_slice(&string_length.to_le_bytes()); // StringLength
    body.extend_from_slice(&0u16.to_le_bytes()); // fwOpts (ETO 없음 → rectangle 스킵)
    body
}

#[test]
fn textout_negative_string_length_does_not_panic() {
    // 0x7FFF(32767) 도 넣는다: `string_length + (string_length % 2)` 가 i16 에서
    // 오버플로하던 두 번째 경로다.
    for n in [-1i16, i16::MIN, 0x7FFF] {
        let bytes = synth_wmf(META_TEXTOUT, &textout_body(n));
        assert!(
            run_result(&bytes).is_ok(),
            "META_TEXTOUT StringLength={n} 이 패닉했습니다 (DoS)"
        );
    }
}

#[test]
fn exttextout_negative_string_length_does_not_panic() {
    for n in [-1i16, i16::MIN] {
        let bytes = synth_wmf(META_EXTTEXTOUT, &exttextout_body(n));
        assert!(
            run_result(&bytes).is_ok(),
            "META_EXTTEXTOUT StringLength={n} 이 패닉했습니다 (DoS)"
        );
    }
}

/// 양성 대조 — 음수가 아닌 길이는 이 가드에 걸리지 않는다("전부 Err" 수정 방지).
#[test]
fn non_negative_string_length_is_not_rejected_by_the_guard() {
    for n in [0i16, 2, 4] {
        for func in [META_TEXTOUT, META_EXTTEXTOUT] {
            let body = if func == META_TEXTOUT {
                textout_body(n)
            } else {
                exttextout_body(n)
            };
            let bytes = synth_wmf(func, &body);
            let r = run_result(&bytes);
            assert!(
                r.is_ok(),
                "func={func:#06x} StringLength={n} 이 패닉했습니다"
            );
            if let Ok(Err(e)) = r {
                let msg = format!("{e:?}");
                assert!(
                    !msg.contains("must not be negative"),
                    "func={func:#06x} StringLength={n} 인데 음수 가드가 걸렸습니다: {msg}"
                );
            }
        }
    }
}
