# Task 383: 자리차지+TAC 혼합 문단 pagination 높이 계산 수정 (v2)

## 문제

한 문단에 TAC 표와 자리차지(vert_offset) 표가 공존할 때:
1. **pagination**: 두 표의 높이를 개별 합산 + host_spacing 이중 적용 → current_height 과대
2. **layout**: TAC→자리차지 순서일 때 `deferred_table_bottom` 미적용 → y_offset 과대

### 재현 파일
| 파일 | 순서 | 문제 |
|------|------|------|
| bodo-01.hwp | 자리차지→TAC | layout: 순서 역전 (Task 382 해결), pagination: 정상 |
| bodo-02.hwp | TAC→자리차지 | pagination: pi=21 분할 후 pi=22가 3페이지로 넘김, layout: 간격 과대 |

### 근본 원인

**pagination**:
```
ci=0 (TAC): current_height += TAC높이 + host_spacing₁
ci=1 (자리차지): current_height += 자리차지높이 + host_spacing₂
합산: TAC높이 + 자리차지높이 + host_spacing₁ + host_spacing₂
```

**실제 차지 공간**:
```
max(TAC하단, vert_offset + 자리차지높이) + host_spacing (1회만)
```

합산 > 실제 → 후속 문단이 더 아래로 밀려 페이지 넘김 발생.

**layout** (bodo-02 전용):
- `has_following_tac` 조건이 "자리차지 뒤에 TAC가 있는 경우"만 처리
- TAC→자리차지 순서에서는 deferred가 활성화되지 않음

## 해결 방안: 후행 보정 방식

### 1단계: pagination 후행 보정

`process_controls` 호출 전후에 높이를 측정하여, 혼합 문단이면 통합 높이로 보정.

**위치**: `paginate_text_lines` 내부, `process_controls` 호출 후 (line 260 부근)

```rust
// process_controls 호출 전
let height_before = st.current_height;

self.process_controls(st, para_idx, para, measured, measurer, ...);

// 같은 문단에 TAC + 자리차지(vert_offset>0) 공존 시 후행 보정
if has_mixed_tac_and_positioned_table(para) {
    let added = st.current_height - height_before;
    let integrated = compute_integrated_table_height(para, measured, measurer);
    if integrated < added {
        st.current_height = height_before + integrated;
    }
}
```

**`compute_integrated_table_height`**:
```
tac_max_bottom = 0
positioned_max_bottom = 0

for each table control in para:
    effective_h = measured_table.total_height (또는 셀 높이 합산)
    if TAC:
        tac_max_bottom = max(tac_max_bottom, effective_h)
    else if 자리차지 && vert_offset > 0:
        positioned_max_bottom = max(positioned_max_bottom, vert_offset_px + effective_h)

integrated = max(tac_max_bottom, positioned_max_bottom) + host_spacing (1회)
```

### 2단계: layout deferred 확장 (TAC→자리차지 순서)

**위치**: `layout_table_item` 내부, `has_following_tac` 조건 확장

현재:
```rust
let has_following_tac = vert_offset > 0 && !tac && TopAndBottom
    && 뒤에 TAC가 있는가;
```

확장:
```rust
// 같은 문단에 자리차지(vert_offset>0) 표가 존재하고, 현재 표가 TAC인 경우
let has_positioned_sibling = tac && para.controls.iter()
    .any(|c| matches!(c, Control::Table(t)
        if !t.common.treat_as_char
        && t.common.vertical_offset > 0
        && matches!(t.common.text_wrap, TopAndBottom)));
```

TAC 처리 시 `has_positioned_sibling=true`이면 y_offset을 유지하고, 자리차지 표 처리 후 max(y_offset, 자리차지 하단)으로 갱신.

## 엣지 케이스 처리

| 케이스 | 처리 |
|--------|------|
| TAC 1개 + 자리차지 1개 (어느 순서든) | 통합 높이 = max(TAC하단, vert+자리차지높이) |
| TAC 2개 + 자리차지 1개 | tac_max_bottom = max(TAC₁, TAC₂), 통합 = max(tac_max, positioned) |
| 자리차지 2개 (TAC 없음) | positioned_max = max(vert₁+h₁, vert₂+h₂), tac_max=0, 통합=positioned_max |
| vert_offset=0 비-TAC + TAC | positioned 조건 불충족 → 보정 미적용 → 기존 동작 유지 |
| 단일 표 문단 | has_mixed 조건 불충족 → 보정 미적용 → 기존 동작 유지 |
| 자리차지 표 페이지 분할 | process_controls에서 split 처리 → 후행 보정은 전체 fits 경우에만 적용 |

## 구현 단계

| 단계 | 내용 | 파일 |
|------|------|------|
| 1 | `has_mixed_tac_and_positioned_table` 헬퍼 | pagination/engine.rs |
| 2 | `compute_integrated_table_height` 헬퍼 | pagination/engine.rs |
| 3 | process_controls 후행 보정 삽입 | pagination/engine.rs |
| 4 | layout `has_positioned_sibling` 조건 추가 | layout.rs |
| 5 | bodo-01, bodo-02, 기존 테스트 검증 | — |

## 테스트 계획
- bodo-01.hwp: 자리차지→TAC 순서 정상 유지
- bodo-02.hwp: TAC→자리차지 순서, 2페이지에 pi=22 배치
- kps-ai.hwp: 기존 TAC 표 간격 정상 유지
- 기존 755개 단위 테스트 통과
