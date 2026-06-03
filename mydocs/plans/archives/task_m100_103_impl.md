# 구현 계획서 — Task #103

**이슈**: [#103](https://github.com/edwardkim/rhwp/issues/103)
**타이틀**: 비-TAC wrap=위아래 표의 out-of-flow 레이아웃 처리
**마일스톤**: M100
**작성일**: 2026-04-12
**브랜치**: `local/task103`

---

## 구현 목표

비-TAC `wrap=위아래(TopAndBottom)` 표를 한컴과 동일하게 **out-of-flow 2패스**로 배치한다.
re-flow 시 `para_start_y` 스냅샷 기반으로 FloatingTable을 독립 재배치한다.

---

## 현재 구조 요약

| 구성요소 | 파일 | 현재 동작 |
|---------|------|----------|
| `PageItem` enum | `src/renderer/pagination.rs:142` | Table, PartialTable, Shape, FullParagraph, PartialParagraph — FloatingTable 없음 |
| 비-TAC TopAndBottom 표 | `engine.rs:826-836` | `PageItem::Table`로 in-flow 처리 |
| Shape 2패스 | `layout.rs:2597` | `layout_column_shapes_pass()` — z-order 정렬 후 앵커 문단 y 기준 배치 |
| 앵커 추적 | `layout.rs:1244` | `para_start_y: HashMap<usize, f64>` — 문단별 y 위치 기록 |

---

## 단계별 구현 계획

### 1단계: `PageItem::FloatingTable` 변형 추가

**파일**: `src/renderer/pagination.rs`

`PageItem` enum에 새 변형 추가:

```rust
/// 비-TAC wrap=위아래 표 — out-of-flow 독립 배치
FloatingTable {
    para_index: usize,
    control_index: usize,
},
```

- `Table`/`PartialTable`과 동일한 필드 구조
- `Shape`과 마찬가지로 레이아웃 2패스에서 처리

---

### 2단계: 페이지네이션 엔진 — 비-TAC TopAndBottom 표 분류 변경

**파일**: `src/renderer/pagination/engine.rs`

**현재 코드** (`engine.rs:826-836`):
```rust
if !table.common.treat_as_char
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
{
    st.current_items.push(PageItem::Table {
        para_index: para_idx,
        control_index: ctrl_idx,
    });
    // ...
}
```

**변경 후**:
- `treat_as_char=false` + `text_wrap=TopAndBottom` + `vert_rel_to=Para` + `vertical_offset > 0` 조건 → `PageItem::FloatingTable`로 분류
- 기존 조건(Page/Paper 기준 고정 배치 등)은 `PageItem::Table` 유지
- `FloatingTable`은 in-flow 높이 기여 없음 — 앵커 문단의 흐름에 영향을 주지 않음

분류 조건 정리:

| 조건 | PageItem |
|------|----------|
| `treat_as_char=true` | `Table` (TAC, in-flow) |
| `treat_as_char=false` + `TopAndBottom` + `vert_rel_to=Para` + `vertical_offset > 0` | `FloatingTable` (out-of-flow) |
| 나머지 비-TAC | `Table` (기존 유지) |

---

### 3단계: 레이아웃 엔진 — `FloatingTable` 2패스 배치

**파일**: `src/renderer/layout.rs`

#### 3-1. `layout_column_content()` — FloatingTable 수집

`PageItem::FloatingTable`을 `PageItem::Shape`와 동일하게 1패스에서 **건너뜀** (y_offset 기여 없음).

#### 3-2. `layout_column_shapes_pass()` — FloatingTable 포함

기존 Shape 수집 루프에 `PageItem::FloatingTable` 추가:

```rust
PageItem::FloatingTable { para_index, control_index } => {
    let para_y = para_start_y.get(para_index).copied().unwrap_or(col_area.y);
    let v_off = /* table.common.vertical_offset → px 변환 */;
    // Shape와 동일하게 z_order 기준 정렬 후 배치
}
```

#### 3-3. FloatingTable 실제 배치

앵커 문단 y + `vertical_offset`(px) 위치에 표 레이아웃 실행:

```
float_y = para_start_y[para_index] + v_off
```

기존 `layout_table_item()` 호출 — 표 렌더링 로직 재사용.

---

### 4단계: re-flow — `para_start_y` 스냅샷 기반 독립 재배치

**설계 원칙**: `FloatingTable`의 배치 기준은 오직 `para_start_y` 스냅샷이다.

#### 동작 흐름

```
1패스 (in-flow 조판 완료)
  → para_start_y 스냅샷 확정 (모든 문단 y좌표 기록)

2패스 (FloatingTable 배치)
  → 각 FloatingTable은 para_start_y[앵커] + v_off 로 독립 배치
  → FloatingTable끼리 서로 참조하지 않음
```

#### re-flow 시

```
편집 발생 (FloatingTable 높이 변경 등)
  → 기존 페이지네이션 전파 실행 (in-flow 재계산)
  → para_start_y 스냅샷 갱신
  → 2패스 재실행 → 갱신된 스냅샷 기준으로 FloatingTable 전부 재배치
```

#### 이 설계의 장점

- **순환 의존 없음**: FloatingTable A가 B의 앵커에 영향을 주는 경로 차단
- **반복 수렴 불필요**: 1패스 스냅샷이 확정되면 2패스는 1회로 완결
- **엉킴 없음**: FloatingTable이 몇 개든 각자 스냅샷만 보고 독립 배치
- **기존 엔진 재사용**: PartialTable 분리, 다음 페이지 전파는 기존 로직 그대로

---

### 5단계: 검증

1. `hwpspec.hwp` 93페이지 pi=238 확인
   - `export-svg --debug-overlay` → 비-TAC 표가 앵커 문단 y + 33.7mm 위치에 배치되는지
   - `dump-pages -p 92` → FloatingTable이 in-flow 높이에 영향을 주지 않는지
2. 기존 TAC 표 / Shape 동작 회귀 없음 확인
3. `cargo test` 전체 통과 확인

---

## 파일 수정 범위

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/pagination.rs` | `PageItem::FloatingTable` 변형 추가 |
| `src/renderer/pagination/engine.rs` | 비-TAC TopAndBottom 표 → `FloatingTable` 분류 |
| `src/renderer/layout.rs` | `layout_column_content()` FloatingTable 스킵, `layout_column_shapes_pass()` FloatingTable 배치 추가 |

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계부터 구현을 시작하겠습니다.
