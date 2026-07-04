# PR #1844 리뷰 — 같은 문단 float 스택 이월 규칙

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1844 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1831` |
| 작성 시점 규모 | 9 files, +460 / -2 |
| 작성 시점 참고 head | `ff0c7ac2c3629ec780179f5e682f685bc0c085d2` |
| 최종 merge head | `93312688500e6a026758947e57e203fbe100e808` |
| 작성 시점 참고 상태 | `MERGEABLE`, `Build & Test` 대기 중 |
| merge 결과 | 2026-07-03 KST merge 완료, merge commit `a987ae10e85128f75997d7b826b61a668b0eef7f` |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `src/renderer/typeset.rs`
  - 같은 문단에 선행 float 표가 있고, 뒤따르는 row-break 표 전체가 잔여 단 공간에 들어가지 않으면 첫 조각을 만들지 않고 다음 단/쪽으로 이월한다.
  - 이월 후 단 상단에서 전체 표가 2px 이하 측정 오차로만 넘치는 경우 전체 배치를 허용한다.
- `src/main.rs`
  - `dump` 진단에 셀 `hdr=` 값을 추가한다.
- `samples/float-stack-defer.hwp`, `pdf/float-stack-defer-2022.pdf`
  - 한글 2022 기준 PDF와 재현 샘플을 추가한다.
- `tools/patch_cell_flags.py`
  - 셀 플래그 인과 실험용 보조 패처를 추가한다.
- `mydocs/*`, `mydocs/tech/hwp_spec_errata.md`
  - #1831 조사/보고/스펙 정정 문서를 추가 또는 갱신한다.

## PR 내용 기준 판단

이 PR의 핵심은 일반적인 visual diff 점수를 높이는 것이 아니라, 같은 문단에 앵커된 선행 float 표 때문에 뒤따르는
다쪽 표가 단 하단에 첫 행 조각을 만들던 문제를 한글 2022처럼 다음 쪽 상단으로 통째 이월하는 것이다.

로컬에서 `samples/float-stack-defer.hwp`를 확인한 결과:

- `rhwp info samples/float-stack-defer.hwp`: 2 pages.
- `rhwp dump-pages samples/float-stack-defer.hwp`: p1은 제목과 첫 번째 표, p2는 두 번째 표 전체로 구성된다.
- 기준 PDF `pdf/float-stack-defer-2022.pdf`: Hancom PDF 1.3.0.550, Hwp 2022 12.0.0.4547, 2 pages.

따라서 PR 본문에서 말한 "수정 전 p1 하단에 표2 row0 조각이 남고, 수정 후 p2 상단으로 표2 전체 이월"이라는 목적은
현재 head에서 맞게 구현된 것으로 판단한다.

## 시각 검증

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1844-float-stack-defer \
  --hwp samples/float-stack-defer.hwp \
  --pdf pdf/float-stack-defer-2022.pdf \
  --pages 1-2 \
  --out output/pr1844_visual \
  --rhwp-bin target/debug/rhwp
```

결과:

- visual sweep: `flagged=0/2`
- SVG/PDF/RHWP export page count: 2 / 2 / 2
- review p1: `output/pr1844_visual/pr1844-float-stack-defer/review/review_001.png`
- review p2: `output/pr1844_visual/pr1844-float-stack-defer/review/review_002.png`
- overlay metrics: `output/pr1844_visual/pr1844-float-stack-defer/overlay/overlay_metrics.json`
- PR comment용 asset 후보:
  - `mydocs/pr/assets/pr_1844_float_stack_defer_review_p1.png`
  - `mydocs/pr/assets/pr_1844_float_stack_defer_review_p2.png`

자동 일치율 보조값:

- p1 `visual_accuracy_proxy_percent`: 약 11.15%
- p2 `visual_accuracy_proxy_percent`: 약 9.93%

이 값은 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다. 폰트/래스터 차이 때문에 낮게 나오지만,
이번 PR의 판정 기준인 페이지 수, p1/p2 표 배치, p2 표 전체 이월 여부는 기준 PDF와 맞는다.

추가 baseline 비교:

```bash
target/debug/rhwp export-pdf samples/float-stack-defer.hwp -o output/pr1844_float_stack_defer_rhwp.pdf
/tmp/rhwp-pr1844-venv/bin/python tools/compare_line_baselines.py \
  output/pr1844_float_stack_defer_rhwp.pdf \
  pdf/float-stack-defer-2022.pdf
```

- p1: `n=58 Δbaseline median=-1.00pt min=-1.27 max=+0.09`
- p2: `n=70 Δbaseline median=-2.27pt min=-2.70 max=+0.65`

`export-pdf` 중 `LAYOUT_OVERFLOW` 경고 1건이 출력되었다.

```text
LAYOUT_OVERFLOW: page=1, sec=0, col=0, para=1, type=Table, first=true, y=1060.5, bottom=1031.8, overflow=28.7px
```

visual sweep 자동 판정과 기준 PDF 대조에서는 p1 하단 표 조각 재발이 보이지 않아 merge blocker로 보지는 않는다. 다만 같은
샘플의 table/caption overflow 진단이 남는 점은 후속 회귀 관찰 포인트로 기록한다.

## 로컬 검증

검토 브랜치: `local/pr1844-review`

- `find target -mindepth 1 -maxdepth 1 -exec rm -rf {} +`
- `env CARGO_INCREMENTAL=0 cargo build` 통과
- `git diff --check upstream/devel...HEAD` 통과
- `cargo fmt --check` 통과
- `python3 -m py_compile tools/patch_cell_flags.py tools/compare_line_baselines.py` 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과

스팟체크:

- `target/debug/rhwp info samples/rowbreak-problem-pages.hwp`: 18 pages
- `target/debug/rhwp info samples/rowbreak-problem-pages.hwpx`: 18 pages
- `target/debug/rhwp info samples/byeolpyo4.hwp`: 26 pages
- `target/debug/rhwp info samples/byeolpyo1.hwp`: 4 pages

## GitHub CI

최종 merge 전 최신 head `93312688500e6a026758947e57e203fbe100e808` 기준 확인:

- CodeQL: 통과
- Render Diff / Canvas visual diff: 통과
- Analyze rust/python/javascript-typescript: 통과
- CI preflight: 통과
- `Build & Test`: 통과
- WASM Build: skip

## GitHub 후속 처리

- PR merge: https://github.com/edwardkim/rhwp/pull/1844
- merge commit: `a987ae10e85128f75997d7b826b61a668b0eef7f`
- #1831은 PR 본문 `Closes #1831` 대상이며 merge 후 close 처리 확인 대상이다.
- #1842는 PR 본문에서 분리한 잔여 이슈이므로 열린 상태 유지가 맞다.

## 결론

PR 내용 기준으로 #1831의 핵심 재현 케이스는 수정된 것으로 판단했고, 최신 head 기준 GitHub Actions 통과 후 merge
완료했다. #1842는 PR 본문에서도 분리한 잔여 이슈이므로 별도로 열린 상태를 유지하는 것이 맞다.
