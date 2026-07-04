# Task 1293 Stage 66: 공식 미주 gap baseline 계측

## 목적

Stage65에서 공식 `미주 모양` 의미와 2024-11 샘플의 UI 값을 확정했다. 이번 단계는 소스 수정 전에
현재 rhwp가 각 값을 실제 렌더링 gap으로 어떻게 소비하는지 PDF와 비교한다.

이 단계의 목표는 증상별 page tail이 아니라 다음 공식 gap을 분리해 계측하는 것이다.

- 미주 block top gap: 본문과 구분선 사이, 또는 구분선 없는 경우 본문과 첫 미주 내용 사이
- separator-to-content gap: 구분선과 첫 미주 내용 사이
- between-notes gap: 앞 번호 미주 visible bottom과 다음 번호 미주 visible top 사이

## 대상 샘플

| target | 샘플 | 공식 UI 값 |
|---|---|---|
| `2024-11-practice-shape987` | `구분선위9미주사이8구분선아래7` | 위 9mm, 사이 8mm, 아래 7mm |
| `2024-11-practice-above0-between0-below0` | `구분선위0미주사이0구분선아래0` | 위 0mm, 사이 0mm, 아래 0mm |
| `2024-11-practice-above0-between7-below2` | `구분선위0미주사이7구분선아래2` | 위 0mm, 사이 7mm, 아래 2mm |
| `2024-11-practice-above0-between7-below20` | `구분선위0미주사이7구분선아래20` | 위 0mm, 사이 7mm, 아래 20mm |
| `2024-11-practice-above0-between20-below2` | `구분선위0미주사이20구분선아래2` | 위 0mm, 사이 20mm, 아래 2mm |
| `2024-11-practice-above20-between0-below20` | `구분선위20미주사이0구분선아래20` | 위 20mm, 사이 0mm, 아래 20mm |
| `2024-11-practice-above20-between7-below2` | `구분선위20미주사이7구분선아래2` | 위 20mm, 사이 7mm, 아래 2mm |
| `2024-11-practice-no-separator-above20-between20-below20` | `구분선없음구분선위20미주사이20구분선아래20` | 선 없음, 위 20mm, 사이 20mm, 아래 20mm |

## 계측 방법

1. `scripts/task1274_visual_sweep.py --target <target>`로 SVG/PDF/render-tree/annotation을 생성한다.
2. `metrics.json`의 `endnote_separator_gap`을 우선 확인한다.
   - separator 후보가 있으면 rhwp/PDF separator y와 first content y를 비교한다.
   - separator 후보가 없으면 first content y와 page/body 흐름을 별도로 확인한다.
3. `render_tree_*.json`에서 첫 미주 content와 문항 marker의 bbox를 추출한다.
4. gap 계측이 script에서 부족하면 Stage67에서 sweep metric을 확장한다.

## 검증 명령

```sh
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between7-below2 \
  --target 2024-11-practice-above0-between7-below20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage66_official_gap_baseline \
  --rhwp-bin target/debug/rhwp
```

## 판단 기준

- `separatorBelow`가 다른 샘플에서 separator-to-content gap이 같은 값만큼 변해야 한다.
- `separatorAbove`가 0/20mm로 바뀌면 본문과 separator 사이 또는 미주 block top이 변해야 한다.
- `betweenNotes`가 0/7/20mm로 바뀌면 첫 미주 block 내부가 아니라 note boundary gap만 변해야 한다.
- 구분선 없음 샘플은 선 후보가 없어도 `separatorAbove`와 `separatorBelow` 값이 content start를
  결정해야 한다.

## baseline 실행 결과

명령:

```sh
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between7-below2 \
  --target 2024-11-practice-above0-between7-below20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage66_official_gap_baseline \
  --rhwp-bin target/debug/rhwp
```

결과 요약:

| target | page count | hard gate | qflow 후보 |
|---|---:|---|---|
| `shape987` | 21/21/21 | frame/title/order 없음 | `[16, 17, 18, 20, 21]` |
| `above0-between0-below0` | 21/21/21 | frame/title/order 없음 | `[11, 12, 13, 17, 19, 20]` |
| `above0-between7-below2` | 21/21/21 | frame/title/order 없음 | `[17, 20]` |
| `above0-between7-below20` | 21/21/21 | frame/title/order 없음 | `[14, 17, 20]` |
| `above0-between20-below2` | 22/22/22 | frame/title/order 없음 | `[21, 22]` |
| `above20-between0-below20` | 21/21/21 | frame/title/order 없음 | `[11, 16, 17, 18, 19, 20, 21]` |
| `above20-between7-below2` | 21/21/21 | frame/title/order 없음 | `[17, 20]` |
| `no-separator-above20-between20-below20` | 23/23/23 | frame/title/order 없음 | `[18, 22]` |

## separator gap 관찰

대표 시작 page의 `endnote_separator_gap`:

| target | UI 위/사이/아래 | rhwp gap | PDF gap | 판단 |
|---|---|---:|---:|---|
| `shape987` | 9/8/7mm | 27.6px | 27.0px | 구분선 아래 7mm는 일치 |
| `above0-between0-below0` | 0/0/0mm | 59.6px | 43.0px | outlier. detector/첫 content 후보 확인 필요 |
| `above0-between7-below2` | 0/7/2mm | 8.6px | 7.0px | 구분선 아래 2mm는 일치 |
| `above0-between7-below20` | 0/7/20mm | 76.6px | 75.0px | 구분선 아래 20mm는 일치 |
| `above0-between20-below2` | 0/20/2mm | 8.6px | 7.0px | 구분선 아래 2mm는 일치 |
| `above20-between0-below20` | 20/0/20mm | 77.0px | 76.0px | 구분선 아래 20mm는 일치 |
| `above20-between7-below2` | 20/7/2mm | 9.0px | 8.0px | 구분선 아래 2mm는 일치 |

`구분선 위 20mm`는 separator-to-content gap이 아니라 separator line 자체의 y를 이동시킨다.
예를 들어 `above0-between7-below2`의 첫 content y는 rhwp/PDF `361/353px`이고,
`above20-between7-below2`는 `437/429px`이다. 둘 다 약 76px 증가하므로 20mm 상단 gap은
대표 시작 page에서 적용되고 있다.

## 남은 문제

1. `above0-between0-below0`만 대표 page gap이 16.6px 차이난다. 이 page는 0mm라 line과 첫 content
   후보를 더 엄격히 봐야 한다.
2. 구분선 없음 target은 separator detector가 본문/그림 가로선을 후보로 잡는다. 공식 gap 판단에
   쓸 수 없다.
3. 현재 sweep은 `betweenNotes`를 직접 계측하지 못한다. qflow/marker drift로 간접 관찰할 뿐,
   “앞 미주 visible bottom과 다음 미주 visible top 사이”를 측정하지 않는다.

## 다음 단계

Stage67에서는 source behavior 변경보다 먼저 `scripts/task1274_visual_sweep.py`를 보강한다.

- `has_visible_separator=false`이면 separator candidate를 찾지 않고 no-separator content-start
  metric을 별도로 기록한다.
- note-shape의 공식 UI 값(`above/between/below`, separator visible)을 page metric에 함께 기록한다.
- `betweenNotes` 검증을 위해 red question marker 또는 render tree text bbox 기준의 note boundary
  gap 후보를 산출한다.
