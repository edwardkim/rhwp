# PR #1887 처리 계획 — #1811 HWPX saved bounds RowBreak 조판 보정

## 대상

- PR: #1887
- 작성자: @jangster77
- 관련 이슈: #1811
- 기준 브랜치: `devel`
- head branch: `task/m100-1811-saved-bounds-clean`
- 문서 작성 시점 코드 head: `6c6037ca9d4961d4c6fdd08e6aede5f9ddfc3943`
- 처리 경로: collaborator self-merge 후보, 옵션 1

## 커밋

1. `f32761b39`
   - `task 1811: HWPX RowBreak 합성 lineSeg 컷 보정`
   - RowBreak 합성 lineSeg와 row cut 계산 보정
2. `cb1a8b732`
   - `task 1811: HWPX RowBreak host 순서 보정`
   - mixed text/table RowBreak partial 순서 보정
3. `5dd28a51c`
   - `task 1811: missing lineSeg 폭 보정 범위 분리`
   - synthetic lineSeg와 HWP/HWP3-origin missing lineSeg fallback 폭 계산 분리
4. `6c6037ca9`
   - `task 1811: Stage6 회귀 분석 기록 보존`
   - draft PR #1875 계열 커밋이 섞였을 때의 `issue_1139_inline_picture_duplicate` 회귀 분석 기록 보존

## Stage 진행

### Stage 1. 이슈 재현 및 보정 방향 확인

완료.

- #1811은 #1752 merge 뒤 남은 `saved_bounds_cumulative_page_break.hwpx` p5 tail/line drift 후속이다.
- PR 범위는 HWPX RowBreak/saved bounds 조판 흐름 보정이다.

### Stage 2. RowBreak 합성 lineSeg 컷 보정

완료.

- `src/document_core/commands/document.rs`
- `src/renderer/composer.rs`
- `src/renderer/layout/table_layout.rs`
- `tests/issue_1749_saved_bounds_page_break.rs`

### Stage 3. HWPX RowBreak host 순서 보정

완료.

- 같은 셀 문단 내부에서 visible host text 가 nested table fragment 보다 먼저 배치되는지 확인했다.
- HWP 경로의 기존 cut 은 유지하고, HWPX 입력에서 필요한 pre-emit 경로만 적용했다.

### Stage 4. HWP3-origin missing lineSeg 회귀 분리

완료.

- synthetic lineSeg 경로에는 paragraph margin/indent 차감 폭을 유지했다.
- HWP/HWP3-origin missing lineSeg fallback 은 기존 `cell_inner_width_px` 기준으로 분리했다.
- `issue_1035_alignment` 회귀를 해소했다.

### Stage 5. Stage 6 분석 기록 보존

완료.

- `pr1875-review-latest` 에서 draft PR #1875 계열 미주 커밋이 섞였을 때 분석한 회귀 기록을 보존했다.
- clean 브랜치에서는 해당 미주 코드 수정 없이 `issue_1139_inline_picture_duplicate` 85개가 통과함을 확인했다.

### Stage 6. 옵션 1 PR 문서/asset/오늘할일 반영

진행.

- PR #1887 생성 완료
- review 문서: `mydocs/pr/archives/pr_1887_review.md`
- 처리 계획서: `mydocs/pr/archives/pr_1887_review_impl.md`
- 대표 visual asset:
  - `mydocs/pr/assets/pr_1887_issue1811_saved_bounds_review_p004.png`
  - `mydocs/pr/assets/pr_1887_issue1811_saved_bounds_review_p005.png`
- 오늘할일: `mydocs/orders/20260704.md`

## 검증

PR 생성 전 clean 브랜치 기준 완료:

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

추가 focused 확인:

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1035_alignment -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과

## merge 전 조건

1. 옵션 1 문서/asset/오늘할일 커밋을 PR head 에 push
2. 이전 code-only head SHA 의 run 이 남아 있으면 force-cancel
3. 최신 PR head 기준 GitHub Actions 통과 확인
4. 작업지시자 merge 승인 확인

## merge 후 후속 처리

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. PR #1887 merge
2. `upstream/devel` fetch 및 로컬 동기화
3. #1811 auto-close 여부 확인
4. 필요 시 #1811 수동 close comment 작성
5. 옵션 1 경로이므로 별도 docs-only PR 생성 없음
6. local/remote PR branch 정리
