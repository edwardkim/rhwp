# Stage 2b 보고서 — Task #485 Bug-2 정정 (boundary epsilon)

**작성일**: 2026-05-07
**브랜치**: `local/task485`
**대상**: `src/renderer/layout/table_layout.rs` `compute_cell_line_ranges`

---

## 1. 변경 요약

`abs_limit` 와 cell-clip-rect bottom 의 미세 어긋남 + 글자 descender 여유분으로 인해, `line_end_pos` 가 abs_limit 와 ~0~2px 차이로 fit 하면 시각적으로 본문 경계를 침범하던 결함 정정.

`SPLIT_LIMIT_EPSILON = 2.0px` 마진을 적용한 `effective_limit` 도입. break/exceed 비교에 사용.

## 2. 변경 내용 (diff)

### `src/renderer/layout/table_layout.rs:2214 부근`

```rust
// [Task #485 Bug-2] boundary epsilon — abs_limit 와 cell-clip-rect bottom 의 미세 어긋남 +
// descender 여유분 흡수. line_end_pos 가 abs_limit 와 ~0~2px 차이로 fit 하면
// 시각적으로 본문 경계 침범 → 다음 페이지로 밀어냄.
const SPLIT_LIMIT_EPSILON: f64 = 2.0;
let effective_limit = if has_limit { content_offset + content_limit - SPLIT_LIMIT_EPSILON } else { 0.0 };
```

### atomic 분기

```rust
// [Task #485 Bug-2] boundary epsilon 적용 — descender 여유분
let exceeds_limit = has_limit && para_end_pos > effective_limit && !bigger_than_page;
```

### 일반 분기 (line 단위)

```rust
if has_limit && line_end_pos > effective_limit {
    // [Task #485 Bug-2] boundary epsilon 적용 — line_end_pos 가 abs_limit 와 ~0~2px 차이로 fit 하면
    // cell-clip-rect bottom 과 descender 가 충돌 → 다음 페이지로 밀어냄.
    // [Task #485 Bug-1] outer 루프도 차단 — 후속 단락의 작은 line_h slip 방지.
    limit_reached = true;
    break;
}
```

이전 `abs_limit` 변수는 제거 (effective_limit 으로 통일).

## 3. epsilon 값 결정 근거

### 3.1 측정 데이터 (Stage 1)

| 페이지 | 마지막 visible 줄 gap (limit-end) |
|--------|----------------------------------|
| p21 pi=108 | +0.947 |
| p20 pi=169 (post 2a 차단) | +1.120 |
| p15 pi=84 (post 2a 차단) | +1.973 |

### 3.2 epsilon = 2.0px 선택 사유

- p21 의 0.947px gap 차단을 위해 ε ≥ 1.0px 필요
- 마진 안전성 위해 **2.0px** (descender 영역 + 부동소수점 오차)
- line_h (13~24px) 대비 ~10% — 폰트 크기 변화에 적절
- 2.5px 이상은 너무 보수적으로 페이지 적재량 손실 위험
- 고정값 채택: `line_h × 0.1` 비례안은 작은 line_h 에서 ε 가 1.3px 미만으로 작아져 효과 부족

## 4. 검증 결과

### 4.1 시각 검증 (`samples/synam-001.hwp` 페이지 15·20·21)

| 페이지 | Stage 2a 후 | **Stage 2b 후** |
|--------|-------------|-----------------|
| p15 | 클립 해소 ✓ | 동일 (영향 없음 — 마지막 줄 gap=16.6px ≫ ε) |
| p20 | 클립 해소 ✓ | 동일 (영향 없음 — 마지막 줄 gap=14.5px ≫ ε) |
| p21 | **클립 잔존** | **클립 해소** ✓ — pi=108 (gap=0.947) 차단됨 |

### 4.2 cargo test 결과

```
passed: 1199, failed: 0
```

전 테스트 통과 — Task #431/#362/#398/#474 회귀 없음.

### 4.3 회귀 점검 (kps-ai.hwp Task #362 영역)

- p56/p67/p68/p69/p70/p72/p73 export-svg 정상 출력 (시각 확인)
- 표 분할 + 빈 페이지 미발생 (Task #431 의도 보존)
- 분할 표 페이지 적재량 정상 (Task #362 의도 보존)

### 4.4 인접 페이지 흐름 (synam-001 p13~p22)

- p13~p16: 분할 표 흐름 정상 (PDF p14~p16 과 시각 정합)
- p20~p22: 분할 표 흐름 정상 (PDF p20~p22 과 시각 정합)

## 5. 위험 / 부작용

### 5.1 epsilon 임의성

2.0px 는 측정 데이터 기반이지만 본질적으로 휴리스틱. 폰트 크기 변경(예: ㅎfont-size=20px) 시 마진이 부족할 가능성. 본 이슈 범위에서는 일반 한컴 문서의 표 분할 케이스에 적합한 값.

### 5.2 페이지 적재량 미세 감소

이론적으로 모든 분할 표가 마지막 ~2px 영역을 양보 — 일부 케이스에서 분할 줄이 1줄 더 다음 페이지로 밀릴 수 있음. 검증한 회귀 샘플에서는 미발생.

### 5.3 layout drift 본질 통일은 별도 이슈

`typeset_layout_drift_analysis.md` 의 "단일 모델 통합" 은 본 정정 영역 밖. 본 epsilon 은 그 구조적 본질의 임시 흡수.

## 6. 산출물 / 커밋

- 변경 파일: `src/renderer/layout/table_layout.rs` (epsilon 상수 + effective_limit + 2곳 비교 변경)
- 커밋 예정: `Task #485 Stage 2b: boundary epsilon (2.0px) 적용`

## 7. 작업지시자 승인 요청

1. epsilon = 2.0px 선택 동의?
2. 시각 검증 결과 (p21 클립 해소, 인접 페이지 정상) 동의?
3. Stage 3 (회귀 검증 + 보강) 진행 동의?

승인 후 Stage 3 진행.
