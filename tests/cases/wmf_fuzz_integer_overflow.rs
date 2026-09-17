//! WMF 변환기가 파일의 정수 끝값에서 넘침으로 패닉하지 않는다.
//!
//! `fuzz-smoke` 워크플로의 `parse_wmf` 타깃(`cargo +nightly fuzz run parse_wmf`)이 찾은 입력을
//! `tests/fixtures/wmf_fuzz/` 에 둔다. 모두 파일에서 읽은 `i16`/`u32` 값에 무검사 정수 산술을
//! 적용해 "attempt to negate/add/subtract with overflow" 로 패닉했다(overflow-checks 빌드 —
//! fuzz·debug·test 프로필).
//!
//! | 입력 | 넘친 자리 |
//! | --- | --- |
//! | `window_ext_i16_min.wmf` | `Window::ext` 의 `x.abs()` (SetWindowExt x = 0x8000) |
//! | `main_window_origin_abs.wmf` | 상류 `main` 예약 실행 입력 — `main` 의 `point_s_to_absolute_point` `abs()`(devel 은 #6617 로 제거) |
//! | `emf_escape_data_size_u32_max.wmf` | META_ESCAPE ENHANCED_METAFILE `data_size + 34` |
//! | `ellipse_rect_i16_extremes.wmf` | 타원 사각형 반지름 `(right - left) / 2` |
//! | `font_escapement_i16_min.wmf` | 글꼴 `-escapement / 10` |
//! | `get_color_table_start_after_byte_count.wmf` | META_ESCAPE GETCOLORTABLE `byte_count - start` |
//! | `pen_dash_width_i16_extreme.wmf` | 펜 점선 간격 `width.x * 10` |
//! | `bitmap_line_past_pixel_data.wmf` | 비트맵 줄 슬라이스가 픽셀 데이터 밖(범위 밖 인덱스 — **릴리스에서도 패닉**) |
//! | `eps_size_below_header.wmf` | META_ESCAPE ENCAPSULATED_POSTSCRIPT `size - 머리 길이` |
//! | `text_baseline_offset_i16_extreme.wmf` | ExtTextOut 기준점 `y + 세로 정렬 보정` |
//!
//! 주의: CI 전체 회귀(`release-test` 프로필)는 overflow-checks 가 꺼져 수정 전에도 통과한다.
//! 결함 검출 증거는 test 프로필(`node scripts/run-rust-test.mjs wmf_fuzz_integer_overflow`)과
//! fuzz 재생이다.

use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

#[test]
fn wmf_fuzz_regression_inputs_do_not_panic() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wmf_fuzz");
    let mut inputs: Vec<_> = std::fs::read_dir(&dir)
        .expect("fuzz 입력 폴더")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "wmf"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 10,
        "fuzz 회귀 입력이 누락됐다: {} 개",
        inputs.len()
    );

    let panicked: Vec<String> = inputs
        .iter()
        .filter(|path| {
            let data = std::fs::read(path).expect("fuzz 입력");
            std::panic::catch_unwind(|| {
                let _ = WMFConverter::new(data.as_slice(), SVGPlayer::new()).run();
            })
            .is_err()
        })
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(panicked.is_empty(), "WMF 변환이 패닉한 입력: {panicked:?}");
}

/// META_TEXTOUT와 META_EXTTEXTOUT는 같은 TOP/BOTTOM 기준점 보정을 쓴다.
/// 파일의 i16 좌표에 ascent/descent를 더하는 경계와 정상 범위를 함께 검사한다.
#[test]
fn textout_vertical_alignment_handles_both_i16_limits() {
    let seed = include_bytes!("../fixtures/pr7239_review/textout_baseline_i16_max.wmf");
    for (align, y) in [
        (0u16, i16::MAX),
        (8, i16::MIN),
        (0, 100),
        (8, 100),
        (24, i16::MAX),
    ] {
        let mut bytes = seed.to_vec();
        let mut pos = 18; // 표준 WMF header
        while u16::from_le_bytes(bytes[pos + 4..pos + 6].try_into().unwrap()) != 0x0521 {
            pos += u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize * 2;
        }
        // 한 글자와 WORD padding 뒤에 YStart가 있다.
        bytes[pos + 10..pos + 12].copy_from_slice(&y.to_le_bytes());
        let mut alignment = 4u32.to_le_bytes().to_vec();
        alignment.extend_from_slice(&0x012eu16.to_le_bytes());
        alignment.extend_from_slice(&align.to_le_bytes());
        bytes.splice(pos..pos, alignment);
        let words = bytes.len() as u32 / 2;
        bytes[6..10].copy_from_slice(&words.to_le_bytes());
        let svg = WMFConverter::new(bytes.as_slice(), SVGPlayer::new())
            .run()
            .unwrap_or_else(|e| panic!("align={align}, y={y}: {e}"));
        let svg = String::from_utf8(svg).unwrap();
        assert!(
            svg.contains(">A</text>"),
            "글자를 숨기지 않아야 한다: {svg}"
        );
    }
}
