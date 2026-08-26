---
kind: pr-review
status: self-review-ci-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5930 self-review — W5 Oracle Profile·controlled ladder

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `67567931369b0317339c6dc5534e39f0ce91c570`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#5930](https://github.com/edwardkim/rhwp/pull/5930) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4963](https://github.com/edwardkim/rhwp/issues/4963), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4963` |
| 게시 code head | `384a6712190d06e2dd36229bedabe273708e37b8` |
| self-review code candidate | `67567931369b0317339c6dc5534e39f0ce91c570` — 원격 push 전 |
| 규모 | 76 files, +18,056 / -0 — 보정·review tail 전 참고값 |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=BLOCKED` — 작성 시점 참고값 |

1,000줄을 넘지만 제품 구현을 한 덩어리로 추가한 변경은 아니다. schema·negative fixture·17개 readiness와
disposition·Oracle profile·실행/복구 기록처럼 기계 검증 가능한 증적이 대부분이며, 계약→분석기→관찰→
controlled ladder→재현 canary가 12개 단계 커밋으로 분리돼 있다. 즉시 admin merge하지 않고 self-review,
최신-base 정합성, 최신 head Full CI와 메인테이너 merge 승인을 독립 gate로 유지한다.

별도 `pr_5930_review_impl.md`는 만들지 않는다. 외부 PR 보정이나 복수 PR 누적이 아니며, self-review에서
발견한 단일 privacy 계약 오류는 code candidate commit으로 분리해 수정했다.

## 변경 범위와 목적 정합성

#4963의 목적은 제품 fallback을 곧바로 바꾸는 것이 아니라 W4 상위 17개 face의 exact/substitution/missing
상태를 통제하고 selection→glyph→advance→line→page의 최초 차이를 재현 가능한 Oracle evidence로 만드는
것이다.

- 공개 deterministic HWPX fixture, profile schema와 negative mutation을 추가한다.
- SFNT `hmtx`와 PDF observed advance를 서로 다른 envelope로 보존한다.
- exact, alias, official successor, document substitution, metric surrogate와 Hancom missing-font를 합치지
  않는다.
- source 부재·보호된 system/HFT provider·document face mismatch를 값이 없는 정상 terminal disposition으로
  기록하며 추정값을 만들지 않는다.
- rank 1·7·8 acceptance ladder와 rank 16 capability mismatch를 고정해 17개 queue의 actionable rank를
  0개로 정리한다.
- 외부 Hyper-V controller가 Standard checkpoint restore 전후, managed font set, interactive HWP task와
  recovered manifest를 강제하고 새 환경의 path-free reproduction summary를 만든다.
- `src/**`, Cargo manifest/lock, 제품 metric DB·fallback·paint·layout은 변경하지 않는다.

## code·계약 self-review

### 발견 및 보정

공개 `oracle_profile_contract.json`은 `absoluteFontPathsPublished=false`를 선언하면서 같은 privacy 객체에
메인테이너의 실제 local font root 절대 경로를 넣고 있었다. 실행기는 이미 font root를 runtime 인자로
받으므로 이 값은 기능에 필요하지 않고 제3자 재현성과 privacy 주장에 모순됐다.

`675679313`에서 이를 `fontRootProvisioning=runtime-argument-outside-repository` 계약으로 바꾸고 validator와
회귀 검사를 함께 수정했다. 외부 `ttfs` font 보관과 SHA-256 identity 정책은 그대로 유지하며 공개 계약에는
실제 local root를 넣지 않는다. 기존 원격 head의 GitHub CI는 이 code correction을 포함하지 않으므로
merge 근거로 재사용하지 않고, review tail과 함께 push될 최신 head에서 Full CI를 요구한다.

### 보호 불변식 확인

- 모든 Python input/output은 declared root 아래 regular file만 허용하고 symlink·traversal·oversize를
  fail-closed로 거부한다.
- PDF는 byte/page/object/glyph/tool-output 상한과 child timeout을 갖고 malformed item을 batch 전체
  무한 대기로 확장하지 않는다.
- Hyper-V controller는 raw VM/checkpoint identity, Standard checkpoint, automatic checkpoint 비활성,
  explicit restore approval과 `ShouldProcess`를 검사한다.
- managed font 제거 API는 제공하지 않고, 상태 제거는 외부 checkpoint restore만 사용한다. 실패 경로도
  `finally` 복원과 recovered manifest 확인을 우선한다.
- public investigation JSON에는 실제 absolute path·raw GUID·credential·font/PDF bytes가 없다. generic
  guest example path는 재현 가이드 안의 교체 가능한 recipe일 뿐 실행 identity가 아니다.
- private 10k corpus 원문·파일명·document hash·개별 결과를 사용하거나 공개하지 않는다.

잔여 차단 결함은 발견하지 못했다.

## 관측·시각 증적 판정

rank 8 exact-only는 `KoPubWorldBatangLight`, substitution-only와 none-related는
`HCRBatang-Bold`를 사용했다. exact projection `38f83a79…b4c7`은 none과 다르고, substitution/none은
`59801255…27be`로 같았다. 공개 Hyper-V 재현 canary는 기존 acceptance ladder의 세 projection을 모두
정확히 재현했고 각 상태 뒤 baseline manifest를 복구했다.

메인테이너는 exact/substitution/none side-by-side를 확인해 기계 projection과 같은 외형 관계임을 승인했다.
비교 이미지 SHA-256은
`9d4da59dfaba6f4dcb0fd06e1268fcb490690c66998bd0574df607f376bb90cb`이며 원시 PDF·manifest와 함께
owner-only local evidence로 유지한다. 이 이미지는 제품 renderer 변경의 golden이나 visual sweep
정본이 아니라 외부 한컴 Oracle 관측의 보조 증거다. PR은 제품 렌더 출력을 바꾸지 않으며, 공개 재현은
fixture·controller·path-free observation/projection으로 수행하므로 별도 PR asset으로 승격하지 않는다.

## 완료한 로컬 검증

최신 `upstream/devel@2078a2629c0d1cfd85437383cbbfb7b72418fe7b`을 정상 merge한 후보와 self-review
보정 뒤 다음을 완료했다.

| 검증 | 결과 |
| --- | --- |
| Stage 2·3·4·4 profile·5 queue Python tests | 42/42 통과 |
| Oracle Node contract tests | 13/13 통과 |
| Node executable contract·Stage 4 contract check | 각각 통과 |
| 신규 Oracle Python AST / 공개 JSON parse | 16개 / 35개 통과 |
| deterministic HWPX ZIP | CRC 정상, 첫 entry `mimetype`, embedded font 0 |
| Windows PowerShell AST parse | 6/6 통과 |
| investigation의 배포 금지 font/PDF bytes | 0개 |
| 변경 Markdown 내부 상대 링크 | 76파일 범위, 이상 없음 |
| `cargo fmt --all`·`cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

Rust 제품 source·Cargo·integration test source 변경이 0개이므로 release-test nextest, clippy, WASM과
Native Skia 전체 실행은 이 PR의 로컬 변경 범위 게이트가 아니다. 최신 PR head의 GitHub 영향 분류와
required aggregate는 별도로 모두 확인한다.

## GitHub Actions와 남은 조건

게시 head `384a67121`의 GitHub Actions는 self-review 작성 중 실행되고 있었으나, 그 뒤 privacy contract와
validator/test를 고친 code candidate `675679313`이 생겼다. 따라서 게시 head의 성공 여부와 관계없이
stale 결과로 취급한다. code candidate와 이 review·오늘할일 trailing commit을 push한 최신 head는 source와
test 변경을 포함하므로 review-only fast-pass로 우회하지 않고 Full CI를 통과해야 한다.

최종 merge 조건은 다음과 같다.

1. 최신 remote head가 local trailing head와 같다.
2. 최신 head의 CI, CodeQL, Proptest, Adapter와 영향 범위 required check가 모두 success 또는 정책상
   정상 skip이다.
3. `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`을 merge 직전에 다시 확인한다.
4. 작업지시자가 별도로 merge를 승인한다.

## 권고

self-review에서 발견한 공개 절대 경로 계약 모순은 code candidate에서 보정했고, 로컬 재검증에서 잔여
blocker를 발견하지 못했다. 따라서 최신 trailing head의 Full CI와 mergeability를 조건으로 **merge를
권고**한다. 현재는 review 기록 push와 최신 CI 전이므로 merge 보류다.
