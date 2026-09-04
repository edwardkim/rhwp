---
kind: report
status: active
canonical: mydocs/report/archives/task_m100_5955_report.md
last_verified: 2026-08-25
---

# Task M100 #5955 최종 보고서 — W7.5 font rule lifecycle·evidence delta

- **Issue**: #5955
- **상위 tracker**: #4960
- **후속**: #4967 W8
- **작업 브랜치**: `task_m100_5955`
- **현재 판정**: PR #6049 Draft, PR self-review semantic guard 정정의 새 code head 제출 전

## 1. 최종 판정

#5955의 완료 조건은 로컬 후보에서 충족됐다. W7의 schema 1.0 registry는 원 byte를 보존한 역사 anchor로
남았고, 830개 rule을 의미 변화 없이 lifecycle schema 2.0으로 이행했다. current runtime projection은 v2의
active rule만 읽으며 초기 population은 830 active/0 retired, backend별 `171/67/281/153/158`이다.

제품 규칙의 추가·evidence 보강·retirement·replacement는 side-effect-free reducer와 append-only change-set
계약으로 표현된다. same-ID selection tuple 변경, stale parent, 교차 decision plane·projection, evidence graph
손상과 unsafe path는 fail-closed한다. W2 Font Decision Trace 원문과 renderer API는 바꾸지 않고 offline
lifecycle audit에서 유지·신규·retired·replaced·historical·dangling 계보를 설명한다.

이 결과는 W8이 change set을 작성할 수 있는 기반일 뿐 #4967의 rank 8 qualification이나 실제 font mapping
변경을 승인하지 않는다.

## 2. 단계별 결과

| Stage | 결과 |
| --- | --- |
| W7.5-1 | schema 2.0 registry·change set·migration의 RED 계약과 보안 상한 고정 |
| W7.5-2 | side-effect-free reducer, 830건 initial carry-forward와 migration query model 생성 |
| W7.5-2C | `sourceBoundaryId`를 evidence가 아닌 immutable selection semantic으로 정정 |
| W7.5-3 | 다섯 backend projection을 v2 active-only 단일 authority로 전환, semantic 0-delta |
| W7.5-4 | W2 trace의 offline lifecycle resolver·audit와 입력 상한 구현 |
| W7.5-5 | evidence/add/retire/replace와 rollback의 synthetic W8 rehearsal 고정 |
| W7.5-6 | Rust·Studio·Native Skia·Docker WASM·공개 SVG 전체 제품 0-delta 검증 |
| W7.5-7 | self-review blocker 2건 정정, 최신 `upstream/devel` merge tree 검증 |
| W7.5-8 | PR self-review에서 v1 semantic guard 이관 누락을 재현하고 validator·negative contract 정정 |

세부 명령과 단계별 수치는 [Stage W7.5-1~6 기록](../../working/task_m100_5955_w7_5_stage1.md),
[Stage W7.5-7 기록](../../working/task_m100_5955_w7_5_stage7.md)과
[Stage W7.5-8 정정 기록](../../working/task_m100_5955_w7_5_stage8.md)에 보존했다.

## 3. canonical artifact와 population

| artifact | SHA-256 |
| --- | --- |
| v2 registry raw | `fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16` |
| v2 rules | `bd9469aa16156a16ea262f608015cb0b78e925700ae7df69c38602ba6670c029` |
| v1→v2 migration | `54b17603e3ae52eac7b37f32b1f36b778ad01343501fdcf05bb3ae145f82fb5a` |
| projection semantic bundle | `090b4403832a739b7e2928fdc83741126a5cb7e05b4d3ae3fc8be17833e863a6` |
| generated content bundle | `3ba1d6c14b7514143bff42d5e1c690b4d87f41a09ef04424395f3327f772fcaa` |

| projection | active | semantic SHA-256 |
| --- | ---: | --- |
| `rust-layout-name` | 171 | `595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee` |
| `rust-layout-metric` | 67 | `c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2` |
| `canvas2d-paint` | 281 | `c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4` |
| `canvas2d-webfont` | 153 | `730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee` |
| `canvaskit-sfnt` | 158 | `d9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668` |

schema 1.0 registry·schema, W7 migration과 pre-migration baseline은 수정하지 않았다. generated Rust·TypeScript는
generator 결과만 반영했고 integration source는 기존 `tests/cases/issue_4966_font_rule_projection.rs`를
현행화했다. generated suite·manifest는 제출 diff에 포함하지 않았다.

## 4. 전체 제품 검증

Stage W7.5-6의 code candidate `270e321c8`에서 다음을 순차 실행해 모두 통과했다.

- production release build
- release library 4,071 pass/13 ignore
- release-test nextest 8,208 pass/41 skip
- Native Skia library 4,128 pass/13 ignore, missing-picture 2/2, direct PDF 4/4
- Clippy `-D warnings`, rustdoc, `cargo fmt --all -- --check`, diff check
- Studio TypeScript, Node 1,070 pass/1 skip, production build 223 modules
- Docker optimized WASM과 fresh WASM Decision Trace 3/3
- native/WASM 공개 SVG: W1 7문서 167쪽과 W2 6문서 6쪽, byte mismatch 0

W7.5은 사용자-visible mapping을 바꾸지 않으므로 새 HWP/HWPX/PDF fixture, golden, 기준 PDF와 visual review
PNG를 만들지 않았다. renderer 경계는 동일 source의 native/WASM SVG 173쪽 전건 byte parity로 판정했다.
private corpus, Hyper-V Oracle과 로컬 font bytes는 재사용하거나 공개하지 않았다.

## 5. self-review 정정

Stage W7.5-7에서 두 blocker를 발견해 정정했다.

1. reducer는 replacement의 동일 projection 승계를 강제했지만 독립 registry validator는 같은 `supply`
   decision plane 안의 `canvas2d-webfont → canvaskit-sfnt` successor를 허용했다. successor graph validator에
   동일 projection 검사를 추가하고 재현 negative contract를 고정했다.
2. Stage W7.5-6 보고서가 비식별 경계를 선언하면서 Cargo target을 사용자 절대경로로 한 줄 기록했다.
   저장소 상대경로 `target/pr-review`로 정정했다.

정정 뒤 v2 focused 23/23과 전체 `scripts/tests/font_rule_*.test.mjs` 93/93이 통과했다. registry·projection
결정성 및 `git diff --check`도 다시 통과했다.

PR #6049의 위 code head가 Full Actions를 통과한 뒤 수행한 두 번째 self-review에서는 더 근본적인 blocker를
발견했다.

3. v2 validator가 v1의 projection별 relation allowlist와 metric/supply semantic guard를 이관하지 않아
   `canvas2d-paint + supply-source`, paint rule의 metric anchor와 `file://` webfont payload를 허용했다.
   registry와 change-set payload가 공유하는 semantic validator로 v1 경계를 전수 이관했다.
4. malformed `evidenceRecords`와 `projections`가 오류 목록 대신 `TypeError`를 만들었다. 잘못된 collection을
   안전한 빈 순회 입력으로 분리하되 원 자료형 오류는 보존해 validator totality를 복구했다.

정정 뒤 최초 재현 세 건은 모두 거부됐고 v2 focused 26/26, 전체 font-rule Node 계약 96/96, v1/v2 registry,
projection과 pre-migration baseline check, Rust unit-tier 4,221 tests/299 modules가 통과했다. canonical registry,
migration, generated source와 제품 mapping은 바뀌지 않았다. 기존 녹색 Actions head는 새 negative contract를
포함하지 않으므로 merge 근거로 재사용하지 않으며, correction head의 Full CI가 새 외부 게이트다.

## 6. 최신 devel 병합 시뮬레이션

2026-08-25 `upstream/devel@385e93b2c317d1f50d874fd655e88cf4b2a1ba07`은 최초 기준선보다 40커밋
전진해 있었다. 양쪽이 같은 파일을 수정한 경로는 0개였고 `git merge-tree --write-tree`는 충돌 없는 tree를
생성했다.

정정된 후보를 임시 Git object로 고정해 최신 base와 합친 독립 review worktree에서 다음을 확인했다.

- v1/v2 registry, projection generator와 pre-migration baseline check 통과
- font-rule Node 계약 93/93
- Rust unit-tier 4,221 tests/299 modules, drift 0
- integration inventory 911 source/4,267 static test attribute, 32 suite+9 exception, 41/48 target
- `issue_4966_font_rule_projection` 3/3
- merge tree `git diff --check`와 `cargo fmt --all -- --check` 통과

검증용 integration suite·manifest와 worktree는 종료 뒤 제거했다. 최신 base에서 full release·Native
Skia·Docker WASM을 반복하지 않은 이유는 Stage W7.5-6 전체 검증 뒤 변경이 validator negative guard와 문서에
한정되고, 최신 base와 파일 overlap이 없으며 merge tree의 제품 focused test가 통과했기 때문이다. GitHub의
최신 PR head Full CI는 merge 전 별도 필수 조건으로 남는다.

## 7. 보호 불변식

| 불변식 | 결과 |
| --- | --- |
| schema 1.0 역사 artifact byte 보존 | 충족 |
| initial v2 830 active/0 retired, selection tuple delta 0 | 충족 |
| 한 rule/한 decision plane/한 projection | 충족 |
| projection별 relation·metric·supply 소유 경계 | Stage W7.5-8 정정 뒤 충족 |
| same-ID in-place semantic mutation 금지 | 충족 |
| retired row 보존·runtime 제외 | 충족 |
| evidence parent·digest와 stale parent 검증 | 충족 |
| Canvas2D supply와 CanvasKit SFNT capability 분리 | 충족 |
| W2 trace·renderer API와 native/WASM output 0-delta | 충족 |
| private 자료·font bytes·식별 host path 비공개 | 충족 |

## 8. 운영 인계

current 규칙 권위와 변경 절차는 [폰트 fallback 전략](../../tech/font_fallback_strategy.md)에 반영했다. W8은
[Issue #5955 조사 정본](../../tech/investigations/issue-5955/README.md)의 reducer·rehearsal 계약을 사용하되
다음 순서를 별도 계획과 승인으로 시작해야 한다.

1. #4967의 공개 evidence와 정확히 한 decision plane을 고른다.
2. 최신 v2 raw digest를 parent로 하는 issue-scoped change set을 만든다.
3. pre/post tuple과 대상 projection delta, 비대상 네 projection 무변화를 검증한다.
4. 실제 renderer 영향에 맞는 full·시각 gate를 다시 실행한다.

PR #6049는 첫 code head의 녹색 CI 뒤 self-review blocker를 발견해 Draft로 전환했다. 현재 남은 #5955 절차는
Stage W7.5-8 correction head의 승인된 remote push, 새 Full GitHub Actions, self-review archive와 오늘할일
trailing commit, 최신 CI·Ready 전환·merge 승인이다. 이 보고서는 그 외부 상태가 완료됐다고 주장하지 않는다.
