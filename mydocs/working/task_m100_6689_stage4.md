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

## 5. Stage 4-B — canonical release gate

계약 테스를 먼저 추가했을 때 collector, CI job, aggregate 연결, 운영 문서가 없어
예상한 RED가 나왔다. collector·gate를 구현한 뒤 focused suite는 40건 중 39건이 통과했고,
남은 RED 1건은 두 정본 문서의 운영 경계 누락이었다. 문서를 연결한 뒤 40건이
모두 GREEN으로 전환됐다.

`scripts/workflow_promotion_evidence.py`는 GitHub REST API를 read-only로 사용한다. run·job·artifact의
pagination을 상한 내에서 전건 확인하고, exact candidate SHA와 workflow content hash를 함께
고정한다. Oracle verdict ZIP은 압축을 풀어 파일시스템에 쓰지 않고 크기·파일 수·digest를
검증한 뒤 필요한 JSON만 읽는다. trusted maintainer comment의 waiver도 API가 반환한
작성자·URL을 사용하며, 본문이 이 필드를 위조해도 신뢰하지 않는다.

Stage 3 candidate `e33792ce9e2aaa1f959a362249c67dbd90120107`을 대상으로 collector를
실제 조회한 결과 9개 run에서 8개 workflow path를 모두 수집했고 pagination은 완결됐다.
offline verifier는 기존 Stage 3 수작업 판정과 동일하게 7개를 수락하고, CI run 실패와
`Build & Test` 실패 두 건만 거부했다. 임시 API snapshot·token·artifact는 저장소에 남기지 않았다.

이 과정에서 waiver가 permission·secret·security·deployment 위험 표면까지 가릴 수 있는
틈을 발견했다. 검증기가 이 표면의 waiver를 거부하도록 수정했고, 삭제된 workflow는
존재하지 않는 after hash가 아니라 before hash에 예외를 귀속시켰다. 두 경계를 회귀 테스로
고정했다.

CI의 `Workflow promotion preflight`는 same-repository `devel -> main` PR에서만 실행된다.
exact head checkout, `main` ancestor, merge tree 동일성을 확인하고 read-only 증적을 수집한 뒤
offline verifier를 실행한다. 성공·실패 모두 `workflow-promotion-evidence-<run-id>`에
30일간 증적을 남기고, `Build & Test`가 이 job을 의존하므로 fast-pass가 거부를
건너뛸지 못한다. 일반 PR·fork·push에서는 promotion job이 skip되어야 하며 aggregate가
그 skip을 정상 경계로만 수락한다.

`github_operations.md`에는 실행·waiver·stale head·main drift 복구를, `publish_guide.md`에는
릴리스 checklist와 #6634 publish chaining의 독립 판정을 연결했다. Stage 4의 남은
작업은 로컬 정적 검증과 최종 candidate exact-head 실증이다.
운영 문서에는 현재 8개 workflow의 실행 명령·mode·input을 고정했다. dispatch ref가
commit SHA를 직접 받지 않는 한계도 명시하고, 실제 run의 `head_sha`를 dispatch 직전에 기록한
candidate와 다시 대조하도록 했다.

## 6. Stage 4-B 로컬 검증

| 검사 | 결과 |
| --- | --- |
| collector·gate·wiring·preflight focused Python | 40건 통과 |
| CI `Validate workflow contracts` 동일 명령 묶음 | Node 18건 + Python 178건, 총 196건 통과 |
| Python `py_compile` | collector·verifier 통과 |
| `ci.yml` YAML parse | 통과 |
| actionlint | v1.7.12 통과 |
| 변경한 정본 문서 상대 링크 | 2개 모두 통과 |
| `git diff --check` | 통과 |

전체 문서 metadata 검사는 이 절편이 바꾸지 않은 기존 문서 4개의 front matter 누락 16건을
계속 보고했다. 변경한 `github_operations.md`와 `publish_guide.md`의 `kind`, `status`,
`canonical`, `last_verified`는 유효하다. 관련 없는 기존 문서를 이 운영 절편에 섞어 수정하지 않았다.

로컬 정적 검증은 완료됐다. 최종 exact-head 여덟 workflow 재실행은 현재 변경을 commit·push해
새 candidate SHA를 고정한 뒤에만 의미가 있으므로 다음 승인 단위로 남겨 둔다.
