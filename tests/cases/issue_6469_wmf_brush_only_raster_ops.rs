//! [#6469] WMF 의 미구현 래스터 연산을 **통째로 버리지 않는다**.
//!
//! `samples/issue6469/wmf_fill_shapes.hwpx` 는 과학기술정보통신부 보도참고자료
//! (156627451, 4.07MB)의 구조 보존 슬라이스다 — 시험 대상인 2쪽 두 도해
//! (`BinData/image4.wmf`·`image5.wmf`)만 원본 그대로 두고 나머지 BinData 를 줄여
//! 0.43MB 로 만들었다.
//!
//! **증상.** 2·3쪽 WMF 도해의 옅은 회색 배경 패널이 없다. 아이콘·글자·구분선은
//! 나오는데 그 뒤를 받치는 면만 비어 도해가 흰 바탕에 떠 있다.
//!
//! **근인 — 지원 ROP 이 4종뿐이고 나머지는 조용히 버려진다.**
//!
//! `ternary_raster_operator.rs` 의 `run()` 은 `BLACKNESS`·`SRCCOPY`·`PATCOPY`·
//! `WHITENESS` 만 처리하고 나머지는 `Ok(None)` 을 돌려준다. 호출부는 그걸 받으면
//! **레코드를 흔적 없이 지운다** — 로그도 남지 않는다.
//!
//! 이 패널은 소스 없는 `DibBitBlt` 세 개가 그린다.
//!
//! ```text
//! rec37  PATINVERT  brush #D9D9D9      D ⊕ P
//! rec43  DPa        brush = DIB 패턴    D ∧ P
//! rec48  PATINVERT  brush #D9D9D9      D ⊕ P
//! ```
//!
//! 흰 바탕에서 이 조합의 최종 결과는 **브러시 색 자체**다.
//!
//! ```text
//! 0xFFFFFF ⊕ 0xD9D9D9 = 0x262626
//! 0x262626 ∧ 0xD9D9D9 = 0x000000
//! 0x000000 ⊕ 0xD9D9D9 = 0xD9D9D9   ← 브러시 색
//! ```
//!
//! **수정.** 소스를 쓰지 않고 브러시만 쓰는 ROP 은 `PATCOPY` 로, 소스를 쓰는 ROP 은
//! `SRCCOPY` 로 근사한다. 같은 사각형·같은 브러시의 `PATINVERT` 쌍은 서로 상쇄하므로
//! 두 번째는 건너뛴다(그러지 않으면 가운데 패턴 blit 이 칠한 그림을 덮는다).
//!
//! **이 갈래는 종전에 아무것도 그리지 않던 경우에만 걸린다** — 이미 그려지던 출력은
//! 하나도 바뀌지 않는다.
//!
//! **오라클 — 한글 2022** (`producer=Hancom PDF 1.3.0.550`, 15쪽 일치). 좌측 패널의
//! 비백 픽셀 비율:
//!
//! | 영역 | 종전 | **수정 후** | 한글 2022 |
//! |---|---:|---:|---:|
//! | 2쪽 위 | 11.3% | **63.3%** | 42.7% |
//! | 2쪽 아래 | 14.8% | **69.1%** | 69.1% |
//! | 3쪽 | 14.8% | **59.6%** | 61.7% |
//!
//! 2쪽 아래는 **정확히 일치**하고 3쪽은 2.1%p 차다. 2쪽 위가 넘치는 것은 패널 위에
//! 한글이 그리는 **흰 원**이 아직 안 나오기 때문이다 — 그 원은 `DPa` 의 **DIB 패턴
//! 브러시**가 담고 있는데 `svg2pdf` 가 `<pattern>` 채움을 그리지 않는다. 별도 축이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 한글 2022 실측 — 2쪽 아래 도해 좌패널의 비백 픽셀 비율(%).
const HANGUL_LOWER_PANEL_PCT: f64 = 69.1;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6469/wmf_fill_shapes.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6469-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 2쪽 SVG.
fn page2_svg() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-svg",
            &sample(),
            "-p",
            "1",
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "svg"))
        .expect("SVG 가 없다");
    std::fs::read_to_string(path).unwrap()
}

/// 내장 WMF SVG(data URI)들을 디코드해 돌려준다.
fn embedded_wmf_svgs(svg: &str) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in svg.split("data:image/svg+xml;base64,").skip(1) {
        let end = chunk
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '+' && c != '/' && c != '=')
            .unwrap_or(chunk.len());
        if let Ok(bytes) = base64_decode(&chunk[..end]) {
            if let Ok(s) = String::from_utf8(bytes) {
                out.push(s);
            }
        }
    }
    out
}

/// 의존성 없이 쓰는 최소 base64 디코더.
fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut map = [255u8; 256];
    for (i, c) in T.iter().enumerate() {
        map[*c as usize] = i as u8;
    }
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for b in s.bytes() {
        if b == b'=' {
            break;
        }
        let v = map[b as usize];
        if v == 255 {
            return Err(());
        }
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

/// 회색 패널이 산출 SVG 에 실제로 들어간다.
///
/// 종전에는 이 색이 **한 번도** 나오지 않았다 — 브러시는 원본에 있는데 그리는 레코드가
/// 미지원 ROP 이라 통째로 버려졌다.
#[test]
fn gray_panel_brush_reaches_the_output() {
    let svgs = embedded_wmf_svgs(&page2_svg());
    assert_eq!(svgs.len(), 2, "2쪽에는 WMF 도해 2개가 있다");
    for (i, inner) in svgs.iter().enumerate() {
        assert!(
            inner.contains("#D9D9D9"),
            "도해 {i}: 원본 브러시 #D9D9D9 로 그리는 도형이 방출되어야 한다 (종전 0개)"
        );
    }
}

/// 같은 사각형의 `PATINVERT` 쌍은 상쇄해 **한 번만** 그린다.
///
/// 그러지 않으면 두 번째 XOR 이 가운데 패턴 blit 이 칠한 그림을 평면 색으로 덮는다.
#[test]
fn xor_pair_is_cancelled_so_gray_rect_is_emitted_once() {
    let svgs = embedded_wmf_svgs(&page2_svg());
    for (i, inner) in svgs.iter().enumerate() {
        let grays = inner.matches("fill=\"#D9D9D9\"").count();
        assert_eq!(
            grays, 1,
            "도해 {i}: 회색 사각형은 XOR 상쇄 후 1개여야 한다 (상쇄 없으면 2개)"
        );
    }
}
