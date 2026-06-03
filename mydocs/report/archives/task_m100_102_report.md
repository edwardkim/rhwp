# Task #102: 최종 완료보고서 — 문단 줄 단위 페이지 경계 방어 로직

> **이슈**: [#102](https://github.com/edwardkim/rhwp/issues/102)
> **브랜치**: `local/task102`
> **작성일**: 2026-04-11
> **마일스톤**: M100 (v1.0.0)

---

## 목표

hwpspec.hwp 18페이지(page=5) s2:pi=65 PartialParagraph 첫 줄이 편집용지 높이를 13.2px 초과하여 시각적으로 클리핑되는 현상 수정.

```
LAYOUT_OVERFLOW: page=5, col=0, para=65, type=PartialParagraph, y=1047.3, bottom=1034.1, overflow=13.2px
```

---

## 구현 경위

### 1차 시도: 페이지네이션 단계 방어 (레이어 1 guard)

PartialParagraph 배치 직전, 페이지 끝 근방 구역에서 overflow 예상 시 다음 페이지로 이동하는 guard 구현.

**결과**: page 22 회귀 발생.
- pi=65가 page 18→19로 밀리면서 연쇄 페이지 밀림
- `#62` 이슈(선/선/표/표 구조 pi=126 ci=3) 재발

### 2차 방향: 렌더링 단계 시각적 클램핑 (최종 채택)

작업지시자 지시에 따라 페이지네이션은 그대로 유지하고,
**렌더링 시점에 줄 y 위치가 단 하단(col_bottom)을 초과하면 col_bottom 바로 위로 클램핑**하는 방식으로 전환.

---

## 최종 구현

### 수정 파일: `src/renderer/layout/paragraph_layout.rs`

#### `layout_composed_paragraph` (일반 경로)

줄 배치 직전, 셀 외부에서 줄 하단이 col_bottom을 초과하면 y를 클램핑:

```rust
// TODO: 높이 계산 오차에 대한 임시 방어 로직.
// 줄 하단(text_y + line_height)이 단 하단(col_bottom)을 초과하면 col_bottom 바로 위로
// 클램핑하여 줄이 페이지 경계를 벗어나 시각적으로 잘리는 현상을 방지한다.
// current_height 누적이 정확해지면 이 코드는 제거 가능하다.
let col_bottom = col_area.y + col_area.height;
let text_y = if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
    let clamped = (col_bottom - line_height).max(col_area.y);
    y = clamped;
    clamped
} else {
    text_y
};
```

#### `layout_raw_paragraph` (fallback 경로)

동일한 클램핑 로직 적용.

### 추가 필드: `src/renderer/pagination/state.rs`

레이어 2 관련 필드(`defense_counts`, `overflow_carry`, `layer1_advancing`) 및
`check_last_item_overflow`, `reinsert_carry_with_height` 함수 추가 (현재 비활성).
향후 페이지네이션 단계 방어 로직 추가 시 활용 가능.

---

## 검증 결과

### SVG 렌더링 클램핑 확인 (hwpspec.hwp 18페이지 pi=65)

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| 줄 baseline y (translate) | 1037.3px | 1032.0px |
| col_bottom | 1034.1px | 1034.1px |
| 시각적 클리핑 | 발생 | 해소 |

### 전체 226개 샘플 LAYOUT_OVERFLOW

| 항목 | 건수 |
|------|------|
| 수정 전 (기준) | 70건 |
| 수정 후 | 69건 |
| 변화 | -1건 (개선) |

### page 22 회귀 없음

pi=126 ci=3 테이블이 22페이지에 정상 위치 확인.

### 단위 테스트

785개 전체 통과.

### WASM 빌드

성공.

---

## 제약 사항 및 향후 과제

- LAYOUT_OVERFLOW 경고는 페이지네이션 계산 기반이므로 계속 출력됨 (렌더링 개선과 별개)
- 표(`Table`, `PartialTable`) 클리핑은 이번 범위 외
- 근본 수정(current_height 정확도 향상) 시 클램핑 코드 제거 가능 (TODO 주석 명시)
- 비-TAC wrap=위아래 표 out-of-flow 레이아웃 문제 → 이슈 [#103](https://github.com/edwardkim/rhwp/issues/103)으로 별도 추적
