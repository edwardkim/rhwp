---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 66 — HWPX p144 behind-text nested-table position

## 범위와 정답지

Stage 65가 고정한 것은 outer block-TAC table의 쪽 소유와 하위 2×3 예시 표의 body
clip이다. 그 결과를 전체 p144 fidelity 완료로 해석하면 안 된다. 같은 direct PDF sweep의
`review_144.png`에서 자동날인 안내의 빈 1×1 표 세 개가 rhwp에서는 세로로 쌓이고,
한컴 PDF에서는 같은 y 좌표에 가로로 놓인다.

| 항목 | 경로 |
| --- | --- |
| HWPX 입력 | `samples/2025 행정업무운영 편람(최종).hwpx` |
| PDF oracle | `pdf/2025 행정업무운영 편람(최종)-2024.pdf` |
| 직접 증적 | `mydocs/pr/assets/task_m100_3820_stage65_hwpx_noninline_tac_fit/review_144.png` |
| 대상 XML | `Contents/section3.xml`, outer table `id=1723619577`, r2c0 paragraph 4 |

PDF의 세 점선 상자는 비어 있는 표 자체가 맞다. 결함은 빈 상자를 이미지로 대체하는 것이
아니라 다음 HWPX 좌표를 무시해 같은 문단의 세 control을 flow로 연속 배치하는 것이다.

| table id | `treatAsChar` | `textWrap` | `flowWithText` | `horzOffset` |
| --- | ---: | --- | ---: | ---: |
| `1723619590` | 0 | `BEHIND_TEXT` | 1 | 22830 |
| `1723619591` | 0 | `BEHIND_TEXT` | 1 | 13553 |
| `1723619592` | 0 | `BEHIND_TEXT` | 1 | 4868 |

세 control은 모두 `vertRelTo=PARA`, `horzRelTo=COLUMN`, `vertOffset=0`, 1×1 cell이다.
같은 host paragraph의 stored `vpos=11824HU`에 anchor되어야 한다. 따라서 PDF의 x 순서는
offset 4868 → 13553 → 22830이고 y는 동일하다.

## 원인

`src/renderer/layout/table_layout.rs`의 nested non-TAC `Control::Table` 분기는
`inline_x_override=None`으로 `layout_table`을 호출한다. depth>0의 일반 x 계산은 작은
non-TAC table을 host cell 중앙에 놓고 `horizontal_offset`을 의도적으로 버린다. 이어서
`para_y = nested_y + table_h`로 갱신하므로 같은 host paragraph의 세 BehindText table이
각각 다음 줄에 세로로 쌓인다.

이것은 source의 `BehindText` 의미(본문 흐름을 소비하지 않는 overlay)와 맞지 않는다.
다만 모든 depth>0 non-TAC table의 기존 정렬 계약을 바꾸지 않는다. HWPX stored-layout,
`BehindText`, `vertRelTo=PARA`, `horzRelTo=COLUMN`, `flowWithText=true`로 한정해 parent
cell origin을 explicit x anchor로 넘기고, 해당 control은 paragraph flow cursor를 전진시키지
않는 최소 보정만 허용한다.

## 구현 및 검증 계획

1. 위 한정 predicate에서 `inline_x_override=Some(inner_area.x)`를 사용해 existing
   `compute_table_x_position`의 non-TAC offset 합산 경로를 재사용한다.
2. 같은 predicate에서는 `para_y`를 table 높이만큼 누적하지 않아 세 table의 common
   paragraph anchor를 보존한다.
3. production p144 tree에서 56.7px 1×1 표 세 개의 y가 같고 x가 source offset 순으로
   단조 증가하는 focused regression을 추가한다.
4. p143--p146 direct PDF sweep, Stage 65 focused test 및 overflow-cell baseline으로
   owner·physical clip·일반 nested-cell 회귀를 재확인한다.

## 중단 조건

이 predicate가 다른 HWPX inline/TopAndBottom 표 또는 nested-cell normal flow를 바꾸면
보정을 폐기한다. `BehindText`와 해당 stored anchor가 없는 표에는 offset을 강제하지 않는다.

## 구현 결과

`Control::Table`의 nested non-TAC branch에
`hwpx_nested_behind_text_overlay` predicate를 추가했다. predicate가 참이면
`inline_x_override=Some(inner_area.x)`를 전달해 기존 `compute_table_x_position()`의
non-TAC horizontal-offset 합산을 재사용하고, `para_y`를 table height로 갱신하지 않는다.
따라서 같은 host paragraph의 세 control은 공통 y를 유지하면서 `horzOffset`만큼 분리된다.

`tests/issue_3930_hwpx_hwp_save_layout.rs`에는 p144에서 56.7px 1×1 점선 표 세 개의
수와 PDF 기준 좌표 `x=182.0, 297.8, 421.5`, 공통 y를 고정했다. table owner만 맞고
상자를 세로로 쌓는 회귀를 render tree 수준에서 차단한다.

180 DPI p143--p146 direct PDF sweep은 SVG 386/386, requested raster 4/4로 완료했다.
새 p144 review에서는 세 점선 상자가 PDF와 같이 가로 한 줄로 놓인다. 증적은
`mydocs/pr/assets/task_m100_3820_stage66_hwpx_behind_text_table_positions/review_144.png`,
`review_145.png`, `summary.json`에 보존했다.

## 검증 결과

- `cargo fmt --check` — 통과.
- `cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout -- --nocapture` — 2/2 통과.
- `cargo test --profile release-test --test issue_1891 -- --nocapture` — 4/4 통과.
- `cargo test --profile release-test --test overflow_cell_baseline -- --nocapture` — 통과,
  678 fixtures(3 skip), nonzero 17 documents, total 691 lines. baseline 변경 없음.
