# Task #352 Stage 3 완료보고서: dash run 시각 라인 통합

> 2026-04-28 | Branch: `local/task352`

---

## 변경 사항

### `src/renderer/svg.rs` (line 1882~)

연속 3+ ASCII 하이픈(`-`) 클러스터 시퀀스를 식별하여 dash 글리프 출력을 스킵하고, 필요 시 단일 `<line>` 으로 통합 렌더한다.

추가 항목:
- `dash_run_groups: Vec<(usize, usize)>` — 클러스터 인덱스 기준 연속 dash run 범위
- `dash_line_y_offset = -font_size * 0.32` — baseline 기준 dash 중앙선 근사
- `dash_line_stroke_w = (font_size * 0.07).max(0.5)` — dash 두께 모방
- `cluster_in_dash_run(cluster_idx)` — 해당 클러스터가 run 의 첫 위치인지·내부인지 판정
- `suppress_dash_leader_line = !style.underline.is_none()` — underline 이 있으면 dash leader 라인 생략 (이중선 방지)

shadow loop 와 main loop 양쪽에 동일 패턴 적용. dash 글리프는 항상 스킵, 라인 출력만 underline 부재 시.

### `src/renderer/web_canvas.rs` (line 1284 근처)

WASM/canvas 경로에 동일 로직 적용.

### `src/renderer/layout/text_measurement.rs`

Stage 2 의 `font_size * 0.3` 을 `font_size * 0.32` 로 미세 조정 — Q32 라인 폭이 PDF(~135 px) 보다 약간 짧다는 작업지시자 피드백 반영.

---

## 검증

### Q32 블랭크 라인 (5 페이지)

| 항목 | Stage 2 | Stage 3 (현재) | PDF 목표 |
|------|---------|---------------|----------|
| dash 글리프 수 | 29 | **0** | 0 (PDF 도 글리프가 보이지 않음) |
| 가로선 수 | 2 (dash bar + underline) | **1** (underline 만) | 1 |
| underline 폭 | 126.5 px | **134.94 px** | ~135 px ✓ |
| `of being` 시작 x | 770 | 839 | 컬럼 중앙쯤 |

### 측정 데이터

```
<line x1="597.12" y1="1041.75" x2="732.06" y2="1041.75" stroke="#000000" stroke-width="1"/>
폭 = 134.94 px
```

dash 글리프 출력 스킵 확인:
```bash
grep 'translate\([0-9.]+,1039\.[0-9]+\).*>-<' exam_eng_005.svg | wc -l
→ 0
```

### 작업지시자 피드백 대응

1. **"왜 2줄로 그려지나?"** → underline 이 있는 run 의 dash leader 라인 생략. 단일 underline 만 노출.
2. **"폭이 조금 짧음"** → 0.3 em → 0.32 em 으로 미세 확장. 134.94 px 가 PDF(~135) 와 일치.

### cargo test --release

전 테스트 통과 (1023 + 모든 통합 테스트). 회귀 없음.

---

## 부수 변경 영향

dash leader 라인 (underline 없는 경우) 의 y 위치 / 두께:
- y: baseline - 0.32 × font_size (≈ x-height 중앙)
- 두께: 0.07 × font_size (≈ 1.07 px @ 15.3 font)

가운데점(`is_middle_dot`) 패턴과 동일한 SVG/canvas 의 시각 도형 직접 그리기 방식. 폰트 비의존.

---

## 다음 단계

Stage 4: 전 샘플 회귀 + 최종 보고서 + orders 갱신.
