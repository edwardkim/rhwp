---
kind: pr-review
status: self-review-changes-requested
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5938 self-review — W6 font metric 계보 분리

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `2d15b50ea13cff679d2dae8151bd3721b95d49a1`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#5938](https://github.com/edwardkim/rhwp/pull/5938) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4964](https://github.com/edwardkim/rhwp/issues/4964), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4964` |
| 게시 code head | `2d15b50ea13cff679d2dae8151bd3721b95d49a1` |
| 규모 | 24 files, +115,529 / -46,200, 7 commits |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=BLOCKED` — 작성 시점 참고값 |

1,000줄을 크게 넘지만 65,881행 lineage manifest, 45,921행 기존 metric data 이동과 단계별 계획·보고가
대부분이다. runtime 정책 diff는 facade와 generator에 집중돼 있고 7개 commit이 기준선→manifest→물리
분리→generator→통합 검증→승인 기록 순으로 나뉜다. 대형 PR 규칙에 따라 즉시 admin merge하지 않고
code self-review, 최신-base simulation, Full CI와 메인테이너 merge 승인을 독립 gate로 유지한다.

별도 `pr_5938_review_impl.md`는 만들지 않는다. 외부 PR 보정이나 복수 PR 누적이 아니며, 아래 세 결함은
generator 출력 경계를 닫는 하나의 최소 정정 stage로 처리할 수 있다.

## 변경 범위와 목적 정합성

#4964의 목적은 metric 값을 고치는 것이 아니라 historical generated 595개, #2430 measured overlay 5개,
runtime lookup과 provenance의 소유권을 행동 변화 없이 분리하는 것이다.

- `font_metrics_data.rs`는 type·alias·lookup과 `generated → overlay` iterator만 소유한다.
- generated 595개와 measured 5개는 별도 source에 있고 정렬·dedupe·style 정규화를 하지 않는다.
- lineage manifest는 600개 안정 ID와 W1 600개·W5 2개 link를 가지며, 595개 legacy origin을 증거 없이
  source-exact로 승격하지 않는다.
- generator는 명시적 plan의 순서·face·identity와 provenance metadata를 검사하고 core·overlay ownership을
  침범하지 않아야 한다.
- 신규 font bytes, private corpus 식별 자료, metric 교정과 fallback 정책 변경은 범위 밖으로 유지한다.

## code self-review findings

### [P1] 실행 CWD가 canonical generated DB ownership 판정을 결정한다

`src/tools/font_metric_gen.rs:1093-1130`은 `env::current_dir()`를 repository root로 사용한다. 저장소
상위나 하위 디렉터리에서 실행하면 실제 `src/renderer/font_metrics_generated.rs`와 비교하는 canonical
경로가 달라진다. core·overlay는 파일명 차단도 있지만 generated DB에는 같은 보조 방어가 없으므로,
경로를 CWD 기준으로 다시 적은 canary plan이 canonical generated DB 보호 조건을 우회할 수 있다.

이는 W6-I12의 "불완전 plan은 canonical generated DB를 덮어쓰지 못한다"는 주장과 충돌하므로 merge
blocker다. 실행 위치와 무관한 checkout root를 하나만 결정하고 input·evidence·보호 출력 경로를 모두 그
root에서 해석해야 한다. checkout-relative 입력은 canonicalize 뒤 root 밖으로 나가는 symlink도 거부해야
한다. 저장소 밖 local font 검산은 plan generation이 아니라 기존 read-only diagnostic 경로로 분리한다.

### [P1] generated output이 provenance metadata보다 먼저 변경된다

`src/tools/font_metric_gen.rs:1209-1213`은 입력 파싱·직렬화 뒤 generated 파일을 먼저 쓰고 metadata를
두 번째로 쓴다. metadata 대상이 디렉터리이거나 I/O가 실패하면 명령은 오류를 반환하지만 generated
파일은 이미 바뀐다. canonical generated DB와 그 provenance가 한 실행 단위라는 계약을 만족하지 못한다.

두 산출물을 각 target의 sibling temporary file에 먼저 완성하고, metadata를 먼저 확정한 뒤 generated
data를 마지막 commit point로 교체해야 한다. 중간 실패에서는 temporary file을 정리하고 기존 generated
target hash가 유지되는 negative test가 필요하다.

### [P2] generator Node contract의 내부 Cargo build가 lockfile을 보호하지 않는다

`scripts/tests/font_metric_gen.test.mjs:42-47`은 `cargo build --bin font-metric-gen`을 `--locked` 없이
실행한다. Stage W6-5에서 이 test가 workspace package 두 항목의 `Cargo.lock` 순서를 실제로 바꿨고,
검증 뒤 수동으로 기준 bytes를 복원했다. 모든 검토 Cargo 명령에 `--locked`를 쓰는 프로젝트 규칙과
맞지 않는다.

build 인자에 `--locked`를 추가하고, 같은 Node contract 재실행 뒤 `Cargo.lock` blob이 바뀌지 않았음을
검사한다.

## 대량 data·manifest review

대량 행을 눈으로 전수 비교하는 대신 분리 전 기준선과 fail-closed projection을 사용했다.

- 600개, 401 unique name, style 383/89/79/49와 generated 595·overlay 5를 고정했다.
- 7,062,099 data-bearing entry-codepoint pair, lookup name/alias × 4 style를 전수 직렬화했다.
- composition `d4cdac86…e69a`, metric `025812ea…0bcf`, width `2cd1389a…7d1b`, lookup
  `bb3008f9…bfdf`가 분리 전후 동일했다.
- 순서 교환, 폭 1단위 변경, overlay identity 변경, evidence digest·W1 link 변경 negative test가
  각각 실패했다.
- #2430 5개 overlay는 측정 TSV와 475/475 exact였다.

따라서 대량 데이터 이동 자체에서 별도 수치·순서 회귀는 발견하지 않았다.

## 렌더·시각 증적 판정

renderer source 경계가 바뀌므로 `visual_fixture_evidence.md`를 적용했다. 다만 이 PR은 사용자-visible
개선을 주장하지 않고 출력 불변을 요구한다. 공개 W0 7문서의 native/WASM SVG 167쪽을 byte 단위로
비교해 mismatch 0을 확인했다. 별도 기준 PDF·golden·sample 변경은 없고 사람의 시각 판정을 merge 근거로
사용하지 않았으므로 대표 review PNG를 만들지 않는다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| W1·W5·lineage·generator Node contract | 39/39 통과 |
| baseline·manifest deterministic check | 통과 |
| Rust font metric·legacy lookup | 9/9 통과 |
| full nextest | 8,073/8,073 통과, 정책 skip 39 |
| #2430 measured overlay | 5 face, 475/475 exact |
| Native Skia library·지정 fixture | 통과, 2/2, 4/4 |
| native release·Docker WASM | 통과 |
| native/WASM SVG parity | 7문서 167쪽, mismatch 0 |
| Clippy·doc test·fmt·diff check | 통과 |
| review 파생 suite manifest | 881 sources, 32 suites + 9 exceptions, 통과 |
| unit-tier policy | 4,225 tests / 299 modules, 통과 |

공유 `target/pr-review`에 과거 worktree의 compile-time 절대경로가 남아 발생한 `rhwp-contracts` 1건은
해당 package의 `release-test` 파생 cache만 정리하고 현재 checkout에서 다시 링크한 뒤 전체 8,073건이
통과했다. task branch의 오래된 ignored generated suite도 review 단계에서 `--prepare`한 뒤 표준 fixture
명령이 suite 025·030을 선택해 재통과했다. 파생 파일은 PR diff에 포함하지 않는다.

## GitHub Actions와 남은 조건

code candidate `2d15b50ea`의 preflight 5종은 성공했고 Full CI, CodeQL, Proptest, Adapter와 Render Diff는
self-review 작성 시점에 진행 중이다. 그러나 위 generator blocker를 고칠 새 code head가 필요하므로 현재
실행 결과가 성공하더라도 최종 merge 근거로 재사용하지 않는다.

최소 정정과 focused negative contract를 통과한 새 code candidate에서 변경 범위 검증을 다시 수행하고,
review·오늘할일 trailing commit을 포함한 최신 head의 required checks를 확인해야 한다. code correction이
포함되므로 review-only fast-pass로 Full CI를 우회하지 않는다.

## 현재 권고

metric data 분리와 renderer 불변성에서는 blocker를 발견하지 않았지만 generator ownership·paired output
계약 2건과 lockfile 보호 1건을 고쳐야 한다. 따라서 현재 head는 **changes requested / merge 보류**다.
메인테이너가 최소 정정 계획을 승인한 뒤 보정 code·negative test·재검증을 별도 commit으로 추가하고,
review 문서를 최신 code candidate 기준으로 갱신한다.
