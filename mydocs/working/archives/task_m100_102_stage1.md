# Task #102: 1단계 완료보고서 — 렌더링 클램핑 방어 로직 구현

> **이슈**: [#102](https://github.com/edwardkim/rhwp/issues/102)
> **브랜치**: `local/task102`
> **작성일**: 2026-04-11
> **단계**: 1단계 (최종 구현)

---

## 구현 방향 변경 경위

구현계획서 v2(레이어 1/2 방어 로직)를 기준으로 레이어 1 guard를 먼저 구현하였으나,
page 22 회귀(pi=126 ci=3 테이블이 23페이지로 밀림)가 발생하였다.

원인 분석:
- 레이어 1 guard가 pi=65를 page 18 → 19로 이동
- 연쇄 페이지 밀림으로 page 22의 [선][선][표][표] 구조(pi=126)에서 ci=3 테이블이 23페이지로 밀림
- `#62` 이슈 재발

**작업지시자 지시**: "PartialParagraph 배치를 막는 방식이 아니라, 이미 배치된 PartialParagraph가 클리핑될 때 시각적으로 보완하는 방향으로 접근"

---

## 실제 구현: 렌더링 단계 시각적 클램핑

### 핵심 원칙

페이지네이션 결과는 그대로 유지하고, **렌더링 시점에 줄 y 위치가 단 하단(col_bottom)을 초과하면 col_bottom 바로 위로 클램핑**하여 시각적 클리핑을 방지한다.

### 수정 파일: `src/renderer/layout/paragraph_layout.rs`

#### 1. `layout_composed_paragraph` 함수 (줄 545줄 근방 TextLine 생성 직전)

```rust
// TODO: 높이 계산 오차에 대한 임시 방어 로직.
// 줄 하단(text_y + line_height)이 단 하단(col_bottom)을 초과하면 col_bottom 바로 위로
// 클램핑하여 줄이 페이지 경계를 벗어나 시각적으로 잘리는 현상을 방지한다.
// current_height 누적이 정확해지면 이 코드는 제거 가능하다.
let col_bottom = col_area.y + col_area.height;
let text_y = if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
    let clamped = (col_bottom - line_height).max(col_area.y);
    // 클램핑 결과를 y에도 반영하여 이 줄의 모든 자식 노드(TextRun 등)가
    // 클램핑된 y를 기준으로 배치되도록 한다.
    y = clamped;
    clamped
} else {
    text_y
};
```

**적용 조건**:
- `cell_ctx.is_none()`: 셀 내부 레이아웃은 제외 (셀 내부 overflow는 정상 케이스)
- `text_y + line_height > col_bottom + 0.5`: 줄 하단이 단 하단 + 0.5px 초과 시에만

#### 2. `layout_raw_paragraph` 함수 (fallback 경로)

동일한 클램핑 로직을 `layout_raw_paragraph`에도 추가 (ComposedParagraph 없는 폴백 경로 보완):

```rust
// TODO: 높이 계산 오차에 대한 임시 방어 로직.
let col_bottom = col_area.y + col_area.height;
let y_clamped = if y + line_height > col_bottom + 0.5 {
    (col_bottom - line_height).max(col_area.y)
} else {
    y
};
```

### 비활성 코드: `src/renderer/pagination/state.rs`

레이어 2 관련 필드 및 함수가 추가되어 있으나 현재 호출되지 않음.
향후 페이지네이션 단계 방어 로직 추가 시 활용 가능. 현재는 참고용으로 유지.

---

## 검증 결과

### LAYOUT_OVERFLOW 건수

| 구분 | 건수 |
|------|------|
| 기준 (Task #101 완료 후) | 70건 |
| 현재 (렌더링 클램핑 적용 후) | 69건 |
| 변화 | -1건 (개선) |

### hwpspec.hwp LAYOUT_OVERFLOW (4건 유지)

```
LAYOUT_OVERFLOW: page=5, col=0, para=65, type=PartialParagraph, y=1042.1, bottom=1034.1, overflow=8.0px
LAYOUT_OVERFLOW: page=9, col=0, para=127, type=FullParagraph, y=1042.1, bottom=1034.1, overflow=8.0px
LAYOUT_OVERFLOW: page=15, col=0, para=170, type=Table, y=1038.9, bottom=1034.1, overflow=4.8px
LAYOUT_OVERFLOW: page=40, col=0, para=344, type=Table, y=1036.6, bottom=1034.1, overflow=2.5px
```

> 참고: LAYOUT_OVERFLOW는 페이지네이션 계산 기반 경고로, 렌더링 클램핑과 무관하게 계속 출력된다.
> 실제 렌더링에서는 pi=65, pi=127이 클램핑되어 시각적 클리핑 해소.
> 기존 Table overflow 2건(page=15, page=40)은 이번 범위 외.

### 22페이지 회귀 없음

pi=126 ci=3 테이블이 22페이지에 정상 위치 (Layer 1 guard 제거로 해소).

### SVG 렌더링 클램핑 확인

- 이전: translate y=1037.3 (col_bottom=1034.1 기준 3.2px 초과)
- 이후: translate y=1032.0 (col_bottom 2.1px 이내)

### 단위 테스트

785개 테스트 전체 통과.

---

## 제약 사항

- 표(`Table`, `PartialTable`)는 클램핑 대상 아님 (이번 이슈 범위 외)
- LAYOUT_OVERFLOW 경고는 페이지네이션 계산 결과를 반영하므로 계속 출력됨
- 렌더링 클램핑은 시각적 보완이며, 페이지네이션 정확도 향상은 별도 과제
