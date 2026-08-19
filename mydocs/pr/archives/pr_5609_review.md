---
kind: pr-review
status: approved-pending-integration-ci
pr: 5609
author: planet6897
integration_pr: 5617
---

# PR #5609 검토: 앞 run 소유 titleMark를 조각 말미에 방출한다

원본: [#5609](https://github.com/edwardkim/rhwp/pull/5609)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `a4d01b9a777dfba80f37f48cf061994bfc847302`  
관련 issue: #5537

## 검토 결과

**승인, maintainer 보정 반영, 통합 PR CI 대기.** 앞 run이 소유한 titleMark는 fragment 말미에 방출하고, 다음 run의 선두로 옮기지 않는다.

원본 PR의 source-side `#[cfg(test)]` 두 건은 현재 정책을 위반하므로, 구현 의미는 보존한 채 `tests/cases/issue_5537_titlemark_run_ownership.rs`의 integration 계약 두 건으로 이관했다. 이 보정은 `b250788c6`에 기록돼 있다.

## 검증 근거

- titleMark focused integration: 2 passed
- `node scripts/rust-unit-test-tiers.mjs --check`: source-side 4,225 tests / 298 modules, 통과
- 누적 Rust integration: 7,782 passed

## 병합 후 처리

통합 PR #5617 병합 뒤 원본 PR #5609와 issue #5537에 체리픽 통합·maintainer 보정·검증 결과를 댓글로 남기고 각각 close한다.
