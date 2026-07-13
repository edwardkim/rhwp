# PR #2231 리뷰 - HWPX 비정상 줄 metrics 재조판

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/2231 |
| 제목 | `task 2093: HWPX 비정상 줄 metrics 재조판` |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_2093_stale_hwpx_line_metrics` |
| 관련 이슈 | https://github.com/edwardkim/rhwp/issues/2093 |
| 규모 | 문서 작성 시점 참고값: 8 files, +406/-77 |
| reviewer | `edwardkim` 지정 |
| mergeable | 문서 작성 시점 참고값: `BLOCKED` (필수 CI 대기) |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 관련 이슈와 판단

#2093의 실문서 `1192000_hydrogen_policy_research.hwp` 17→16쪽 정합은 이미 `devel`에
반영돼 있다. 다만 원본 합성 fixture `saved_single_line_spacing_after.hwpx`는 HWP 2020/2022가
1쪽으로 재조판하는 반면 rhwp는 저장 `LINE_SEG`를 917.3px로 신뢰해 2쪽으로 보였다.

이 PR은 fixture를 PDF oracle과 같은 1쪽으로 고친다. 원본 fixture와 기준 PDF는 수정하지
않고, 렌더/조판 경로의 저장 metrics 해석만 보정한다.

## 변경 범위

- `SectionDef`와 `ColumnDef`가 함께 있는 HWPX 구역 첫 순수 텍스트 줄만 재조판 후보로
  한정했다. 표·그림·글상자·필드와 일반 문단은 대상이 아니다.
- 저장 `line_height`와 `text_height`가 예상 줄 advance의 40배를 모두 넘을 때만 글꼴 크기와
  줄간격으로 line height, spacing, baseline을 복원한다.
- 재조판 뒤에도 남는 stale `vertical_pos`를 순차 흐름으로 접고, 후속 near-top reset이
  빈 새 페이지를 만들지 않게 한다.
- #2098의 의도된 55000HU 큰 구역 첫 줄은 보존하고, #1692 HWPX 미주 페이지 범위도 보호한다.

## 렌더 영향과 Visual Sweep

renderer/typeset/layout 경로와 페이지 수가 바뀌므로 visual sweep 대상이다.

- 원본: `samples/task2093/saved_single_line_spacing_after.hwpx`
- HWP 2020 MCP 기준 PDF: `pdf/task2093/saved_single_line_spacing_after-2020.pdf`
- 페이지 수: rhwp SVG 1쪽 / 기준 PDF 1쪽
- 자동 후보: `flagged=0/1`, frame/tail/order 후보 0건
- pixel match: `99.75843%`
- visual accuracy proxy: `10.51101%`
  - serif fallback과 한컴 PDF 글꼴 외형 차이에 따른 보조값이므로 페이지 수와 문단 순서
    판정과 분리해 해석했다.

증적:

- `mydocs/pr/assets/pr_2231/task2093_original_p001_review.png`
- `mydocs/pr/assets/pr_2231/task2093_original_overlay_metrics.json`

## 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --quiet`
  - 2200 passed, 0 failed, 7 ignored
- 최신 rebase 후 focused regression:
  - `issue_2093_saved_single_line_spacing_after`
  - `issue_2093_1192000_real_doc_pin`
  - `issue_2098_margin_boundary_split`
  - `issue_1692`
  - 모두 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `cargo fmt --check`, `git diff --check` 통과
- 작업지시자 브라우저 검증 완료

전체 회귀는 사용자 지시에 따라 이미 통과한 결과를 사용했으며, rebase 후에는 영향권 회귀와 WASM build를
재확인했다.

## 운영 기록

collaborator self PR 경로에 따라 `mydocs/orders/20260712.md`에 PR #2231 처리 내역을 같은
PR head에 포함했다.

## 최종 권고

PR head 최신 GitHub Actions 통과와 작업지시자 승인 후 merge를 권고한다. merge 후
[https://github.com/edwardkim/rhwp/issues/2093](https://github.com/edwardkim/rhwp/issues/2093)의
자동 close를 확인하고, 기준 PDF와 visual review asset 링크를 포함한 후속 코멘트를 남긴다.
