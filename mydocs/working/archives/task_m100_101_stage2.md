# Task #101: 2단계 완료 보고서

> **작성일**: 2026-04-11
> **단계**: 2단계 — 비-TAC 표 캡션 `current_height` 보정

---

## 수행 내용

### 근본 원인 (v4에서 확정)

비-TAC 표의 `table_total_height = effective_height + host_spacing`에 캡션 높이가 포함되지 않아
`current_height`가 레이아웃 실제 y_offset보다 작게 누적되는 문제.

- pi=179 캡션 누락: 20.22px
- pi=180 캡션 누락: 24.03px
- 누적 discrepancy: ~36.65px → page_avail 과도 계산 → rows=0..13 선택 → overflow 28.2px

### 수정 내용 (`src/renderer/pagination/engine.rs`)

1. **`paginate_table_control`**: `caption_extra_for_current` 계산 추가
   - 비-TAC 표이고 Top/Bottom 캡션이 있을 때만 캡션 높이 + spacing 계산
   - TAC 표, Left/Right 캡션 표는 0.0

2. **`place_table_fits`**: 시그니처에 `caption_extra_for_current: f64` 파라미터 추가
   - `is_independent_float` 분기 제외, 나머지 `current_height` 업데이트에서 포함:
     `st.current_height += table_total_height + caption_extra_for_current`

3. **피트 판단 변경 없음**: `effective_table_height`는 기존 그대로 (`table_total_height` 기반)
   - 캡션이 피트 판단에 영향을 주지 않아 기존 표 배치 로직 유지

---

## 검증 결과

### 30페이지 pi=181

```
수정 전: PartialTable pi=181 ci=0 rows=0..13 cont=false 33x4  →  LAYOUT_OVERFLOW 28.2px
수정 후: Table pi=181 ci=0 (31페이지 전체 배치)              →  LAYOUT_OVERFLOW 없음
```

pi=180, pi=181이 31페이지로 이동하여 pi=181 전체(33행)가 한 페이지에 배치됩니다.

### LAYOUT_OVERFLOW 총 건수

```
수정 전: 43개
수정 후: 20개
해결: 23개 (pi=181 포함), 새 회귀: 0건
```

### 단위 테스트

```
785 passed; 0 failed; 1 ignored
```

---

## 수정 파일

- `src/renderer/pagination/engine.rs`

