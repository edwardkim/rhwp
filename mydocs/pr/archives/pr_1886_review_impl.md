# PR #1886 처리 계획 — #1770 HWPX-origin 마커 기반 변환 HWP pagination 자기정합

## 대상

- PR: #1886
- 작성자: @planet6897
- 관련 이슈: #1770
- 기준 브랜치: `devel`
- head branch: `fix/1770-hwpx-origin-tolerance`
- 문서 작성 시점 참고 head: `dd14511d3353abb3ad0c8862d9697f52a62fe4e8`
- 원 코드 PR merge commit: `86c3addd92cfb43fa1aaa487fdde6eb9aaaefa7c`
- 처리 경로: maintainer 일반 review + 옵션 2 후속 docs-only PR

## Stage

### Stage 1. PR 메타 및 reviewer 지정

완료.

- base: `devel`
- PR author: @planet6897
- reviewer: @jangster77 지정
- PR 상태 작성 시점 참고값: `MERGEABLE` / `CLEAN`

### Stage 2. 첨부 PDF 확보

완료.

- PR comment 첨부 PDF 를 `pdf/issue1770_rowsplit_tolerance-2024.pdf` 로 복사했다.
- `pdfinfo` 기준 Hancom PDF 2024 생성물, 4페이지 A4임을 확인했다.
- 옵션 2 처리이므로 PDF 는 원 PR head 에 push 하지 않고, merge 후 docs-only PR 에 포함한다.

### Stage 3. 코드 검토

완료.

- HWPX source 에서만 `/RhwpHwpxOrigin` 마커를 추가한다.
- HWP parser 가 unknown root stream 을 `extra_streams` 로 수집하고, 마커를 감지해 `Document::is_hwpx_variant` 를 설정한다.
- pagination/rendering 양쪽에서 `source_format == Hwpx || is_hwpx_variant` 를 같은 의미로 사용한다.
- native HWP5 오인 방지 및 2-round idempotent 테스트가 포함됐다.

Blocking finding 없음.

### Stage 4. 로컬 검증

완료.

- `env CARGO_INCREMENTAL=0 cargo build`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1770_hwpx_origin_marker -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwp5_roundtrip_baseline -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwpx_to_hwp_adapter -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

### Stage 5. 시각 검증

완료.

- HWPX 원본 vs 기준 PDF: 4/4페이지, visual sweep flagged 0/4
- 변환 HWP vs 기준 PDF: 4/4페이지, visual sweep flagged 0/4
- `render-diff --via hwp`: PASS, page count 4/4, max displacement 0.00px
- 대표 asset 후보:
  - `mydocs/pr/assets/pr_1886_issue1770_hwpx_review_p003.png`
  - `mydocs/pr/assets/pr_1886_issue1770_converted_hwp_review_p003.png`

### Stage 6. merge 전 조건

완료.

1. PR head 최신 상태 재확인: `dd14511d3353abb3ad0c8862d9697f52a62fe4e8`
2. GitHub Actions 최신 head 기준 통과 재확인: 완료
3. 작업지시자 merge 승인 확인: 완료
4. admin merge: `86c3addd92cfb43fa1aaa487fdde6eb9aaaefa7c`

### Stage 7. merge 후 옵션 2 후속 처리

대기.

1. PR #1886 merge: 완료
2. `devel` 을 `upstream/devel` 로 fast-forward sync: 완료
3. #1770 close 여부 확인: OPEN, docs-only PR merge 후 asset 링크와 함께 close/comment 예정
4. docs-only PR 생성
   - `pdf/issue1770_rowsplit_tolerance-2024.pdf`
   - `mydocs/pr/archives/pr_1886_review.md`
   - `mydocs/pr/archives/pr_1886_review_impl.md`
   - `mydocs/pr/assets/pr_1886_issue1770_hwpx_review_p003.png`
   - `mydocs/pr/assets/pr_1886_issue1770_converted_hwp_review_p003.png`
   - `mydocs/orders/{yyyymmdd}.md`
5. docs-only PR merge 후 #1770 issue comment + close
6. docs-only PR merge 후 원 PR comment 에 검증 결과와 asset 링크 기록
7. PR branch/worktree 정리
