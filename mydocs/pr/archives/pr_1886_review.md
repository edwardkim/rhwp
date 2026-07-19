# PR #1886 리뷰 — #1770 HWPX-origin 마커 기반 변환 HWP pagination 자기정합

- PR: #1886 `Issue #1770: HWPX-origin 마커로 변환-HWP pagination 자기정합 (옵션 1)`
- 작성자: @planet6897
- 기준 브랜치: `devel`
- head branch: `fix/1770-hwpx-origin-tolerance`
- 문서 작성 시점 참고 head: `dd14511d3353abb3ad0c8862d9697f52a62fe4e8`
- 원 코드 PR merge commit: `86c3addd92cfb43fa1aaa487fdde6eb9aaaefa7c`
- 관련 이슈: #1770
- 검토 경로: maintainer 일반 review + 옵션 2 후속 문서/asset PR
- 원 코드 PR merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인 완료

## 변경 요약

HWPX 를 rhwp 로 HWP 변환하면 LINE_SEG 를 verbatim 직렬화하므로 산출 HWP5 의 IR 은 HWPX 시멘틱을
그대로 유지한다. 그런데 변환 HWP 를 다시 파싱하면 `source_format == Hwp` 로 인식되어 RowBreak 분할
tolerance 같은 HWPX 전용 렌더 분기가 꺼지고, 같은 IR 이 다른 쪽수로 조판되는 문제가 있었다.

이번 PR 은 HWPX 에서 HWP 로 변환할 때 `/RhwpHwpxOrigin` 마커 스트림을 넣고, HWP 파서가 이를 감지해
`Document::is_hwpx_variant` 를 세우도록 한다. 이후 pagination/rendering 경로는
`source_format == Hwpx || is_hwpx_variant` 를 HWPX source 로 해석한다.

## 첨부 PDF 반영

PR comment 에 첨부된 한컴 기준 PDF 를 로컬 `pdf/` 디렉터리에 복사해 검증 기준으로 사용했다.

- comment: https://github.com/edwardkim/rhwp/pull/1886#issuecomment-4881027960
- 원본 첨부: `issue1770_rowsplit_tolerance-2024.pdf`
- 로컬 경로: `pdf/issue1770_rowsplit_tolerance-2024.pdf`
- PDF 메타:
  - Creator: `Hwp 2024 13.0.0.3622`
  - Producer: `Hancom PDF 1.3.0.550`
  - Pages: 4
  - Page size: A4

옵션 2 처리이므로 이 PDF 는 원 PR head 에 push 하지 않는다. #1886 merge 후 별도 docs-only PR 에서
review 문서, 대표 visual asset, 오늘할일과 함께 포함한다.

## 변경 범위 검토

변경 파일:

- `samples/issue1770_rowsplit_tolerance.hwpx`
- `src/document_core/converters/hwpx_to_hwp.rs`
- `src/document_core/queries/rendering.rs`
- `src/model/document.rs`
- `src/parser/hwpx/mod.rs`
- `src/parser/mod.rs`
- `src/serializer/cfb_writer/tests.rs`
- `tests/issue_1770_hwpx_origin_marker.rs`

코드 검토 결과:

- HWPX source 에서만 `/RhwpHwpxOrigin` 마커를 추가하며, HWP/HWP3 native 입력에는 마커를 추가하지 않는다.
- `extra_streams` 보존 경로를 사용하므로 다중 roundtrip 에서 마커가 1개로 유지되는지 테스트가 있다.
- HWP 파서가 마커를 감지해 `Document::is_hwpx_variant` 를 설정한다.
- pagination pass 와 render query pass 양쪽 모두 같은 조건으로 HWPX source 여부를 계산한다.
- native HWP5 오인 방지 테스트가 포함되어 있다.

Blocking finding 없음.

## 렌더 영향 판정

pagination, RowBreak tolerance, page count, HWPX/HWP 변환 roundtrip 렌더링에 영향을 주므로 visual sweep 대상이다.

검증 기준:

- HWPX 원본: `samples/issue1770_rowsplit_tolerance.hwpx`
- 변환 HWP: `output/pr1886-issue1770/converted/issue1770_rowsplit_tolerance.hwp`
- 기준 PDF: `pdf/issue1770_rowsplit_tolerance-2024.pdf`

## 로컬 검증

target 정리 후 macOS 로컬에서 실행했다.

- `env CARGO_INCREMENTAL=0 cargo build`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1770_hwpx_origin_marker -- --nocapture`: 통과, 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter -- --nocapture`: 통과, 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`: 통과, 20 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwp5_roundtrip_baseline -- --nocapture`: 통과, 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwpx_to_hwp_adapter -- --nocapture`: 통과, 50 passed / 15 ignored
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

추가 render-diff:

- `target/debug/rhwp render-diff samples/issue1770_rowsplit_tolerance.hwpx --via hwp`
- 결과: `status: PASS`, 페이지 수 A=4/B=4, 최대 변위 0.00px, 구조 불일치 0
- 동일 위치의 `LAYOUT_OVERFLOW` 로그가 출력되지만 A/B 모두 같은 구조로 비교되어 render-diff 는 PASS 다.

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 한컴 PDF 와 비교했다.

### HWPX 원본 vs 기준 PDF

- command:
  `python3 scripts/task1274_visual_sweep.py --key pr1886-issue1770 --hwp samples/issue1770_rowsplit_tolerance.hwpx --pdf pdf/issue1770_rowsplit_tolerance-2024.pdf --out output/pr1886-issue1770`
- SVG/PDF pages: 4/4
- flagged: 0/4
- p3 compare: `output/pr1886-issue1770/pr1886-issue1770/compare/compare_003.png`
- p3 overlay: `output/pr1886-issue1770/pr1886-issue1770/overlay/overlay_003.png`
- p3 review: `output/pr1886-issue1770/pr1886-issue1770/review/review_003.png`
- p3 asset 후보: `mydocs/pr/assets/pr_1886_issue1770_hwpx_review_p003.png`
- p3 visual_accuracy_proxy_percent: 15.84273

### 변환 HWP vs 기준 PDF

- 변환 command:
  `target/debug/rhwp convert samples/issue1770_rowsplit_tolerance.hwpx output/pr1886-issue1770/converted/issue1770_rowsplit_tolerance.hwp`
- sweep command:
  `python3 scripts/task1274_visual_sweep.py --key pr1886-issue1770-converted --hwp output/pr1886-issue1770/converted/issue1770_rowsplit_tolerance.hwp --pdf pdf/issue1770_rowsplit_tolerance-2024.pdf --out output/pr1886-issue1770-converted`
- SVG/PDF pages: 4/4
- flagged: 0/4
- p3 compare: `output/pr1886-issue1770-converted/pr1886-issue1770-converted/compare/compare_003.png`
- p3 overlay: `output/pr1886-issue1770-converted/pr1886-issue1770-converted/overlay/overlay_003.png`
- p3 review: `output/pr1886-issue1770-converted/pr1886-issue1770-converted/review/review_003.png`
- p3 asset 후보: `mydocs/pr/assets/pr_1886_issue1770_converted_hwp_review_p003.png`
- p3 visual_accuracy_proxy_percent: 15.84273

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 15.84%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

시각 판정:

- PR 본문에서 문제 삼은 4쪽/5쪽 page count divergence 는 HWPX 원본과 변환 HWP 모두 4쪽으로 정리됐다.
- p3 STRUCT drift 후보도 visual sweep flagged 0/4 및 render-diff 0.00px 로 해소된 것으로 판단한다.
- overlay 차이는 폰트 raster/anti-aliasing 중심으로 보이며, PR 범위의 page count/RowBreak origin 문제에 대한 blocker 로 보지 않는다.

## GitHub CI

merge 전 최신 head 기준 확인 결과:

- `Build default-feature tests`: pass
- `Native Skia tests`: pass
- `Canvas visual diff`: pass
- `CodeQL`: pass
- `WASM Build`: skipped
- `mergeable`: `MERGEABLE`
- `mergeStateStatus`: `CLEAN`

## 리뷰 결론

#1886 원 코드 PR 은 merge 완료.

단, 옵션 2 처리로 한다. 즉 #1886 원 PR 은 코드/샘플 변경만 merge 하고, 아래 검증 기록/자산은 merge 후
별도 docs-only PR 로 반영한다.

후속 docs-only PR 포함 대상:

- `pdf/issue1770_rowsplit_tolerance-2024.pdf`
- `mydocs/pr/archives/pr_1886_review.md`
- `mydocs/pr/archives/pr_1886_review_impl.md`
- `mydocs/pr/assets/pr_1886_issue1770_hwpx_review_p003.png`
- `mydocs/pr/assets/pr_1886_issue1770_converted_hwp_review_p003.png`
- `mydocs/orders/{yyyymmdd}.md` 오늘할일 갱신

docs-only PR merge 후에는 `mydocs/manual/pr_review_workflow.md` 에 따라 #1770 에 asset 링크 포함 후속 코멘트와
close 처리를 수행하고, #1886 PR comment 및 branch/worktree 정리를 수행한다.
