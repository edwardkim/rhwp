# PR #1954 리뷰 - 단일 시리즈 자동 제목 fallback

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1954 |
| 제목 | C1c 후속: 단일 시리즈 자동 제목 fallback (#1882 잔여) |
| 작성자 | johndoekim |
| base | `devel` |
| head | `local/task1882_v2` |
| 문서 작성 시점 head SHA | `297141762fbd0e342b616f31a94278dd605e60a2` |
| merge commit | `79b4ce216039a08dd0d1957facf027e714f425e0` |
| mergedAt | `2026-07-05T14:43:19Z` |
| 규모 | 7 files, +226 / -7 |
| mergeable | merge 전 최종 확인: `MERGEABLE` |
| CI | 문서 작성 시점 head 기준 CI/CodeQL/Render Diff 관련 checks 모두 통과 |
| closingIssuesReferences | 비어 있음. merge 후 #1882 close/후속 코멘트는 별도 확인 필요 |

## 변경 범위

- `src/ooxml_chart/renderer.rs`
  - 자동 제목 우선순위를 `명시 제목 -> 단일 시리즈 이름 -> "차트 제목"`으로 확장했다.
  - 기존 `has_title_elem && !auto_title_deleted` gate는 유지한다.
  - 단위 테스트로 단일 시리즈 이름, 빈 이름 fallback, 명시 제목 우선, `autoTitleDeleted=1`, 다계열 placeholder를 확인한다.
- `tests/issue_1882_chart_style_gaps.rs`
  - 원형 단일 시리즈 `판매`와 특이케이스 가로막대 `계열 1` 제목을 검증한다.
  - 다계열 chart의 `"차트 제목"` placeholder 유지도 계속 검증한다.
- `samples/chart/특이케이스/`, `pdf/chart/특이케이스/`
  - 단일 시리즈 가로막대 HWP/HWPX 샘플과 한컴 기준 PDF를 추가했다.
- `mydocs/plans/task_m100_1882_v2.md`, `mydocs/report/task_m100_1882_v2_report.md`
  - #1882 잔여 자동 제목 fallback 근거와 검증 결과를 기록한다.

## 렌더 영향 및 visual sweep 판정

`src/ooxml_chart/renderer.rs`와 샘플/PDF가 바뀌므로 visual sweep 대상이다. PR이 추가한 단일 시리즈 가로막대 HWP/HWPX 샘플을 동일 기준 PDF와 1쪽 비교했다.

실행 명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1954-hwp \
  --hwp "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwp" \
  --pdf "pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf" \
  --page 1 \
  --out output/pr1954-visual-hwp

python3 scripts/task1274_visual_sweep.py \
  --key pr1954-hwpx \
  --hwp "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx" \
  --pdf "pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf" \
  --page 1 \
  --out output/pr1954-visual-hwpx
```

산출물:

| 입력 | compare | overlay | review | asset |
|---|---|---|---|---|
| HWP | `output/pr1954-visual-hwp/pr1954-hwp/compare/compare_001.png` | `output/pr1954-visual-hwp/pr1954-hwp/overlay/overlay_001.png` | `output/pr1954-visual-hwp/pr1954-hwp/review/review_001.png` | `mydocs/pr/assets/pr_1954_hwp_single_series_title_review_001.png` |
| HWPX | `output/pr1954-visual-hwpx/pr1954-hwpx/compare/compare_001.png` | `output/pr1954-visual-hwpx/pr1954-hwpx/overlay/overlay_001.png` | `output/pr1954-visual-hwpx/pr1954-hwpx/review/review_001.png` | `mydocs/pr/assets/pr_1954_hwpx_single_series_title_review_001.png` |

visual sweep 결과:

- HWP: SVG/PDF 페이지 수 1/1, 자동 후보 `flagged=0/1`, pixel match `97.37344%`, 내용 픽셀 중심 자동 일치율 보조값 `42.02253%`
- HWPX: SVG/PDF 페이지 수 1/1, 자동 후보 `flagged=0/1`, pixel match `97.37344%`, 내용 픽셀 중심 자동 일치율 보조값 `42.02253%`

사람 판정 메모:

- HWP/HWPX 모두 rhwp 출력 제목이 `계열 1`로 표시된다.
- 기준 PDF도 같은 위치의 제목이 `계열 1`이므로 PR의 핵심 수정 의도와 일치한다.
- `visual_accuracy_proxy_percent`는 chart 축/막대 raster 위치 차이와 잉크 차이를 크게 반영해 낮지만, 이번 PR의 판단 기준인 자동 제목 텍스트 자체는 기준 PDF와 맞는다.

## 로컬 검증

검토 시작 시 cargo cache 비대화 영향을 줄이기 위해 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다. 이후 PR head가 갱신되어 최신 head `297141762fbd0e342b616f31a94278dd605e60a2` 기준으로 다시 fetch하고 순차 재검증했다.

```bash
git fetch upstream +pull/1954/head:local/pr1954
git switch -C review/pr1954 local/pr1954
git merge upstream/devel --no-commit --no-ff
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo test --lib ooxml_chart
env CARGO_INCREMENTAL=0 cargo test --test issue_1882_chart_style_gaps
env CARGO_INCREMENTAL=0 cargo test --test issue_1431_scatter
env CARGO_INCREMENTAL=0 cargo test --test issue_1453_chart_3d_ofpie_routing
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- `git merge upstream/devel --no-commit --no-ff`: `Already up to date`, 충돌 없음
- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test --lib ooxml_chart`: 56 passed
- `cargo test --test issue_1882_chart_style_gaps`: 4 passed
- `cargo test --test issue_1431_scatter`: 1 passed
- `cargo test --test issue_1453_chart_3d_ofpie_routing`: 2 passed
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과. `svg_snapshot.rs` 8 tests 포함 통과

## 검토 결과

### 1. 자동 제목 fallback 규칙은 기존 model/parser에 무리 없이 들어간다

`OoxmlSeries.name`은 이미 파싱되어 있고, 이번 변경은 renderer의 `effective_title` 결정 단계만 조정한다. `has_title_elem && !auto_title_deleted` gate를 유지하므로 제목 요소가 없거나 자동 제목 삭제가 명시된 chart에는 새 fallback이 적용되지 않는다.

### 2. 단일/다계열 경계가 테스트로 가드된다

단일 시리즈는 시리즈 이름을 제목으로 렌더하고, 다계열은 기존 placeholder `"차트 제목"`을 유지한다. 명시 제목이 있으면 시리즈 이름보다 우선하고, 빈 시리즈 이름은 placeholder로 떨어진다. 이 경계가 단위 테스트와 fixture 통합 테스트 양쪽에 들어가 있어 회귀 감지가 가능하다.

### 3. PR 범위 밖 항목은 문서상 분리되어 있다

보고서는 축 간격, 범례 순서, stock/3D 계열을 범위 밖 후속 후보로 분리한다. 이번 PR은 #1882 잔여 중 단일 시리즈 자동 제목 fallback만 수용하는 범위로 보는 것이 맞다.

## 최종 권고

merge 완료로 정리한다.

- merge commit: `79b4ce216039a08dd0d1957facf027e714f425e0`
- merge 방식: admin merge
- merge 시각: `2026-07-05T14:43:19Z`

후속:

- `closingIssuesReferences`가 비어 있으므로 #1882 상태를 확인하고, 필요하면 #1954에서 단일 시리즈 자동 제목 fallback 항목이 처리됐다는 후속 코멘트를 남긴다.
- review 문서와 visual asset은 merge 후 절차에 따라 archive/docs-only 경로로 정리한다.
