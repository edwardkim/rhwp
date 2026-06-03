# Task #1001 Stage 1 — 정밀 진단 보고서

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
수행/구현 계획서: [`task_m100_1001.md`](../plans/task_m100_1001.md), [`task_m100_1001_impl.md`](../plans/task_m100_1001_impl.md)

## 1. 진단 도구 / 데이터

- `RHWP_DEBUG_PAGE_BORDER=1` (Task #987 도입)
- `rhwp dump samples/hwp3-sample16-hwp5.hwp` 등
- `rhwp dump-pages -p N` (페이지 분할 측정)
- HWPX XML 직접 unzip + grep
- HWP5 스펙: `mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 표 135/136 (4.3.10.1.3 쪽 테두리/배경)

## 2. 권위 규격 확인 — HWP5 spec 표 136

쪽 테두리/배경 속성 (UINT, 4바이트):

| bit | 구분 | 값 0 | 값 1 |
|-----|------|------|------|
| **0** | 위치 기준 | 본문 기준 | 종이 기준 |
| **1** | 머리말 포함 | 미포함 | 포함 |
| **2** | 꼬리말 포함 | 미포함 | 포함 |
| 3~4 | 채울 영역 | 종이 / 쪽 / 테두리 | |

**핵심**: bit 1/2 (머리말/꼬리말 포함 여부) 는 외곽선이 머리말/꼬리말 영역까지 확장되는지 결정.

## 3. 격차 A — 페이지 번호 외곽선 안/밖

### 3-1. HWP5 binary attr 값 (parse_page_border_fill at body_text.rs:858)

| 파일 | `attr` | bit 0 | bit 1 | bit 2 | 의미 |
|------|--------|-------|-------|-------|------|
| `hwp3-sample16.hwp` (HWP3 원본) | `0x00000000` | 0 | 0 | 0 | 본문 기준 + 머/꼬 미포함 |
| `hwp3-sample16-hwp5.hwp` (HWP5 변환본) | `0x00000001` | 1 | 0 | 0 | **종이 기준 + 머/꼬 미포함** |

### 3-2. HWPX 변환본 확인 (samples/hwp3-sample16-hwp5.hwpx)

```xml
<hp:pageBorderFill type="BOTH" borderFillIDRef="2"
  textBorder="PAPER"
  headerInside="0"
  footerInside="0"
  fillArea="PAPER">
```

XML 속성과 binary attr 비트가 완전 일치:
- `textBorder="PAPER"` ↔ bit 0 = 1 (종이 기준)
- `headerInside="0"` ↔ bit 1 = 0 (머리말 미포함)
- `footerInside="0"` ↔ bit 2 = 0 (꼬리말 미포함)

### 3-3. rhwp 현재 처리 (src/renderer/layout.rs:986-1020 `build_page_borders`)

```rust
let paper_based = (pbf.attr & 0x01) != 0;  // bit 0만 사용
// ... bit 1, 2 처리 없음 ...
let (base_x, base_y, base_w, base_h) = if paper_based {
    (0.0, 0.0, layout.page_width, layout.page_height)  // 종이 전체
} else {
    (layout.body_area.x, layout.body_area.y,
     layout.body_area.width, layout.body_area.height)
};
// spacing 적용 후 그대로 그림 (머/꼬 clip 없음)
```

**rhwp 는 bit 0 만 사용하고 bit 1/2 를 완전히 무시**.

### 3-4. 격차 수치 (페이지 16 기준, 297mm 페이지)

| 항목 | 값 |
|------|-----|
| 용지 | 210 × 297 mm |
| 여백 | 좌/우=15, 상/하=10, 머리말/꼬리말=10 mm |
| body_area.y | 20 mm (top + header) |
| body_area.height | 257 mm (= 297 - 20 - 20) |
| body_area 하단 | 277 mm |
| footer_area.y | 277 mm |
| footer_area.height | 10 mm |
| Page number baseline | ~282 mm (footer 중앙 + font/3) |
| pbf.spacing | L/R/T/B = 1420 HU = 5 mm |

**Paper 기준 (현재 rhwp HWP5 변환본 동작)**:
- 외곽선 하단 = 297 - 5 = **292 mm** (paper bottom - spacing)
- Page number (282 mm) < border bottom (292 mm) → **외곽선 안** ✗

**Body 기준 + 꼬리말 미포함 clip (한컴 정합 예상)**:
- 외곽선 하단 = min(body 하단 + spacing, footer_area.y) = min(282, 277) = **277 mm**
- Page number (282 mm) > border bottom (277 mm) → **외곽선 밖** ✓

**Body 기준 (HWP3 원본, 현재 rhwp 동작)**:
- 외곽선 하단 = body 하단 + spacing = 277 + 5 = **282 mm**
- Page number (282 mm) = border bottom (282 mm) → 경계선 위 (실제 시각: 외곽선 밖)
- HWP3 변형본은 body_area 자체가 header/footer 영역을 제외하므로, bit 1/2 무시되어도 결과 정합

### 3-5. Root cause 결론

**페이지 번호가 외곽선 안에 들어가는 회귀의 root cause**: rhwp 의 `build_page_borders` 가 HWP5 spec 표 136 의 bit 1 (머리말 포함) 과 bit 2 (꼬리말 포함) 를 처리하지 않음.

한컴은 `textBorder=PAPER` + `footerInside=0` 시 paper 기준 외곽선의 하단을 꼬리말 영역 진입 전 (footer_area.y) 으로 clip — rhwp 는 spacing 만 빼고 그대로 그림.

같은 root cause 가 머리말 (bit 1 = 0) 에도 적용됨: 외곽선 상단도 머리말 영역 진입 전 (header_area 끝) 으로 clip 되어야 함.

## 4. 격차 B — 페이지 분할 drift

### 4-1. 페이지 경계 자체는 정합

`dump-pages -p 15/-p 16` 비교 결과:

| 페이지 | HWP5 변환본 시작 paragraph | HWP3 원본 시작 paragraph |
|--------|-------------------------|------------------------|
| 16 (idx 15) | pi=331 "(3) 유지보수 방안" | pi=331 "(3) 유지보수 방안" |
| 17 (idx 16) | pi=374 "Ⅳ. 프로젝트 과업범위" | pi=374 "Ⅳ. 프로젝트 과업범위" |

**페이지 분할 경계는 일치** — 같은 paragraph 가 같은 페이지에서 시작.

### 4-2. 잔존: paragraph 내부 spacing_before / vpos 격차

| paragraph | HWP5 변환본 sb (spacing_before) | HWP3 원본 sb | h (높이) |
|-----------|-----------------------------|-------------|----------|
| pi=331 | 7.6 | 3.8 | 24.9 vs 21.1 |
| pi=342 | 11.4 | 5.7 | 32.7 vs 27.0 |
| pi=343 | 7.6 | 3.8 | 42.2 vs 38.5 |
| pi=374 | 11.4 | 5.7 | 38.0 vs 32.3 |

HWP5 변환본의 `spacing_before` 가 일관되게 **2 배** (Task #998 의 그 격차 패턴 동일).

Task #998 은 `line_segs` 누락 paragraph 에 대해 spacing_before=0 override 로 page count 정합 달성. line_segs 가 있는 paragraph 에 대해서는 미적용 → 위 사례처럼 2배 격차 잔존.

### 4-3. 격차 B 의 시각적 영향

페이지 16 dump-pages 출력 기준:
- 단 0 (items=43, used=931.4px, hwp_used≈932.3px, diff=-0.9px) — 거의 정합
- 단 0 (items=18 HWP3, used=883.5px, hwp_used≈811.2px, diff=+72.3px) — drift 잔존

격차 B (사용자가 보여준 다이어그램 위치 차이) 는 dump-pages 페이지 16 의 직접 비교에서는 페이지 분할이 동일하므로, **시각 격차의 주된 원인은 격차 A (외곽선 안/밖) + paragraph 내부 spacing 격차의 누적**.

## 5. Fix 방향 (Stage 2 후보)

### 5-1. 핵심 Fix (격차 A)

**`src/renderer/layout.rs:986-1020 `build_page_borders` 에서 bit 1/2 처리 추가**:

```rust
let paper_based = (pbf.attr & 0x01) != 0;
let header_inside = (pbf.attr & 0x02) != 0;
let footer_inside = (pbf.attr & 0x04) != 0;
// base 계산은 그대로 (paper or body)
// spacing 적용 후 추가 clip:
//   !header_inside 이면 border 상단 = max(border_top, body_area.y)
//   !footer_inside 이면 border 하단 = min(border_bottom, body_area.y + body_area.height)
```

### 5-2. 변경 영향 분석

| Sample | bit 0 (paper) | bit 1/2 (header/footer inside) | 현재 결과 | Fix 후 결과 |
|--------|-------------|--------------------------------|----------|------------|
| hwp3-sample16.hwp | 0 (body) | 0/0 | 외곽선 밖 ✓ | 외곽선 밖 ✓ (body_area 한계 = 자기 자신) |
| hwp3-sample16-hwp5.hwp | 1 (paper) | 0/0 | 외곽선 안 ✗ | 외곽선 밖 ✓ (header/footer clip 활성화) |
| 시험지 (#952/#987) | 추가 확인 필요 | 추가 확인 필요 | 정합 ✓ | 변동 가능성 — Stage 2 sweep 필요 |

### 5-3. 격차 B (페이지 분할 내부 drift) — 후속 분리

`line_segs` 가 있는 paragraph 의 HWP5 변환본 `spacing_before` 가 HWP3 원본의 2 배인 격차. Task #998 의 line_segs 없는 케이스 override 와 짝을 이룸. **본 Task #1001 범위 외 — 후속 issue 등록 권고**.

## 6. Stage 1 산출물 / 다음 단계

- 본 보고서: `mydocs/working/task_m100_1001_stage1.md`
- Stage 2 진입: Fix 후보 평가 (5-1 방향 + 회귀 risk + 시험지/sample sweep)
- 격차 B 후속 issue 등록 결정은 Stage 5 보고서에 명시
