# Task #62 재오픈: 같은 문단 내 [선][선][표][표] 레이아웃 오류 — 수행계획서

> **이슈**: [#62](https://github.com/edwardkim/rhwp/issues/62)
> **브랜치**: `local/task62-reopen`
> **작성일**: 2026-04-10

---

## 배경

이슈 #62는 `hwpspec.hwp` 22페이지, 문단 s2:pi=126에서 발생하는 레이아웃 오류다.
이전에 부분 수정이 이루어졌으나 두 가지 문제가 미해결 상태로 클로즈되었다.

## 현상

`s2:pi=126` 문단 구조: `[Shape(선) ci=0][Shape(선) ci=1][Table ci=2][Table ci=3]`

- **문제 1**: ci=3 표(히스토리 아이템)가 22페이지에 통째로 배치되어야 하는데 22/23페이지로 분할(PartialTable)됨
- **문제 2**: ci=1 선이 22페이지 경계를 넘어 23페이지까지 이어짐 (ci=3 표 분할로 인한 연쇄 오류)

한컴 기대값: ci=3 표는 22페이지에 완전히 배치되고, 두 선은 두 표 사이를 잇는 형태.

## 원인 규명

### 구조 데이터

```
pi=126: tac=false, LINE_SEG 1개 (lh=1000, ls=600)
  ci=0: Shape(선), InFrontOfText
  ci=1: Shape(선), InFrontOfText
  ci=2: 표, wrap=위아래, vert=Para(0mm),  크기=49.4×28.4mm
  ci=3: 표, wrap=위아래, vert=Para(53mm), 크기=42.4×36.7mm
```

### 잘못된 피트 판단 로직

`paginate_table_control`의 ci=3 처리 시 `effective_table_height` 계산:

```rust
// engine.rs:1070-1079
let effective_table_height = if !is_tac_table
    && matches!(table_text_wrap, TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Para)
    && table.common.vertical_offset > 0
{
    effective_height + host_spacing + v_off  // ← vert_offset(200px)을 더함
} else {
    table_total_height
};
```

피트 판단:
```
st.current_height (ci=2 처리 후 누적) + effective_table_height(139 + 200) > available_height
= (문단누적 + 107) + 339 > 930  → 페이지 초과 판정 → PartialTable
```

### 혼동의 본질

ci=3 표는 `vert=Para(53mm)` — 문단 시작점에서 독립적으로 53mm 아래에 배치된다.
ci=2와 ci=3은 **순차 배치가 아닌 독립 위치 배치**이므로,
ci=3의 피트 기준점은 `st.current_height`(ci=2 높이 누적 후)가 아닌 **문단 시작 y**여야 한다.

올바른 피트 판단:
```
para_start_y + vert_offset + 표_높이 ≤ 페이지_하단
= para_y + 200 + 139 ≤ 1034  → 페이지 내 배치 가능 (통과)
```

## 수행 목표

1. ci=3 표(vert=Para + vert_offset > 0인 비-TAC 자리차지 표)의 페이지 피트 판단을 **문단 시작 y 기준**으로 수정
2. ci=3 표가 22페이지에 통째로 배치되도록 수정
3. ci=1 선이 페이지 경계를 넘지 않도록 수정 (ci=3 표 위치 수정의 연쇄 효과)
4. 회귀 0건 유지

## 수행 범위

- `src/renderer/pagination/engine.rs` — `paginate_table_control` 피트 판단 수정
- `src/renderer/layout.rs` (필요시) — 레이아웃 연동 확인
- 회귀 테스트: 전체 샘플 SVG 비교

## 제약 조건

- 기존 225개 파일 회귀 0건 유지
- TAC 표, 어울림 표, Page/Paper 기준 표 등 다른 케이스에 영향 없어야 함
- 수정 범위는 `vert=Para + vert_offset > 0 + wrap=TopAndBottom + tac=false` 조건으로 한정

## 수행 절차

1. 구현계획서 작성 → 승인
2. 단계별 구현 → 단계별 완료보고서 → 승인
3. 최종 완료보고서 → 승인
4. 커밋 + local/devel 머지
