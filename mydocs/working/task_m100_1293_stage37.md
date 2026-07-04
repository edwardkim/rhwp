# Task 1293 Stage 37: 구분선위20 미주사이7 문26 continuation overflow 분석

## 배경

Stage36에서 `2024-11-practice-above20-between7-below2`의 page 18
`pi=884~890` TAC 그림 뒤 연쇄 overflow는 제거했다. focused sweep 결과 이 target의
잔여 overflow는 같은 샘플의 뒤쪽 문26 주변 `pi=914` 3건으로 축소되었다.

## 목적

`구분선 위=20mm`, `미주 사이=7mm`, `구분선 아래=2mm` 조합에서 문26 제목 뒤
continuation 본문이 renderer에서는 frame 아래로 밀리지만 pagination에서는 fit으로 판단되는
차이를 분석하고, 미주 흐름 공통 로직으로 해결한다.

## 분석 대상

- target: `2024-11-practice-above20-between7-below2`
- sample: `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp`
- 실제 문서 위치: 19쪽 문26 주변
- 잔여 overflow:
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=914 line=1 y=1122.0 col_bottom=1092.3`
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=914 line=2 y=1140.0 col_bottom=1092.3`
  - `LAYOUT_OVERFLOW: page=18, sec=0, col=0, para=914, type=FullParagraph, first=false`

## 초기 관찰

- Stage36 임시 분석에서 pagination은 `pi=914` 직전 current height를 약 917px로 보았다.
- renderer는 문26 제목/본문의 vpos cursor 때문에 `pi=914`를 약 1076px에서 시작한다.
- 단순 `current_height + en_fit <= available` 판정은 실제 renderer y를 반영하지 못한다.

## 확인 계획

1. `export-render-tree`의 `render_tree_019.json`에서 `pi=912~914` bbox와 line y를 확인한다.
2. `dump-pages -p 18`로 pagination의 문단 배치와 current height 추정을 확인한다.
3. `RHWP_TYPESET_DRIFT` 로그가 있으면 문26 제목/본문의 split, vpos, current height를 비교한다.
4. renderer `HeightCursor`와 pagination이 같은 continuation y를 사용하도록 공통 조건을 좁혀 수정한다.
5. focused sweep으로 stage36 target과 회귀 target을 다시 확인한다.

## 분석 결과

- Stage36 render tree에서 `pi=914`는 왼쪽 단 하단 `y=1076.3`에서 시작하고 마지막 줄이
  `y=1128.0`까지 내려가 frame bottom `1092.3`을 넘었다.
- `dump-pages` 기준 pagination은 `pi=912~914`를 모두 왼쪽 단에 둘 수 있다고 판단했다.
  하지만 PDF/Hancom 비교에서는 문26 제목(`pi=912`)부터 오른쪽 단 상단에서 시작한다.
- 원인은 새 미주 제목의 `current_height`는 약 824px로 아직 여유가 있다고 보지만,
  현재 단 첫 미주 vpos 기준으로 환산한 저장 vpos 위치는 이미 단 하단에 가까웠기 때문이다.
- 처음 시도한 본문 tail 기준 이동은 `pi=914`만 오른쪽 단으로 넘겨 `pi=553`, `pi=936`
  새 overflow를 만들었다. 이 조건은 제거했다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 보이는 구분선 + compact 미주 + 기본/작은 미주 사이에서 새 미주 제목(`ep_idx=0`)이
    단 하단에 걸릴 때, 제목 포함 초기 3개 문단의 저장 vpos span을 현재 단 첫 미주 vpos 기준으로
    환산한다.
  - 이 초기 묶음이 현재 단에 들어가지 않으면 제목 단계에서 다음 단으로 넘긴다.
  - `미주사이8` shape987 같은 큰 gap 샘플은 기존 경로를 유지하도록 `default_between_notes_gap`
    조건으로 제한했다.

## 검증 결과

- `cargo build --bin rhwp`: 통과
- focused sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-above20-between0-below20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage37_focused3 --rhwp-bin target/debug/rhwp`
  - 결과 파일: `output/task1293_stage37_focused3/summary.json`

| target | page count | overflow_lines | hard 후보 | frame 후보 |
|---|---:|---:|---:|---:|
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 1 | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 0 |
| `2024-11-practice-shape987` | 21/21/21 | 9 | 0 | 0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 14 | 0 | 0 |

## 판단

- Stage36의 `2024-11-practice-above20-between7-below2` 잔여 `pi=914` 3건은 사라졌다.
- 문26 제목과 초기 풀이가 오른쪽 단 상단으로 이동해 PDF/Hancom의 큰 흐름과 맞아졌다.
- 회귀 target 중 `shape987`은 Stage36과 같은 overflow 9건으로 돌아왔다.
- `above0-between0-below0`은 Stage36의 16건에서 14건으로 줄었다.
- `above20-between7-below2`에는 `pi=932` 7.1px overflow가 남아 있어 다음 스테이지에서
  문28 초반 수식 줄 하단 처리와 PDF 대비 흐름 차이를 계속 분석한다.
