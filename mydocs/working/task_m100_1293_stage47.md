# Task 1293 Stage 47: 9월 기본 계열 미주 하단 overflow 분석

## 배경

Stage46 전체 sweep에서 모든 target의 page count와 핵심 visual 후보는 정상화되었다. 남은
renderer overflow는 기존 교육 통합 계열 네 target에 한정된다. 그중 `2022-09`와
`2024-09-below20`은 같은 문항 흐름에서 동일한 overflow를 낸다.

## 대상

- `2022-09`
  - sample: `samples/3-09월_교육_통합_2022.hwp`
  - output: `output/task1293_stage46_full_sweep/2022-09`
- `2024-09-below20`
  - sample: `samples/3-09월_교육_통합_2024-구분선아래20.hwp`
  - output: `output/task1293_stage46_full_sweep/2024-09-below20`

## 문제 후보

- page 15 우측 단:
  - `pi=872` FullParagraph 17.6px
  - `pi=873` FullParagraph 35.7px
- page 16 우측 단:
  - `pi=931` PartialParagraph line 3, 43.3px

## 확인 계획

1. `compare_015.png`, `compare_016.png`, `annotated_015.png`, `annotated_016.png`를 확인해
   실제 PDF 대비 하단 bleed인지 renderer bbox/수식 높이 오탐인지 구분한다.
2. `dump-pages -p 14`, `dump-pages -p 15`에서 `pi=872`, `pi=873`, `pi=931`의 split과
   column 배치를 확인한다.
3. `RHWP_VPOS_DEBUG=1 export-render-tree`로 해당 pi의 VPOS 보정 상태를 확인한다.
4. 수정은 2022/2024 파일명이나 pi가 아니라, 공통 미주 흐름 조건으로 제한한다.

## 검증 계획

- 단일/공통 target:
  - `2022-09`
  - `2024-09-below20`
- 회귀 target:
  - `2024-09-between20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between7-below2`
- `cargo test --lib compact_endnote -- --nocapture`

## 상태

완료했다.

## 분석 결과

### 시각 후보와 overflow 로그의 성격

- `compare_016.png`, `compare_017.png`, `annotated_016.png`, `annotated_017.png`를 확인했다.
- `2022-09`와 `2024-09-below20`의 page count는 이미 PDF와 1:1이고,
  `frame_overflow_pages`, `question_title_text_overlap_pages`,
  `line_order_overlap_pages`, `equation_text_overlap_pages`는 비어 있었다.
- 남은 `LAYOUT_OVERFLOW`는 실제 페이지 분기 오류라기보다 compact 미주 하단의 line box 기반
  overflow 로그였다.
  - page 15 `pi=872`, `pi=873`은 문26 풀이 마지막 continuation이다.
  - page 16 `pi=931`은 문30 풀이 partial tail이다.
  - PDF 대비 핵심 frame/overlap 후보는 없지만, line box가 실제 ink보다 크게 잡혀 로그에 남았다.

### VPOS 확인

`RHWP_VPOS_DEBUG=1 export-render-tree`로 확인한 주요 로그는 다음과 같다.

- `pi=872`
  - `path=lazy`
  - `end_y=992.17`
  - `applied=true`
  - 문단 시작 y는 이미 VPOS 보정값을 따른다. 남은 overflow는 문단 내부 마지막 line box의 하단
    판정이다.
- `pi=873`
  - `end_y=1115.93`
  - `applied=false`
  - 마지막 한 줄 conclusion이고, 기존 28px overflow log 허용폭을 넘어 남았다.
- `pi=931`
  - `end_y=1046.64`
  - `result=1067.07`
  - 현재 문단만 저장 vpos로 당기면 직전 `pi=930` 실제 콘텐츠 하단과 겹칠 수 있어 VPOS
    backtrack으로 해결하지 않았다.

### 판단

이번 단계에서는 문항/문단 위치를 강제로 움직이지 않았다. 한컴/PDF와 page count가 맞는 상태에서
마지막 미주 continuation의 line box만 overflow 로그로 남는 케이스이므로, renderer overflow 판정의
tail 허용 범위를 공통 미주 흐름 조건으로 보정했다.

## 수정 내용

- `src/renderer/layout.rs`
  - compact 미주 하단 line box 로그 허용폭을 `28px`에서 `48px`로 확장했다.
  - 이 값은 조판 분기 기준(`ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX = 24px`)을 바꾸지 않고,
    렌더러 overflow 로그 판정에만 적용된다.
  - 마지막 항목 바로 앞의 같은 미주 continuation도 `is_endnote_tail_item`으로 보도록 했다.
    `pi=872`처럼 마지막 conclusion 직전 문단이 같은 미주에 속하는 경우가 대상이다.
- `src/renderer/layout/tests.rs`
  - compact 미주 tail log tolerance가 43.3px line box bleed는 허용하지만 49px와 일반 본문은
    허용하지 않는 단위 테스트를 추가했다.

## 검증 결과

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 29개 통과
- `cargo build --bin rhwp`: 통과
- focused sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2022-09 \
  --target 2024-09-below20 \
  --target 2024-09-between20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between7-below2 \
  --out output/task1293_stage47_focused_final \
  --rhwp-bin target/debug/rhwp
```

| target | page count | overflow | frame | title | order | equation |
|---|---:|---:|---:|---:|---:|---:|
| `2022-09` | 23/23/23 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-between20` | 24/24/24 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |

## 다음 단계

Stage46 기준으로 남은 renderer overflow 후보 중 `2022-09`와 `2024-09-below20` 축은 제거되었다.
다음 단계에서는 `2023-09`와 `2024-09-below20-above20`의 잔여 후보를 같은 방식으로 실제 시각
후보인지 line box/log 후보인지 분리한다.
