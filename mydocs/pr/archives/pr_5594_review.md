---
kind: pr-review
status: approved-pending-integration-ci
pr: 5594
author: planet6897
integration_pr: 5617
---

# PR #5594 검토: 평문 HWPX 문단 여백 표기를 보존한다

원본: [#5594](https://github.com/edwardkim/rhwp/pull/5594)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `4ea70b285c5d780e5d218f58aeca58564ffbb104`  
관련 issue: #4898

## 검토 결과

**승인, 통합 PR CI 대기.** HWPX 문단 여백이 switch 표기인지 평문 표기인지 구분해 원래 표기 체계를 유지한다. 이 변경은 round-trip에서 의도하지 않은 여백 축소와 쪽수 증가를 막는다.

## 검증 근거

- 누적 Rust integration: 7,782 passed
- serializer의 평문·switch 여백 회귀 계약을 전체 integration 실행에 포함했다.

## 병합 후 처리

통합 PR #5617 병합 뒤 원본 PR #5594와 issue #4898에 통합 및 검증 결과를 댓글로 남기고 각각 close한다.
