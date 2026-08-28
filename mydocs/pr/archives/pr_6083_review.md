---
kind: pr-review
status: accepted-maintainer-corrected-with-visual-residual
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 21:27 KST
pr: 6083
issue: 5952
author: kevin9327
---

# PR #6083 review - 셀 저장 2줄이 1줄로 접힌 유의사항 상자 재래핑

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6083
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `cfb2646ee19ffb794fbb389779d56114c6f97807`
- 통합 검토 브랜치: `review/kevin9327-6073-6083-20260828`
- 기준: `upstream/devel@a6c7e7bb3ae09470c225a4c90c0fc1ad88b6b5a6`
- 원 PR 상태: non-draft, GitHub CI 성공, `mergeable=CONFLICTING`, `mergeStateStatus=DIRTY`
- 원 PR 코멘트: `postmelee`가 p69 시각 비교와 overflow-cell baseline 증가를 근거로 재작업 요청

## 검토 판단

**메인터너 보정 포함 수용 후보.** #5952의 원인 분석은 맞다. 편람 HWP p69의 유의사항 상자에서 저장
`LINE_SEG`는 2줄인데 compose 결과가 1줄로 접히면서 오른쪽 사이드바 "공문서" 영역으로 글자가
튀어나간다.

다만 원 PR head는 오른쪽 겹침만 줄이고, 재래핑으로 늘어난 줄높이를 표 행/셀 높이 측정 경로가
동일하게 반영하지 못했다. 그 결과 메인터너 코멘트에서 지적된 것처럼 상자 하단이 다음 본문
`4) 문서의 "끝" 표시` 줄과 겹쳤고, `tests/fixtures/overflow_cell_baseline.tsv`의 HWPX 편람 행을
`51 -> 52`로 늘렸다. `local_validation.md` 4.3.1은 기존 문서의 overflow-cell 증가를 baseline으로
숨기지 말라고 규정하므로 그대로 수용할 수 없었다.

보정 내용:

- `composer`에 `recompose_horizontal_cell_lines_for_width`를 추가해 렌더/측정 공통 경로를 만들었다.
- `height_measurer`, `table_layout`, `table_partial`의 가로쓰기 셀 재구성 경로가 같은 helper를 쓰게 해
  #5952 fresh 재래핑 결과가 행/셀 높이에 반영되도록 했다.
- fresh 재래핑 보정은 `native_hwp5_layout()`에서만 켜서 원 HWPX 저장 layout의 overflow-cell 원장을
  임의로 늘리지 않게 했다.
- 하단 겹침을 놓치지 않도록 `note_box_bottom_stays_separate_from_following_body_heading` 회귀를 추가했다.

## 증적과 검증

- 대상 fixture: `samples/2025 행정업무운영 편람(최종).hwp`
- SHA-256: `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`
- `rhwp info --json`: `mydocs/pr/assets/pr_6083_issue5952_handbook_info.json`
  - `format=hwp5`
  - `lastSavedWith=hancom-office-2024 13.0.0.3622`
  - `pageCount=384`
- 기준 PDF: `pdf/2025 행정업무운영 편람(최종)-2024.pdf`
- visual sweep:
  - command:
    `python3 scripts/visual_sweep.py --key pr6083-issue5952-p69 --hwp "samples/2025 행정업무운영 편람(최종).hwp" --pdf "pdf/2025 행정업무운영 편람(최종)-2024.pdf" --page 69 --rhwp-bin /home/tsjang/rhwp/target/pr-review/release-test/rhwp --out output/pr6083-maintainer-check/visual_sweep_pr6083_issue5952`
  - page: p69
  - representative review: `mydocs/pr/assets/pr_6083_issue5952_p69_review.png`
  - compare: `mydocs/pr/assets/pr_6083_issue5952_p69_compare.png`
  - overlay: `mydocs/pr/assets/pr_6083_issue5952_p69_overlay.png`
  - summary: `mydocs/pr/assets/pr_6083_issue5952_visual_sweep_summary.json`
  - overlay metrics: `mydocs/pr/assets/pr_6083_issue5952_p69_overlay_metrics.json`
  - analysis metrics: `mydocs/pr/assets/pr_6083_issue5952_p69_analysis_metrics.json`
  - `visual_accuracy_proxy_percent=50.8623%`
  - `flagged=1/1`: `render_tree_frame_tail_overflow`, `content_bottom_drift`, `line_band_drift`,
    `column_line_band_drift`, `large_ink_region_drift`
- 사람 판정:
  - 유의사항 상자 오른쪽 글자가 사이드바 "공문서" 열로 침범하는 #5952 핵심 결함은 focused 회귀에서
    해소됨을 확인했다.
  - 상자 하단과 다음 본문 heading의 직접 글자 겹침은 새 회귀 test로 해소됨을 확인했다.
  - 한컴 2024 기준 PDF 대비 p69 전체 ink 위치는 아직 크게 다르며, visual sweep 자동 flag도 남는다.
    이는 #5952 우측/하단 겹침 blocker와 분리해 merge comment에서 잔여 시각 fidelity 후보로 명시한다.
- overflow-cell 원장:
  - `RHWP_OVERFLOW_CELL_DUMP=output/pr6083-maintainer-check/overflow_cell_current.tsv cargo test --locked --target-dir target/pr-review --test overflow_cell_baseline -- --nocapture`
  - 945 samples, skip 3, 0 아닌 문서 13종, 총 352줄
  - `test overflow_cell_lines_do_not_grow ... ok`
  - dump: `mydocs/pr/assets/pr_6083_issue5952_overflow_cell_current.tsv`
- focused tests:
  - `issue_5952_cell_note_overflow`: 4 pass
  - `issue_3931`: 5 pass
- 통합 head 공통 검증:
  - `cargo fmt --all -- --check` 통과
  - `git diff --check` 통과
  - `node scripts/rust-test-suite-manifest.mjs --check` 통과
  - `node scripts/rust-unit-test-tiers.mjs --check` 통과
  - `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과
  - `cargo test --locked --doc --target-dir target/pr-review`: 8 pass / 3 ignored
  - 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
    8551 pass / 43 skipped
  - Native Skia lib: 3946 pass / 13 ignored
  - Native Skia `issue_2225_missing_picture_placeholder`: 2 pass
  - Native Skia `render_p37_direct_pdf_export`: 4 pass
  - `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과

## 코멘트 처리

원 PR/issue 코멘트에는 메인터너 보정 사유를 자세히 남긴다.

- 원 PR head의 문제는 "오른쪽 겹침 해소"만 검증했고, 재래핑으로 늘어난 줄높이를 행/셀 높이 측정에
  반영하지 못한 것이다.
- `overflow_cell_baseline.tsv` 증가는 기존 렌더 회귀를 baseline으로 숨기는 형태라 보정에서 되돌렸다.
- 보정 후 focused #5952 4건, overflow-cell 원장, 전체 nextest, clippy, Native Skia, WASM 검증을 통과했다.
- visual sweep p69는 여전히 자동 flag와 낮은 proxy 값이 있으므로, 이는 #5952 해결과 분리한 잔여 fidelity
  후보로 추적한다.
- merge SHA가 확정되면 `pr_6083_issue5952_p69_review.png`를 raw URL로 첨부한다.

Visual Sweep comment 문구:

~~~text
코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 50.86%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다
~~~

## 후속

- p69 전체 한컴 2024 기준 PDF와의 line/ink drift는 별도 fidelity 후보로 남긴다.
- 통합 PR을 만들 경우 본문에 #6083 메인터너 보정 사유와 잔여 visual sweep flag를 반드시 포함한다.
