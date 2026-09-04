//! [Issue #5057] 같은 문서를 HWP5 로 읽을 때와 직접 HWPX 로 읽을 때 쪽수가 갈리던 결함.
//!
//! 저장 **첫 조각 source frame** 이 주는 마지막 행 초과 허용치
//! (`source_first_fragment_overflow_allowance`)가 네이티브 HWP5 프로파일에만 열려
//! 있었다. 그 기록은 컨테이너와 무관하게 파일에 그대로 있는데도, `META-INF/rhwp-hwp5-origin`
//! 표식 하나로 적용 여부가 갈렸다.
//!
//! `21484591` 실측 — 같은 바이트에서 표식만 뺀 사본과의 A/B:
//!
//! ```text
//!   두 프로파일 모두  avail_for_rows = 523.4   (host_before 3.8 · vert_off 20.9 동일)
//!   hwp5    r=7 에서 source_frame_whole_row_fits=true  → consumed 528.8 (5.4px 초과 수용)
//!           → 8행 전부, 13쪽
//!   direct  r=7 에서 false → 7행에서 끊고 8행은 다음 단 → 14쪽
//!   한/글 2024 = 13쪽
//! ```
//!
//! ⚠ 예산은 두 프로파일이 **동일**하다. 갈리는 것은 마지막 행의 초과 수용뿐이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/ordin_downloads/김천시/
///  21484591_【별지 제1호-제13호 서식】(김천시 하수도 사용 조례 시행규칙).hwp`
///
/// ⚠ `.hwp` 를 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/` 전체를 스윕해
/// 무관한 직렬화 발산을 끌고 온다. `RHWP_ISSUE5057_SAMPLE` 로 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE5057_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(
            r"C:\Users\planet\hwpdocs_10k_share",
            r"\ordin_downloads\김천시"
        ),
        concat!(r"D:\hwpdocs_10k_share", r"\ordin_downloads\김천시"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("21484591") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// 원본 HWP5 는 한/글 2024 와 같은 13쪽이어야 한다.
#[test]
fn native_hwp5_matches_hangul_page_count() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        13,
        "원본 HWP5 는 한/글 2024 와 같은 13쪽이어야 한다 — #5057 회귀"
    );
}

/// **표식을 뺀 직접 HWPX 판도 같은 쪽수**여야 한다 — 이것이 이 이슈의 축이다.
///
/// `export-hwpx` 산출물에서 `META-INF/rhwp-hwp5-origin` 만 제거해 direct-HWPX 프로파일로
/// 읽게 한다. 종전에는 14쪽이었다.
#[test]
fn direct_hwpx_profile_agrees_with_native_hwp5() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let Ok(hwpx) = core.export_hwpx_native() else {
        return;
    };

    // 표식만 뺀 사본을 만든다 — 나머지 엔트리는 그대로다.
    let cursor = std::io::Cursor::new(hwpx.clone());
    let Ok(mut zip) = zip::ZipArchive::new(cursor) else {
        return;
    };
    let mut out = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for index in 0..zip.len() {
            let Ok(mut entry) = zip.by_index(index) else {
                return;
            };
            let name = entry.name().to_string();
            if name.contains("rhwp-hwp5-origin") {
                continue;
            }
            use std::io::Read as _;
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_err() {
                return;
            }
            use std::io::Write as _;
            if writer.start_file(name, options).is_err() || writer.write_all(&buf).is_err() {
                return;
            }
        }
        if writer.finish().is_err() {
            return;
        }
    }

    let direct = DocumentCore::from_bytes(&out).expect("표식 없는 HWPX 로드");
    assert_eq!(
        direct.page_count(),
        core.page_count(),
        "같은 바이트인데 출처 표식 하나로 쪽수가 갈리면 안 된다 — #5057 회귀 \
         (direct {} vs hwp5 {})",
        direct.page_count(),
        core.page_count()
    );
}
