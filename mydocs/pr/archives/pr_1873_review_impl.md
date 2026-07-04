# PR #1873 Review Impl — HWP3 LINE_SEG rewind 페이지 경계 반영

## Stage 1. 이슈 및 기존 상태 확인

완료.

- 이슈 #1695 는 SO-SUEOP HWP3 샘플의 `LINE_SEG` `vpos` reset/rewind 가 page/column boundary hint 로
  처리되지 않는 문제를 다룬다.
- 기존 `--respect-vpos-reset` 옵션은 legacy `Paginator` 경로에만 의미가 있고, 기본 `TypesetEngine` 경로에는
  실질 영향이 없음을 확인했다.
- devel 기준 `samples/SO-SUEOP.hwp` 는 46페이지였으나, 후보 문단의 내부 rewind 경계가 page split 으로
  반영되지 않아 하단 clipping/overflow 후보가 남아 있었다.

## Stage 2. 구현

완료.

- `TypesetState` 에 원본 HWP3 source 여부를 추가했다.
- `DocumentCore` 의 `source_format == Hwp3` 정보를 typeset 경로로 전달했다.
- `internal_vpos_page_break_line` 의 일반 rewind 감지를 원본 HWP3 텍스트 문단으로 제한했다.
- 기존 sample16 전용 reset 예외는 유지했다.
- HWP3 변환본에는 새 일반 rewind 규칙을 적용하지 않도록 좁혀 sample16-hwp5-2022 page count 회귀를 막았다.
- `tests/issue_1695.rs` 를 추가해 page count, split 위치, body bottom overflow, continuation 위치를 고정했다.

## Stage 3. 로컬 검증

완료.

순차 실행으로 확인했다.

```bash
env CARGO_INCREMENTAL=0 cargo test --test issue_1695 -- --nocapture
env CARGO_INCREMENTAL=0 cargo test --test issue_1035_alignment
env CARGO_INCREMENTAL=0 cargo test --test issue_1692
env CARGO_INCREMENTAL=0 cargo test --test issue_1375_endnote_rewind_column_overflow
env CARGO_INCREMENTAL=0 cargo test --test issue_1733
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
git diff --check
```

결과:

- 전체 통과
- 첫 full release-test 에서 sample16-hwp5-2022 64→65 page count 회귀가 확인됐고, 일반 rewind 규칙을 원본
  HWP3 source 전용으로 축소한 뒤 focused 및 full release-test 를 재통과했다.

## Stage 4. Visual sweep

완료.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1695-so-sueop-hwp \
  --hwp samples/SO-SUEOP.hwp \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --out output/issue1695_after_visual \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG 46 pages / render-tree 46 pages / 기준 PDF 46 pages
- 자동 flagged pages 0 / 46
- 대표 review PNG 는 p006, p024, p029, p044 를 선택했다.
- 대표 PNG 는 `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p*.png` 로 복사했다.

## Stage 5. PR 생성 및 update branch

완료.

- 커밋: `e937e29ec1ce98539752cd2d73b8128e2ea3c291`
- PR: #1873
- 최초 PR head 는 `BEHIND` 였다.
- `upstream/devel` 기준 rebase 후 `6fd39bb70076fc85e7d7445f357edcc7a478d147` 로 force-with-lease push 했다.
- 이전 SHA `e937e29ec1ce98539752cd2d73b8128e2ea3c291` 의 `CI`, `CodeQL`, `Render Diff` run 은 force-cancel 완료했다.

## Stage 6. Option 1 문서/asset 준비

완료.

- `mydocs/pr/archives/pr_1873_review.md`
- `mydocs/pr/archives/pr_1873_review_impl.md`
- `mydocs/orders/20260704.md`
- `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p006.png`
- `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p024.png`
- `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p029.png`
- `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p044.png`

## Stage 7. CI 및 merge 후속 확인

완료.

- option 1 문서/asset/오늘할일 커밋: `3e64bbbfdc1fb3d378ae398906880ca76dc91941`
- 최종 head 기준 GitHub Actions:
  - `CI`: 통과
  - `CodeQL`: 통과
  - `Render Diff`: 통과
- PR #1873 merge 완료: `e5ff8ab9311ff0a22c127f5a5081c4f1cacc421d`
- #1695 auto-close 완료: `2026-07-03T16:32:57Z`

## 남은 작업

- 후속 기록 fast-pass PR 생성 및 merge
- #1873 PR comment 작성
- `task/m100-1695-vpos-reset`, `task/m100-1695-post-merge-docs` 브랜치 정리
