# Stage 4 진행보고서 — #1022: 검증 + 잔여 분석

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 4 — Stage 3 정합 검증 + 페이지 22 잔여 잔존 원인 추적

## 1. Stage 3 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 무경고 |
| `cargo clippy --release` | 무경고 |
| `cargo test --release` | **1302 passed**, 0 failed, 6 ignored |
| `svg_snapshot` | **8 passed** (form-002 부동소수 갱신) |
| `LAYOUT_OVERFLOW` (비공개 184페이지) | **42 → 38건** (4건 감소) |

베이스라인(8be5e0c2) LAYOUT_OVERFLOW 도 **42건** — 이전 grep 정규식 오류로 0 으로 잘못 보고. task993 회귀가 아니라 사전 존재 issue 였다.

## 2. 페이지 22 18.3px 잔존 — 깊이 분석

### 2-1. 항목별 paginator vs 렌더러 advance

| 항목 | paginator total_height | 렌더러 advance |
|------|----------------------|----------------|
| pi=221 PartialTable (cont=true) | ~506.6 | 508.55 |
| pi=222 FullParagraph (빈, ps_id=985 lh=1300 ls=1040) | 31.2 | 31.20 |
| pi=223 Table (TAC inline) | ~288.1 + host | 319.61 |
| pi=224 FullParagraph (pi=222 와 동일 구조) | 31.2 | 31.20 |
| pi=225 FullParagraph (pi=222·224 와 동일 구조) | 31.2 | **45.07** |
| pi=226 PartialTable (cont=false) | ~21.9 | 23.78 |
| **합계** | 902.2 | 959.41 |

차이 57.21 = 18.3 (페이지 초과) + 38.91 (본문~viewBox 여백에 흡수).

### 2-2. 핵심 발견 — VPOS_CORR (`layout.rs:2455~2469`)

pi=225 가 pi=224 와 동일 구조(ps_id=985 lh=1300 ls=1040)인데 13.87px 더
advance 하는 원인은 렌더러의 **HWP LINE_SEG.vpos 기반 위치 보정**:

```
[dbg1022b] page=21 item=Full pi=224 y_offset_after=996.360
[dbg1022b-enter] pi=225 y_start=1010.227 ...     ← VPOS_CORR 가 +13.867 추가
[dbg1022b-line] pi=225 line=0 lh=17.333 ls=13.867 y_before=1010.227
[dbg1022b] page=21 item=Full pi=225 y_offset_after=1041.427
```

`layout.rs:2455` 의 `VPOS_CORR` 블록:
- pi=N+1 의 LINE_SEG.vpos 절대값 + col_area.y = `end_y`.
- `if end_y >= col_area.y && end_y <= col_bottom && end_y >= y_offset - 8px && !stale_table_host: y_offset = end_y`.

pi=225 의 vpos=1248996 HU 가 pi=224 자연 종료점보다 13.87px 아래를 가리키므로 렌더러가 y_offset 을 vpos 위치로 정정. 페이지네이터(typeset.rs) 의 `MeasuredParagraph.total_height` 누적은 vpos 보정을 모름 — paginator 902.2 vs renderer 959.4 차이의 주요 원인.

pi=223 의 31.6px 도 동일 부류로 추정(Table item 의 vpos 기반 위치 보정).

## 3. 진정한 원인 — VPOS_CORR ↔ paginator 미정합

페이지 22 잔여 57.21px(18.3 페이지초과 + 38.9 본문~viewBox margin 흡수)
의 주요 원인은 **렌더러의 HWP LINE_SEG.vpos 기반 위치 정정과 paginator
의 `MeasuredParagraph` 누적 사이 미정합**:

- 렌더러: 각 paragraph/table 의 시작 y_offset 을 HWP LINE_SEG.vpos 절대값
  으로 정정. 항목 사이 자연 누적과 vpos 기반 정정이 다를 때 vpos 우선.
- 페이지네이터: `MeasuredParagraph.total_height` 만으로 누적. vpos 기반
  보정 미반영.

이는 본 #1022 의 명시 범위(`HeightMeasurer ↔ cell_units`)와 **별개의
부류** — paginator vs renderer 의 paragraph/table 위치 정합 문제.

## 4. 작업량 재산정

당초 5~10시간 예상 → **실제 작업량은 multi-day 작업**:

- VPOS_CORR 모든 케이스 감사 (어떤 조건에서 발동, 어떤 결과 가져오는지).
- 두 방향 중 선택:
  - (방향 1) paginator 가 vpos-기반 위치를 추적. typeset.rs 의 모든
    `current_height +=` 가 vpos 보정을 반영하도록 광범위 변경.
  - (방향 2) 렌더러의 VPOS_CORR 비활성화. 단 이미 다수 문서에서
    VPOS_CORR 가 정정 효과를 내고 있어 회귀 위험 큼.
- 공개 골든 SVG 광범위 회귀 점검.

## 5. Stage 3 까지의 성과·옵션

- **본 #1022 Stage 3 완료분**: `cell_units` ↔ `HeightMeasurer` 정합 완료
  (42→38 events, -4건). cell_units 알고리즘이 HeightMeasurer 와 동일한
  per-cell `max(cell.height, content+pad)` 규칙으로 통일. task993 컷
  모델의 측정 기반 일치.

- **잔여 38건 (페이지 22 18.3 포함)**: VPOS_CORR ↔ paginator 정합이
  본질이며 본 타스크 명시 범위 밖. multi-day 작업.

## 6. 추천

(a) **본 타스크 #1022 를 현 Stage 3 완료로 마무리**하고 VPOS_CORR ↔
    paginator 정합을 별도 이슈로 분리. 본 타스크는 명시 범위
    (HeightMeasurer ↔ cell_units) 를 달성했다.

(b) 추가 multi-day 작업으로 VPOS_CORR 정합까지 시도 — 회귀 위험·작업량 모두 큼.
