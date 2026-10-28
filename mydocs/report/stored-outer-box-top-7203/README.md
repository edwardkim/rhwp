---
kind: report
status: active
last_verified: 2026-09-20
---

# [#7203] 저장 사다리가 상자 전체를 증명하면 `vpos` 는 **바깥 여백 상자의 위끝**이다

- 대상 SHA: `upstream/devel` `a3de5826c` 위의 한 줄 수정
- 기준 PDF: `pdf/hwpctl_API_v2.4-hwp-2020.pdf` (engine 2020, 105쪽)
- 계측: 전폭 427.7px 표의 **윗변·아랫변을 표 높이로 짝지어** 잠근 뒤 정본 가로 괘선과
  `export-render-tree` 의 `Table` bbox 를 대조(190건 짝지음)

## 1. 사다리가 두 갈래를 가른다

`stored_topbottom_object_span` 은 다음 저장 앵커의 advance 로 `vpos` 의 뜻을 정한다.
같은 문서·같은 선언 여백(283HU)인데 사다리 간격만 다른 두 무리가 있고, 정본이 갈라 준다.

| 사다리 advance | n | 종전 윗변 | 정본 − rhwp | 판정 |
| --- | ---: | --- | ---: | --- |
| `높이 + 위 + 아래` (정확 일치) | 18 | `vpos` | **+3.41px** | 위여백 한 개만큼 **위였다** |
| `높이 + 66HU` | 16 | `vpos − 위여백` | **+0.55px** | 이미 맞다 |

곧 앞 갈래에서 `vpos` 는 **바깥 여백 상자의 위끝**이고 표 윗변은 `vpos + outMargin.top` 이다.
종전 offset `0` 은 paint 와 예약을 맞추려던 값이었지 기하가 아니었다(모듈 주석이 그렇게 적고
있었다). 점유 끝(`outer_box_height`)은 양쪽 모두 그대로라 **예약 높이는 변하지 않는다**.

사방 균등일 때만 이 읽기를 쓴다. 저장소의 좁은 술어 셋(`#6378` · `#3820 Stage 120` ·
`native_empty_host_physical_outer_box_paint_inset`)이 모두 같은 조건을 요구하고, 정본도 같은
말을 한다 — `76076_regulatory_analysis.hwp` pi=323·324 는 `(0/0/566/566)` 이고 한/글이
여백을 싣지 않는다(정본 괘선 400.52 로 확인).

## 2. 결과 — 정본 일치율

| | base `a3de5826c` | 수정 후 |
| --- | ---: | ---: |
| 표 190건 중 \|Δy\| ≤ 1.5px | **130** | **144** |
| `+3px` 무리 | 18 | **9** |
| Δy 중앙값 | −0.19 | −0.25 |

## 3. 직접 판독

`table-top-vs-oracle-p17.png` — 정본 17쪽 표(pi=309)의 외곽 괘선에 빨간 기준선을 긋고
세 장을 겹쳐 놓았다. 수정 전에는 표 윗변이 기준선 **위로** 벗어나 있고, 수정 후에는 겹친다.

그 쪽의 정본에는 가로 괘선이 둘 있어 구별이 필요하다 — 폭으로 갈린다.

```text
   y=472.76  x=182.62  w=469.35   ← 칸 안쪽 괘선 (종전 시험 핀이 여기에 맞아 있었다)
   y=476.28  x=177.03  w=480.70   ← 표 외곽 (rhwp 표 x=177.1 w=480.5 와 일치)
```

## 4. Visual Sweep

`scripts/visual_sweep.py --file-target t7203 samples/hwpctl_API_v2.4.hwp
pdf/hwpctl_API_v2.4-hwp-2020.pdf --pages 17,18,25 --dpi 96`

| | base | 수정 후 |
| --- | ---: | ---: |
| 평균 pixel match | 93.925% | **94.040%** |
| 평균 ink match | 16.744% | **17.719%** |
| flagged page | 0 | 0 |

## 5. samples 전수 A/B (1,139문서)

**모든 지표가 동일하다** — 쪽수 변동 0, overflow 2,577 = 2,577, text-overlap 3,541 = 3,541,
overlap 125 = 125, off-canvas 343 = 343, empty-page 95 = 95. 회귀 비용 없이 표적만 움직인다.
