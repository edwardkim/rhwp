---
kind: pr-review
status: approved-pending-integration-ci
pr: 5604
author: planet6897
integration_pr: 5617
---

# PR #5604 검토: HWP3 SectionDef와 8유닛 슬롯을 좌표에 반영한다

원본: [#5604](https://github.com/edwardkim/rhwp/pull/5604)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `db80a116d8f6fb21605d1838c10ca7d551e1228f`  
관련 issue: #5542, #5532

## 검토 결과

**승인, 통합 PR CI 대기.** HWP3 구역 첫 문단에 SectionDef를 합성하고 `secd`/`cold`의 8유닛 슬롯을 좌표 계산에 계상한다. 구역 속성과 위치 단위를 함께 보존하는 parser 계약으로 범위를 고정했다.

## 검증 근거

- 누적 Rust integration: 7,782 passed
- HWP3 SectionDef와 좌표 단위 회귀 계약을 전체 integration 실행에 포함했다.

## 병합 후 처리

통합 PR #5617 병합 뒤 원본 PR #5604에 체리픽 통합 사실을 댓글로 남기고 close한다. issue #5542와 #5532는 통합 결과를 확인한 뒤 각각 해결 댓글과 함께 close한다.
