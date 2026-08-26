//! 차이 목록 → 카테고리별 건수 집계.
//!
//! 집계는 [`FindingKind::ALL`] 을 순서대로 훑는다. `HashMap` 을 돌지 않는 것이
//! 핵심이다 — 해시 순회 순서는 실행마다 달라질 수 있고, 그러면 "같은 두 문서면
//! 같은 결과"라는 약속이 요약 계층에서 깨진다.

use super::model::{DiffSummary, DocumentDiff, FindingKind};

/// 차이 목록을 카테고리별로 센다.
///
/// 상한(`max_findings`)에 걸려 버린 차이는 세지 않는다 — 집계는 **보고된 것**의
/// 회계이고, 더 있었다는 사실은 [`DiffSummary::truncated`] 가 말한다.
pub fn summarize(diff: &DocumentDiff) -> DiffSummary {
    let mut by_kind = Vec::new();
    for kind in FindingKind::ALL {
        let count = diff.findings.iter().filter(|f| f.kind == kind).count();
        if count > 0 {
            by_kind.push((kind, count));
        }
    }
    DiffSummary {
        total: diff.findings.len(),
        truncated: diff.truncated,
        by_kind,
    }
}
