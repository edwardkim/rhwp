# PR #1744 처리 계획 — RowBreak continuation 부분 보정

## 대상

- PR: #1744
- 작성자: @planet6897
- 관련 이슈: #1728
- 문서 작성 시점 PR head: `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5`
- merge commit: `4da379b4385570be5b73d75fa9facc41bf5717dd`
- 처리 결과: GitHub Actions 통과 후 admin merge 완료

## 커밋

1. `c10c8e4854eb06d1a1d742f513d88e019edc4f71`
   - `Task #1728: 자동 쪽번호 세로 위치 + RowBreak 셀-내 continuation 상단 space-before 정합`
   - contributor 원 변경 커밋
2. `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5`
   - `Merge branch 'devel' into pr/devel-1728`
   - 메인터너 conflict 해소 merge commit

## Stage 1. PR 메타 확인

완료.

- base branch: `devel`
- draft: false
- maintainerCanModify: true
- 최초 상태: `CONFLICTING` / `DIRTY`
- conflict 해소 후 상태: `MERGEABLE` / `CLEAN`
- 규모: 8 files, +219/-5

## Stage 2. conflict 해소

완료.

- 충돌 파일: `src/renderer/layout.rs`
- 충돌 원인:
  - #1744가 `keep_continuation_column_top_spacing_before` 필드를 추가
  - #1743이 같은 위치에 `hwpx_page_preview` 필드를 추가
- 해소 내용:
  - 두 필드를 모두 유지
  - 생성자 초기화도 두 필드를 모두 유지
  - `layout.rs` conflict marker 제거
- push:
  - `planet6897/rhwp:pr/devel-1728`
  - 새 head `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5`

## Stage 3. 변경 내용 검토

완료.

- footer 쪽번호는 `margin_footer` 기반으로 이동하며 aift처럼 `margin_footer == margin_bottom`인 샘플은 불변이다.
- continuation spacing flag는 `table_partial.rs`의 단일 호출 전후 set/reset으로 scope가 좁다.
- 1x1 linear 셀 제외 조건은 `issue_rowbreak_chart_overlap` 회귀 테스트와 맞물려 있다.
- #1728 전체 해결이 아니라 footer y와 giant continuation 상단 spacing 갈래의 부분 해결이다.

## Stage 4. 로컬 검증

완료.

- `cargo fmt --all -- --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib test_634_aift`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692 issue_1692_so_sueop_hwpx_title_ole_renders_from_embedded_preview`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test svg_snapshot form_002_page_0`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_rowbreak_chart_overlap`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_874_ktx_toc_page_number_right_align`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과
- `git diff --cached --check`: 통과

## Stage 5. GitHub Actions 확인

완료.

최신 PR head 기준으로 다음을 확인했다.

- GitHub Actions required checks 전체 통과
- `MERGEABLE` + `CLEAN`
- 작업지시자 merge 승인

## merge 후 후속 처리 결과

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. merge 직전 최신 head SHA와 GitHub Actions 확인 완료
2. PR #1744 admin merge 완료: `4da379b4385570be5b73d75fa9facc41bf5717dd`
3. `devel` fast-forward sync 완료
4. PR 감사 코멘트 작성 완료: https://github.com/edwardkim/rhwp/pull/1744#issuecomment-4865221629
5. #1728은 부분 해결이므로 open 유지 확인 및 남은 갈래 코멘트 작성 완료: https://github.com/edwardkim/rhwp/issues/1728#issuecomment-4865223824
6. 리뷰 문서 archive 이동 및 오늘할일 갱신은 별도 문서-only PR로 반영

## 후속 코멘트 요지

- conflict는 #1743의 `hwpx_page_preview`와 #1744의 continuation spacing flag가 같은 구조체 위치를 건드려 발생했고,
  두 필드를 모두 유지하는 방식으로 해소했다.
- footer 쪽번호 y와 giant continuation 상단 spacing 갈래는 검증 후 merge했다.
- #1728의 scattered 하단 over-fill / 남은 PDF 하단 차이는 open으로 계속 추적한다.
