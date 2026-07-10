# PR #2140 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2140`
- 제목: `C1d: 라인 누적(stacked/percentStacked) + 표식 렌더 (#1431 Track C)`
- base: `devel` `595f1a486a32503149698615727bd634293d840c`
- head: `local/task2129` `66660e99c2ba6703bd5067c4cbb4c0cf10403dcc`
- merge commit: `38d1b5a2d7a7b6f068f6d436b94066dcf48497dd`
- 변경량: 11 files, +1388/-30

## 사전 상태

- `gh pr edit 2140 --repo edwardkim/rhwp --add-reviewer jangster77` 수행 완료.
- `gh pr checks 2140 --repo edwardkim/rhwp` 결과 CI pass.
- 로컬 `upstream/devel`과 GitHub PR base가 달라 `595f1a486...local/pr2140` 기준으로 diff 확인.
- `gh pr review 2140 --repo edwardkim/rhwp --approve --body-file -` 수행 완료.
- `gh pr merge 2140 --repo edwardkim/rhwp --merge --delete-branch` 수행 완료.
- 옵션 2 지시에 따라 후속 기록은 `docs/pr2140-review-assets-20260710` 브랜치의 docs-only PR 로 분리.

## 주요 명령

```bash
git diff --name-status 595f1a486a32503149698615727bd634293d840c...local/pr2140
git diff --check 595f1a486a32503149698615727bd634293d840c...local/pr2140
cargo fmt --check
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
CARGO_INCREMENTAL=0 cargo test --profile release-test --lib ooxml_chart -- --nocapture
CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2129_line_stacked -- --nocapture
CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1882_chart_style_gaps \
  --test issue_1431_scatter --test issue_1453_chart_3d_ofpie_routing -- --nocapture
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

시각 sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr2140-line-plain-hwpx samples/chart/라인/꺽은선형.hwpx pdf/chart/라인/꺽은선형-2022.pdf \
  --file-target pr2140-line-plain-hwp samples/chart/라인/꺽은선형.hwp pdf/chart/라인/꺽은선형-2022.pdf \
  --file-target pr2140-line-marker-hwpx samples/chart/라인/표식이있는꺽은선형.hwpx pdf/chart/라인/표식이있는꺽은선형-2022.pdf \
  --file-target pr2140-line-marker-hwp samples/chart/라인/표식이있는꺽은선형.hwp pdf/chart/라인/표식이있는꺽은선형-2022.pdf \
  --file-target pr2140-line-stacked-hwpx samples/chart/라인/누적꺽은선형.hwpx pdf/chart/라인/누적꺽은선형-2022.pdf \
  --file-target pr2140-line-stacked-hwp samples/chart/라인/누적꺽은선형.hwp pdf/chart/라인/누적꺽은선형-2022.pdf \
  --file-target pr2140-line-stacked-marker-hwpx samples/chart/라인/표식이있는누적꺽은선형.hwpx pdf/chart/라인/표식이있는누적꺽은선형-2022.pdf \
  --file-target pr2140-line-stacked-marker-hwp samples/chart/라인/표식이있는누적꺽은선형.hwp pdf/chart/라인/표식이있는누적꺽은선형-2022.pdf \
  --file-target pr2140-line-percent-hwpx samples/chart/라인/백프로기준누적꺽은선형.hwpx pdf/chart/라인/백프로기준누적꺽은선형-2022.pdf \
  --file-target pr2140-line-percent-hwp samples/chart/라인/백프로기준누적꺽은선형.hwp pdf/chart/라인/백프로기준누적꺽은선형-2022.pdf \
  --out output/pr2140-chart-c1d
```

## 산출물

- sweep root: `output/pr2140-chart-c1d`
- 대표 보존 이미지:
  - `mydocs/pr/assets/pr_2140_line_stacked_marker_hwpx_review.png`
  - `mydocs/pr/assets/pr_2140_line_percent_hwpx_review.png`
  - `mydocs/pr/assets/pr_2140_line_marker_hwpx_review.png`

## 리뷰 코멘트 초안

```text
검토했습니다. #2129 C1d 범위의 line stacked/percentStacked 렌더와 plot-level marker 렌더는
코드 경로, 신규 통합 테스트, 기준 PDF 대비 visual sweep 기준으로 확인했습니다.

로컬에서도 fmt, diff check, clippy, ooxml_chart 단위 테스트, 신규 issue_2129_line_stacked 통합
테스트, 관련 차트 회귀 테스트, 전체 release-test 통합 테스트를 모두 통과했습니다. GitHub CI도
pass 상태입니다.

범례 순서 역전과 세부 스타일 fidelity는 PR 본문에 기록된 대로 C2 잔차로 남기는 것이 타당해
보이며, 이번 PR의 merge blocker로 보지 않습니다. approve 의견입니다.
```
