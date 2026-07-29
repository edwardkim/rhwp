# PR #3235 검토 — 문서-only Update branch merge fast-pass 재사용

| 항목 | 내용 |
| --- | --- |
| PR | [#3235](https://github.com/edwardkim/rhwp/pull/3235) |
| 관련 이슈 | [#3233](https://github.com/edwardkim/rhwp/issues/3233) |
| base / 최초 검증 head | `devel` `0f0d92e687b1a0006e4d33c3892f883fb718f810` / `2a41243a034cd58701c4058d865adc7286b3383d` |
| 최초 변경 규모 | 7 files, +135 -20 |
| 검토 경로 | maintainer 후속 기록 fast-pass 검증 PR |

## 배경과 변경 범위

- #3123에서 `Update branch`가 만든 문서-only merge 뒤에 오늘할일·검토 기록만 추가했을 때, 기존 preflight가 merge commit을 일괄 거절해 전체 CI를 다시 실행한 사례를 #3233으로 기록했다.
- CI, CodeQL, Render Diff의 review-only 탐색은 trailing single-parent 문서 커밋을 건너뛰되, 그 직전 candidate가 **현재 PR base를 부모로 갖는 정확히 2-parent 문서-only Update branch merge**일 때만 기존 검증 결과를 재사용한다.
- merge diff가 허용된 review 경로만 바꾸는지, trailing 문서 커밋이 실제로 하나 이상 있는지까지 확인한다. 어느 하나라도 불일치하면 기존처럼 full CI로 안전하게 되돌린다.
- `pr_review_workflow.md` 9.3.2절에 같은 guard와 fallback 조건을 명시했다. 코드·테스트·workflow 변경, base 불일치, 3개 이상 parent merge, 허용 경로 밖 변경은 fast-pass 대상이 아니다.

## 검증 근거

- `actionlint`로 CI·CodeQL workflow를, Ruby YAML 파서로 세 workflow를 확인했다. Render Diff의 actionlint SC2086은 변경하지 않은 기존 shell 구문 경고다.
- 실제 #3123 SHA를 사용한 Node 회귀 검사로, `a4a3826` 문서 커밋과 `f8eebc4`의 2-parent/base 일치 merge가 수용되고, base 불일치·trailing 문서 커밋 부재·3-parent merge는 거절되는 것을 확인했다.
- 최초 code/workflow head `2a41243a`에서는 fast-pass를 적용하지 않고 전체 GitHub Actions를 수행했다. Build & Test(아카이브 및 8 shards), Lint, Native Skia tests, Frontend package gates, CodeQL 3개 분석, Canvas visual diff가 모두 성공했고 WASM Build는 조건에 따라 skipped였다.

## 후속 fast-pass 검증

- 이 문서와 오늘할일 기록만을 뒤따르는 single-parent 문서 커밋으로 push한다. 변경 경로는 `mydocs/orders/20260724.md`와 이 archive review 파일뿐이다.
- 문서 head `82844486b`에서 CI·CodeQL·Render Diff preflight가 모두 성공했고 Build & Test 집계도 성공했다. Lint, Native Skia tests, Frontend package gates, Build test archive, 8개 default-feature shard, CodeQL 분석, Canvas visual diff는 모두 `skipped`였다.
- 이 head는 `CLEAN`·`MERGEABLE` 상태가 됐다. 기대와 달리 full CI가 재실행되거나 preflight가 실패하지 않아, #3123에서 확인한 base 일치 문서-only Update branch merge 재사용 경로가 실제로 동작함을 확인했다.

## 최종 권고

- fast-pass 조건과 merge 가능 상태를 모두 충족했다. 최종 문서 보완 head도 같은 문서-only 경로로 확인한 뒤 #3235를 merge 후보로 권고한다.
