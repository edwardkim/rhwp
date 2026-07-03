# PR #1862 리뷰 기록

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1862 |
| 작성자 | `planet6897` |
| base | `devel` |
| head | `task1858-paper-anchor-coanchored-flow` |
| 관련 이슈 | #1858 |
| merge commit | `c8c13b173f4f45c80806af2fb0475fe5b159d030` |
| 처리 방식 | 원 코드 PR merge 후 옵션 2 방식으로 review 문서/asset 후속 PR 분리 |

## PR 요지

#1858 의 두 발현 중 발현 1만 다룬다. `vert=용지(Paper)` 기준 페이지 절대좌표에 co-anchored 된
자리차지 상자들이 첫 상자 이후 generic flow 경로로 빠지면서 페이지를 소비하던 문제를, 같은 host 문단의
후속 Paper 자리차지 상자도 0-flow 절대배치로 처리하도록 보정한다.

이 PR 은 발현 2인 `vert=쪽(Page)` + `valign=Bottom` 하단 블록 세로 오프셋 문제를 닫지 않는다. 따라서
#1858 자체는 후속 발현 2 추적을 위해 open 유지가 맞다.

## 변경 범위

- `src/renderer/typeset.rs`
  - `is_paper_topbottom_block` 경로에서 같은 host 문단에 선행 Paper 자리차지 표가 있으면 후속 상자도
    절대배치 경로로 처리한다.
  - 첫 박스만 `current_height` 를 host vpos 로 sync 하고, 후속 co-anchored Paper 상자는 flow 를 소비하지
    않는다.
- `tests/issue_1858.rs`
  - `samples/issue1858_paper_anchor_float_stack.hwpx` 가 1쪽이어야 한다는 회귀 게이트를 추가한다.
- `samples/issue1858_paper_anchor_float_stack.hwpx`
  - 발현 1 재현 샘플.

## 로컬 검증

작업 위치: `/private/tmp/rhwp-pr1862-review`

```bash
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1858
```

결과: 통과. `paper_anchored_coanchored_boxes_do_not_inflate_pages` 1건 성공.

```bash
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
./target/debug/rhwp dump-pages samples/issue1858_paper_anchor_float_stack.hwpx | rg -c '^=== 페이지'
```

결과: `1`

`upstream/devel` 기준 같은 샘플은 3쪽이었다. 따라서 PR 요지인 `3쪽 -> 1쪽` 페이지 폭발 해소는 로컬에서
재현 확인했다.

영향권 회귀 테스트도 함께 확인했다.

```bash
env CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_1858 \
  --test issue_1853 \
  --test issue_1611_footer_page_bottom_pagination \
  --test issue_1624_footer_overpush_pagination \
  --test issue_1658_page_bottom_fixed_exclusion \
  --test issue_1418_textbox_table_overlap \
  --test issue_1510 \
  --test issue_1663
```

결과: 통과. 8개 test binary, 총 15개 테스트 성공.

`git diff --check upstream/devel...HEAD` 도 통과했다.

## 시각 검증

기준 PDF:

- `pdf/issue1858_paper_anchor_float_stack-2024.pdf`
- Creator: `Hwp 2024 13.0.0.3622`
- Producer: `Hancom PDF 1.3.0.550`
- 페이지 수: 1쪽

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1862-issue1858 \
  --hwp 'samples/issue1858_paper_anchor_float_stack.hwpx' \
  --pdf 'pdf/issue1858_paper_anchor_float_stack-2024.pdf' \
  --page 1 \
  --out output/pr1862_visual \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG pages: 1
- PDF pages: 1
- selected pages: `[1]`
- `flagged=0/1`
- `average_visual_accuracy_proxy_percent`: 약 `15.86%`

후속 PR asset:

- `mydocs/pr/assets/pr_1862_issue1858_review_p001.png`
- `mydocs/pr/assets/pr_1862_issue1858_baseline_2024.pdf`

사람 판정:

- 페이지 수와 큰 구조는 PR 요지에 맞게 1쪽으로 정합한다.
- 기준 PDF와 rhwp 출력의 세부 위치, 회색 채움, 표 영역 차이는 남는다.
- 이 PR 의 목적은 발현 1의 페이지 폭발 해소이므로 merge blocker 로 보지 않는다.
- 정밀 시각 일치는 발현 2 또는 별도 렌더링 보정 후속으로 분리하는 것이 맞다.

## GitHub CI

최신 PR head `ce9c8f80934f090709c81880c8e203e9a5f232e4` 기준:

- Build & Test: pass
- CodeQL / Analyze rust/python/javascript-typescript: pass
- Render Diff / Canvas visual diff: pass
- WASM Build: skipped

## 코멘트/후속

- PR merge 후 페이지 수나 렌더링 위치 변화가 핵심인 PR 에서는 한컴 2020/2024 등에서 저장한 기준 PDF를
  함께 첨부해 달라는 추가 코멘트를 남겼다.
- 코멘트: https://github.com/edwardkim/rhwp/pull/1862#issuecomment-4875744947
- #1858 은 발현 2가 남아 있으므로 close 하지 않는다.

## 결론

PR 내용 기준으로 merge 적합하다. 실제 merge 는 `c8c13b173f4f45c80806af2fb0475fe5b159d030` 으로 완료했다.
