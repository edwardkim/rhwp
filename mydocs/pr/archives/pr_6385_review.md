---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
---

# PR #6385 self-review — F5 셀 블록 계산과 한글 IME 셀 명령

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`, `visual_verification_governance.md`
- 작성자 본인 self-review이므로 GitHub reviewer를 지정하지 않는다.
- 제품 코드 candidate: `5240e38f130b35e07d277eab3098617f52bc129c`

Studio 셀 선택 오버레이는 문서 renderer·layout·저장 출력이 아닌 제품 UI다. 따라서 시각 검증 거버넌스의
`studio/확장 UI (렌더 엔진 무관)` 경로를 적용해 실제 브라우저 기능 스모크와 작업지시자의 수동 승인을
근거로 사용했다. 재현 가능한 임시 스크린샷·build 산출물은 source PR에 포함하지 않는다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6385](https://github.com/edwardkim/rhwp/pull/6385) |
| 작성자 | `postmelee` (repository write collaborator) |
| 관련 이슈 | [#4135](https://github.com/edwardkim/rhwp/issues/4135) |
| base / head | `devel` / fork `postmelee:codex/issue-4135-contextual-shortcut` |
| local 제품 candidate 규모 | 41 files, +2,613 / -118, 23 commits |
| latest merge simulation target | `upstream/devel@067a8134b` |
| remote 상태 | Open, non-draft. 기존 head `feadbd1e8`은 `MERGEABLE/CLEAN`이고 당시 CI 성공 |

이 PR은 collaborator가 자기 fork에서 연 기존 PR이다. collaborator self-merge의 권장 upstream-owned branch와
다르지만 PR을 다시 만들어 review·discussion 연속성을 끊는 이득이 작아 현재 head를 유지한다. GitHub
metadata와 기존 CI는 변동 값이며, 이 문서 뒤 trailing review commit을 push하면 최신 head 기준으로 다시
계산된다. CI 완료 확인과 최종 merge는 작업지시자가 별도로 수행한다.

## 목적과 변경 범위 정합성

- F5 셀 블록의 `Ctrl/Cmd+Shift+S`를 Save As·셀 나누기보다 먼저 블록 합계로 라우팅한다.
- 선택 범위의 오른쪽 또는 아래 빈 결과 가장자리에 행별·열별 합계를 한 undo 단위로 기록한다.
- 한글 IME에서도 물리 `KeyS`·`KeyM` 셀 나누기·합치기를 처리하고 후속 `ㄴ`·`ㅡ` 입력을 억제한다.
- F5 1·2단계는 셀 중앙 회색·주황 마커, 1·2·3단계는 하단 상태 문구로 현재 모드를 설명한다.
- 표 계산식은 `AA` 이상 열 주소와 일반 셀 텍스트 치환을 지원한다.
- 병합으로 covered cell이 물리 배열에서 제거돼도 수식 원본과 결과 셀을 논리 `(row, col)` 좌표로 찾는다.

## self-review findings와 처리

### 수정 완료 — 병합 셀 뒤 수식 좌표가 한 칸씩 밀림

기존 구현은 `row * col_count + col`을 `table.cells`의 물리 index로 사용했다. 그러나 셀 병합은 anchor만
남기고 covered cell을 배열에서 제거하므로, 병합 영역 뒤의 논리 좌표와 물리 index가 달라진다.

3×4 표에서 `A1:B1`을 병합하고 `A2=1`, `B2=2`, `C2=3`, 결과 `D2`에 `SUM(A2:C2)`를 계산하는 실제
API 경로를 재현했다. 보정 전에는 합계가 `5`가 되고 결과가 `A3`에 기록됐다. 원본 셀과 결과 셀을
각 cell의 논리 `row`·`col`로 검색하도록 바꾸고, 결과 셀이 없으면 명시적 오류를 반환했다. 통합 회귀는
합계 `6`, `D2="6"`, `A3` 무변경을 함께 고정한다.

### 추가 blocker 없음

- covered non-anchor 좌표는 독립 셀이 아니므로 수식 원본에서 값 없는 셀로 취급하는 기존 의미를 유지한다.
- 계산 결과의 snapshot·undo와 Studio all-or-nothing preflight 계약은 바꾸지 않았다.
- shortcut 문맥 우선순위와 IME 억제는 `code`·modifier·selection state를 분리해 일반 문자 입력 경로를
  침범하지 않는다.
- F5 3단계와 Escape는 marker·status를 모두 제거하며 보호 셀 내부 선택에는 학습용 상태를 표시하지 않는다.

## 완료한 검토와 로컬 검증

| 검증 | 결과 |
| --- | --- |
| suite manifest prepare/check | 1,032 sources / 4,532 static attrs / 48 targets, 통과 |
| unit tier policy | 4,221 tests / 299 modules, 통과 |
| `document_core::table_calc` | 29/29 통과 |
| WASM API #4135 focused | 1/1 통과 |
| #4135 integration focused | 병합 회귀 포함 3/3 통과 |
| release-test 전체 nextest | 8,635/8,635 통과, 43 skipped, 0 실패 |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`, 통과 |
| 포맷·whitespace | `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`, 통과 |
| 이전 동일 Studio 제품 candidate | 1,272 tests / 1,271 통과 / 1 skip, TypeScript·production build 통과 |
| 실제 사용자 검증 | macOS 한글 IME 셀 나누기와 F5 1·2·3 단계 UX 승인 |

전체 nextest와 Clippy는 generated suite를 준비한 임시 review worktree에서 실행했다. 검증 뒤 임시
worktree와 이 작업이 만든 `target/pr-review` 8.9GiB를 제거했고 generated suite·manifest를 source PR에
포함하지 않았다. 최신 보정은 Rust 표 좌표 조회와 해당 integration test만 바꾸므로 이전 Studio 수동·자동
검증의 UI candidate는 동일하다.

기존 remote head의 CI는 성공했지만 최신 보정을 검증하는 근거로 승격하지 않는다. trailing review commit을
push한 뒤 latest-head CI에서 fmt·Clippy·WASM, 전체 test, Frontend, Native Skia, CodeQL, Render Diff와
보조 회귀 gate를 다시 확인해야 한다.

## 큰 PR 별도 검토 회차

PR이 1,000줄을 넘으므로 metadata 확인과 별도로 제품 diff·테스트·문서 정합성을 검토했다. 이 회차에서
병합 셀 뒤 계산 좌표 blocker를 실제 API로 재현해 보정했고, focused와 전체 release-test를 다시 통과했다.
최신 `upstream/devel`과 merge-tree·trailing head·문서 포함 여부는 push 직전 및 직후에 별도 확인한다.

## 최종 권고

review에서 발견한 병합 표 좌표 blocker를 보정했고 추가 blocker는 발견하지 않았다. self-review는
**완료 / 최신 head CI 및 작업지시자 승인 조건부 merge 권고**다.

이 작업에서는 보정과 검토 기록을 push해 CI를 시작하는 지점까지만 수행한다. CI 완료 확인, merge와
연결 이슈 #4135 종료는 작업지시자의 후속 판단 뒤 별도로 수행한다.

## Maintainer integration review - 2026-08-30

외부 통합 검토에서는 `e83dd818a09ed9632df285d57174bde552bb45b6`까지 반영했다. 첫 실행 skin
onboarding이 test textarea 포커스를 가리는 기존 E2E 준비 결함을 `605bfaa21`로 해제한 뒤 실제
Chromium `undo-contracts.test.mjs`를 통과했다. 제품 코드와 shortcut 의미는 바꾸지 않았다.

- Studio TypeScript, `npm test` 1271 pass / 1 skip, production build 통과.
- `issue_4135_table_calc_multi_letter_columns`: 3/3, Native Skia 지정 회귀 2/2 및 4/4 통과.
- 통합 release-test 전체 8,712/8,712 통과.

따라서 maintainer integration 판단도 **수용 권고**다. 통합 PR merge 후 원 PR에는 F5/IME focused
계약, 실제 browser undo E2E, 최신 통합 회귀 결과를 수용 근거로 남긴다.
