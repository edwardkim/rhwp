# Task M100 Kevin9327 Stage 3 - PR 파생 suite 산출물 정책 이행

## 목적

누적 체리픽 후보를 #5177의 원본-only PR 정책에 맞춘다. 기여 변경은 `tests/cases/**`에 남기고,
generated harness와 manifest는 CI checkout에서만 생성한다.

## 조치

- 최신 `upstream/devel` 위로 기존 체리픽 후보를 재배치한다.
- 후보와 base의 `tests/generated/**`, `tests/suites/manifest.json` 차이를 역적용해 PR 최종 diff에서 제거한다.
- Cargo generated test target block은 base와 동일하게 유지한다.

## 기대 결과

각 원 PR은 원본 regression case만 병합 후보에 남고, 누적 PR은 #5177의 base-diff 게이트를 통과한다.
파생 suite 재생성은 PR review와 CI archive checkout에서 한 번만 수행한다.
