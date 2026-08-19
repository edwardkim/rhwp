---
kind: pr-review
status: approved-pending-integration-ci
pr: 5591
author: planet6897
integration_pr: 5617
---

# PR #5591 검토: 어울림(Square) 표의 문단 기준 세로 오프셋을 렌더 y에 반영한다

원본: [#5591](https://github.com/edwardkim/rhwp/pull/5591)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `f98e743876521366c3ae7109ed4e7b3e083009d2`  
관련 issue: #5566

## 검토 결과

**승인, 통합 PR CI 대기.** Square 표의 직렬화된 문단 기준 세로 오프셋을 렌더 위치에 합산해, 표를 문단 위치와 같은 기준으로 배치한다. 기존 geometry 계약이 오프셋 반영을 검증한다.

## 검증 근거

- 누적 Rust integration: 7,782 passed
- `samples/hwp_table_test-m.hwp` 첫 페이지를 native-Skia PNG로 내보내 셀 경계와 세로 배치를 확인했다.
- native-Skia lib: 58 passed

## 병합 후 처리

통합 PR #5617 병합 뒤 원본 PR #5591에 체리픽 통합 사실을 댓글로 남기고 close한다. issue #5566은 통합 결과와 CI를 확인한 뒤 해결 댓글과 함께 close한다.
