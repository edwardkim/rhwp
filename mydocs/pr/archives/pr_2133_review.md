# PR #2133 리뷰 — 중간-쪽 RowBreak 표 선언-fit 확대

- 작성 시각: 2026-07-10 KST
- PR: https://github.com/edwardkim/rhwp/pull/2133
- 작성자: `planet6897`
- base / head: `devel` / `fix/2097-midpage-rowbreak`
- 문서 작성 시점 참고 head: `fd4b346193d7fd6498f66eee87f6cde594f110c2`
- 문서 작성 시점 참고 mergeable: `MERGEABLE`
- 처리 경로: `codex/planet6897-cherrypick-20260710` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- #2105의 RowBreak 쪽 상단 선언-fit 게이트를 중간-쪽 표까지 제한적으로 확장한다.
- `overshoot <= 16px && excess <= 20px` 또는 near-fit `overshoot <= 4px && excess <= 48px`일 때만 선언 높이를 신뢰한다.
- `RHWP_TABLE_DRIFT` trace에 declared/pageBreak 필드를 보강한다.
- 실문서 `3080901_pii_ledger.hwp`, 한컴 2022 기준 PDF, 합성 fixture와 pin tests를 추가한다.

## 체리픽 검토

- 누적 체리픽 순서: 4/4.
- 공유 선행 커밋: #2109에서 `1c1d2330796652c2e771690a598af10a945aa080`을 이미 적용했다.
- 적용 커밋: `cc08688b5` (`fd4b346193d7fd6498f66eee87f6cde594f110c2`에서 `-x` 체리픽).
- 충돌: 없음.
- 선행 PR 의존: #2109 RowBreak 쪽 상단 선언-fit 게이트가 선행 적용되어야 한다.

## 기준 샘플

| 항목 | 경로 |
|---|---|
| 실문서 HWP | `samples/task2097/3080901_pii_ledger.hwp` |
| 기준 PDF | `pdf/task2097/3080901_pii_ledger-2022.pdf` |
| 합성 HWPX | `samples/task2097/rowbreak_midpage_declared_fits.hwpx` |

## 검증

- 원 PR GitHub Actions: 문서 작성 시점 기준 `CI`, `CodeQL`, `Render Diff` 계열 check 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2105_rowbreak_table_declared_fits --test issue_2097_3080901_real_doc_pin --test issue_2097_rowbreak_midpage_declared_fits --test issue_1842`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0). `tests/issue_2097_3080901_real_doc_pin.rs`, `tests/issue_2097_rowbreak_midpage_declared_fits.rs` 모두 release-test에서 통과.

## 판단

- 체리픽 통합 가능.
- 중간-쪽 RowBreak 확대는 overshoot/excess 2티어 노이즈 한도로 제한되어 있으며, 실문서 기준 PDF와 합성 fixture가 함께 보존된다.
- 원 PR은 통합 PR이 merge된 뒤 supersede close/comment 처리 대상이다.
