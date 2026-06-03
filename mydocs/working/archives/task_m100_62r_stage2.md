# Task #62 재오픈 — 2단계 완료보고서

> **이슈**: [#62](https://github.com/edwardkim/rhwp/issues/62)
> **브랜치**: `local/task62-reopen`
> **작성일**: 2026-04-10

---

## 완료 내용

### 수정 파일
- `src/renderer/pagination/engine.rs`
- `src/renderer/svg.rs`

---

### engine.rs 변경 사항

#### 1. `place_table_fits` — current_height 업데이트 방식 수정

비-TAC 자리차지 표(wrap=TopAndBottom, vert=Para, vert_offset>0)는 문단 시작 y 기준으로 독립 배치되므로,
후속 문단의 배치 기준이 되는 `current_height`를 `float_bottom` 방식으로 업데이트한다.

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

**효과**: pi=129 전체 패턴이 올바르게 23페이지로 배치됨.

---

### svg.rs 변경 사항 — 디버그 오버레이 개선

#### 2. `s0:pi=0` 오류 레이블 제거

- **원인**: 구역 정의 섹션(섹션 0)의 문단 0이 body에 렌더링될 때 TextLine이 오버레이에 잡힘
- **수정**: 페이지 메인 섹션 자동 감지 (`overlay_page_section`) — 처음 등장하는 섹션을 메인으로 설정하고 다른 섹션 문단은 오버레이에서 제외

#### 3. 텍스트 없는 문단(pi=126)의 경계 박스 누락 수정

- **원인**: 오버레이 문단 경계는 TextLine에서만 수집 — [선][선][표][표] 구조처럼 텍스트 줄 없는 문단은 경계 박스 미표시
- **수정**: Table 노드에서도 같은 para_index의 문단 bounds를 확장

#### 4. TextBox/Group 노드 skip_depth 적용

- **원인**: TextBox, Group 내부의 TextLine이 skip_depth=0 상태에서 잡혀 오버레이에 포함됨
- **수정**: `RenderNodeType::TextBox | RenderNodeType::Group(_)`을 skip_depth 증가/감소 목록에 추가

#### 5. `begin_page`에서 오버레이 상태 초기화

- `overlay_para_bounds`, `overlay_table_bounds`, `overlay_skip_depth`, `overlay_page_section`을 페이지 시작 시 초기화

---

## 검증 결과

### 22페이지 디버그 오버레이
```
s2:pi=126 y=627.0  경계박스 x=142.7 y=627.0 w=409.7 h=332.4 (하단=959.5)
  → ci=2 표(y=627.0), ci=3 표(y=820.7) 모두 문단 경계 내 포함
s2:pi=127 y=998.2
s2:pi=128 y=1026.2
s0:pi=0 제거됨
```

### 23페이지 디버그 오버레이
```
s2:pi=129 y=103.6  문단 경계 정상
  → ci=2 표(y=103.6), ci=3 표(y=281.2) 모두 23페이지에 정상 배치
```

### 페이지네이션
```
22페이지: pi=126 (ci=2, ci=3 표 완전 배치) + pi=127, pi=128
23페이지: pi=129 (ci=0, ci=1 선, ci=2, ci=3 표 전체)
```

### 전체 샘플 회귀
- 226개 샘플 파일 내보내기 오류 0건

### WASM 빌드
- 성공 (1m 21s)

---

## 미완료 사항

- 없음 (구현계획서 2단계 항목 모두 완료)

