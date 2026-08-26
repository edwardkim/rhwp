//! [#3505] `convert --verify` 코퍼스 래칫 — 저장 손실이 새로 들어오는 것을 막는다.
//!
//! 이 세션에서 고친 저장 손실 두 건(#3492 개요번호 마커, #3524 책갈피)은 **둘 다
//! `convert --verify` 가 이미 종료코드로 판정할 수 있었는데** 아무도 돌리지 않아 남아
//! 있었다. 개별 결함을 더 찾는 것보다 이 계열이 다시 들어오지 못하게 막는 쪽이 낫다.
//!
//! ## 왜 지금 걸 수 있게 됐나
//!
//! 종전에는 샘플 346건 중 **22건**이 exit 3 였다. 그중 대부분은 손실이 아니라 **포맷 간
//! 비교가 무의미한 항목**이었다 — `diff_documents` 는 HWPX 왕복(같은 포맷) 가드로 만들어졌기
//! 때문이다. 그 항목들을 `strip_cross_format_noise` 로 걷어내고 빈 BinData placeholder 를
//! 세지 않게 하니 **6건**만 남았다. 그 6건은 아래에 근거와 함께 등재한다.
//!
//! ## 이 테스트가 하는 일
//!
//! 샘플 전수를 변환·재파싱해 `--verify` 와 같은 비교를 돌리고, **실패하는 문서 집합이
//! 등재 목록과 정확히 같은지** 본다.
//!
//! - 새 문서가 실패하면 → 저장 손실이 새로 들어왔다. 고치거나, 근거를 적어 등재하라.
//! - 등재된 문서가 통과하면 → 고쳐졌다. **목록에서 지워라** (래칫을 조인다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::FileFormat;
use rhwp::serializer::hwpx::roundtrip::{
    diff_documents, strip_cross_format_noise, strip_hwpx_to_hwp_noise,
};

/// 현재 저장 손실이 남아 있는 문서 — 각 항목은 **왜 아직 열려 있는지**를 밝힌다.
///
/// 고치면 여기서 지운다. 새로 추가할 때는 반드시 근거를 함께 적는다.
const EXPECTED_FAILURES: &[(&str, &str)] = &[
    // #3505 로 등재했던 `flowWithText` 축 4건은 #3834 수정으로 전부 통과한다 — 래칫을
    // 네 칸 조였다. 뒤집힘의 근인은 HWPX→HWP5 어댑터가 공통 속성 bit 13 을 무조건 OR 한
    // 것이었고, 한글 2022 는 같은 변환에서 원본 `0` 을 그대로 둔다(실측 13문서 표 84개).
    // issue1891_external_bindata_link.hwpx 는 #3528 수정으로 통과한다 — 래칫을 한 칸 조였다.
    // hwp3-sample10-hwpx.hwpx 는 제목 차례 표시(`<hp:titleMark/>`) 보존으로 통과한다 —
    // 래칫을 한 칸 더 조였다. 등재 당시 "lineseg 를 어긋나게 만든 상류를 고쳐야 한다"고
    // 적어 둔 그 상류가 이것이었다: 파서가 표시를 흘려 문단 축이 표시 하나당 8유닛씩
    // 짧아졌고, 그래서 둘째 lineseg 의 `text_start` 가 PARA_TEXT 끝을 넘어섰다
    // (문단 15487: char_count=37 인데 text_start=40 — 표시 하나를 되찾으면 45 가 된다).
    // 이 픽스처에는 표시가 290 개 들어 있다.
];

/// 게이트에서 제외하는 문서 — 크기에 비해 변환이 지나치게 오래 걸린다(각 2분+).
///
/// 제외는 **결함이 없다는 뜻이 아니다.** 별도 스윕으로 확인한다.
const TOO_SLOW: &[&str] = &[
    "[2027] 온새미로 1 본교재.hwp",
    "[2027] 온새미로 1 본교재.hwpx",
];

/// 크기 상한 — 이보다 큰 문서는 게이트에서 제외한다(등재 문서는 예외로 항상 검사).
///
/// 코퍼스 346건 중 287건이 이 상한 안이고 바이트로는 37MB / 280MB 다. 비용은 대형
/// 50여 건이 지배하므로, 상한을 두면 **문서 수 83%를 유지하면서 실행 시간을 크게 줄인다.**
/// 상한을 넘는 문서의 저장 손실은 이 게이트가 잡지 못한다 — 별도 스윕 대상이다.
const SIZE_CAP_BYTES: u64 = 1024 * 1024;

/// 코퍼스를 나눌 조각 수 — nextest 가 조각마다 프로세스를 병렬로 띄운다.
const PARTITIONS: usize = 4;

fn samples_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples")
}

/// 디렉터리 아래 `.hwp`/`.hwpx` 를 재귀로 모은다 (#4099).
///
/// 파일명이 조각 분배와 등재 목록의 키다. `samples/chart/**` 56건은 최상위 355건과
/// 이름이 겹치지 않고 자기들끼리도 유일함을 확인했다.
fn collect_recursive(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, out);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("hwp") | Some("hwpx")
        ) {
            out.push(path);
        }
    }
}

/// `convert --verify` 와 같은 판정: 변환 → 재파싱 → IR 비교.
///
/// 비교 강도는 CLI 와 같다 — 포맷을 넘으면 대상 포맷에 자리가 없는 항목을 걷어낸다.
fn verify_roundtrip(data: &[u8]) -> Option<usize> {
    let source_format = rhwp::parser::detect_format(data);
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(data).ok()?;
    doc.convert_to_editable_native().ok()?;
    let bytes = doc.export_hwp_with_adapter().ok()?;
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).ok()?;

    let diff = diff_documents(doc.document(), reloaded.document());
    let diff = match source_format {
        FileFormat::Hwp => diff,
        FileFormat::Hwpx => strip_hwpx_to_hwp_noise(diff),
        _ => strip_cross_format_noise(diff),
    };
    Some(diff.differences.len())
}

/// 코퍼스를 `PARTITIONS` 조각으로 나눠 검사한다.
///
/// 전수를 한 테스트로 돌리면 415초가 걸려 CI 샤드 하나가 그만큼 길어진다. nextest 는
/// `#[test]` 단위로 프로세스를 병렬 스케줄하므로, 조각으로 나누면 **같은 커버리지에
/// 벽시계만 1/N** 이 된다. 나눔 기준은 정렬된 파일 목록의 인덱스라 결정적이다.
fn run_partition(part: usize) {
    let expected: std::collections::BTreeMap<&str, &str> =
        EXPECTED_FAILURES.iter().copied().collect();
    let skip: std::collections::BTreeSet<&str> = TOO_SLOW.iter().copied().collect();

    let mut failed: std::collections::BTreeMap<String, usize> = Default::default();
    let mut checked = 0usize;
    let mut unreadable = 0usize;
    let mut oversized = 0usize;
    let mut seen: std::collections::BTreeSet<String> = Default::default();

    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(samples_dir())
        .expect("samples 디렉터리")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("hwp") | Some("hwpx")
            )
        })
        .collect();
    // [#4099] 차트 코퍼스를 재귀로 합친다.
    //
    // 위 수집은 비재귀라 하위 디렉터리가 통째로 빠진다(디렉터리는 확장자가 없어
    // 필터에서 탈락한다). 그래서 `samples/chart/**` 28종 × 2포맷이 이 게이트 밖에
    // 있었고, HWPX→HWP5 변환이 차트 참조를 끊는 결함(#4099)이 `--verify` 로 처음부터
    // exit 3 를 내고 있었는데도 아무도 보지 못했다.
    //
    // `samples/` 전체를 재귀로 열지 않고 `chart` 만 합치는 이유는 범위 통제다 —
    // 나머지 하위 디렉터리 68개는 이 이슈가 다루는 축이 아니고, 한꺼번에 넣으면
    // 무관한 신규 실패를 이 PR 에서 등재해야 한다. 필요하면 별도 이슈로 넓힌다.
    collect_recursive(&samples_dir().join("chart"), &mut entries);
    // 크기 내림차순으로 정렬한 뒤 인덱스로 나눈다 — 큰 문서가 한 조각에 몰리면 그 조각이
    // 전체 벽시계를 지배한다(실측: 이름순 분배 시 42/50/134/191초로 4.5배 편차).
    // 이름을 2차 키로 둬서 크기가 같아도 순서가 결정적이다.
    entries.sort_by_key(|p| {
        let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        (std::cmp::Reverse(size), p.clone())
    });

    for (index, path) in entries.into_iter().enumerate() {
        if index % PARTITIONS != part {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if skip.contains(name.as_str()) {
            continue;
        }
        // 등재된 문서는 크기와 무관하게 항상 검사한다 — 그래야 고쳐졌을 때 래칫을 조일 수
        // 있고, 다시 나빠졌을 때도 잡힌다.
        let pinned_doc = expected.contains_key(name.as_str());
        if !pinned_doc {
            let too_big = std::fs::metadata(&path)
                .map(|m| m.len() > SIZE_CAP_BYTES)
                .unwrap_or(false);
            if too_big {
                oversized += 1;
                continue;
            }
        }
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        // 암호 문서 등 열 수 없는 표본은 이 게이트의 대상이 아니다.
        let Some(count) = verify_roundtrip(&data) else {
            unreadable += 1;
            continue;
        };
        checked += 1;
        seen.insert(name.clone());
        if count > 0 {
            failed.insert(name, count);
        }
    }

    assert!(
        checked > 40,
        "조각 {part} 이 검사한 문서가 너무 적다({checked}건) — 표본 수집이 깨졌는지 확인하라"
    );

    let now: std::collections::BTreeSet<&str> = failed.keys().map(|s| s.as_str()).collect();
    // 이 조각이 실제로 검사한 문서만 비교 대상이다 — 다른 조각의 등재 문서를 여기서
    // "고쳐졌다" 로 오판하면 안 된다.
    let pinned: std::collections::BTreeSet<&str> = expected
        .keys()
        .copied()
        .filter(|n| seen.contains(*n))
        .collect();

    let regressions: Vec<String> = now
        .difference(&pinned)
        .map(|n| format!("  + {n} (차이 {}건)", failed[*n]))
        .collect();
    let fixed: Vec<String> = pinned
        .difference(&now)
        .map(|n| format!("  - {n}"))
        .collect();

    assert!(
        regressions.is_empty(),
        "저장 손실이 새로 들어왔다 ({}건):\n{}\n\
         고치거나, 근거를 적어 EXPECTED_FAILURES 에 등재하라.\n\
         상세: rhwp convert <파일> /tmp/out.hwp --verify\n\
         (검사 {checked}건 / 열 수 없음 {unreadable}건 / 크기 상한 초과 {oversized}건)",
        regressions.len(),
        regressions.join("\n")
    );

    assert!(
        fixed.is_empty(),
        "등재된 문서가 이제 통과한다 ({}건):\n{}\n\
         EXPECTED_FAILURES 에서 지워 래칫을 조여라.",
        fixed.len(),
        fixed.join("\n")
    );
}

#[test]
fn ratchet_partition_0() {
    run_partition(0);
}

#[test]
fn ratchet_partition_1() {
    run_partition(1);
}

#[test]
fn ratchet_partition_2() {
    run_partition(2);
}

#[test]
fn ratchet_partition_3() {
    run_partition(3);
}
