# PR #1494 검토 기록

## PR 메타

| 항목 | 내용 |
|---|---|
| 번호 | #1494 |
| 제목 | task 1491: 표 셀 리사이즈와 균등화 회귀 보정 |
| 작성자 | jangster77 |
| base | devel |
| head | task_m100_1491 |
| 관련 이슈 | #1491 |
| 규모 | 문서 작성 시점 참고값: 23 files, +1564 / -70 |
| head SHA | 문서 작성 시점 참고값: `b1ff3be2990c6dc15209b9de80e39cb0f18f3643` |
| draft | 문서 작성 시점 참고값: false |
| mergeable | 문서 작성 시점 참고값: MERGEABLE / mergeStateStatus BLOCKED |
| CI 상태 | 문서 작성 시점 참고값: GitHub Actions pending, merge 전 최신 상태 확인 필요 |

## 관련 이슈 요약

#1491은 `셀 너비를 같게` 기능이 선택 행/셀의 local width hint를 유지하지 못하고 전역 grid 기준으로 되돌아가는 회귀를 다룬다. 작업 중 Shift+마우스 개별 셀 resize, undo 이후 resize cache, `셀 높이를 같게` 명령의 같은 계열 회귀도 함께 확인되어 같은 PR에 포함했다.

추가 확인된 동반 회귀는 #1491에 코멘트로 기록했다: https://github.com/edwardkim/rhwp/issues/1491#issuecomment-4781531372

## 변경 범위 분석

- 표 셀 resize 모델에 local resize hint와 렌더 bbox 기준 보존 경로를 보강했다.
- Shift+마우스 개별 셀 너비/높이 조절에서 선택/hover/cache 상태가 섞여도 대상 셀만 조정되도록 보정했다.
- `셀 너비를 같게`, `셀 높이를 같게` 명령을 마우스 resize와 같은 snapshot operation 경로로 적용하도록 정리했다.
- undo/redo 및 문서 전환 시 표 resize 런타임 cache가 stale 상태로 남지 않도록 정리했다.
- 회귀 방지를 위해 Rust 통합 테스트와 rhwp-studio node tests를 추가했다.
- `samples/rowbreak-problem-pages.*` 및 PDF는 추후 검증 자료로 보관하기 위해 포함했다.

## 사전 검증 결과

로컬 검증은 2026-06-24 KST 기준 PR head `b1ff3be2`에서 수행했다.

- `cargo build --release`: passed
- `cargo test --release --lib`: passed, 1923 passed / 0 failed / 6 ignored
- `cargo test --profile release-test --tests`: passed
- `cargo fmt --check`: passed
- `git diff --check`: passed
- `cargo clippy --all-targets -- -D warnings`: passed
- `cargo test --doc`: passed, 0 passed / 0 failed / 1 ignored
- `cargo test --test svg_snapshot`: passed, 8 passed / 0 failed
- `cd rhwp-studio && npx tsc --noEmit`: passed
- `cd rhwp-studio && npm test`: passed, 147 passed / 0 failed
- `wasm-pack build --target web --out-dir pkg`: passed

## 주요 문제점 / 리스크

- 표 resize는 renderer bbox, local resize hint, undo cache가 맞물리는 영역이라 시각 회귀 가능성이 있다. PR head 최신 커밋 기준 GitHub Actions와 작업지시자 시각 판단을 merge 전 최종 조건으로 둔다.
- `samples/rowbreak-problem-pages.*`는 기능 코드가 아니라 추후 검증 자료 보관 목적이다. 용량과 목적을 PR description 및 리뷰 문서에 명시했다.
- mergeable/CI 상태는 작성 시점 참고값이므로 merge 직전 최신 상태를 다시 확인해야 한다.

## 최종 권고

조건부 merge 후보로 본다.

최종 merge 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- 이 review 문서와 `pr_1494_review_impl.md`가 PR diff에 포함됨
- 작업지시자 승인
