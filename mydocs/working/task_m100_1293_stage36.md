# Task 1293 Stage 36: 구분선위20 미주사이7 하단 overflow 분석

## 배경

Stage35 전체 sweep에서 page count와 hard 후보는 모두 통과했지만, renderer overflow가
7개 target에 남아 있었다. 새 미주 설정 샘플 중에서는
`2024-11-practice-above20-between7-below2`의 page 17 `pi=884~890` 연쇄 overflow가
가장 큰 후보로 남았다.

## 목적

`구분선 위=20mm`, `미주 사이=7mm`, `구분선 아래=2mm` 조합에서 우측 단 하단의
여러 미주 문단이 연속으로 body bottom 아래에 배치되는 원인을 확인하고, 기존 stage들의
개별 y 보정이 아니라 미주 흐름 공통 로직으로 해결한다.

## 분석 대상

- target: `2024-11-practice-above20-between7-below2`
- sample: `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp`
- page: render log 내부 0-based page 17, 실제 문서 18쪽
- 대표 overflow:
  - `pi=884` FullParagraph overflow 7.9px
  - `pi=885~889` FullParagraph 연쇄 overflow
  - `pi=890` PartialParagraph overflow 205.4px

## 확인 계획

1. `dump-pages -p 16`으로 page 17의 단/문단 배치와 높이를 확인한다.
2. `export-render-tree`로 `pi=884~890`의 실제 bbox와 line y를 확인한다.
3. `dump-note-shape`로 미주 모양 설정이 `20/7/2`로 파싱되는지 확인한다.
4. 필요하면 `RHWP_TYPESET_DRIFT` 로그로 internal rewind/split 후보 계산을 추적한다.
5. 수정 후 focused sweep으로 다음 target을 함께 확인한다.
   - `2024-11-practice-above20-between7-below2`
   - `2024-11-practice-above20-between0-below20`
   - `2024-11-practice-shape987`
   - `2024-11-practice-above0-between0-below0`

## 분석 결과

- `dump-note-shape` 확인:
  - UI `구분선 위=20mm`, `미주 사이=7mm`, `구분선 아래=2mm`로 파싱된다.
  - HWP raw 필드는 `separatorMarginBottom=20mm`, `rawUnknown=7mm`,
    `noteSpacing=2mm`에 저장되어 있고, normalized UI 접근자가 이를 올바르게 해석한다.
- `pi=884~890`는 실제로 `render_tree_018.json`에 있다.
  - sweep overflow 로그의 `page=17`은 0-based 내부 index다.
- 원인:
  - page 18 우측 단에서 `pi=882` TAC 그림 문단이 `vpos=936911`로 되감긴다.
  - pagination은 이 TAC 그림을 앞쪽에 겹쳐 배치되는 개체로 보고 높이를 거의 누적하지 않았다.
  - renderer는 직전 텍스트 침범을 피하기 위해 되감김을 버리고 순차 y에서 그림 높이
    약 215px을 실제로 소비했다.
  - 이 차이 때문에 뒤따르는 `pi=884~890`이 frame 아래로 연쇄 overflow했다.

## 수정 내용

- `src/renderer/typeset.rs`
  - compact 미주 + 보이는 구분선 + 기본 미주 사이 흐름에서 마지막 단 TAC 그림의
    계산된 rewind end가 이미 현재 높이보다 낮거나 같은 경우, renderer처럼 순차 y를 유지한
    것으로 보고 `en_advance`를 누적한다.
  - 앞쪽에 실제로 겹쳐 배치할 수 있는 TAC 그림은 기존처럼 `max(rewind_start + en_advance)`
    경로를 유지한다.

## 검증 결과

- `cargo build --bin rhwp`: 통과
- focused sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-above20-between0-below20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage36_focused --rhwp-bin target/debug/rhwp`
  - 결과 파일: `output/task1293_stage36_focused/summary.json`

| target | page count | overflow_lines | hard 후보 |
|---|---:|---:|---:|
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 3 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 |
| `2024-11-practice-shape987` | 21/21/21 | 9 | 0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 16 | 0 |

## 판단

- Stage35 기준 `2024-11-practice-above20-between7-below2`의 overflow는 15건이었다.
  이번 수정 후 `pi=884~890` 연쇄 overflow는 사라지고 `pi=914` 3건만 남았다.
- 회귀 확인 target의 page count와 overflow count는 Stage35와 동일하다.
- 잔여 `pi=914`는 TAC 그림 높이 누락과 별개로, 문26 제목 이후 본문에서 pagination
  current height와 renderer vpos cursor y가 크게 벌어지는 문제다.

## 다음 단계

Stage37에서는 같은 target의 잔여 `pi=914`를 분석한다.

- 위치: `2024-11-practice-above20-between7-below2`, 실제 문서 19쪽 문26 주변
- 관찰:
  - pagination은 `pi=914` 직전 current height를 약 917px로 보지만,
    renderer는 같은 줄을 약 1076px에서 시작한다.
  - 단순 `current_height + en_fit` 판정으로는 부족하므로, compact 미주에서
    title backtrack 이후 continuation 본문을 어떻게 다음 단으로 넘길지 별도로 봐야 한다.
