---
kind: pr-review
status: in-review
pr: 5548
issue: 5534
base: devel
code_candidate: 0a334bd3ca3a68f45fbba53d71169a5f2610fe5d
last_verified: 2026-08-19
---

# PR #5548 검토: 수식 Thin 공백 첨자 결합과 폭 보존

## 접수

- PR: [#5548](https://github.com/edwardkim/rhwp/pull/5548)
- 관련 이슈: [#5534](https://github.com/edwardkim/rhwp/issues/5534), PR 본문의 `closes #5534`
- 작성자: `planet6897`; base: `devel`; 작성 시점 head: `0a334bd3ca3a68f45fbba53d71169a5f2610fe5d`
- reviewer: `jangster77`
- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 라우팅 문서

## 변경 범위와 계보

- contributor 원 변경 `095ab811b71c2bfa7daf26a6d4f5c892f3909d8d`는 첨자 앞 Thin 공백을
  `Row[base, Space(Thin)]`의 base 폭으로 보존하고, 함수명 뒤 첨자 경로가 이를 소비하지 않게 했다.
- collaborator 보정 `648cef952`는 괄호 그룹도 같은 규칙을 따르도록 했다. `paren_then_script`가
  닫는 괄호 뒤 Thin 공백 하나를 건너뛰어 `_`와 `^`를 식별하고, 기존 `try_parse_scripts`가 공백을
  base에 보존한다.
- 포맷 commit `0a334bd3`은 위 보정의 Rust 포맷만 반영한다.
- 원 contributor commit은 재작성하지 않았고, 모든 collaborator commit은 원 head 뒤에 추가됐다.

## 검토 결과

- 차단 결함: 없음.
- 메인터너 보정 전 `(a)`_{2}`와 `(a)`^{2}`는 닫는 괄호와 스크립트 사이 Thin 공백 때문에
  `Paren` 그룹으로 묶이지 않아 스크립트 결합 경로를 놓칠 수 있었다.
- 보정은 공백을 삭제하거나 일반 괄호 파싱을 바꾸지 않는다. 스크립트 바로 앞의 Thin 공백 하나에만
  적용되며, 최종 base의 `Space(Thin)` 폭은 기존 공통 스크립트 처리에 맡긴다.
- 회귀 계약은 일반 base, 함수 base, 위첨자, 괄호 base의 아래·위 첨자에서 AST 결합과 Thin 공백 보존을
  확인한다.

## 검증 증적

### 로컬

- `cargo fmt --all -- --check` 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare`로 무시되는 파생 suite를 준비한 뒤
  `node scripts/rust-test-suite-manifest.mjs --check` 통과.
- `node scripts/rust-unit-test-tiers.mjs --check` 통과.
- 이번 보정의 focused Rust test와 renderer 시각 검증은 아직 실행하지 않았다.

### 시각·fixture

- 이 변경은 수식 parser AST와 첨자 앵커 폭을 보존하는 구조 보정이다. 독립 HWP/HWPX fixture와 기준
  PDF는 PR에 첨부되지 않았고, 이슈 #5534의 한컴 비교 이미지는 참고 근거다.
- `Space(Thin)`가 base의 오른쪽 경계를 넓혀 첨자 위치를 이동시키는 기존 layout 계약을 활용한다.
  따라서 별도 시각 비교는 참고 자료로 유지하며, parser AST 계약과 최신 CI를 merge 조건으로 삼는다.

### GitHub Actions

- `0a334bd3`의 Full CI aggregate, lint, archive builder, slow/regular shard 4개, Native Skia, CodeQL Rust,
  Canvas visual diff, Proptest, adapter inter-diff가 모두 성공했다. frontend 및 WASM은 영향 정책에 따라 skipped다.
- `cancel-stale-runs`의 실패는 이전 run 정리 workflow 기록이며, 최신 code candidate의 Build & Test aggregate와
  required code 검증 성공을 대체하거나 무효화하지 않는다.
- contributor source branch의 원격 head가 이 candidate와 같은지 확인한 뒤 review·오늘할일 docs-only trailing
  commit을 push하고, 그 새 head의 review-only aggregate를 다시 확인한다.

## 다음 게이트

**code candidate 검증 완료.** 이 문서와 오늘할일만 담은 single-parent docs-only commit을 contributor
source branch에 추가한다. 해당 trailing commit은 review-only fast-pass 조건과 최신 aggregate를 다시 확인한
뒤 merge 판단으로 진행한다.
