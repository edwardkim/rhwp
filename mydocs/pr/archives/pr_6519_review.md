---
kind: pr-review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6519
issue: 4969
author: edwardkim
---

# PR #6519 review - W10 support matrix와 bounded guard

## 결론 - 승인, review-only 기록 검증 대기

[PR #6519](https://github.com/edwardkim/rhwp/pull/6519)은 #4969 W10-Q5의 세 lane 결산과 Q6의
최종 support matrix·tracker 인계 자료를 제출한다. self-review 대상 코드 후보는
`27f037fd5c755660d3bc81e06c7f894ba86bef54`, base는
`upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`이다. 로컬 검증과 코드 후보의
GitHub Full CI에서 차단 결함을 발견하지 않았다. 문서 작성 시점에 PR은 `MERGEABLE/CLEAN`이며,
이 review-only 기록의 fast-pass와 최신 mergeability를 확인하기 전에는 병합하지 않는다.

## 검토 경로와 metadata

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- self PR이므로 reviewer를 지정하지 않는다.
- 변경 규모: 22 files, +3,107/-2, 9 commits
- 변경 경계: mydocs 19 paths, 기존 `tests/cases/` integration source 3 paths
- 제품·Studio·workflow source, 신규 integration source, generated suite·manifest: 0
- 관련 tracker: #4969, #4960. PR 본문은 `Refs`만 사용해 merge 전 자동 close를 막았다.

## 변경과 지원 경계

- Q2 horizontal old Hangul은 동결된 Source Han exact-source tuple에서만 `bounded-subset`이며,
  그 밖의 surface는 W9 K1 또는 K0 TextRun fallback을 유지한다.
- Q3 variable instance는 Happiness Sans VF의 explicit default/interior/max 계약에서 `qualified`다.
  invalid·incomplete instance와 미지원 backend는 원자적으로 TextRun으로 되돌아간다.
- Q4 vertical은 exact Noto source와 `vhea`·`vmtx`, HWP5 table-cell v1, CanvasKit이 모두 성립하는
  tuple만 `bounded-subset`이다. 나머지는 TextRun과 기존 vertical geometry를 보존한다.
- 전체 분류는 `bounded-subset`이다. deferred surface를 현재 지원이나 실패 건수로 계산하지 않았다.
- tracker 문안의 PR·merge·CI placeholder는 로컬 초안에만 있으며 merge 전 게시를 막는 receipt가 있다.

## self-review 결과

- Q2 1/2/8 run resource matrix는 run 수와 무관하게 font blob·face가 exact source당 하나이고,
  TextRun과 GlyphRun 수가 요청 수와 일치하는지 확인한다.
- Q3 guard는 exact-source slot과 instance-request 상한이 모두 4,096으로 결합됐음을 확인한다. 작은 공개
  fixture 하나를 공유해 4,096 slots/requests를 채우고, source 없는 4,097번째 요청이 count·generation을
  바꾸지 않는지 검증한다.
- Q4 guard는 4,096 sidecar를 채운 뒤 4,097번째 attach가 `EntryLimitExceeded`로 끝나고 len·generation이
  그대로인지 확인한다. oversized payload나 무제한 allocation을 만들지 않는다.
- 세 guard는 기존 #4969 integration source 안에 있으며 crate-private owner를 직접 포함하는 기존 계약을
  따른다. 제품 API나 지원 predicate를 넓히지 않는다.
- support matrix, lane 판정, 성능·WASM ledger와 테스트 상수 사이의 불일치를 발견하지 않았다.
- 새 sample, PDF, font binary, private corpus 결과, Hyper-V·한컴 Oracle 자료는 포함하지 않았다.
- 차단 finding: 없음.

## 렌더·시각 영향 판정

제품 renderer·layout·WASM adapter·Studio source diff가 0이므로 이 PR 자체의 신규 시각 sweep은 필요하지
않다. Q5-A에서 PR #6493의 제품 tree와 증적 계보를 자격화했고, Q5-B에서 같은 제품 tree의 native↔WASM
canonical mismatch 0, CanvasKit replay·pixel mismatch 0을 다시 고정했다. 이 증적 재사용은 제품 source가
변하지 않은 범위에 한정한다.

## 로컬 검증

- integration manifest: 1,075 sources / 4,718 attrs / 28 suites + 20 exceptions = 48/48
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` - passed
- native root, wasm32 library, workspace all-target Clippy `-D warnings` - passed
- workspace build - passed
- #4969 focused nextest: 117 passed / 0 failed / 8,795 non-selected
- Q5-C bounded·malformed·atomic rollback: 50/50 passed
- Q5-D resource focused: 2/2 passed
- Studio font resource reuse: 5/5 passed
- source unit tier: 4,221 tests / 299 modules, policy check passed
- shared `target/pr-review`만 사용했고 generated suite·manifest를 stage하지 않았다.

## 코드 후보 GitHub Actions

코드 후보 `27f037fd5`에서 24 success, 5 expected skip, 실패·대기 0을 확인했다. GHAS 집계
`CodeQL` 1건은 `NEUTRAL`이고 같은 run의 JavaScript/TypeScript·Python·Rust Analyze는 모두 성공했다.
CI Build & Test, Lint, archive A/B/C/D와 각 default-feature shard, Proptest, Adapter inter-diff도 성공했다.
제품·Studio source가 없는 변경 범위에 따른 Native Skia, WASM Build, frontend gate skip은 정책상 정상이다.

## 최종 판정

- 판정: 승인
- 근거: support matrix와 구현 경계가 일치하고, bounded guard·fallback·resource 회계가 로컬 및 최신 코드
  후보 CI를 통과했으며 self-review 차단 finding이 없다.
- merge 전 조건: review-only trailing head의 fast-pass와 required aggregate 성공, 최신 head SHA,
  `MERGEABLE/CLEAN`, 메인테이너의 별도 merge 승인.
- 원격 조치: 이 기록 자체는 issue edit/comment/close 또는 merge를 수행하지 않는다.
