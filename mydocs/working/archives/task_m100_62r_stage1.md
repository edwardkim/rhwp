# Task #62 재오픈 — 1단계 완료보고서

> **이슈**: [#62](https://github.com/edwardkim/rhwp/issues/62)
> **브랜치**: `local/task62-reopen`
> **작성일**: 2026-04-10

---

## 완료 내용

### 수정 파일
- `src/renderer/pagination/engine.rs`

### 변경 사항

#### 1. `para_start_height` 인자 추가
`process_controls` 및 `paginate_table_control` 함수에 `para_start_height: f64` 인자 추가.
호출부에서 `height_before_controls`(문단 처리 시작 시점의 `st.current_height` 스냅샷)를 전달한다.

#### 2. `effective_table_height` 피트 판단 수정

**수정 전** (잘못된 로직):
```rust
// vert_offset을 current_height에 더해 이중 누적
effective_height + host_spacing + v_off
```

**수정 후** (올바른 로직):
```rust
// 표 절대 하단 = 문단 시작 y + vert_offset + 표 높이
// effective_table_height = abs_bottom - current_height (피트 판단식 변환)
let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
(abs_bottom - st.current_height).max(effective_height + host_spacing)
```

조건: `!is_tac_table && wrap=TopAndBottom && vert_rel_to=Para && vertical_offset > 0`

---

## 검증 결과

### pi=126 표 분할 수정 확인
```
수정 전: PartialTable pi=126 ci=3 rows=0..3 (22페이지) + rows=2..5 (23페이지)
수정 후: Table      pi=126 ci=3 5x1 160.2x138.8px (22페이지)
```

### 22페이지 배치 확인 (SVG 디버그 레이블)
```
s2:pi=126 ci=2 3x1 y=627.0  ← 정상
s2:pi=126 ci=3 5x1 y=820.7  ← 22페이지 완전 배치 (하단=959.5 < 페이지하단=1034)
```

### ci=1 선 범위 확인
```
ci=0: (331.64, 639.25) → (392.27, 821.28)  ← 22페이지 내
ci=1: (331.64, 658.15) → (392.27, 960.51)  ← 22페이지 내 (960 < 1034)
```

### 전체 샘플 회귀
- 226개 샘플 파일 내보내기 오류 0건

---

## 미완료 사항 (2단계로 이월)

- 비-TAC 자리차지 표의 `current_height` 누적 적절성 검토
- 점선(stroke-dasharray) 문제: ci=0, ci=1 선이 여전히 점선으로 렌더링됨 (별도 원인 분석 필요)
