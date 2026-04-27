//! Issue #353: NewNumber 컨트롤이 모든 후속 페이지에 매번 재적용되어
//! page_number 가 1, 2, 3, 1, 1, 1, ... 식으로 고정되는 회귀 버그.
//!
//! 정상 동작: NewNumber 가 그 컨트롤의 소유 문단이 처음 등장하는 페이지에서
//! 1회만 적용되고, 이후 페이지는 +1 로 단조 증가해야 한다.

use std::fs;
use std::path::Path;

/// dump_page_items 출력에서 페이지별 page_num 시퀀스를 추출한다.
fn extract_page_numbers(dump: &str) -> Vec<u32> {
    let mut nums = Vec::new();
    for line in dump.lines() {
        if let Some(idx) = line.find("page_num=") {
            let rest = &line[idx + "page_num=".len()..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            if let Ok(n) = rest[..end].parse::<u32>() {
                nums.push(n);
            }
        }
    }
    nums
}

/// `samples/2022년 국립국어원 업무계획.hwp` :
/// - NewNumber Page=1 컨트롤이 1개 (표지 다음 본문 첫 페이지에서 트리거)
/// - 트리거 이후의 모든 페이지는 +1 로 단조 증가해야 한다.
///
/// 버그: 트리거 이후의 페이지가 모두 1 로 고정됨.
#[test]
fn gugeo_업무계획_post_new_number_monotonic() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/2022년 국립국어원 업무계획.hwp");
    let Ok(bytes) = fs::read(&hwp_path) else {
        eprintln!("skip: sample not available at {}", hwp_path.display());
        return;
    };

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 2022년 국립국어원 업무계획.hwp");

    let dump = doc.dump_page_items(None);
    let nums = extract_page_numbers(&dump);

    assert!(!nums.is_empty(), "page_num 추출 실패 — dump 형식 변경 가능성");

    // NewNumber 트리거 위치 찾기 (직전 page_num > 현재 page_num 인 첫 지점)
    let trigger_idx = nums.windows(2).position(|w| w[0] >= w[1])
        .map(|i| i + 1)
        .unwrap_or(0);

    // 트리거 이후 단조 증가 검증
    for (i, win) in nums[trigger_idx..].windows(2).enumerate() {
        let abs_idx = trigger_idx + i;
        assert_eq!(
            win[1], win[0] + 1,
            "페이지 {} 의 page_num={} 가 직전 page_num={} +1 이 아님 (NewNumber 트리거 idx={}). 전체 시퀀스: {:?}",
            abs_idx + 2, win[1], win[0], trigger_idx, nums,
        );
    }

    // 트리거 이후 페이지 수가 적어도 (전체-3) 이상 (표지/목차/표지표 등 앞부분 제외)
    let post_count = nums.len() - trigger_idx;
    assert!(
        post_count >= nums.len().saturating_sub(3),
        "트리거 이후 페이지({}/{})가 너무 적음 — 트리거 위치가 비정상.",
        post_count, nums.len(),
    );
}

/// 트리거 이후 page_num 의 최댓값이 PDF 의 마지막 푸터 번호(35)와 근접해야 한다.
/// (정확히 35 가 아니더라도 33 이상이면 합격 — 별첨 등 후행 구역 영향 허용)
#[test]
fn gugeo_업무계획_max_page_number_close_to_count() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/2022년 국립국어원 업무계획.hwp");
    let Ok(bytes) = fs::read(&hwp_path) else {
        eprintln!("skip: sample not available at {}", hwp_path.display());
        return;
    };
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");

    let total_pages = doc.page_count();
    let dump = doc.dump_page_items(None);
    let nums = extract_page_numbers(&dump);

    assert_eq!(
        nums.len() as u32,
        total_pages,
        "추출된 page_num 개수({})와 page_count({}) 불일치",
        nums.len(),
        total_pages,
    );

    let max = *nums.iter().max().unwrap();
    let expected_min = total_pages.saturating_sub(3);
    assert!(
        max >= expected_min,
        "page_num 최댓값({})이 (페이지 수-3)({})보다 작음 — NewNumber 가 매 페이지 재적용되는 회귀. 시퀀스: {:?}",
        max, expected_min, nums,
    );
}
