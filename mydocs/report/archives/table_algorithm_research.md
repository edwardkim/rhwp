# 표 아키텍처 개선을 위한 알고리즘 조사 보고서

## 1. 개요

현재 아키텍처(Rust WASM + Canvas/JS)의 Pain Point 6개에 대해, 브라우저 엔진/조판 시스템/스프레드시트 엔진에서 검증된 알고리즘을 조사하고 적용 방안을 정리한다.

---

## 2. Pain Point별 알고리즘 분석

### 2.1 플랫 셀 배열 O(n) 탐색

**현재**: `find_cell_at_row_col()`이 `Vec<Cell>`를 매번 선형 스캔

#### Dense Grid (행×열 밀집 배열)

```rust
// grid[row * col_count + col] = Some(cell_idx)
cell_grid: Vec<Option<usize>>  // 크기: row_count × col_count
```

- **복잡도**: O(1) 조회, O(R×C) 공간, O(n) 구축
- **병합 셀**: span 영역 전체를 앵커 셀 인덱스로 채움
- **사용 사례**: Chrome Blink LayoutNG, Firefox Gecko 표 레이아웃, CSSWG 표 알고리즘 스펙
- **WASM 적합성**: 캐시 친화적, 해시 오버헤드 없음. HWP 표는 대부분 100×100 미만이므로 최적

> **HashMap 대비 장점**: HWP 표는 차원이 고정되어 있으므로 밀집 배열이 해시보다 빠르고 메모리 예측 가능

**권장**: Dense Grid 채택. 구조 변경(행/열 추가/삭제) 후 O(R×C) 재구축 — 일반 표에서 무시 가능한 비용.

---

### 2.2 전체 섹션 리플로우

**현재**: 셀 한 글자 편집에 `compose_section()` + `measure_section()` + `paginate()` 전체 실행

#### 2.2.1 Double Dirty Bit (Gecko/Blink 모델)

```
노드별 2개 비트:
  IS_DIRTY              — 이 노드 자체가 재레이아웃 필요
  HAS_DIRTY_CHILDREN    — 하위 노드 중 최소 1개가 dirty

마킹: 변경된 노드에 IS_DIRTY 설정 → 루트까지 HAS_DIRTY_CHILDREN 전파 (O(depth))
레이아웃: 두 비트 모두 clean인 서브트리는 건너뜀
```

- **복잡도**: 마킹 O(depth), 레이아웃 O(dirty 노드 수)
- **사용 사례**: Firefox Gecko (`NS_FRAME_IS_DIRTY` + `NS_FRAME_HAS_DIRTY_CHILDREN`), Chrome Blink
- **참고**: 2025년 PLDI 논문 "Spineless Traversal"은 우선순위 큐 기반으로 Double Dirty Bit 대비 1.80x 추가 성능 향상 달성

#### 2.2.2 Relayout Boundary (Flutter 모델)

```
특정 노드를 "relayout boundary"로 선언
  → 하위 변경이 상위 레이아웃에 영향 없음 보장
  → dirty 전파가 boundary에서 중단

자연스러운 boundary:
  - 고정 열 폭 표 → 셀 내용 변경이 표 폭에 영향 없음
  - 고정 높이 셀 → 내용 변경이 행 높이에 영향 없음 (오버플로 시 클리핑)
```

- **복잡도**: dirty 전파가 가장 가까운 boundary에서 중단 → O(서브트리) 레이아웃
- **사용 사례**: Flutter `RenderObject._relayoutBoundary`, Chrome Blink "layout root"
- **적용**: 표가 자연스러운 relayout boundary. 셀 편집 → 행 높이 불변이면 표 바깥 리플로우 불필요

#### 2.2.3 Constrained Memoization (Typst/Comemo 모델)

```rust
#[comemo::memoize]
fn compose_paragraph(para: &Paragraph) -> ComposedParagraph { ... }

#[comemo::memoize]
fn measure_table(table: &Table, styles: &StyleSet) -> MeasuredTable { ... }
```

- **원리**: 순수 함수에 `#[memoize]` 어노테이션. 프레임워크가 함수의 실제 접근 입력을 추적. 접근한 입력이 변경되지 않았으면 캐시 결과 반환
- **복잡도**: 캐시 히트 O(1), 미스 O(함수 비용)
- **사용 사례**: **Typst** (Rust 조판 엔진) — comemo로 서브초 증분 컴파일 달성. **rust-analyzer** — Salsa로 증분 의미 분석
- **WASM 적합성**: [comemo](https://github.com/typst/comemo) Rust 크레이트, WASM 컴파일 가능
- **적용**: `compose_paragraph()`, `measure_paragraph()`, `measure_table()`이 모두 순수 함수 → comemo 적용으로 **변경되지 않은 문단/표는 자동 스킵**

**권장**: 3가지 조합 — Double Dirty Bit(신호) + Relayout Boundary(전파 제한) + Comemo(자동 캐싱)

---

### 2.3 높이 이중 계산

**현재**: `measure_table()`과 `layout_table()`이 동일한 행 높이 계산을 각각 독립 실행

#### 측정 결과 전달 (Measure-Once, Layout-Reuse)

```rust
// 현재: layout_table()이 행 높이를 독자적으로 재계산
// 개선: MeasuredTable을 레이아웃 단계에 전달

fn layout_table(
    &self,
    measured: &MeasuredTable,  // ← 사전 측정 결과
    // ...
)
```

- **원리**: 브라우저 엔진은 측정과 배치를 단일 패스로 수행 (Flutter `performLayout()`). 그러나 페이지네이션이 있는 문서 엔진은 측정→페이지 분할→렌더가 불가피하게 분리됨. 해결책: 측정 결과를 캐시하여 렌더에 전달
- **복잡도**: O(n) → O(n) (총 비용 불변이나 중복 제거로 상수 계수 1/2)
- **사용 사례**: TeX (박스 측정 후 페이지 빌더에 전달), Typst (region 기반 레이아웃)

**권장**: `MeasuredTable`을 `layout_table()`에 직접 전달. Comemo 적용 시 자동 해결.

---

### 2.4 중첩 표 높이 역전파

**현재**: `calc_cell_controls_height()` → 0 반환. 중첩 표 높이 변경이 부모에 전파 안 됨.

#### 2.4.1 Bottom-Up Dirty Propagation (Gecko/Flutter 모델)

```
리프 노드(문단) 높이 변경
  → IS_DIRTY 설정
  → 부모 셀 → 부모 행 → 부모 표 → ... 루트까지 HAS_DIRTY_CHILDREN 전파
  → 재레이아웃 시 bottom-up 처리:
    가장 안쪽 dirty 표부터 높이 재계산
    → 부모 행 높이 갱신
    → 부모 표 높이 갱신
    → ...
```

- **복잡도**: 전파 O(depth), 재레이아웃 O(depth × 레벨당 셀 수)
- **사용 사례**: Gecko `NS_FRAME_IS_DIRTY` 표 프레임 전파, Flutter `markNeedsLayout`

#### 2.4.2 Constraint Propagation with Fixed-Point (CSS 표 알고리즘)

```
행 높이 = max(셀별 콘텐츠 높이)
셀 콘텐츠 높이 = padding + Σ(문단 높이) + Σ(중첩 표 높이)
중첩 표 높이 = Σ(행 높이) + 셀 간격

→ 비순환 제약조건 → O(depth) 반복으로 수렴 보장
```

- **복잡도**: O(depth × 레벨당 셀 수)
- **사용 사례**: CSS 2.1 표 레이아웃 알고리즘, CSSWG 표 알고리즘 드래프트
- **적용**: 현재 `height_measurer.rs`의 병합 셀 제약 해결 루프를 중첩 표로 확장

**권장**: Bottom-Up Dirty Propagation(신호) + Constraint Propagation(계산). 기존 `height_measurer.rs` 제약 해결 루프의 자연스러운 확장.

---

### 2.5 표 페이지 분할 최적화

**현재**: 행 단위 선형 스캔으로 페이지 경계 결정

#### 2.5.1 Prefix Sum + Binary Search

```rust
// 누적 행 높이 사전 계산
cumulative[0] = 0;
cumulative[r+1] = cumulative[r] + row_heights[r] + cell_spacing;

// 페이지 분할점: "잔여 높이 이하인 최대 행" → 이진 탐색
let break_row = cumulative.partition_point(|&h| h <= available_height);
```

- **복잡도**: 전처리 O(R), 분할점 결정 O(log R) (현재 O(R))
- **사용 사례**: PDF 생성 라이브러리, 대용량 표 페이지네이션
- **적용**: `paginate_with_measured()`의 행 루프를 이진 탐색으로 대체

#### 2.5.2 Penalty 기반 분할 (TeX 페이지 빌더)

```
각 분할 후보에 페널티 부여:
  - 첫 행 후 분할 (orphan)     → 높은 페널티
  - 마지막 행 전 분할 (widow)   → 높은 페널티
  - 병합 셀 span 내부 분할      → 매우 높은 페널티
  - 제목행 직후 분할            → 낮은 페널티
  - 일반 행 경계               → 0 페널티

greedy 방식으로 페이지 채우되, 페널티가 낮은 분할점 우선
```

- **복잡도**: O(R) greedy, O(R²) 최적 DP (Knuth-Plass 확장)
- **사용 사례**: TeX 페이지 빌더, Typst region 기반 레이아웃
- **적용**: 현재 `MIN_SPLIT_CONTENT_PX = 10.0` 임계값을 페널티 모델로 일반화

**권장**: Prefix Sum(성능) + Penalty 가중치(품질) 조합.

---

### 2.6 경로 기반 중첩 구조 접근

**현재**: `get_table_mut(sec, ppi, ci)` — 3단계 고정 인덱싱. 중첩 표 접근 불가.

#### 2.6.1 Path Encoding (경로 인코딩)

```rust
enum PathSegment {
    Paragraph(usize),       // 문단 인덱스
    Control(usize),         // 컨트롤 인덱스 (Table, Picture 등)
    Cell(u16, u16),         // 셀 (row, col)
}

type DocumentPath = Vec<PathSegment>;

// 사용 예
let nested_path = vec![
    PathSegment::Paragraph(5),   // 본문 문단 5
    PathSegment::Control(0),     // 첫 번째 컨트롤 (최상위 표)
    PathSegment::Cell(2, 1),     // 셀 (2,1)
    PathSegment::Paragraph(0),   // 셀 내 문단 0
    PathSegment::Control(0),     // 중첩 표
    PathSegment::Cell(0, 0),     // 중첩 셀 (0,0)
];
```

- **복잡도**: 접근 O(depth), dirty 전파 O(depth)
- **사용 사례**: Google Docs API 구조적 요소 모델, XPath/JSONPath 주소 체계
- **적용**: 현재 `DocumentPosition`의 7필드를 `DocumentPath`로 통합 가능

#### 2.6.2 Arena-Based Tree (indextree)

```rust
// 모든 노드를 플랫 Arena에 저장
let arena: Arena<DocumentNode> = Arena::new();

// 노드 접근: O(1)
let node = arena[node_id];

// 부모 추적: O(1)
let parent = node.parent();

// 중첩 표 접근: arena 인덱스 시퀀스
let path = [section_id, para_id, table_id, cell_id, nested_table_id];
```

- **복잡도**: O(1) 노드 접근, O(1) 부모 접근, O(depth) 경로 순회
- **사용 사례**: Servo 레이아웃 엔진, Rust DOM 구현체 ([indextree](https://github.com/saschagrunert/indextree)), [generational-arena](https://github.com/fitzgen/generational-arena)
- **장점**: 부모 포인터 자연 지원 → dirty 역전파에 최적. Rust 소유권 문제 해소 (재귀 구조 대신 플랫 벡터)
- **단점**: 기존 데이터 모델 전면 재구성 필요

**권장**: 단기 — Path Encoding (최소 변경). 장기 — Arena-Based Tree (근본 해결).

---

## 3. 핵심 참조 시스템 비교

### 3.1 레이아웃 엔진 비교

| 시스템 | Dirty 메커니즘 | 전파 방향 | 경계 개념 | 증분 전략 |
|--------|-------------|----------|----------|----------|
| **Gecko** (Firefox) | 2비트 (IS_DIRTY, HAS_DIRTY_CHILDREN) | 바텀업 마킹, 탑다운 리플로우 | Reflow Root | 프레임별 dirty 추적 |
| **Blink** (Chrome) | Dirty 플래그 + 무효화 집합 | 바텀업 + 수평 | Layout Root | LayoutNG 입출력 분리 |
| **Flutter** | markNeedsLayout | 바텀업 → Relayout Boundary | RelayoutBoundary | 제약 하향, 크기 상향 |
| **Typst** | Comemo 캐시 | 함수 단위 자동 | 함수 경계 | 제약 메모이제이션 |
| **TeX** | 없음 (단일 패스) | 탑다운 | 페이지 | 증분 없음 (배치 처리) |

### 3.2 Rust WASM 호환 크레이트

| 크레이트 | 용도 | WASM | 통합 난이도 |
|---------|------|------|-----------|
| [comemo](https://github.com/typst/comemo) | 제약 메모이제이션 | 가능 | 낮음 (어노테이션) |
| [salsa](https://github.com/salsa-rs/salsa) | 쿼리 기반 증분 계산 | 가능 | 높음 (프로그램 재구조화) |
| [indextree](https://github.com/saschagrunert/indextree) | Arena 기반 트리 | 가능 | 중간 |
| [generational-arena](https://github.com/fitzgen/generational-arena) | 세대별 Arena | 가능 | 중간 |
| [slotmap](https://github.com/orlp/slotmap) | 키-값 Arena | 가능 | 중간 |

---

## 4. 통합 적용 전략

### 4.1 3계층 증분 레이아웃 아키텍처

```
┌─────────────────────────────────────────────────┐
│ 계층 1: 신호 (Double Dirty Bit)                   │
│ - IS_DIRTY + HAS_DIRTY_CHILDREN 2비트             │
│ - 변경 시 O(depth) 상향 전파                       │
│ - clean 서브트리 완전 스킵                          │
└──────────────────────┬──────────────────────────┘
                       ▼
┌─────────────────────────────────────────────────┐
│ 계층 2: 경계 (Relayout Boundary)                  │
│ - 표 = 자연스러운 relayout boundary               │
│ - 셀 내 변경 → 행 높이 불변이면 표 바깥 리플로우 없음 │
│ - 행 높이 변경 → 표까지만 전파, 표 높이 불변이면 중단  │
└──────────────────────┬──────────────────────────┘
                       ▼
┌─────────────────────────────────────────────────┐
│ 계층 3: 캐싱 (Comemo Memoization)                 │
│ - compose_paragraph() → 캐시 히트 시 O(1)         │
│ - measure_table() → 변경 없는 표 자동 스킵          │
│ - measure_section() 재호출해도 실질 비용 O(dirty)   │
└─────────────────────────────────────────────────┘
```

### 4.2 중첩 표 역전파 통합

```
셀(depth=2) 편집
  │
  ├── [Comemo] compose_paragraph() 캐시 미스 → 재계산
  │
  ├── [Constraint Propagation] depth=2 행 높이 재계산
  │     row_height = max(cell_content_heights)
  │
  ├── [Dirty Bit] depth=2 표 total_height 변경?
  │     ├── NO → [Relayout Boundary] 전파 중단 (종료)
  │     └── YES → HAS_DIRTY_CHILDREN → depth=1 부모 셀
  │               │
  │               ├── [Constraint Propagation] depth=1 행 높이 재계산
  │               ├── [Dirty Bit] depth=1 표 total_height 변경?
  │               │     ├── NO → 전파 중단
  │               │     └── YES → depth=0 최상위 표
  │               │               │
  │               │               ├── [Prefix Sum] 페이지 분할 재계산
  │               │               └── 섹션 리페이지네이션
  │
  └── [Comemo] 다른 모든 문단/표 → 캐시 히트 → O(1) 스킵
```

---

## 5. 구현 우선순위

| 순서 | Pain Point | 알고리즘 | 공수 | 효과 | 선행 조건 |
|------|-----------|---------|------|------|----------|
| 1 | 셀 탐색 O(n) | Dense Grid | 소 | 높음 | 없음 |
| 2 | 높이 이중 계산 | MeasuredTable 전달 | 소 | 중간 | 없음 |
| 3 | 경로 기반 접근 | Path Encoding | 중 | 높음 | 없음 |
| 4 | 중첩 높이 역전파 | Bottom-Up Dirty + Constraint | 중 | 높음 | #3 |
| 5 | 페이지 분할 | Prefix Sum + Penalty | 중 | 중간 | #2 |
| 6 | 전체 섹션 리플로우 | Dirty Bit + Boundary + Comemo | 대 | 극대 | #1~#5 |

- **#1, #2**: 독립 구현 가능한 quick win
- **#3**: #4(중첩 역전파)의 전제 조건
- **#6**: 다른 모든 개선이 기반을 제공. 최대 효과지만 최대 공수

---

## 6. 참고 문헌

### 브라우저 엔진
- [BlinkNG 파이프라인](https://developer.chrome.com/docs/chromium/blinkng)
- [LayoutNG 아키텍처](https://developer.chrome.com/docs/chromium/layoutng)
- [Gecko Reflow 문서](https://www-archive.mozilla.org/newlayout/doc/reflow.html)
- [Firefox Layout Overview](https://firefox-source-docs.mozilla.org/layout/LayoutOverview.html)
- [Spineless Traversal (PLDI 2025)](https://arxiv.org/html/2411.10659v8) — Double Dirty Bit 대비 1.80x 성능 향상

### 조판 시스템
- [Typst Layout Models](https://laurmaedje.github.io/posts/layout-models/)
- [Comemo: Constrained Memoization](https://github.com/typst/comemo)
- [TeX Page Breaking](https://link.springer.com/chapter/10.1007/978-1-4613-9142-5_1)
- [Knuth-Plass 알고리즘](https://en.wikipedia.org/wiki/Knuth%E2%80%93Plass_line-breaking_algorithm)

### 텍스트 에디터
- [Xi-editor: 증분 워드 래핑](https://xi-editor.io/docs/rope_science_05.html)
- [VS Code: Piece Table](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation)

### Flutter / React
- [Flutter markNeedsLayout](https://api.flutter.dev/flutter/rendering/RenderObject/markNeedsLayout.html)
- [Flutter Relayout Boundary](https://flutter.megathink.com/rendering/layout)

### 스프레드시트
- [Excel 재계산 알고리즘](https://saturncloud.io/blog/what-algorithm-does-excel-use-to-recalculate-formulas/)
- [스프레드시트 재계산 방법론](https://lord.io/spreadsheets/)

### CSS 표 알고리즘
- [CSSWG Table Algorithms Draft](https://drafts.csswg.org/css3-tables-algorithms/Overview.src.htm)

### Rust 크레이트
- [Salsa (증분 계산)](https://github.com/salsa-rs/salsa)
- [indextree (Arena 트리)](https://github.com/saschagrunert/indextree)
- [generational-arena](https://github.com/fitzgen/generational-arena)

---

*작성일: 2026-02-15*
