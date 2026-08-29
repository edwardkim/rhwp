//! [#6175] 어울림 개체 옆 문단이 저장된 좁은 폭을 지킨다.
//!
//! `samples/issue6175/seed_expo_square_float_body.hwpx` 는 원본(농촌진흥청 156655489,
//! 940KB)의 **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 101KB 로
//! 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **형상.** 1쪽 오른쪽에 용지 기준 `SQUARE` 그림이 있고, 문단 5~7 은 **문단 전체가**
//! 그 옆에 들어간다. 그래서 저장 사다리가 `cs=0 · sw=26692` 로 **균일하게** 좁다.
//!
//! **종전 결함.** `stored_rows_require_external_geometry` 는 한 문단 안에서 폭이
//! **변할** 때만 외부 기하의 증거로 인정했다. 문단 전체가 개체 옆이면 그 변화가
//! 사라져 증거가 소멸하고, 프레임(전폭)과 대조 → 불일치 → 전폭 재래핑 → 본문이
//! 그림 아래로 들어가 가려졌다.
//!
//! **판별자 — 개체 폭 대조.** 결손 폭이 문서에 실재하는 어울림 개체의 흐름 폭과
//! 맞으면 좁음의 출처는 문단 자신이 아니라 외부 기하다.
//!
//! ```text
//! 본문 폭 48188 − 저장 (cs 0 + sw 26692) = 결손 21496
//! 용지기준 SQUARE 그림 폭 21212 (offset 32361 → 프레임 좌표 26692)
//!   → 저장 사다리의 끝이 개체 왼쪽 변과 단위까지 일치
//! ```
//!
//! ⚠ "균일하게 좁다"만으로 판정하면 **문단 테두리 박스의 inset** 을 어울림으로
//! 오인해 #547·#1440 핀이 깨진다(#6129 에서 국소 판별자 2종이 그렇게 반증됐다).
//! 셀에서는 #5818 이 같은 혼동을 "같은 셀에 Square float 실재"로 갈랐고, 이것은 그
//! 계약의 본문 판이다.
//!
//! 한글 2022 실측(문서 편집 버전 = 한글 2020, major 11 → 가장 가까운 설치본):
//! 1쪽 본문 줄의 오른쪽 끝이 323~329pt.
#![cfg(not(target_arch = "wasm32"))]

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6175/seed_expo_square_float_body.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// `pi=<index>` 본문 줄의 (y, width) 목록.
fn body_line_widths(document: &str, para_index: usize) -> Vec<(f64, f64)> {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", document])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let needle = format!(" pi={para_index} ");
    let mut rows = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.starts_with("TextLine") || !line.contains(&needle) {
            continue;
        }
        let y = line
            .split("y=")
            .nth(1)
            .and_then(|r| r.split("..").next())
            .and_then(|v| v.trim().parse::<f64>().ok());
        let w = line
            .split(" w=")
            .nth(1)
            .and_then(|r| r.split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok());
        if let (Some(y), Some(w)) = (y, w) {
            rows.push((y, w));
        }
    }
    rows
}

struct TempSample {
    path: PathBuf,
}

impl TempSample {
    fn path(&self) -> &str {
        self.path
            .to_str()
            .expect("임시 HWPX 경로는 UTF-8이어야 한다")
    }
}

impl Drop for TempSample {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// 같은 폭의 square float가 문단의 저장 행보다 아래에 있으면, 폭 일치만으로는
/// 해당 행을 외부 geometry로 인정할 수 없다.
fn sample_with_square_float_below_body_rows() -> TempSample {
    let source = File::open(sample()).expect("#6175 sample 열기");
    let mut archive = ZipArchive::new(source).expect("#6175 sample zip 열기");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "rhwp-6175-square-float-below-{}-{nonce}.hwpx",
        std::process::id()
    ));
    let output = File::create(&path).expect("임시 HWPX 생성");
    let mut writer = ZipWriter::new(output);

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("sample zip 항목");
        let name = entry.name().to_owned();
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if name.ends_with('/') {
            writer
                .add_directory(name, options)
                .expect("zip 디렉터리 쓰기");
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).expect("zip 항목 읽기");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).expect("section XML utf-8");
            let (before, picture) = xml
                .split_once("<hp:pic id=\"1651052852\"")
                .expect("#6175 square picture");
            let shifted = picture.replacen("vertOffset=\"28980\"", "vertOffset=\"70000\"", 1);
            assert_ne!(shifted, picture, "#6175 picture vertical offset");
            bytes = format!("{before}<hp:pic id=\"1651052852\"{shifted}").into_bytes();
        }
        writer.start_file(name, options).expect("zip 항목 쓰기");
        writer.write_all(&bytes).expect("zip 내용 쓰기");
    }
    writer.finish().expect("임시 HWPX 완료");
    TempSample { path }
}

#[test]
fn paragraph_beside_square_float_keeps_its_stored_narrow_width() {
    // 저장 sw=26692HU = 355.9px. 전폭 재래핑하면 642.5px 로 벌어져 그림을 덮는다.
    for para_index in [5usize, 7] {
        let rows = body_line_widths(&sample(), para_index);
        assert!(!rows.is_empty(), "pi={para_index} 본문 줄을 찾지 못했다");
        for (y, w) in &rows {
            assert!(
                (*w - 355.9).abs() <= 6.0,
                "pi={para_index} y={y:.1} 줄이 전폭으로 재래핑됐다: w={w:.1} (기대 355.9)"
            );
        }
    }
}

#[test]
fn stored_rows_are_not_dropped_by_reflow() {
    // 전폭 재래핑은 줄 수도 줄인다 — pi=5 는 저장 3줄이다.
    let rows = body_line_widths(&sample(), 5);
    assert_eq!(
        rows.len(),
        3,
        "pi=5 의 저장 줄 수가 유지되지 않았다: {rows:?}"
    );
}

#[test]
fn square_float_outside_stored_row_band_does_not_preserve_narrow_rows() {
    let shifted = sample_with_square_float_below_body_rows();
    let rows = body_line_widths(shifted.path(), 5);
    assert!(
        !rows.is_empty(),
        "이동한 fixture에서 pi=5 본문 줄을 찾지 못했다"
    );
    assert!(
        rows.iter().any(|(_, width)| *width > 600.0),
        "본문 행과 겹치지 않는 같은 폭의 square float 때문에 pi=5가 좁은 폭으로 남았다: {rows:?}"
    );
}
