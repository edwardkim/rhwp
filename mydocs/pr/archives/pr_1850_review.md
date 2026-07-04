# PR #1850 리뷰 — visible-host 표 직후 outer_margin_bottom 반영

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1850 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1841` |
| 작성 시점 규모 | 6 files, +138 / -17 |
| 작성 시점 참고 head | `9519b3aca3796e00e62c0aeea1b6e02fd57ba46a` |
| 최종 상태 | GitHub CI 통과 후 merge 완료 |
| merge commit | `bceed75f8b1db049e45a620de4e80bf294dad38d` |
| reviewer assign | @jangster77 지정 완료 |

## 관련 이슈

- #1841: visible-host 자리차지(TopAndBottom) 표 직후 본문 재개 위치가 표 `outer_margin_bottom`
  만큼 위로 붙는 결재문서 계열 오프셋 문제.
- PR 본문 기준 #1841을 처리하고, 하단 결재부 -34pt 누적, 셀 내부 국소 차이, 36388711 다페이지 흐름 시프트는
  비범위로 분리한다.

## 변경 범위

- `src/renderer/layout.rs`
  - visible-host TopAndBottom 표 아래 host 본문 재개 시 `outer_margin_bottom`을 y 재개 위치에 더한다.
- `src/renderer/typeset.rs`
  - pagination 북키핑에서도 `is_tac || (is_para_topbottom_float && para_has_non_whitespace_text)` 조건으로
    `outer_bottom`을 반영해 layout 축과 맞춘다.
- `tests/issue_1789_exclusion_probe_line_spacing.rs`
  - 36385142 재현 샘플의 저장 vpos 기준 핀을 `541.3px`로 정정한다.
- `tests/issue_1692.rs`
  - SO-SUEOP p22 관계도 표 아래 본문 시작과 후속 질문 기준값을 권위 PDF 재측정값으로 정정한다.
- `mydocs/plans/task_m100_1841.md`, `mydocs/report/task_m100_1841_report.md`
  - 원인, 적용 범위, 검증 결과를 기록한다.

## PR 내용 기준 판단

PR은 모든 비-TAC 표에 `outer_margin_bottom`을 전면 적용하지 않고, 실제 렌더 좌표가 바뀌는
visible-host float 형상으로 한정한다. 이 범위 제한은 PR 본문에 적힌 1086/1156 계열 핀 충돌 회피와
일치하며, `layout.rs`와 `typeset.rs` 양쪽에 대칭으로 반영되어 있다.

테스트 핀 변경도 "기존 보상 오차를 고정하던 값"을 권위 PDF와 저장 vpos 산술 기준으로 정정하는 성격이다.
focused test와 기준 PDF 대조에서 PR 본문 방향과 일치함을 확인했다.

## 로컬 검증

검토 브랜치: `local/pr1850-review`

- `find target -mindepth 1 -maxdepth 1 -exec rm -rf {} +`
- `git diff --check upstream/devel...HEAD` 통과
- `cargo fmt --check` 통과
- `env CARGO_INCREMENTAL=0 cargo build` 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1692 --test issue_1789_exclusion_probe_line_spacing` 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과
- `python3 -m py_compile scripts/task1274_visual_sweep.py` 통과

추가 baseline 비교:

```bash
target/debug/rhwp export-pdf samples/task1789/exclusion_probe_line_spacing.hwpx -o output/pr1850_exclusion_probe_rhwp.pdf
target/debug/rhwp export-pdf 'samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx' -o output/pr1850_36389312_rhwp.pdf
/tmp/rhwp-pr1844-venv/bin/python tools/compare_line_baselines.py output/pr1850_exclusion_probe_rhwp.pdf pdf/exclusion_probe_line_spacing-2024.pdf
/tmp/rhwp-pr1844-venv/bin/python tools/compare_line_baselines.py output/pr1850_36389312_rhwp.pdf 'pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf'
```

- `exclusion_probe_line_spacing` p1: `n=80 Δbaseline median=+0.07pt min=+0.01 max=+0.96`
- `exclusion_probe_line_spacing` p2: 하단 결재부 계열 `-30~-34pt` step 잔존. PR 본문 비범위와 일치.
- `36389312` p1: `n=69 Δbaseline median=-0.43pt min=-34.02 max=+0.35`
- `36389312` 하단 결재부 `-34pt` step 잔존. PR 본문 비범위와 일치.

## 시각 검증

### 36385142 / exclusion_probe p1

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1850-exclusion \
  --hwp samples/task1789/exclusion_probe_line_spacing.hwpx \
  --pdf pdf/exclusion_probe_line_spacing-2024.pdf \
  --page 1 \
  --out output/pr1850_visual \
  --rhwp-bin target/debug/rhwp
```

- visual sweep: `flagged=0/1`
- compare: `output/pr1850_visual/pr1850-exclusion/compare/compare_001.png`
- overlay: `output/pr1850_visual/pr1850-exclusion/overlay/overlay_001.png`
- review: `output/pr1850_visual/pr1850-exclusion/review/review_001.png`
- asset: `mydocs/pr/assets/pr_1850_exclusion_review_p001.png`
- `visual_accuracy_proxy_percent`: 약 21.64%
- 사람 판정: 표 직후 본문이 과도하게 아래로 밀리지 않고 기준 PDF와 큰 흐름이 맞는다.

### 36389312 p177 / 기준 PDF p1

`rhwp export-svg`가 원문 내부 페이지 번호 `177`로 산출하고 기준 PDF는 단일 페이지라 `rhwp_177.png`와
`pdf-1.png`처럼 번호가 다르다. visual sweep에 단일 페이지 산출물 fallback을 보정한 뒤 `--page 1` 명령으로
p177 ↔ PDF p1을 자동 1:1 매칭해 확인했다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1850-36389312 \
  --hwp 'samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx' \
  --pdf 'pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf' \
  --page 1 \
  --out output/pr1850_visual \
  --rhwp-bin target/debug/rhwp
```

- visual sweep: `flagged=0/1`
- compare: `output/pr1850_visual/pr1850-36389312/compare/compare_177.png`
- overlay: `output/pr1850_visual/pr1850-36389312/overlay/overlay_177.png`
- review: `output/pr1850_visual/pr1850-36389312/review/review_177.png`
- asset: `mydocs/pr/assets/pr_1850_36389312_review_p177.png`
- `visual_accuracy_proxy_percent`: 약 7.33%
- 사람 판정: 상단 헤더 표 이후 본문 흐름은 PR 목적대로 맞고, 하단 결재부 시프트는 PR 본문 비범위와 일치한다.

### SO-SUEOP HWP p22

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1850-so-sueop-hwp-p22 \
  --hwp samples/SO-SUEOP.hwp \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --page 22 \
  --out output/pr1850_visual \
  --rhwp-bin target/debug/rhwp
```

- visual sweep: `flagged=0/1`
- compare: `output/pr1850_visual/pr1850-so-sueop-hwp-p22/compare/compare_022.png`
- overlay: `output/pr1850_visual/pr1850-so-sueop-hwp-p22/overlay/overlay_022.png`
- review: `output/pr1850_visual/pr1850-so-sueop-hwp-p22/review/review_022.png`
- asset: `mydocs/pr/assets/pr_1850_so_sueop_hwp_review_p022.png`
- `visual_accuracy_proxy_percent`: 약 14.30%
- 사람 판정: 관계도 표 아래 본문 시작과 후속 질문 앵커가 PR 목적 범위에서 맞는다. 나머지는 폰트/글자폭/래스터 차이로 본다.

## GitHub CI

최신 PR head `9519b3aca3796e00e62c0aeea1b6e02fd57ba46a` 기준:

- CI preflight: 통과
- Render Diff preflight: 통과
- CodeQL preflight: 통과
- Canvas visual diff: 통과
- Analyze python/javascript-typescript/rust: 통과
- Build & Test: 통과

## 결론

PR 내용 기준으로 merge 후보로 판단했고, 최신 PR head 기준 GitHub CI 통과 후 merge 완료했다. #1841은 후속
리뷰 기록/asset PR 이 merge 된 뒤 close comment 와 함께 close 대상이다. PR 본문 비범위로 분리한 하단
결재부/셀 내부/36388711 흐름 축은 별도 이슈로 유지한다.
