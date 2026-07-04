# PR #1890 리뷰 — C1c 차트 스타일 4갭 보정

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1890 |
| 제목 | C1c: 차트 스타일 4갭 보정 (#1431 Track C) |
| 작성자 | johndoekim |
| base | `devel` |
| head | `local/task1882` |
| 최신 head SHA | `6d1eda74a7713e42612333685063bec73fca9ef1` |
| merge commit | `4708f08beecfacfe590af55a53418102a904a757` |
| 규모 | 12 files, +1582 / -111 |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE`, `CLEAN` |
| CI | 문서 작성 시점 최신 head 기준 Build & Test, CodeQL, Canvas visual diff 통과 |

## 변경 범위

- `src/ooxml_chart/{mod,parser,renderer}.rs`
  - OOXML chart 모델에 `has_title_elem`, `auto_title_deleted`, `legend_pos`, `is_3d` 계열 상태 추가
  - 기본 팔레트, 자동 제목, 우측 범례, 축 nice-scale/headroom, 3D 축 정책 보정
- `tests/issue_1882_chart_style_gaps.rs`
  - HWP/HWPX 대표 샘플 기준 자동 제목, 팔레트, 축 라벨, 우측 범례 회귀 가드 추가
- `mydocs/plans`, `mydocs/working`, `mydocs/report`
  - #1882 계획/단계/결과 보고서 추가

## 렌더 영향 및 visual sweep 판정

렌더러와 차트 SVG 출력이 직접 바뀌므로 visual sweep 대상이다. 기준 PDF가 `pdf/chart/**` 아래에 존재하므로 대표 샘플 7개를 기준 PDF와 비교했다.

판정 기준은 `mydocs/manual/pr_review_workflow.md` 의 visual sweep 규칙에 따라 "PDF와 픽셀이 완전히 같은가"가 아니라, PR 이 실제로 약속한 변경 범위가 맞게 반영됐는가로 둔다. 따라서 3D 입체감, marker shape, 정확한 차트 크기/위치처럼 PR 본문이 범위 밖 또는 C2 후보로 분리한 차이는 merge blocker 로 보지 않는다.

실행 명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --out output/pr1890-visual \
  --rhwp-bin target/debug/rhwp \
  --page 1 \
  --file-target chart-column samples/chart/세로막대형/묶은세로막대형.hwpx pdf/chart/세로막대형/묶은세로막대형-2022.pdf \
  --file-target chart-stacked-column samples/chart/세로막대형/누적세로막대형.hwpx pdf/chart/세로막대형/누적세로막대형-2022.pdf \
  --file-target chart-bar samples/chart/가로막대형/묶은가로막대형.hwpx pdf/chart/가로막대형/묶은가로막대형-2022.pdf \
  --file-target chart-scatter samples/chart/분산형/표식만있는분산형.hwpx pdf/chart/분산형/표식만있는분산형-2022.pdf \
  --file-target chart-pie samples/chart/원형/2차원원형.hwpx pdf/chart/원형/2차원원형-2022.pdf \
  --file-target chart-3d-column samples/chart/세로막대형/3차원묶은세로막대형.hwpx pdf/chart/세로막대형/3차원묶은세로막대형-2022.pdf \
  --file-target chart-3d-stacked-column samples/chart/세로막대형/3차원누적세로막대형.hwpx pdf/chart/세로막대형/3차원누적세로막대형-2022.pdf
```

요약 파일:

- `output/pr1890-visual/summary.json`

대표 결과:

| target | 임시 review PNG | 증적 asset | 자동 후보 | pixel match | 내용 픽셀 중심 자동 일치율 보조값 |
|---|---|---|---:|---:|---:|
| chart-column | `output/pr1890-visual/chart-column/review/review_001.png` | `mydocs/pr/assets/pr_1890_chart_column_review.png` | 0 | 97.40799% | 32.58663% |
| chart-stacked-column | `output/pr1890-visual/chart-stacked-column/review/review_001.png` | `mydocs/pr/assets/pr_1890_chart_stacked_column_review.png` | 0 | 97.24257% | 32.75626% |
| chart-bar | `output/pr1890-visual/chart-bar/review/review_001.png` | `mydocs/pr/assets/pr_1890_chart_bar_review.png` | 0 | 96.58727% | 15.11381% |
| chart-scatter | `output/pr1890-visual/chart-scatter/review/review_001.png` | `mydocs/pr/assets/pr_1890_chart_scatter_review.png` | 0 | 99.26329% | 18.15350% |
| chart-pie | `output/pr1890-visual/chart-pie/review/review_002.png` | `mydocs/pr/assets/pr_1890_chart_pie_review.png` | 0 | 98.29476% | 54.72951% |
| chart-3d-column | `output/pr1890-visual/chart-3d-column/review/review_003.png` | `mydocs/pr/assets/pr_1890_chart_3d_column_review.png` | 0 | 95.75220% | 12.51646% |
| chart-3d-stacked-column | `output/pr1890-visual/chart-3d-stacked-column/review/review_003.png` | `mydocs/pr/assets/pr_1890_chart_3d_stacked_column_review.png` | 0 | 97.21643% | 20.40535% |

사람 판정 메모:

- 막대/누적/분산/3D 계열은 제목 표시, 팔레트, 우측 범례, 축 headroom 등 PR 의 핵심 주장 일부가 반영된 것을 확인했다.
- 3D 입체감, marker shape, 정확한 차트 크기/위치, 일부 범례 순서 차이는 PR 본문/보고서상 C2 후보 또는 픽셀 parity 범위 밖으로 분리되어 있어 단독 merge blocker 로 보지는 않는다.
- 원형 `2차원원형`은 기준 PDF의 제목이 `판매`인데 rhwp 결과는 `차트 제목`이다. 이 항목은 PR 본문이 직접 약속한 ① 제목 gap 및 "정답지 PDF와 4갭 정합" 범위 안의 잔여 차이지만, 이번 PR 이 기존 공통 4갭을 크게 개선하고 회귀 가드를 추가했으므로 merge blocker 로 보지 않고 후속 작업 요청으로 분리한다.

## 로컬 검증

새 PR review 시작 전 cargo cache 비대화 영향을 줄이기 위해 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

```bash
git diff --check
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib ooxml_chart
env CARGO_INCREMENTAL=0 cargo test --test issue_1882_chart_style_gaps
env CARGO_INCREMENTAL=0 cargo test --test issue_1431_scatter
env CARGO_INCREMENTAL=0 cargo test --test issue_1453_chart_3d_ofpie_routing
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `cargo test --lib ooxml_chart`: 54 passed
- `cargo test --test issue_1882_chart_style_gaps`: 4 passed
- `cargo test --test issue_1431_scatter`: 1 passed
- `cargo test --test issue_1453_chart_3d_ofpie_routing`: 2 passed
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과. `svg_snapshot` integration test 포함 실행 확인

## 주요 검토 결과

### 1. 원형 차트 제목 정책이 기준 PDF와 다름

PR 은 `c:title` 요소가 있고 `autoTitleDeleted=0` 이며 명시 텍스트가 없으면 `"차트 제목"`을 렌더하도록 구현했다.

- `src/ooxml_chart/parser.rs:272`-`276`: 모든 `<c:title>`에서 `has_title_elem = true`
- `src/ooxml_chart/renderer.rs:100`-`104`: 명시 제목이 없으면 `"차트 제목"` fallback
- `tests/issue_1882_chart_style_gaps.rs:30`-`39`: `원형/2차원원형`도 `"차트 제목"`을 기대

하지만 기준 PDF `pdf/chart/원형/2차원원형-2022.pdf` 와 visual sweep 결과는 제목 `판매`를 보여준다.

- 시각 자료: `output/pr1890-visual/chart-pie/review/review_002.png`
- 샘플 XML 확인: `samples/chart/원형/2차원원형.hwpx` 의 `Chart/chart1.xml` 에서 `<c:title>` 텍스트는 없지만 첫 series 이름이 `판매`
- 코퍼스 확인: 원형 계열의 첫 series 이름은 `판매`, 막대/라인 계열은 `계열 1`, 분산형은 `Y1 값`

#1882 의 완료 기준이 "4갭 모두 정답지 PDF와 시각 정합"인 만큼, 원형 제목은 이번 PR 범위 안에서 발견된 잔여 차이다. 다만 이번 PR 은 공통 제목 placeholder, 팔레트, 우측 범례, 축 headroom/3D 축 정책을 넓은 범위에서 개선하고 테스트를 추가했으므로 merge 로 수용한다. 원형 차트에서 첫 series name 을 제목 fallback 으로 쓰는지, 또는 차트 종류별 자동 제목 정책이 다른지 확인하는 작업은 후속으로 요청한다.

### 2. 보고서 상태와 PR 본문이 일부 어긋남

`mydocs/report/task_m100_1882_report.md:6` 은 "작업지시자 시각판정 대기"라고 되어 있으나, PR 본문은 "작업지시자 studio(WASM) 판정 완료"라고 쓴다. 또한 위 원형 제목 차이 때문에 "대표 확인 — 4갭 전부 정답지 정합"(`mydocs/report/task_m100_1882_report.md:63`-`65`)은 현재 로컬 재검증 결과와 맞지 않는다.

이 문서는 PR 에 포함되는 기록물이므로 보완이 좋다. 다만 PR 의 코드 변경과 테스트는 통과했고, 잔여 차이를 후속 작업으로 요청하는 방향이면 merge blocker 로 보지는 않는다.

### 3. PR 본문은 `Refs #1882` 이며 auto-close 되지 않음

PR 본문이 현재 `Refs #1882` 이므로 merge 후 auto-close 는 기대하지 않는다. 원형 제목 정책의 잔여 차이를 후속 요청으로 남기는 결론이므로 #1882 는 후속 코멘트 후 open 유지가 적절하다.

## 최종 권고

merge 로 수용하고, 원형 차트 제목 정책과 보고서 문구는 후속 작업으로 요청하는 결론이다.

후속 요청:

1. 원형 계열에서 기준 PDF가 `판매`를 제목으로 렌더하는 원인을 반영한다.
2. `tests/issue_1882_chart_style_gaps.rs` 의 원형 제목 기대값을 기준 PDF와 맞게 보정한다.
3. `mydocs/report/task_m100_1882_report.md` 의 시각판정 상태와 "4갭 전부 정합" 표현을 재검증 결과에 맞춰 갱신한다.
4. 수정 후 PR head 최신 커밋 기준 GitHub Actions와 로컬 focused test, visual sweep 을 다시 확인한다.

merge 및 후속 처리:

- 최신 PR head 기준 GitHub Actions 통과 확인
- PR #1890 merge 완료: `4708f08beecfacfe590af55a53418102a904a757`
- 기준 PDF 대비 대표 visual sweep 결과와 잔여 차이 확인
- 위 visual sweep 증적 asset 을 후속 문서/asset PR 에 포함
- #1882 는 후속 코멘트 후 open 유지
