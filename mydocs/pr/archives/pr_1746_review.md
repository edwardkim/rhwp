# PR #1746 리뷰 — Task #1745 텍스트-anchor 어울림 표 후속 문단 wrap 흡수

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1746 |
| 작성자 | planet6897 |
| base | devel |
| head | pr/devel-1745 |
| 제목 | Task #1745: 텍스트-anchor 어울림 표의 후속 문단 wrap 흡수 실패 수정 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE |
| 로컬 검토 브랜치 | `local/pr_page2_batch_check` |

## 체리픽 검토

- 오래된 순서 누적 검토: #1746 -> #1751 -> #1752 -> #1754
- 적용 커밋: `4cbf94df7958556b9163303643d2cb7a569157e5`
- 로컬 체리픽 커밋: `018bbed6f`
- PR 내부 merge commit `881c5ad48b1c0c59cee81e1ce111b64c4e11d24f`는 체리픽 검토에서 제외했다.
- `upstream/devel` 기준 누적 체리픽 충돌 없음.

## 변경 범위

- `src/renderer/mod.rs`: `text_anchor_square_table_strip`로 텍스트 혼합 anchor 문단의 Square wrap 띠를 표 geometry에서 도출.
- `src/renderer/typeset.rs`: 도출된 wrap 띠를 후속 문단 매칭에 사용하고, 다쪽 분할 표의 wrap 문단 기록을 첫 fragment column으로 소급.
- `src/renderer/layout.rs`: host text 영역과 후속 wrap strip 영역을 분리하고, 분할 표 텍스트 혼합 anchor의 host text 이중 렌더를 방지.
- `samples/task1745/table_text_anchor_wrap.hwp`와 관련 문서 추가.

## 검토 결과

- 표 단독 anchor는 기존 첫 LINE_SEG 기반 경로를 유지하고, 텍스트가 있는 left-aligned Square table anchor만 새 strip 도출 경로를 탄다.
- `record_wrap_around_para`는 현재 column에 첫 fragment가 없을 때만 과거 page/column을 찾아 소급 기록하므로 일반 표 배치와 충돌하지 않는다.
- layout 경로에서 `render_host_text=false`는 분할 표 텍스트 혼합 anchor에 한정되어 host text 중복을 막는다.
- 신규 단위 테스트 2건이 strip 도출 케이스와 기존 경로 보존 케이스를 직접 검증한다.

## 로컬 검증

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib text_anchor_square_table_strip -- --nocapture`: 2 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## PR 내용 기준 검증

- 이 PR의 핵심은 텍스트가 섞인 left-aligned Square table anchor가 후속 문단 wrap strip을 잃지 않는지다.
- 로컬 검증은 새 strip 도출 단위 테스트, 전체 integration test, tracked HWP 샘플의 PDF 기준 visual sweep으로 확인했다.
- 따라서 PR 자체 판정은 `samples/task1745/table_text_anchor_wrap.hwp`를 기준으로 한다.

## 시각 검증

- `samples/task1745/table_text_anchor_wrap.hwp` vs `pdf/table_text_anchor_wrap-2024.pdf`
  - command: `python3 scripts/task1274_visual_sweep.py --key pr1746-table-text-anchor --hwp samples/task1745/table_text_anchor_wrap.hwp --pdf pdf/table_text_anchor_wrap-2024.pdf --pages 1-3 --out output/pr1746-visual --rhwp-bin target/release-test/rhwp --pixel-diff-threshold 32`
  - SVG/PDF pages: 3/3
  - selected pages: 1-3
  - flagged: 0/3
  - review contact sheet: `output/pr1746-visual/pr1746-table-text-anchor/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: p1 4.1875, p2 6.5884, p3 5.92948
- untracked 추가 검증 샘플 `samples/table_text_anchor_wrap.hwpx` vs `pdf/table_text_anchor_wrap-2024.pdf`
  - command: `python3 scripts/task1274_visual_sweep.py --key pr1746-table-text-anchor-hwpx --hwp samples/table_text_anchor_wrap.hwpx --pdf pdf/table_text_anchor_wrap-2024.pdf --pages 1-3 --out output/pr1746-visual-hwpx --rhwp-bin target/release-test/rhwp --pixel-diff-threshold 32`
  - SVG/PDF pages: 3/3
  - selected pages: 1-3
  - flagged: 1/3
  - flagged page: p2 (`frame_overflow_pixels`, `render_tree_frame_tail_overflow`)
  - review page: `output/pr1746-visual-hwpx/pr1746-table-text-anchor-hwpx/review/review_002.png`
  - review contact sheet: `output/pr1746-visual-hwpx/pr1746-table-text-anchor-hwpx/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: p1 4.1572, p2 5.99399, p3 4.99249

해석:
- PR tracked HWP 샘플은 자동 후보가 0건이다.
- untracked HWPX 샘플은 PR에 포함된 변경 파일은 아니며, 보조 검증에서 p2 하단 frame/tail 후보가 남았다. 이는 #1746 merge blocker라기보다 후속 검토 후보로 분리해 판단한다.
- `visual_accuracy_proxy_percent`는 낮게 나오지만, 이는 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다.

## PR 코멘트 처리 시 PNG 위치

- 정상 대표 샘플 1장의 로컬 위치만 기록한다.
- 코멘트 처리 시 제시할 PNG: `mydocs/pr/assets/pr_1746_visual_review_p2.png`
- 코멘트 요지: PR 포함 HWP 샘플의 p2 기준으로 후속 문단 wrap strip이 유지되며, visual sweep 자동 후보는 0/3이다.
- 코멘트 요지에 다음 요청도 포함한다: 이후 시각 대조가 필요한 샘플을 추가할 때는 한컴 2020, 한컴 2024 등 기준 환경에서 저장한 PDF 파일도 함께 업로드해 달라고 요청한다.
- 보조 untracked HWPX p2 후보 이미지는 PR 범위 밖 후속 검토 자료이므로 #1746 머지 코멘트에는 첨부하지 않는다.

## 결론

PR 내용 기준으로는 로컬 기본 검증과 tracked HWP 샘플 기준 visual sweep이 통과해 merge 후보로 판단한다. untracked HWPX p2 frame/tail 후보는 PR 범위 밖 보조 검증 결과이므로, 별도 후속 이슈/PR로 다루는 것이 맞다.

## 후속 처리 결과

- 통합 PR: #1810
- merge commit: `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR 코멘트: https://github.com/edwardkim/rhwp/pull/1746#issuecomment-4866609523
- 원 PR 상태: superseded close 완료
- 관련 이슈: #1745 close 완료, https://github.com/edwardkim/rhwp/issues/1745#issuecomment-4866620601
