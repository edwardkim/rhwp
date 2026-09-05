# #6689 Stage 4 — release gate·문서 연결

## 1. Stage 3 인계

exact candidate `e33792ce9e2aaa1f959a362249c67dbd90120107`에서 여덟 workflow를 실제 실행했다.
Adapter inter-diff, CodeQL, Pages verify-only, Gym contracts-only, Oracle advisory, Proptest roundtrip,
Render Diff는 모두 계약을 통과했다. CI의 제품·WASM·test archive·frontend·Skia 검증도 통과했지만,
`Validate workflow contracts`가 새 promotion 계약 테스트의 CI 미배선을 정확히 검출했다.

offline verifier의 거부 사유는 CI run과 집계 `Build & Test`가 녹색이 아니라는 두 항목뿐이다. 둘은
`test_workflow_promotion_preflight.py` 미배선이라는 하나의 원인에서 파생됐다. 따라서 Stage 4는 이 RED를
우회하거나 waiver하지 않고 CI의 canonical 계약 목록에 연결하는 것으로 시작한다.

## 2. Stage 4-A — promotion 계약 테스트 최소 배선

`.github/workflows/ci.yml`의 `Validate workflow contracts` 단계에 다음 실행을 추가한다.

```text
python3 -m unittest scripts/tests/test_workflow_promotion_preflight.py
```

이 절편은 기존 Lint job의 조건·권한·timeout, 다른 job의 dependency, workflow trigger를 바꾸지 않는다.
`test_workflow_contract_wiring.py`가 새 계약 테스트의 존재와 Lint job 내부 배선을 모두 강제하므로, 이후 같은
유형의 누락은 CI에서 fail-closed 된다.

검증 순서는 promotion·wiring focused test, CI workflow YAML parse, actionlint, `git diff --check`다. 로컬
검증 뒤 새 exact candidate를 commit·push하고 CI를 재실행하는 작업은 각각 다음 승인 단위로 분리한다.

## 3. Stage 4-A 로컬 결과

| 검사 | 결과 |
| --- | --- |
| promotion·wiring·Oracle·Gym focused Python | 40건 통과 |
| `ci.yml` YAML parse | 통과 |
| actionlint | v1.7.12 통과 |
| `git diff --check` | 통과 |

이전 exact-head CI에서 실패한 `test_workflow_contract_wiring.py`의 두 assertion은 새 실행 줄을 Lint job
내부에서 발견해 GREEN으로 전환됐다. Rust source와 test source는 바뀌지 않았고, 이 절편의 실행 의미 변경은
기존 promotion 계약 테스트를 CI에서 실제로 실행하게 만든 것뿐이다.

## 4. Stage 4-A exact-head CI 실증

commit `cd4f6cea286b8f8bc03615fc3c6cfd31cd2e3350`을 원격 `task_m100_6689`에 push하고
`release_grade=false`로 CI를 수동 실행했다.

| 항목 | 결과 |
| --- | --- |
| run | `33954443178` |
| event / branch | `workflow_dispatch` / `task_m100_6689` |
| head SHA | `cd4f6cea286b8f8bc03615fc3c6cfd31cd2e3350` |
| run conclusion | success |
| `CI preflight` | success |
| `Lint (fmt, clippy, WASM check)` | success |
| `Build & Test` | success |

이전 run `33952874349`에서 실패한 `Validate workflow contracts`는 통과했고 이후 native·WASM·workspace
Clippy도 모두 성공했다. WASM Build, Frontend package gates, Native Skia tests, 네 개 test archive의
build·실행도 전부 성공했다. `Frontend unit gates`와 nextest duration refresh의 skip은 입력과 갱신 정책에
따른 정상 상태다.

이 결과로 Stage 4-A의 RED→GREEN 전환은 완료됐다. 다만 Stage 4의 최종 promotion 판정은 release gate와
운영 문서 연결까지 구현한 최종 candidate에서 여덟 workflow exact-head 증적을 다시 수집한 뒤 수행한다.
중간 절편마다 나머지 일곱 workflow를 반복 실행해 runner를 낭비하지 않는다.
