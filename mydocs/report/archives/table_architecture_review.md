# 표 객체 처리 아키텍처 현황 분석 보고서

## 1. 개요

현재 프로젝트의 표 객체 처리 아키텍처를 **백엔드 추상화 트리 관리**, **이벤트 핸들러**, **사용자 UI** 세 계층으로 분석한다.

---

## 2. 백엔드 추상화 트리 관리 (Rust)

### 2.1 데이터 모델

**`src/model/table.rs:8-40`** — `Table` 구조체:

```rust
pub struct Table {
    pub row_count: u16,
    pub col_count: u16,
    pub cells: Vec<Cell>,       // 플랫 배열 (행 우선 순서)
    pub row_sizes: Vec<HwpUnit16>,
    pub border_fill_id: u16,
    pub page_break: TablePageBreak,
    pub repeat_header: bool,
    pub caption: Option<Caption>,
    // ...라운드트립 보존 필드
}
```

- `cells`는 **플랫 `Vec<Cell>`** — 2D 인덱스 없이 행 우선 순서로 저장
- `row_count`, `col_count`는 차원 정보만 보유하며, 셀 접근에 활용되지 않음
- 행/열 수와 실제 셀 배치 사이의 정합성은 런타임에 보장되지 않음

**`src/model/table.rs:67-96`** — `Cell` 구조체:

```rust
pub struct Cell {
    pub col: u16,
    pub row: u16,
    pub col_span: u16,
    pub row_span: u16,
    pub width: HwpUnit,
    pub height: HwpUnit,
    pub padding: Padding,
    pub border_fill_id: u16,
    pub paragraphs: Vec<Paragraph>,
    pub text_direction: u8,
    pub vertical_align: VerticalAlign,
    pub is_header: bool,
    // ...라운드트립 보존 필드
}
```

- 각 셀이 절대 좌표 `(row, col, row_span, col_span)`를 개별 보유
- 셀 내 콘텐츠는 `paragraphs: Vec<Paragraph>`로 리치 텍스트 지원
- 병합 셀은 `row_span > 1` 또는 `col_span > 1`로 표현

### 2.2 셀 탐색 방식

**`src/wasm_api.rs:7014-7018`** — `find_cell_at_row_col()`:

```rust
fn find_cell_at_row_col(table: &Table, target_row: u16, target_col: u16) -> Option<usize> {
    table.cells.iter().position(|cell| {
        cell.row <= target_row && target_row < cell.row + cell.row_span
            && cell.col <= target_col && target_col < cell.col + cell.col_span
    })
}
```

- **모든 셀 접근마다 O(n) 선형 스캔** 발생
- 커서 이동, 셀 편집, 표 구조 변경 등 모든 조작에서 반복 호출
- 병합 셀 탐색 시 span 범위를 매번 계산

### 2.3 표 접근 경로 (트리 탐색)

**`src/wasm_api.rs:2005-2034`** — `get_table_mut()`:

```
Document → sections[sec] → paragraphs[ppi] → controls[ci] → Control::Table(table)
```

- 3단계 인덱스 역참조로 표에 접근
- 표 수정 시마다 이 경로를 반복 탐색 (캐싱 없음)
- `Control` 열거형에서 `Table` 패턴 매칭 필요

### 2.4 수정 후 무효화 패턴

표 수정이 발생하면 **전체 섹션 재직렬화 + 재레이아웃**이 트리거된다:

```rust
// 8곳에서 동일 패턴 반복 (wasm_api.rs)
self.document.sections[section_idx].raw_stream = None;       // 직렬화 캐시 무효화
self.composed[section_idx] = compose_section(...);           // 전체 섹션 재레이아웃
```

발생 위치:
- 텍스트 삽입/삭제 (`insert_text`, `delete_text`)
- 문단 분할/병합 (`split_paragraph`, `merge_paragraph`)
- 행/열 추가/삭제 (`insert_row`, `insert_column`, `delete_row`, `delete_column`)
- 셀 병합/나누기 (`merge_cells`, `split_cell`)
- 서식 적용 (`apply_char_format`)

### 2.5 행/열 구조 변경 시 비용

**`src/model/table.rs`** — `rebuild_row_sizes()`:

- 행 추가/삭제 후 `row_sizes` 벡터를 전체 재구성
- 모든 셀을 순회하며 행별 최대 높이 계산 → O(n×m) 비용
- `row_sizes`는 HWP 직렬화에 필요한 행별 높이 배열

---

## 3. 이벤트 핸들러 (TypeScript)

### 3.1 InputHandler 구조

**`rhwp-studio/src/engine/input-handler.ts:18`** — `InputHandler` 클래스가 **모든 표 관련 이벤트를 단일 클래스에서 처리**:

```typescript
export class InputHandler {
    private cursor: CursorState;
    private caret: CaretRenderer;
    private selectionRenderer: SelectionRenderer;
    private cellSelectionRenderer: CellSelectionRenderer | null = null;
    private tableObjectRenderer: TableObjectRenderer | null = null;
    // ...
}
```

표 전용 이벤트 핸들링이 본문 편집 핸들링과 동일 클래스에 혼재.

### 3.2 상태 머신 (3가지 모드)

| 모드 | 진입 조건 | 주요 동작 | 탈출 조건 |
|------|----------|----------|----------|
| **셀 편집** | 표 내부 클릭, Tab 이동 | 일반 텍스트 편집과 동일 | Esc → 표 객체 선택 |
| **F5 셀 선택** | F5 키 (셀 내부에서) | 화살표 이동, Shift+클릭 범위선택, Ctrl+클릭 토글 | Enter 또는 Esc |
| **표 객체 선택** | Esc (셀 편집에서), 표 테두리 클릭 | Delete 삭제, 파란 테두리+8핸들 표시 | Esc → 표 밖 본문, Enter → 셀 편집 |

### 3.3 키 처리 흐름

`onKeyDown()` 내부 분기:

```
onKeyDown(e)
  ├── F5 셀 선택 모드?
  │     ├── 화살표 → 셀 앵커/포커스 변경
  │     ├── Enter → 모드 종료
  │     └── Esc → 모드 종료
  ├── 표 객체 선택?
  │     ├── Delete → table:delete 커맨드
  │     └── Esc → 표 밖 커서 탈출
  ├── 셀 내부?
  │     ├── Tab/Shift+Tab → 셀 이동 (WASM navigate_cell_horizontal 호출)
  │     ├── 화살표 상/하 → 셀 간 수직 이동 (WASM navigate_cell_vertical 호출)
  │     └── Esc → 표 객체 선택 모드 진입
  └── 본문 → 일반 텍스트 편집
```

### 3.4 컨텍스트 메뉴 분기

| 컨텍스트 | 메서드 | 메뉴 항목 |
|---------|--------|----------|
| 셀 내부 | `getTableContextMenuItems()` | 잘라내기/복사/붙여넣기, 셀/행/열/표 속성, 병합/나누기, 행/열 추가/삭제 |
| 표 객체 선택 | `getTableObjectContextMenuItems()` | 잘라내기/복사/붙여넣기, 표 속성, 표 삭제 |
| 본문 | `getDefaultContextMenuItems()` | 잘라내기/복사/붙여넣기 |

### 3.5 셀 간 이동 로직

**`rhwp-studio/src/engine/cursor.ts:150-158`**:

```typescript
moveHorizontal(delta: number): void {
    if (this.isInTextBox()) this.moveHorizontalInTextBox(delta);
    else if (this.isInCell()) this.moveHorizontalInCell(delta);
    else this.moveHorizontalInBody(delta);
}
```

- 수평 이동: `cellIndex` 기반 (플랫 배열 인덱스 ±1)
- 수직 이동: WASM `navigate_cell_vertical()` 호출 → `find_cell_at_row_col()` O(n) 탐색
- Tab 이동: 다음/이전 셀로 이동, 마지막 셀에서 Tab 시 행 추가

---

## 4. 사용자 UI (TypeScript)

### 4.1 DocumentPosition (커서 위치 표현)

**`rhwp-studio/src/core/types.ts:55-66`**:

```typescript
interface HitTestResult {
    sectionIndex: number;       // 섹션 인덱스
    paragraphIndex: number;     // 본문 문단 인덱스 or 셀 내 문단 인덱스
    charOffset: number;         // 문자 오프셋
    parentParaIndex?: number;   // 표가 속한 본문 문단 (셀 내부일 때만)
    controlIndex?: number;      // 표의 컨트롤 인덱스 (셀 내부일 때만)
    cellIndex?: number;         // 셀 인덱스 — 플랫 배열 기반 (셀 내부일 때만)
    cellParaIndex?: number;     // 셀 내 문단 인덱스 (셀 내부일 때만)
    isTextBox?: boolean;        // 글상자 내부 여부
}
```

- **7개 필드** — 본문과 셀 컨텍스트를 동시에 표현
- 셀 내부에서는 `parentParaIndex + controlIndex + cellIndex + cellParaIndex` 4개 필드가 모두 필요
- 위치 비교 시 본문/셀 혼합 케이스 처리 복잡 (`comparePositions()` 100줄)

### 4.2 이중 좌표계 문제

| 용도 | 좌표계 | 사용 위치 |
|------|--------|----------|
| 셀 순차 이동 (Tab, 수평) | `cellIndex` (플랫 배열 인덱스) | cursor.ts `moveHorizontalInCell()` |
| 셀 범위 선택 (F5) | `(row, col)` 2D 좌표 | cursor.ts `cellAnchor`, `cellFocus` |
| 셀 데이터 접근 | `(row, col)` → `find_cell_at_row_col()` → `cellIndex` | wasm_api.rs |
| 렌더링 | `cellIndex` → `getCellBbox()` | cell-selection-renderer.ts |

두 좌표계 변환이 필요한 모든 곳에서 WASM 호출 발생.

### 4.3 시각적 피드백 렌더러 (3종)

| 렌더러 | 파일 | 용도 |
|--------|------|------|
| `CaretRenderer` | `caret-renderer.ts` | 셀 내부 깜빡이는 커서 |
| `CellSelectionRenderer` | `cell-selection-renderer.ts` | F5 셀 범위 선택 (파란 오버레이) |
| `TableObjectRenderer` | `table-object-renderer.ts` | 표 객체 선택 (검은 테두리 + 8 리사이즈 핸들) |

각 렌더러가 독립적으로 WASM에서 bbox 정보를 가져와 DOM 요소를 생성/갱신.

---

## 5. 핵심 Pain Points

### 5.1 성능

| 문제 | 위치 | 복잡도 | 설명 |
|------|------|--------|------|
| 플랫 `Vec<Cell>` 선형 탐색 | `find_cell_at_row_col()` | O(n) | 모든 셀 접근마다 전체 셀 배열 순회 |
| `rebuild_row_sizes()` | `table.rs` | O(n×m) | 행/열 변경마다 전체 행 높이 재계산 |
| 전체 섹션 리플로우 | `raw_stream=None` + `compose_section()` | O(섹션) | 표 한 글자 편집에도 섹션 전체 재레이아웃 |

### 5.2 복잡도

| 문제 | 설명 |
|------|------|
| 7필드 DocumentPosition | 본문/셀/글상자 컨텍스트를 하나의 타입에 혼합. Optional 필드 4개로 상태 표현이 모호 |
| 이중 좌표계 (`cellIndex` ↔ `(row,col)`) | 플랫 인덱스와 2D 좌표 간 변환이 필요한 모든 곳에서 WASM 호출 + O(n) 탐색 |
| 이벤트 핸들러 단일 클래스 | `InputHandler`가 본문/셀/F5/표객체 4가지 모드를 모두 처리하여 분기 복잡 |
| 표 접근 경로 비캐싱 | `get_table_mut()` 3단계 역참조를 수정마다 반복 |

### 5.3 확장성

| 문제 | 설명 |
|------|------|
| 셀 주소 기반 수식/참조 불가 | 플랫 배열에는 엑셀식 "A1" 주소 체계가 없음 |
| 부분 레이아웃 갱신 불가 | 표 수정 시 항상 전체 섹션 레이아웃 재계산 |
| 병합 셀 탐색 비효율 | span 범위를 매번 모든 셀에 대해 계산 |

---

## 6. 리플로우 파이프라인 분석: 표 페이지 경계 분할

### 6.1 전체 파이프라인 구조

표 수정 시 트리거되는 리플로우는 **4단계 파이프라인**으로 구성된다:

```
[트리거] wasm_api.rs — 텍스트 편집 / 표 구조 변경
    │
    │  raw_stream = None          // 직렬화 캐시 무효화
    │  composed[sec] = compose_section(...)  // 1단계
    │  self.paginate()                       // 2~3단계
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ 1단계: 구성 (Composition)                                    │
│ composer.rs:compose_section() — 라인 78                      │
│ 문단 텍스트 → LineSeg 기반 줄 분할 → CharShapeRef 경계 TextRun │
│ 출력: Vec<ComposedParagraph>                                 │
└────────────────────────┬────────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 2단계: 높이 측정 (Measurement)                               │
│ height_measurer.rs:measure_section() — 라인 107              │
│ 모든 문단 + 표의 실제 렌더링 높이를 사전 계산                   │
│ 표: measure_table() → 행별 높이, 셀별 줄 높이 측정              │
│ 출력: MeasuredSection { paragraphs, tables }                 │
└────────────────────────┬────────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 3단계: 페이지 분할 (Pagination)                              │
│ pagination.rs:paginate_with_measured() — 라인 184            │
│ 측정된 높이로 페이지 경계 결정, 표 행/인트라-로우 분할           │
│ 출력: PaginationResult { pages: Vec<PageContent> }           │
└────────────────────────┬────────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 4단계: 렌더링 (Layout)                                       │
│ layout.rs:build_render_tree() → layout_table() — 라인 1125   │
│ 페이지별 렌더 트리 구축, 열 폭/행 높이 **재계산**               │
│ 출력: PageRenderTree (SVG/Canvas 노드)                       │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 표 페이지 분할 상세 로직

**`pagination.rs:632-907`** — 표가 현재 페이지 잔여 높이를 초과할 때 실행:

#### 6.2.1 행 단위 분할 루프

```rust
// pagination.rs:684-907
let mut cursor_row: usize = 0;        // 현재 처리 중인 시작 행
let mut is_continuation = false;       // 연속 페이지 여부
let mut content_offset: f64 = 0.0;     // 인트라-로우 콘텐츠 오프셋

while cursor_row < row_count {
    // 1. 이 페이지에 들어갈 행 범위 결정 (cursor_row..end_row)
    // 2. 마지막 행이 부분적으로만 들어가면 인트라-로우 분할 시도
    // 3. PageItem::PartialTable 생성 → 다음 페이지로 진행
}
```

#### 6.2.2 인트라-로우 분할 의사결정

**`pagination.rs:737-776`** — 행이 페이지에 안 들어갈 때:

```
행이 페이지 잔여에 안 들어감
  ├── is_row_splittable(r) == true?
  │     ├── 양쪽 모두 ≥ 10px 콘텐츠?
  │     │     └── YES → 인트라-로우 분할 (split_end_limit 설정)
  │     └── NO → 폴백 (행 전체를 다음 페이지로)
  └── is_row_splittable(r) == false?
        └── 행 전체 강제 배치 (오버플로 + clipPath 숨김)
```

#### 6.2.3 `is_row_splittable()` 판정

**`height_measurer.rs:536-544`**:

```rust
pub fn is_row_splittable(&self, row: usize) -> bool {
    let cells_in_row: Vec<&MeasuredCell> = self.cells.iter()
        .filter(|c| c.row == row && c.row_span == 1)
        .collect();
    if cells_in_row.is_empty() { return false; }
    // 행 내 최소 1개 셀이 2줄 이상이면 분할 가능
    cells_in_row.iter().any(|c| c.line_heights.len() > 1)
}
```

- 멀티라인 셀(텍스트 줄바꿈) → 분할 가능
- 단일라인 셀(이미지, 짧은 텍스트) → 분할 불가
- **행 전체가 이미지 셀이면 분할 금지** (타스크 77에서 추가)

#### 6.2.4 `PageItem::PartialTable` 데이터 구조

**`pagination.rs:118-133`**:

```rust
PartialTable {
    para_index: usize,                  // 표가 속한 문단
    control_index: usize,               // 컨트롤 인덱스
    start_row: usize,                   // 시작 행 (inclusive)
    end_row: usize,                     // 끝 행 (exclusive)
    is_continuation: bool,              // 연속 페이지 여부 (제목행 반복용)
    split_start_content_offset: f64,    // 시작행 콘텐츠 오프셋 (px)
    split_end_content_limit: f64,       // 끝행 최대 콘텐츠 높이 (px, 0=전부)
}
```

#### 6.2.5 제목행 반복 처리

**`pagination.rs:706-710`**:

```rust
let header_overhead = if is_continuation
    && mt.repeat_header && mt.has_header_cells && row_count > 1
{
    header_row_height + cell_spacing  // 제목행 높이를 사용 가능 높이에서 차감
} else { 0.0 };
```

연속 페이지에서 행0(제목행)을 자동 반복 렌더링.

### 6.3 높이 측정 이중 계산 문제

현재 행 높이가 **2단계(측정)와 4단계(렌더링)에서 동일 로직으로 중복 계산**된다:

| 계산 위치 | 파일 | 로직 |
|----------|------|------|
| **측정 단계** | `height_measurer.rs:214-380` `measure_table()` | 모든 셀 순회 → `compose_paragraph()` → 줄 높이 합산 |
| **렌더링 단계** | `layout.rs:1230-1400` `layout_table()` | 모든 셀 순회 → `compose_paragraph()` → 줄 높이 합산 |

두 함수 모두:
1. `row_span==1` 셀에서 행별 최대 높이 추출
2. 셀 내 문단마다 `compose_paragraph()` 호출 → 줄 높이 합산
3. 병합 셀 제약조건 해결 (가우스 소거법)
4. 병합 셀 콘텐츠 초과 시 마지막 행 확장

**`compose_paragraph()`가 셀당 2번씩 호출**되므로, n개 셀 표에서 총 2n번 호출.

### 6.4 전체 섹션 리플로우의 영향

현재 **표 셀 한 글자 편집**에도 다음이 모두 실행된다:

```
1. compose_section()        — 섹션의 모든 문단 재구성
2. measure_section()        — 섹션의 모든 문단+표 높이 재측정
3. paginate_with_measured() — 전체 페이지 분할 재수행
4. build_render_tree()      — 요청 페이지 렌더 트리 재구축
```

**비용 분석** (섹션에 문단 P개, 표 T개, 표당 셀 C개 가정):

| 단계 | 비용 | 설명 |
|------|------|------|
| compose_section | O(P) | 모든 문단 재구성 |
| measure_section | O(P + T×C) | 모든 문단 측정 + 표마다 셀 내 문단 compose |
| paginate | O(P + T×R) | 모든 문단 순회 + 표마다 행 순회 |
| layout_table | O(T×C) | 표마다 셀 내 문단 다시 compose |
| **총합** | O(P + T×C) | 표 하나 편집에 섹션 전체 비용 |

---

## 7. 핵심 Pain Points

### 7.1 성능

| 문제 | 위치 | 복잡도 | 설명 |
|------|------|--------|------|
| 플랫 `Vec<Cell>` 선형 탐색 | `find_cell_at_row_col()` | O(n) | 모든 셀 접근마다 전체 셀 배열 순회 |
| `rebuild_row_sizes()` | `table.rs` | O(n×m) | 행/열 변경마다 전체 행 높이 재계산 |
| 전체 섹션 리플로우 | `raw_stream=None` + `compose_section()` | O(섹션) | 표 한 글자 편집에도 섹션 전체 재레이아웃 |
| 행 높이 이중 계산 | `measure_table()` + `layout_table()` | O(2×셀수) | 동일 로직이 측정+렌더에서 반복 |
| `compose_paragraph()` 이중 호출 | 측정 + 렌더 | O(2×셀수×문단) | 셀 내 문단 구성이 2번씩 실행 |

### 7.2 복잡도

| 문제 | 설명 |
|------|------|
| 7필드 DocumentPosition | 본문/셀/글상자 컨텍스트를 하나의 타입에 혼합. Optional 필드 4개로 상태 표현이 모호 |
| 이중 좌표계 (`cellIndex` ↔ `(row,col)`) | 플랫 인덱스와 2D 좌표 간 변환이 필요한 모든 곳에서 WASM 호출 + O(n) 탐색 |
| 이벤트 핸들러 단일 클래스 | `InputHandler`가 본문/셀/F5/표객체 4가지 모드를 모두 처리하여 분기 복잡 |
| 표 접근 경로 비캐싱 | `get_table_mut()` 3단계 역참조를 수정마다 반복 |

### 7.3 확장성

| 문제 | 설명 |
|------|------|
| 셀 주소 기반 수식/참조 불가 | 플랫 배열에는 엑셀식 "A1" 주소 체계가 없음 |
| 부분 레이아웃 갱신 불가 | 표 수정 시 항상 전체 섹션 레이아웃 재계산 |
| 병합 셀 탐색 비효율 | span 범위를 매번 모든 셀에 대해 계산 |

---

## 8. 효율적 리플로우 설계 방향

### 8.1 설계 원칙

표 페이지 분할의 근본 비효율은 **"표 내부 변경 → 섹션 전체 리플로우"** 패턴에 있다. 이를 해소하기 위해 **표 내부 리플로우를 섹션 리플로우에서 분리**하는 설계가 필요하다.

### 8.2 목표 아키텍처: 3단계 분리 리플로우

```
[현재] 표 셀 편집 → 섹션 전체 Compose → 섹션 전체 Measure → 섹션 전체 Paginate

[목표] 표 셀 편집 → 해당 셀만 Compose → 해당 행만 Measure → 표 Paginate만 재수행
                                                             ↓
                                                    표 높이 변경?
                                                     ├── NO → 렌더만 갱신
                                                     └── YES → 섹션 Paginate 재수행
```

#### 레벨 1: 셀 내부 리플로우 (표 높이 불변)

셀 내 텍스트 편집으로 **표 전체 높이가 변하지 않는 경우** (줄 수 변화 없음):
- `compose_paragraph()`를 해당 셀의 수정된 문단에만 적용
- 행 높이/페이지 분할 결과 재사용
- 렌더 트리에서 해당 셀 노드만 갱신

#### 레벨 2: 행 리플로우 (표 높이 변경, 페이지 분할 불변)

셀 내 편집으로 **행 높이가 변했지만 페이지 경계가 이동하지 않는 경우**:
- 변경된 셀의 `compose_paragraph()` 실행
- 해당 행의 높이만 재측정 (`measure_row()`)
- 표 전체 높이 재계산 (행 높이 합산만, O(R))
- 이전 `PartialTable` 분할 결과가 여전히 유효한지 빠르게 검증
- 유효하면 렌더 트리만 갱신

#### 레벨 3: 표 페이지 재분할 (페이지 경계 이동)

행 높이 변경으로 **페이지 경계가 이동하는 경우**:
- 표의 `MeasuredTable`만 재생성 (O(C), 셀 수)
- 표가 속한 위치부터 섹션 끝까지만 재페이지네이션
- 이전 페이지들의 결과는 재사용

### 8.3 이를 위한 데이터 구조 변경

#### 8.3.1 MeasuredTable 캐싱

```rust
// 현재: paginate() 호출마다 MeasuredSection을 임시 생성 후 폐기
// 목표: MeasuredTable을 표 모델에 캐싱

pub struct Table {
    // ...기존 필드...
    /// 캐시된 측정 결과 (표 수정 시 무효화)
    measured: Option<MeasuredTable>,
}
```

- 셀 편집 시: 해당 행의 `MeasuredCell`만 갱신 → `measured.row_heights[row]` 재계산
- 구조 변경 시: `measured = None` → 전체 재측정

#### 8.3.2 layout_table()의 이중 계산 제거

```rust
// 현재: layout_table()이 행 높이를 처음부터 다시 계산
// 목표: MeasuredTable의 캐시된 행 높이를 전달받아 사용

fn layout_table(
    &self,
    measured: &MeasuredTable,  // ← 사전 측정 결과 전달
    start_row: usize,
    end_row: usize,
    // ...
) -> f64
```

### 8.4 2D 그리드 인덱스와의 시너지

2D 그리드 추상화(`grid[row][col]`)가 도입되면:

| 현재 비용 | 개선 비용 | 설명 |
|----------|----------|------|
| 행 높이 측정 O(n) 셀 전체 순회 | O(col_count) 해당 행만 | 그리드에서 행 셀 직접 접근 |
| 병합 셀 제약조건 해결 O(n²) | O(병합셀 수) | 병합 셀 레지스트리에서 직접 조회 |
| 인트라-로우 분할 가능성 판정 O(n) | O(col_count) | 해당 행 셀만 검사 |
| 제목행 반복 재렌더링 | O(col_count) | 행0 셀 직접 접근 |

### 8.5 점진적 무효화 전략

```
셀 텍스트 편집
  ├── 해당 셀의 composed 캐시 무효화
  ├── 해당 행의 measured_row 캐시 무효화
  ├── 표 total_height 재계산 (O(R) — 행 높이 합산)
  ├── total_height 변경?
  │     ├── NO → 렌더 캐시만 무효화 (레벨 1)
  │     └── YES → 표의 PartialTable 분할 재계산
  │               ├── 페이지 경계 변경?
  │               │     ├── NO → 렌더 캐시만 무효화 (레벨 2)
  │               │     └── YES → 해당 표 이후 섹션 재페이지네이션 (레벨 3)
  └── raw_stream = None (직렬화 캐시 무효화는 유지)

행/열 추가/삭제
  └── 표 전체 재측정 + 레벨 3 재페이지네이션
```

### 8.6 기대 효과

| 시나리오 | 현재 비용 | 개선 후 비용 | 개선율 |
|---------|----------|------------|--------|
| 셀 내 글자 1개 입력 (줄 수 불변) | O(P + T×C) | O(1) compose + O(1) render | 극대 |
| 셀 내 줄바꿈 발생 (행 높이 변경) | O(P + T×C) | O(C_row) measure + O(R) paginate check | 대 |
| 행 높이 변경으로 페이지 경계 이동 | O(P + T×C) | O(C) measure + O(P_after) repaginate | 중 |
| 행/열 추가/삭제 | O(P + T×C) | O(C) measure + O(P) repaginate | 소 |

---

## 9. 중첩 표 (표 안의 표) 처리 현황

### 9.1 HWP의 중첩 표 구조

HWP는 셀 안에 또 다른 표를 임의 깊이로 중첩할 수 있다. 데이터 모델에서의 재귀 구조:

```
Table
  └─ cells: Vec<Cell>
       └─ Cell.paragraphs: Vec<Paragraph>
            └─ Paragraph.controls: Vec<Control>
                 └─ Control::Table(Box<Table>)    ← 재귀
                      └─ cells: Vec<Cell>
                           └─ Cell.paragraphs
                                └─ ... (무한 중첩 가능)
```

- `Box<Table>`로 재귀 타입 문제 해결 (`src/model/control.rs:20`)
- 깊이 제한 없음 — Rust 스택 크기에만 의존
- 실제 HWP 문서는 대부분 2~3단계 중첩

### 9.2 계층별 중첩 표 지원 현황

| 계층 | 지원 수준 | 파일 | 핵심 한계 |
|------|----------|------|----------|
| **파싱** | **완전** | `parser/control.rs:54-149` | 재귀 파싱으로 임의 깊이 지원 |
| **직렬화** | **완전** | `serializer/control.rs:305-332` | 재귀 직렬화, level 관리 |
| **렌더링** | **부분** | `layout.rs:1125, 2845` | `layout_table()` ≠ `layout_nested_table()` 이원화 |
| **높이 측정** | **미지원** | `layout.rs:2519-2523` | `calc_cell_controls_height()` → **항상 0 반환** |
| **페이지 분할** | **미지원** | `pagination.rs` | 중첩 표 페이지 분할 로직 없음 |
| **WASM API** | **미지원** | `wasm_api.rs:2005-2034` | 3단계 인덱싱만 가능, 중첩 접근 불가 |

### 9.3 렌더링 이원화 문제

현재 표 렌더링 함수가 **두 개로 분리**되어 있다:

**`layout_table()`** — 최상위 표 전용 (`layout.rs:1125-1755`, 630줄):
- 페이지 분할(`PartialTable`) 지원
- 셀 내 컨트롤 → `Control::Table` 발견 시 `layout_nested_table()` 호출

**`layout_nested_table()`** — 중첩 표 전용 (`layout.rs:2845-3096`, 250줄):
- 페이지 분할 미지원 (부모 셀 경계 내에서만 렌더)
- 자기 자신을 재귀 호출하여 심층 중첩 처리
- `layout_table()`과 열 폭/행 높이 계산 로직이 **중복**

```
layout_table() [최상위 표, 630줄]
  └─ Control::Table(nested) → layout_nested_table() [중첩 표, 250줄]
                                  └─ Control::Table(sub) → layout_nested_table() [재귀]
```

**문제**: 동일한 "표 레이아웃"이라는 관심사가 두 함수에 분산되어, 수정 시 양쪽을 동시에 변경해야 함.

### 9.4 높이 측정의 구조적 결함

**`layout.rs:2519-2523`**:
```rust
/// 셀 내 컨트롤(중첩테이블)의 총 높이를 계산한다.
/// Picture/Table 모두 셀 높이(cell.height)에 이미 반영되어 있으므로 0 반환.
fn calc_cell_controls_height(&self, _cell: &Cell) -> f64 {
    0.0
}
```

**`layout.rs:2484-2517`** — `calc_nested_table_height()`:
- `cell.height` HWP 메타데이터만 사용
- 셀 내 문단의 실제 렌더링 높이를 측정하지 않음
- 중첩 표 내부의 중첩 표 높이는 재귀 측정 안 함

**결론**: 뷰어 모드에서는 HWP 파일의 사전 계산된 높이를 신뢰하면 되지만, **편집기 모드에서 중첩 표 내용이 변경되면 부모 셀/부모 표의 높이가 갱신되지 않는다**.

### 9.5 WASM API 접근 불가

현재 WASM API의 표 접근 경로:

```
get_table_mut(section_idx, parent_para_idx, control_idx)
  → sections[sec].paragraphs[ppi].controls[ci]
  → Control::Table(table)
```

**3단계 인덱싱만 지원** — 중첩 표에 접근하려면:

```
필요한 경로:
  sections[sec]
    .paragraphs[ppi]                   // 1. 본문 문단
      .controls[ci]                    // 2. 최상위 표
        → Table.cells[cell_idx]        // 3. 셀
          .paragraphs[cell_para_idx]   // 4. 셀 내 문단
            .controls[nested_ci]       // 5. 중첩 표
              → Table.cells[...]       // 6. 중첩 셀
                .paragraphs[...]       // 7. ...
```

**현재 API로는 5단계 이상 접근이 불가능** — 중첩 표 편집, 행/열 추가, 셀 병합 등 모든 조작이 차단됨.

### 9.6 뷰어 → 편집기 전환의 한계점

| 기능 | 뷰어 모드 | 편집기 모드 (현재 한계) |
|------|----------|---------------------|
| 중첩 표 렌더링 | HWP 메타데이터 기반 → 동작 | 내용 변경 시 높이 갱신 불가 |
| 중첩 표 페이지 분할 | 부모 셀 경계 내 클리핑 | 중첩 표 독립 분할 불가 |
| 중첩 표 편집 | N/A | WASM API 접근 불가 |
| 중첩 표 구조 변경 | N/A | 행/열/셀 조작 API 없음 |
| 높이 전파 | N/A | 중첩 표 변경 → 부모 셀 → 부모 표 역전파 없음 |

---

## 10. 구조적 재설계 방향

### 10.1 핵심 인식

현재 아키텍처는 **뷰어 패러다임**(읽기 전용, HWP 메타데이터 신뢰)에서 출발했다. 편집기로 진화하면서 다음 구조적 한계에 도달한다:

1. **높이 계산의 역전**: 뷰어는 "HWP가 알려준 높이를 사용"하지만, 편집기는 "콘텐츠로부터 높이를 계산"해야 함
2. **단방향 → 양방향 전파**: 뷰어는 파싱→레이아웃→렌더 단방향이지만, 편집기는 내용변경→높이변경→페이지재분할→렌더 양방향 전파가 필요
3. **플랫 접근 → 트리 접근**: 뷰어는 최상위 표만 접근하면 되지만, 편집기는 임의 깊이의 중첩 표에 접근/수정해야 함

### 10.2 재설계 원칙

#### 원칙 1: 통합 표 레이아웃 엔진

```
[현재] layout_table() (630줄) + layout_nested_table() (250줄) = 이원화

[목표] layout_table_recursive() = 단일 함수
  - 깊이(depth) 파라미터로 최상위/중첩 구분
  - 페이지 분할은 depth==0일 때만 적용
  - 열 폭/행 높이/셀 렌더링 로직 완전 통합
```

#### 원칙 2: 재귀적 높이 측정

```
[현재] calc_cell_controls_height() → 0 (HWP 메타데이터 신뢰)

[목표] measure_cell_recursive() = 재귀 측정
  - 셀 내 문단 높이 합산
  - 셀 내 중첩 표 → measure_table_recursive() 재귀 호출
  - 중첩 표 높이가 부모 셀 행 높이에 반영
```

#### 원칙 3: 경로 기반 표 접근 (Path-Based Access)

```
[현재] get_table_mut(sec, ppi, ci) — 3단계 고정

[목표] get_table_by_path(path: &[TablePathSegment])
  where TablePathSegment = {
      para_idx: usize,    // 문단 인덱스
      control_idx: usize, // 컨트롤 인덱스
      cell_row: u16,      // 셀 행
      cell_col: u16,      // 셀 열
  }
```

경로 예시:
```
최상위 표:  [(ppi=5, ci=0)]
중첩 표:    [(ppi=5, ci=0, row=2, col=1), (ppi=0, ci=0)]
3단 중첩:   [(ppi=5, ci=0, row=2, col=1), (ppi=0, ci=0, row=0, col=0), (ppi=0, ci=0)]
```

#### 원칙 4: 양방향 높이 전파

```
중첩 표 셀 내용 변경
  ↓
중첩 표 행 높이 재계산
  ↓
중첩 표 전체 높이 변경
  ↓
부모 셀 행 높이 재계산 (역전파)
  ↓
부모 표 전체 높이 변경?
  ├── NO → 부모 표 렌더만 갱신
  └── YES → 부모 표 페이지 분할 재계산
              ↓
           부모 표도 중첩?
              ├── NO → 섹션 리페이지네이션
              └── YES → 상위 표로 역전파 계속
```

### 10.3 2D 그리드 + 중첩 표 통합 설계

```rust
/// 표 내부 추상화 (엑셀 시트 + 재귀 중첩 지원)
pub struct TableGrid {
    row_count: usize,
    col_count: usize,
    /// O(1) 셀 접근: grid[row][col] = Some(cell_idx)
    grid: Vec<Vec<Option<usize>>>,
    /// 실제 셀 데이터
    cells: Vec<Cell>,
    /// 캐시된 행 높이 (편집 시 점진적 갱신)
    row_heights: Vec<f64>,
    /// 캐시된 열 폭
    col_widths: Vec<f64>,
    /// 캐시된 표 전체 높이
    total_height: Option<f64>,
}

impl Cell {
    /// 셀 내 문단 리스트 (기존)
    pub paragraphs: Vec<Paragraph>,

    // Paragraph.controls에 Control::Table이 있으면 중첩 표
    // → 각 중첩 표도 자체 TableGrid를 가짐
}
```

### 10.4 재귀적 무효화 + 역전파

```
셀(depth=2) 편집
  ├── 해당 셀의 compose 캐시 무효화
  ├── depth=2 표의 해당 행 높이 재측정
  ├── depth=2 표의 total_height 변경?
  │     ├── NO → depth=2 렌더만 갱신 (종료)
  │     └── YES → depth=1 부모 셀의 행 높이 재측정 (역전파)
  │               ├── depth=1 표의 total_height 변경?
  │               │     ├── NO → depth=1 렌더만 갱신 (종료)
  │               │     └── YES → depth=0 부모 셀의 행 높이 재측정 (역전파)
  │               │               └── depth=0 표의 페이지 분할 재계산
  │               │                    └── 섹션 리페이지네이션
  └── raw_stream = None
```

**핵심**: 높이 변경이 없으면 역전파가 조기 종료되어, 대부분의 편집이 O(1)~O(셀 수) 비용으로 처리된다.

### 10.5 현재 아키텍처 vs 목표 아키텍처 비교

| 관점 | 현재 (뷰어 기반) | 목표 (편집기 기반) |
|------|----------------|-----------------|
| **높이 계산** | HWP 메타데이터 신뢰 | 콘텐츠 기반 재귀 측정 |
| **높이 전파** | 없음 (단방향) | 양방향 역전파 |
| **표 레이아웃** | `layout_table` + `layout_nested_table` 이원화 | 단일 재귀 함수 |
| **표 접근** | 3단계 고정 인덱싱 | 경로 기반 임의 깊이 |
| **셀 탐색** | O(n) 플랫 선형 스캔 | O(1) 2D 그리드 인덱스 |
| **페이지 분할** | 최상위 표만 | 재귀적 분할 (깊이별 정책) |
| **무효화** | 섹션 전체 리플로우 | 점진적 + 조기 종료 역전파 |
| **편집 범위** | 최상위 표만 편집 가능 | 임의 깊이 중첩 표 편집 |

---

*작성일: 2026-02-15*
