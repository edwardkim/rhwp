# Task #62 재오픈 — 최종 완료보고서

> **이슈**: [#62](https://github.com/edwardkim/rhwp/issues/62)
> **브랜치**: `local/task62-reopen`
> **작성일**: 2026-04-10

---

## 수행 목표 달성 여부

| 목표 | 결과 |
|------|------|
| ci=3 표(vert=Para + vert_offset>0)의 피트 판단을 문단 시작 y 기준으로 수정 | ✓ 완료 |
| ci=3 표가 22페이지에 통째로 배치 | ✓ 완료 |
| ci=1 선이 페이지 경계를 넘지 않음 (ci=3 배치 수정의 연쇄 효과) | ✓ 완료 |
| pi=129 전체 패턴이 23페이지로 정상 배치 | ✓ 완료 |
| 디버그 오버레이에서 문단 경계 정상 표시 | ✓ 완료 |
| 회귀 0건 유지 | ✓ 완료 |

---

## 수정 파일 및 변경 내용

### `src/renderer/pagination/engine.rs`

#### 1. `para_start_height` 인자 추가
`process_controls`, `paginate_table_control`, `place_table_fits` 함수에 `para_start_height: f64` 인자 추가.
호출부에서 `height_before_controls`(문단 처리 시작 시점의 `st.current_height` 스냅샷)를 전달한다.

#### 2. `effective_table_height` 피트 판단 수정

조건: `!is_tac_table && wrap=TopAndBottom && vert_rel_to=Para && vertical_offset > 0`

```rust
// 수정 전: vert_offset을 current_height에 더해 이중 누적
effective_height + host_spacing + v_off

// 수정 후: 표 절대 하단 = 문단 시작 y + vert_offset + 표 높이
let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
(abs_bottom - st.current_height).max(effective_height + host_spacing)
```

#### 3. `place_table_fits` — current_height 업데이트 수정

비-TAC 자리차지 표는 독립 배치이므로 후속 문단 기준이 되는 `current_height`를 float_bottom 방식으로 업데이트.

```rust
let is_independent_float = !is_tac_table
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Para)
    && table.common.vertical_offset > 0;

if is_independent_float {
    let v_off = hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
    let float_bottom = para_start_height + v_off + effective_height;
    if float_bottom > st.current_height {
        st.current_height = float_bottom;
    }
} else {
    st.current_height += table_total_height;
}
```

---

### `src/renderer/svg.rs`

디버그 오버레이 4가지 개선:

1. **페이지 메인 섹션 자동 감지**: `overlay_page_section` 필드 추가 — 다른 섹션 문단 오버레이 제외 (`s0:pi=0` 제거)
2. **텍스트 없는 문단 경계 표시**: Table 노드에서도 문단 bounds 확장
3. **TextBox/Group skip_depth 적용**: Shape 내부 TextLine이 오버레이에 잡히는 문제 수정
4. **`begin_page` 오버레이 상태 초기화**: 페이지 간 상태 누적 방지

---

## 검증 결과

### pi=126 표 배치 (22페이지)
```
ci=2: 3x1 186.6x107.3px  y=627.0  22페이지 완전 배치
ci=3: 5x1 160.2x138.8px  y=820.7  22페이지 완전 배치 (하단=959.5 < 1034)
```

### pi=129 패턴 (23페이지)
```
ci=0, ci=1: Shape(선)  23페이지
ci=2: 5x1 160.2x138.8px  y=103.6  23페이지
ci=3: 7x2 292.3x138.3px  y=281.2  23페이지
```

### 디버그 오버레이
```
s2:pi=126 y=627.0  경계박스 y=627.0~959.5 (ci=2, ci=3 표 모두 포함)
s2:pi=129 y=103.6  경계박스 정상 (23페이지)
s0:pi=0 제거됨
```

### 전체 샘플 회귀
- 226개 샘플 파일 내보내기 오류 0건

### WASM 빌드
- 성공

---

## 단계별 수행 이력

| 단계 | 내용 | 결과 |
|------|------|------|
| 1단계 | para_start_height 도입 + effective_table_height 수정 | 완료 |
| 2단계 | current_height 누적 수정 + 디버그 오버레이 개선 | 완료 |
| 3단계 | 전체 샘플 회귀 테스트 | 완료 (오류 0건) |

