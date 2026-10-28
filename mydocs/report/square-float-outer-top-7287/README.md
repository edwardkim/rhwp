---
kind: report
status: active
last_verified: 2026-09-21
---

# [#7287] 빈 host 어울림(Square) 자리차지 표가 자기 위쪽 바깥여백 안으로 들어간다

- 대상 SHA: `fix/7203-stored-outer-box-top`([PR #7286](https://github.com/edwardkim/rhwp/pull/7286)) 위
- 기준 PDF: `pdf/hwpctl_API_v2.4-hwp-2020.pdf` (engine 2020),
  `pdf/hwp_table_test-m-hwp-2020.pdf` (대조군)
- 계측: PyMuPDF `get_drawings()` 의 가로 괘선 중 **표 폭과 일치하는 것**만
  (같은 y 근처에 칸 안쪽 괘선이 함께 있다)

## 1. 왜 여백이 안 실렸나 — 전용 갈래가 없었다

`layout.rs` 의 `table_y_start` 체인을 갈래마다 다른 값으로 태그해 빌드하고 어느 것이
실행되는지 실측했다. 다섯 건 모두 **마지막 폴백**(태그 48)으로 떨어진다.

```text
   probe: 갈래별 +41 .. +48px      →  세 건 모두 정확히 +48.0px 이동
```

곧 어울림 자리차지 표에는 전용 갈래가 없어 흐름 위치(`y_offset`)가 그대로 쓰였다.
저장 앵커 경로는 모두 자리차지(T&B) 전용이라 타지 않는다.

| 경로 | 왜 안 타나 |
| --- | --- |
| `native_empty_single_topbottom_table_saved_top` | `is_para_topbottom_float` 요구 |
| 빈 host lane 의 `fragment_outer_top_px` | 좁은 술어 둘(`RowBreak` · `#6378`) 모두 T&B 전용 |
| `compute_table_y_position` 절대 배치 | `TopAndBottom \| BehindText \| InFrontOfText` 만 받음 |

앞의 둘은 각각 **3000HU(40px)** · **+40px** 을 넣어 **반응이 없음을 확인**해 배제했다.

## 2. 정본 대조 — 5건

| 쪽 | pi | 수정 전 | 정본 | Δ 전 | 수정 후 | Δ 후 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 1555 | 838.60 | 841.48 | **+2.88** | 842.30 | −0.82 |
| 93 | 2404 | 558.70 | 561.95 | **+3.25** | 562.40 | −0.45 |
| 94 | 2426 | 173.90 | 177.57 | **+3.67** | 177.60 | −0.03 |
| 95 | 2469 | 406.30 | 409.79 | **+3.49** | 410.10 | −0.31 |
| 96 | 2509 | 438.50 | 441.92 | **+3.42** | 442.30 | −0.38 |

어긋남이 선언 `outMargin.top` 283HU(3.77px)에서 괘선 stroke/2(0.32px)를 뺀 값과 같다.

## 3. 비범위 — 가시 host (대조군)

host 에 글자가 있는 어울림 자리차지 표는 host 줄의 흐름이 이미 자리를 정한다.
`samples/hwp_table_test-m.hwp` 1쪽 표(pi=3, `vertOffset>0`)가 그 갈래이고 정본
`pdf/hwp_table_test-m-hwp-2020.pdf` 의 250.77(x=117.06 w=555.54)에 대해 rhwp 는 **251.0
으로 이미 맞다**. 여백을 더하면 벗어나므로 이 변경은 **빈 host 로 한정**했다.

그 경계는 `issue_5566_square_table_voffset` 이 이미 지키고 있다 — 한정 없이 켜면 그
계약이 `delta=21.60px (기대 25.39px)` 로 깨진다(실제로 겪었다).

## 4. 검증

- `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast`
  → **10,088 중 10,087 PASS**. 잔여 1건 `wmf_emf_goldens` 는 로컬 CRLF checkout 산물.
- **samples 전수 A/B (1,139문서) — 모든 지표 동일.** 쪽수 변동 0 · overflow 2,577 = 2,577 ·
  text-overlap 3,541 = 3,541 · overlap 125 · off-canvas 343 · empty-page 95.
- **Visual Sweep** (`--pages 64,94,95`)

  | | base | 수정 후 |
  | --- | ---: | ---: |
  | 평균 pixel match | 94.668% | **95.167%** |
  | 평균 ink match | 12.976% | **14.243%** |
  | 최악 pixel match | 94.532% | **94.816%** |

  직접 판독: `square-float-top-vs-oracle-p94.png` — 정본 괘선(177.57)에 빨간 기준선을 긋고
  정본·수정 전·수정 후를 겹쳤다. 수정 전에는 표 윗변이 기준선 **위로** 벗어나 있고,
  수정 후에는 겹친다.
