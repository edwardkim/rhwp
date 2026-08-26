//! [#5967] 한컴 판정 자산의 재생성 계약 — 바이트가 아니라 **스트림**으로 잰다.
//!
//! `samples/issue5447/` 38건은 한컴 2022 가 실제로 열어 판정한 바이트다. 그런데 devel 의
//! `b9eb55107`("검토: CI 완료 외부 PR 7건 통합 수용 (#5912)")이 `mini_cfb` 를 MS-CFB 쪽으로
//! 고치면서 — 디렉터리 red/black 을 실제로 칠하고(`color_deepest_nodes`), FAT·MiniFAT 미사용
//! 슬롯을 FREESECT 로 선채움 — 같은 입력을 지금 라이터로 재포장하면 컨테이너 살림 바이트가
//! 달라진다. **되돌리지 않는다**: 종전 "전 엔트리 black" 은 leaf 깊이가 갈릴 때 black-height 가
//! 어긋나고 미사용 슬롯 `0x00` 은 strict 파서에서 sector 0 참조로 읽힌다.
//! `tests/cases/mini_cfb_strict_contract.rs` 가 그 반대 방향을 이미 상시로 고정한다.
//!
//! 그래서 자산은 **동결**하고(한컴이 연 바로 그 바이트), 잃어버린 "재생성 바이트 동일성" 을
//! 두 계약으로 대체한다.
//!
//! - [`judged_streams_survive_a_current_writer_repack`] — 판정 대상 **내용**은 재현된다.
//! - [`writer_drift_is_confined_to_directory_color_and_fat_fill`] — 드리프트는 컨테이너
//!   살림 두 자리에만 갇힌다. 그 밖이 하나라도 바뀌면 여기서 터진다.
//!
//! 생성기(`b2_variants`)에 기대지 않고 커밋된 자산만으로 자기완결한다 — `output/` 을 쓰지
//! 않으므로 CI 에서 상시로 돈다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::ole_container::{all_ole_streams, ole_root_clsid};
use rhwp::serializer::mini_cfb::build_cfb_with_root_clsid;
use std::path::{Path, PathBuf};

const SECTOR_SIZE: usize = 512;
const DIR_ENTRY_SIZE: usize = 128;
/// 디렉터리 엔트리 안 color flag 의 위치. 0 = red, 1 = black.
const COLOR_FLAG_IN_ENTRY: usize = 67;
const FREESECT: u32 = 0xFFFF_FFFF;
const ENDOFCHAIN: u32 = 0xFFFF_FFFE;
/// 헤더가 직접 물고 있는 DIFAT 슬롯 수.
const HEADER_DIFAT_SLOTS: usize = 109;

/// 원장 38행 = 대조군 9 + 변종 28 + 변환본 1.
const ASSET_TOTAL: usize = 38;
/// 그중 rhwp 라이터가 실제로 쓴 것 — 대조군은 한컴 저작 원본의 무편집 사본이다.
const REWRITTEN_TOTAL: usize = 29;

const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 원장 38행을 `(이름, 역할, 자산 경로)` 로 읽는다.
///
/// 파일명을 코드에 박지 않고 원장에서 받는 것은 판정 자산이 macOS↔Windows 를 오가며 파일명이
/// NFC/NFD 로 갈렸던 전례(#5447 보고서 §6-1) 때문이다. 경로도 원장이 정본이다.
fn ledger_entries() -> Vec<(String, String, PathBuf)> {
    let path = manifest("samples/issue5447/MANIFEST.json");
    let ledger: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("판정 원장 읽기")).expect("원장 JSON");
    let entries = ledger["entries"].as_array().expect("entries");
    let out: Vec<(String, String, PathBuf)> = entries
        .iter()
        .map(|e| {
            let name = e["name"].as_str().expect("name").to_string();
            let role = e["role"].as_str().expect("role").to_string();
            let rel = e["original_path"].as_str().expect("original_path");
            (name, role, manifest(rel))
        })
        .collect();
    assert_eq!(out.len(), ASSET_TOTAL, "원장 38행");
    out
}

/// 판정 자산 안의 중첩 OLE CFB. HWPX 는 ZIP 엔트리, HWP5 는 CFB 스트림에서 오고 둘 다 앞의
/// 4바이트 LE 크기 접두어는 파서가 뗀다 — IR 에는 맨 CFB 가 들어 있다.
fn nested_cfb(path: &Path) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc =
        rhwp::parse_document(&bytes).unwrap_or_else(|e| panic!("{}: 파싱 {e:?}", path.display()));
    doc.bin_data_content
        .iter()
        .map(|c| c.data.load())
        .find(|b| b.starts_with(&CFB_MAGIC))
        .unwrap_or_else(|| panic!("{}: 중첩 CFB 가 없다", path.display()))
}

/// 같은 스트림 집합을 **지금 라이터로** 다시 조립한다.
///
/// `replace_ole_stream` 은 내용이 같으면 원본을 그대로 돌려주는 짧은 회로가 있어(#4100 수용
/// 기준 2) 여기서 쓸 수 없다. 재조립 자체가 관측 대상이므로 조립기를 직접 부른다.
fn repack_with_current_writer(nested: &[u8], label: &str) -> Vec<u8> {
    let streams = all_ole_streams(nested).unwrap_or_else(|| panic!("{label}: 중첩 CFB 열거 실패"));
    let clsid = ole_root_clsid(nested).unwrap_or_else(|| panic!("{label}: 루트 CLSID"));
    let refs: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    build_cfb_with_root_clsid(&refs, clsid).unwrap_or_else(|e| panic!("{label}: 재조립 {e}"))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32"))
}

fn sector_offset(sector: usize) -> usize {
    SECTOR_SIZE + sector * SECTOR_SIZE
}

/// DIFAT 이 가리키는 FAT 섹터 번호들.
///
/// 판정 자산의 중첩 CFB 는 전부 헤더 DIFAT(109 슬롯) 안에서 끝난다 — 그 밖은 여기서 다루지
/// 않고 소리 내어 실패한다. 조용히 일부만 훑으면 분류가 헐거워진다.
fn fat_sectors(cfb: &[u8], label: &str) -> Vec<usize> {
    let fat_count = read_u32_at(cfb, 44) as usize;
    let difat_count = read_u32_at(cfb, 72) as usize;
    assert_eq!(
        difat_count, 0,
        "{label}: DIFAT 섹터를 쓰는 컨테이너다 — 이 분류기는 헤더 DIFAT 만 안다"
    );
    assert!(
        fat_count <= HEADER_DIFAT_SLOTS,
        "{label}: FAT 섹터 {fat_count} 개가 헤더 DIFAT 를 넘는다"
    );
    (0..fat_count)
        .map(|i| read_u32_at(cfb, 76 + i * 4) as usize)
        .collect()
}

fn fat_entry(cfb: &[u8], fat: &[usize], index: usize) -> u32 {
    let per_sector = SECTOR_SIZE / 4;
    let sector = fat[index / per_sector];
    read_u32_at(cfb, sector_offset(sector) + (index % per_sector) * 4)
}

/// FAT 사슬을 따라간 섹터 번호들.
fn chain(cfb: &[u8], fat: &[usize], start: u32, label: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut cursor = start;
    while cursor != ENDOFCHAIN && cursor != FREESECT {
        assert!(
            !out.contains(&(cursor as usize)),
            "{label}: 섹터 사슬이 순환한다"
        );
        out.push(cursor as usize);
        cursor = fat_entry(cfb, fat, cursor as usize);
    }
    out
}

/// 컨테이너 살림 영역 — 디렉터리 섹터와 FAT·MiniFAT 섹터.
struct Housekeeping {
    directory: Vec<usize>,
    allocation: Vec<usize>,
}

fn housekeeping(cfb: &[u8], label: &str) -> Housekeeping {
    let fat = fat_sectors(cfb, label);
    let directory = chain(cfb, &fat, read_u32_at(cfb, 48), label);
    assert!(!directory.is_empty(), "{label}: 디렉터리 섹터가 없다");

    let mut allocation = fat.clone();
    let mini_fat_count = read_u32_at(cfb, 64) as usize;
    if mini_fat_count > 0 {
        allocation.extend(chain(cfb, &fat, read_u32_at(cfb, 60), label));
    }
    Housekeeping {
        directory,
        allocation,
    }
}

/// [#5967] **판정 대상 내용은 지금 라이터로도 재현된다.**
///
/// 컨테이너 살림(섹터 배치·색 플래그·미사용 슬롯 채움값)은 라이터 판마다 달라질 수 있다.
/// 달라져선 안 되는 것은 그 안에 담긴 스트림이다 — 한컴이 판정한 것은 그 내용이기 때문이다.
/// 대조군 9건(한컴 저작 CFB)까지 함께 보아, 원본 작성기가 누구든 성립함을 고정한다.
#[test]
fn judged_streams_survive_a_current_writer_repack() {
    let mut checked = 0usize;
    for (name, _role, path) in ledger_entries() {
        let nested = nested_cfb(&path);
        let before = all_ole_streams(&nested).unwrap_or_else(|| panic!("{name}: 열거 실패"));
        assert!(
            before
                .iter()
                .any(|(p, _)| p.trim_start_matches('/') == "OOXMLChartContents"),
            "{name}: OOXMLChartContents 가 없다 — 차트 판정 자산이 아니다"
        );

        let repacked = repack_with_current_writer(&nested, &name);
        let after = all_ole_streams(&repacked).unwrap_or_else(|| panic!("{name}: 재조립본 열거"));

        assert_eq!(after.len(), before.len(), "{name}: 스트림 수");
        for ((path_before, bytes_before), (path_after, bytes_after)) in before.iter().zip(&after) {
            assert_eq!(path_after, path_before, "{name}: 스트림 이름·순서");
            assert_eq!(
                bytes_after.len(),
                bytes_before.len(),
                "{name}: `{path_before}` 길이"
            );
            assert_eq!(bytes_after, bytes_before, "{name}: `{path_before}` 바이트");
        }
        assert_eq!(
            ole_root_clsid(&repacked),
            ole_root_clsid(&nested),
            "{name}: 루트 CLSID — 떨구면 한컴이 개체를 잃는다 (#4097)"
        );
        checked += 1;
    }
    assert_eq!(checked, ASSET_TOTAL, "판정 자산 38건");
}

/// [#5967] **라이터 드리프트는 컨테이너 살림 두 자리에만 갇힌다.**
///
/// 커밋 자산(생성 시점 라이터)과 그것을 지금 라이터로 다시 조립한 것을 바이트로 대조하고,
/// 차이를 두 버킷으로 **분류**한다.
///
/// - **디렉터리 색 플래그** — 디렉터리 섹터 안 엔트리 `+67`, `01`(black) → `00`(red).
///   `color_deepest_nodes` 가 최심 노드만 red 로 칠하면서 생긴다.
/// - **할당표 미사용 슬롯** — FAT·MiniFAT 섹터 안 4바이트 슬롯, `0x00000000` → FREESECT.
///   값 `0` 은 "다음 섹터가 0번" 이라는 뜻인데 섹터 0 은 언제나 사슬 머리라 후속이 될 수
///   없다. 그러므로 `0` 은 미할당 슬롯을 유일하게 지목한다.
///
/// 두 버킷 밖 오프셋이 하나라도 나오면 실패한다 — 다음 라이터 변경이 판정 자산의 **내용**을
/// 건드리는 순간이 여기다. 차이가 0 이어도 실패한다: 지금 드리프트가 실재함을 이 테스트가
/// 스스로 증명하지 못하면 분류는 공허하다.
///
/// 대조군 9건은 한컴이 쓴 CFB 라 재조립이 섹터 배치부터 다르다. 여기서는 rhwp 라이터가 실제로
/// 쓴 29건(변종 28 + 변환본 1)만 본다.
#[test]
fn writer_drift_is_confined_to_directory_color_and_fat_fill() {
    let mut checked = 0usize;
    let mut color_total = 0usize;
    let mut fill_total = 0usize;

    for (name, role, path) in ledger_entries() {
        if role == "control" {
            continue;
        }
        let committed = nested_cfb(&path);
        let rebuilt = repack_with_current_writer(&committed, &name);
        assert_eq!(
            rebuilt.len(),
            committed.len(),
            "{name}: 재조립본 길이가 다르다 — 섹터 배치 자체가 바뀌었다"
        );

        let regions = housekeeping(&committed, &name);
        let mut color = 0usize;
        let mut fill = 0usize;
        let mut stray: Vec<usize> = Vec::new();

        for offset in 0..committed.len() {
            if committed[offset] == rebuilt[offset] {
                continue;
            }
            assert!(offset >= SECTOR_SIZE, "{name}: 헤더 +{offset} 가 바뀌었다");
            let sector = offset / SECTOR_SIZE - 1;
            let in_sector = offset % SECTOR_SIZE;

            let is_color = regions.directory.contains(&sector)
                && in_sector % DIR_ENTRY_SIZE == COLOR_FLAG_IN_ENTRY
                && committed[offset] == 1
                && rebuilt[offset] == 0;
            let slot = offset - offset % 4;
            let is_fill = regions.allocation.contains(&sector)
                && read_u32_at(&committed, slot) == 0
                && read_u32_at(&rebuilt, slot) == FREESECT;

            if is_color {
                color += 1;
            } else if is_fill {
                fill += 1;
            } else {
                stray.push(offset);
            }
        }

        assert!(
            stray.is_empty(),
            "{name}: 살림 두 자리 밖 차이 {}바이트 (처음 8개 오프셋 {:?}) — 판정 자산의 내용이 바뀌었을 수 있다",
            stray.len(),
            &stray[..stray.len().min(8)]
        );
        assert!(
            color + fill > 0,
            "{name}: 재조립본이 커밋본과 바이트 동일하다 — 드리프트가 사라졌다면 이 테스트와 \
             samples/issue5447/README.md 의 재생성 절차를 함께 되돌려야 한다"
        );

        color_total += color;
        fill_total += fill;
        checked += 1;
    }

    assert_eq!(checked, REWRITTEN_TOTAL, "rhwp 라이터가 쓴 자산 29건");
    println!("  드리프트 합계 — 색 플래그 {color_total}바이트, 할당표 채움 {fill_total}바이트");
}
