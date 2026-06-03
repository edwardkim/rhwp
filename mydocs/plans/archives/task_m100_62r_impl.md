# Task #62 재오픈: [선][선][표][표] 레이아웃 오류 — 구현계획서

> **이슈**: [#62](https://github.com/edwardkim/rhwp/issues/62)
> **브랜치**: `local/task62-reopen`
> **작성일**: 2026-04-10

---

## 구현 전략

수정 대상은 `engine.rs`의 `paginate_table_control` 함수 내 vert_offset 피트 판단 로직 1곳이다.
핵심은 `st.current_height`(누적값) 대신 **문단 시작 y 스냅샷**을 피트 기준으로 사용하는 것이다.

단계는 3단계로 구성한다.

---

## 1단계: 문단 시작 y 스냅샷 도입 + 피트 판단 수정

### 대상 파일
- `src/renderer/pagination/engine.rs`

### 변경 내용

#### 1-1. `process_controls` 진입 시 문단 시작 height 스냅샷 캡처

`process_controls` 호출 직전 또는 `paginate_table_control` 인자로 `para_start_height`를 전달한다.

```rust
// engine.rs — process_controls 호출 전
let para_start_height = st.current_height;

self.process_controls(
    &mut st, para_idx, para, measured, &measurer,
    para_height, para_height_for_fit, base_available_height, page_def,
    para_start_height,  // 추가
);
```

또는 `paginate_table_control` 시그니처에 `para_start_height: f64` 인자 추가.

#### 1-2. `effective_table_height` 피트 판단 수정

현재 (잘못된) 코드:
```rust
// engine.rs:1067-1079
let effective_table_height = if !is_tac_table
    && matches!(table_text_wrap, TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Para)
    && table.common.vertical_offset > 0
{
    effective_height + host_spacing + v_off   // ← current_height에 더해짐
} else {
    table_total_height
};
```

수정 후:
```rust
// vert=Para + vert_offset > 0인 비-TAC 자리차지 표:
// 피트 판단을 current_height 누적 기준이 아닌 문단 시작 y 기준으로 수행
let effective_table_height = if !is_tac_table
    && matches!(table_text_wrap, TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Para)
    && table.common.vertical_offset > 0
{
    // 문단 시작 y + vert_offset + 표높이가 페이지 내에 들어오면 통과
    // effective_table_height를 (para_start_height 기준 잔여 공간) 계산에 맞게 조정
    let v_off = hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
    let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
    // 피트 판단용 가상 높이: abs_bottom - current_height
    // (current_height + effective_table_height <= available 판단식에 대입)
    (abs_bottom - st.current_height).max(effective_height + host_spacing)
} else {
    table_total_height
};
```

단, `st.current_height += table_total_height`(1199줄)는 현행 유지 —
자리차지 표는 본문 높이를 소비하지 않으므로 실질적 current_height 증가는 0이어야 한다.
이 부분은 2단계에서 별도 검토한다.

### 검증
- `dump-pages samples/hwpspec.hwp -p 21` 출력에서 `pi=126 ci=3`이 `PartialTable` → `Table`로 변경 확인
- SVG 내보내기: 22페이지에 ci=3 표가 통째로 배치되는지 확인

---

## 2단계: current_height 누적 오류 검토 및 수정

### 배경

비-TAC 자리차지 표(wrap=위아래, tac=false)는 본문 텍스트 흐름에서 **공간을 차지하지 않는다**.
한컴에서 이런 표는 문단 위치에 독립 배치되고, 다음 문단은 표 아래가 아닌 **문단 시작 기준**으로 배치된다.

그러나 현재 `place_table_fits`에서 `st.current_height += table_total_height`(1199줄)가 실행되어
이런 표도 본문 높이를 소비하는 것으로 처리된다.

### 확인 항목
- ci=2 표 처리 후 `st.current_height`가 얼마나 증가하는지 확인
- ci=2 표의 증가분이 ci=3 피트 판단에 영향을 주는지 검증
- 비-TAC 자리차지 표의 `current_height` 증가를 0으로 해야 하는지, 또는 표 높이만큼 증가가 맞는지 확인 (다른 파일 회귀 확인)

### 수정 방향 (검토 후 결정)
- vert=Para + vert_offset=0인 비-TAC 자리차지 표(ci=2): current_height를 표 높이 대신 **문단 라인 높이**만 증가시키는 방향 검토
- 단, 이 변경은 회귀 위험이 크므로 먼저 1단계 적용 후 영향 범위 확인

---

## 3단계: 회귀 테스트 + 검증

### 테스트 항목

| 항목 | 방법 | 기대 결과 |
|------|------|----------|
| pi=126 표 분할 수정 | `dump-pages -p 21` | `ci=3 Table` (PartialTable 아님) |
| 22페이지 SVG | `export-svg -p 21 --debug-overlay` | ci=3 표가 22페이지에 완전 배치 |
| ci=1 선 범위 | 22페이지 SVG | 23페이지로 넘어가지 않음 |
| 전체 샘플 회귀 | 전체 SVG 내보내기 비교 | 0건 변화 또는 개선만 있음 |
| 기존 PartialTable 케이스 | 다른 표 분할 샘플 | 분할 동작 유지 |

### 회귀 기준 파일
- 이전 수정(Task #62 1차)에서 225개 파일 회귀 0건이었던 기준 유지

---

## 구현 순서 요약

| 단계 | 작업 | 파일 | 위험도 |
|------|------|------|--------|
| 1 | para_start_height 도입 + effective_table_height 수정 | engine.rs | 낮음 |
| 2 | current_height 누적 오류 검토 및 수정 | engine.rs | 중간 |
| 3 | 회귀 테스트 전체 수행 | — | — |
