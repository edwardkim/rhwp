---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4246 검토 - 폰트 메트릭 조회 선계산 색인

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4246](https://github.com/edwardkim/rhwp/pull/4246) / @humdrum00001010 |
| contributor 원 head | `c5a246c438d6e6b04d3325dcc643c27fc4be792f` |
| base / 규모 | `devel`, 3개 파일, +317/-0 |
| 관련 이슈 | [#4168](https://github.com/edwardkim/rhwp/issues/4168) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

`find_metric`의 반복 선형 탐색을 `OnceLock` 기반 이름·굵기·기울임 색인으로 바꾼다. legacy 탐색은
테스트 전용 오라클로 남기며, 첫 매칭 우선순위와 bold fallback 의미를 바꾸지 않는다. 렌더 결과가 아니라
메트릭 조회 비용만 줄이는 변경이다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- 최신 `devel` `e919655a` 위 누적 통합에서 `cargo fmt --check`와
  `cargo test --profile release-test --tests`가 종료 코드 0으로 통과했다.
- renderer/layout 영향이므로 통합 PR #4265의 Full CI와 Native Skia 결과를 merge 전 최신 head에서 다시
  확인한다. 로컬 Native Skia 재실행은 이 문서 작성 시점에 아직 완료하지 않았다.

**통합 수용 권고.** 적용 순서는 #4246 → #4247 → #4248 → #4249이며, 이후 fast-path가 이 색인의
메트릭 측정 경로를 재사용한다.
