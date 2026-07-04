# PR #1850 리뷰 구현 메모

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1850
- 작성자: @planet6897
- 제목: `Task #1841: 자리차지 표 직후 본문 재개에 outer_margin bottom 반영 (결재문서 계열 -8.5pt 상향 해소)`
- base / head: `devel` / `pr/devel-1841`
- 검토 head: `9519b3aca3796e00e62c0aeea1b6e02fd57ba46a`
- merge commit: `bceed75f8b1db049e45a620de4e80bf294dad38d`

## Stage 1. 메타 확인

완료.

- reviewer assign 완료.
- PR은 Draft가 아니다.
- base는 `devel`.
- `maintainerCanModify=true`.
- mergeable 상태는 `MERGEABLE`.
- PR head는 update branch 후 `9519b3aca3796e00e62c0aeea1b6e02fd57ba46a`로 갱신되었다.

## Stage 2. 변경 내용 검토

완료.

- `layout.rs`의 visible-host float 재개 y 계산 두 분기에 `outer_margin_bottom`이 반영되었다.
- `typeset.rs`의 `outer_bottom` 조건이 `is_tac`뿐 아니라 visible-host TopAndBottom 표로 확장되었다.
- 모든 비-TAC 표에 전면 적용하지 않고 visible-host float 형상으로 제한한다.
- `issue_1789`와 `issue_1692` 테스트 핀은 기존 보상 오차 기준이 아니라 저장 vpos/권위 PDF 재측정 기준으로 정정되었다.

## Stage 3. 로컬 검증

완료.

- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제.
- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --check` 통과.
- `env CARGO_INCREMENTAL=0 cargo build` 통과.
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1692 --test issue_1789_exclusion_probe_line_spacing` 통과.
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `python3 -m py_compile scripts/task1274_visual_sweep.py` 통과.

## Stage 4. 시각 검증

완료.

- `samples/task1789/exclusion_probe_line_spacing.hwpx` vs `pdf/exclusion_probe_line_spacing-2024.pdf` p1:
  - flagged 0/1.
  - review: `output/pr1850_visual/pr1850-exclusion/review/review_001.png`
  - asset: `mydocs/pr/assets/pr_1850_exclusion_review_p001.png`
- `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` vs 기준 PDF:
  - 내부 페이지 번호 177 ↔ 단일 PDF p1 자동 fallback 매칭.
  - flagged 0/1.
  - review: `output/pr1850_visual/pr1850-36389312/review/review_177.png`
  - asset: `mydocs/pr/assets/pr_1850_36389312_review_p177.png`
- `samples/SO-SUEOP.hwp` vs `pdf/SO-SUEOP-2024.pdf` p22:
  - flagged 0/1.
  - review: `output/pr1850_visual/pr1850-so-sueop-hwp-p22/review/review_022.png`
  - asset: `mydocs/pr/assets/pr_1850_so_sueop_hwp_review_p022.png`

## Stage 5. 최종 상태

완료.

- 최신 PR head 기준 GitHub `Build & Test`, CodeQL, Render Diff, Canvas visual diff 통과.
- #1850 merge 완료.
- review 문서/asset/오늘할일과 visual sweep 단일 페이지 fallback 도구 보정은 후속 PR 방식으로 처리한다.
- 후속 PR merge 후 #1841 close comment 를 남기고 close 한다.
