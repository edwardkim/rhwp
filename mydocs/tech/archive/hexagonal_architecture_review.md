---
kind: investigation
status: historical
canonical: mydocs/tech/parser_architecture.md
last_verified: 2026-07-17
---

# Hexagonal Architecture 전환 검토 보고서

작성일: 2026-03-23

## 1. 개요

rhwp 프로젝트의 현재 아키텍처를 Hexagonal Architecture(포트/어댑터 패턴) 관점에서 평가하고,
전면 전환의 타당성을 검토한다.

## 2. Hexagonal Architecture 개념

```
[Driving Adapter]  →  [Port]  →  [Domain Core]  ←  [Port]  ←  [Driven Adapter]
(요청자)              (인터페이스)    (비즈니스 로직)     (인터페이스)    (구현체)

Driving (Primary):              Driven (Secondary):
- CLI (main.rs)                 - HWP Parser (파일 입력)
- WASM API (wasm_api.rs)        - HWP Serializer (파일 출력)
- REST API (미래)               - SVG Renderer (출력)
                                - Canvas Renderer (출력)
                                - 폰트 메트릭 (외부 데이터)
```

핵심 원칙:
- Domain Core는 외부 어댑터에 **의존하지 않음**
- 외부 어댑터는 Port(Trait)를 통해 Core에 접근
- 의존성 방향이 항상 **바깥 → 안쪽**

## 3. 현재 구조 분석

### 3.1 모듈 의존성 방향

```
parser → model                    ✅ 올바름
serializer → model                ✅ 올바름
renderer → model                  ✅ 올바름
wasm_api → document_core, model   ✅ 올바름 (driving adapter)
main.rs → document_core, model    ✅ 올바름 (driving adapter)

document_core → model             ✅ 올바름
document_core → renderer          ❌ 100건 결합 (핵심 문제)
model → parser                    ❌ 1건 위반 (HWPTAG 상수)
model → serializer                ❌ 1건 위반
```

### 3.2 이미 헥사고날에 가까운 영역

| 영역 | 헥사고날 역할 | 현재 상태 | 평가 |
|------|-------------|-----------|------|
| model | Domain Core (데이터) | 순수 데이터 구조, 의존성 거의 0 | ✅ 완벽 |
| parser | Driven Adapter (입력) | model만 의존, 다른 모듈과 결합 없음 | ✅ 완벽 |
| serializer | Driven Adapter (출력) | model만 의존 | ✅ 양호 |
| wasm_api.rs | Driving Adapter | DocumentCore의 thin wrapper (Deref 패턴) | ✅ 완벽 |
| main.rs | Driving Adapter | CLI 진입점 | ✅ 완벽 |
| Renderer trait | Port 추상화 | SVG/HTML/Canvas 백엔드 교체 가능 | ✅ 이미 존재 |

### 3.3 헥사고날 원칙 위반: document_core → renderer 결합

**document_core가 renderer에 100건 의존**하고 있다. 헥사고날에서 Domain Core는 외부에 의존해서는 안 된다.

#### 결합 상세 분류

| 결합 유형 | 발생 위치 | 건수 | 분리 난이도 |
|-----------|-----------|------|-------------|
| 구조체 필드 | `document_core/mod.rs` | 9 | 매우 높음 |
| rendering 쿼리 | `queries/rendering.rs` | 25 | 높음 |
| cursor 계산 | `queries/cursor_rect.rs`, `cursor_nav.rs` | 31 | 매우 높음 |
| command 내 레이아웃 | `commands/*.rs` | 35 | 높음 |

#### 구체적 결합 예시

**A. DocumentCore 필드 (mod.rs)**
```rust
pub struct DocumentCore {
    // renderer 모듈의 타입을 직접 필드로 보유
    pub pagination_result: Option<PaginationResult>,    // renderer::pagination
    pub measured_sections: Vec<MeasuredSection>,         // renderer::height_measurer
    pub resolved_styles: Option<ResolvedStyleSet>,       // renderer::style_resolver
    pub composed_paragraphs: Vec<ComposedParagraph>,     // renderer::composer
    // ...
}
```

**B. 커서 위치 계산 (cursor_rect.rs)**
```rust
// RenderNode(renderer 타입)를 직접 탐색
fn find_cursor_in_tree(tree: &PageRenderTree, ...) {
    for node in &tree.nodes {
        match &node.node_type {
            RenderNodeType::TextLine { ... } => { ... }
            // renderer 내부 구조에 깊이 결합
        }
    }
}
```

**C. 편집 명령 내 리플로우 (commands/text_editing.rs)**
```rust
// 텍스트 삽입 후 즉시 라인 리플로우 수행
self.reflow_line_segs(section_idx, para_idx);  // renderer의 composer 호출
```

## 4. 전면 전환 비용 분석

### 4.1 필요한 작업

#### Port(Trait) 정의가 필요한 경계 (4개)

| Port | 추상화 대상 | 영향 범위 |
|------|------------|-----------|
| LayoutPort | compose_paragraph, reflow_line_segs, compute_char_positions | commands 35건 |
| PaginationPort | Paginator, PaginationResult, PageItem | queries 25건 |
| RenderTreePort | PageRenderTree, RenderNode 탐색 | cursor 31건 |
| StyleResolverPort | resolve_styles, ResolvedStyleSet | mod.rs 9건 |

#### 타입 이동 필요 구조체 (7개)

현재 renderer에 정의되어 있으나, document_core가 필드로 보유하는 타입:
- `PaginationResult`, `PageItem`
- `MeasuredSection`, `MeasuredTable`
- `ResolvedStyleSet`, `ResolvedCharStyle`, `ResolvedParaStyle`
- `ComposedParagraph`, `ComposedLine`

#### 코드 변경량

- 직접 영향: 15개 파일 (document_core 내)
- 간접 영향: 26개 파일 (renderer 내)
- 총 변경 대상: **약 65,000줄**

### 4.2 비용

| 항목 | 비용 |
|------|------|
| 코드 변경량 | 65,000줄 (프로젝트의 49%) |
| 소요 기간 | 2~3주 (기능 개발 중단) |
| Trait 간접 호출 오버헤드 | dyn Trait → vtable dispatch 성능 저하 |
| 제네릭 사용 시 | 컴파일 시간 증가, 코드 복잡도 상승 |
| 테스트 전면 수정 | 718개 테스트 중 상당수 영향 |
| 리그레션 위험 | 높음 (레이아웃/페이지네이션 정밀 좌표) |

### 4.3 이점

| 항목 | 이점 | 현재 상태 |
|------|------|-----------|
| Renderer 교체 | SVG↔Canvas↔PDF 교체 | **이미 Renderer trait으로 가능** |
| Parser 교체 | HWP5↔HWPX 교체 | **이미 가능** (parser만 교체) |
| Mock 테스트 | renderer 없이 document_core 테스트 | 현재 608개 테스트 정상 동작 |
| 구조적 순수성 | 의존성 방향 완벽 | 실용적 가치 낮음 |

## 5. 판단: 전면 전환 비권장

### 5.1 비권장 근거

**1. WYSIWYG 워드프로세서의 본질적 특성**

커서 위치 계산이 렌더 트리를 탐색해야 하고, 편집 명령이 라인 리플로우를 즉시 수행해야 하는 것은
WYSIWYG 에디터의 본질적 요구사항이다. 이 결합을 trait으로 추상화하면 indirection만 증가하고
실질적 이점이 없다.

```
// WYSIWYG에서 필연적인 흐름:
텍스트 삽입 → 라인 리플로우 → 커서 위치 재계산 → 화면 갱신
(command)      (layout)         (render tree)        (renderer)
```

이 흐름을 trait으로 분리하면 코드 복잡도만 증가한다.

**2. 이미 확보된 교체 가능성**

- Renderer 교체: `Renderer` trait으로 SVG/HTML/Canvas 이미 교체 가능
- Parser 교체: parser가 model만 의존하므로 HWP5/HWPX 이미 교체 가능
- Driving Adapter: wasm_api/main.rs가 이미 thin wrapper

추가 추상화로 얻을 교체 가능성이 없다.

**3. 비용 대비 이점 부족**

65,000줄 변경 + 2~3주 기능 개발 중단의 비용에 비해,
실질적으로 새로 얻는 이점이 거의 없다.

**4. 오버엔지니어링 위험**

HWP 뷰어/에디터로서 parser나 renderer 구현체가 근본적으로 바뀔 가능성이 매우 낮다.
"추상화를 위한 추상화"가 될 위험이 크다.

### 5.2 유사 프로젝트 참고

| 프로젝트 | 아키텍처 | document↔renderer 결합 |
|----------|----------|----------------------|
| LibreOffice Writer | 직접 결합 | SwDoc ↔ SwLayout 밀접 결합 |
| Google Docs | 직접 결합 | Model ↔ Layout 밀접 결합 |
| VS Code (Monaco) | 직접 결합 | TextModel ↔ ViewLayout 결합 |
| ProseMirror | 분리형 | State ↔ View 분리 (but 웹 특화) |

대부분의 WYSIWYG 에디터는 document와 renderer를 밀접하게 결합한다.
이것은 설계 결함이 아니라 도메인 특성에 따른 합리적 선택이다.

## 6. 권장: 점진적 개선

전면 전환 대신, **데이터 구조체 위치 이동**으로 의존성 방향을 개선한다.

### 6.1 개선 방향

```
현재:
  document_core → renderer (100건, 타입 + 로직)

개선 후:
  document_core → model ← renderer (데이터 구조체는 model로)
  document_core → renderer (로직 호출만, ~35건으로 감소)
```

### 6.2 model로 이동할 후보 타입

| 현재 위치 | 타입 | 이동 후 |
|-----------|------|---------|
| renderer::pagination | PaginationResult, PageItem, PageContent | model::page |
| renderer::height_measurer | MeasuredSection, MeasuredTable | model::measured |
| renderer::style_resolver | ResolvedStyleSet, ResolvedCharStyle, ResolvedParaStyle | model::resolved_style |
| renderer::composer | ComposedParagraph, ComposedLine | model::composed |

### 6.3 이동하지 않을 결합 (수용)

| 결합 | 이유 |
|------|------|
| cursor_rect → RenderNode 탐색 | WYSIWYG 본질적 요구 |
| text_editing → reflow_line_segs | 편집 후 즉시 리플로우 필수 |
| rendering.rs → LayoutEngine | 렌더 트리 생성은 renderer의 책임 |

### 6.4 예상 효과

- document_core → renderer 의존: 100건 → ~35건 (65% 감소)
- model 순수성: 유지 (데이터 구조체만 추가, 로직 없음)
- 변경 규모: ~5,000줄 (전면 전환의 1/13)
- 리그레션 위험: 낮음 (타입 위치만 변경, 로직 불변)

## 7. 결론

| 항목 | 전면 전환 | 점진적 개선 |
|------|----------|------------|
| 변경량 | 65,000줄 | 5,000줄 |
| 소요 기간 | 2~3주 | 2~3일 |
| 의존성 개선 | 100건 → 0건 | 100건 → 35건 |
| 리그레션 위험 | 높음 | 낮음 |
| 추가 이점 | 구조적 순수성 | 실용적 의존성 정리 |
| 판단 | **비권장** | **권장** |

rhwp의 현재 구조는 "불완전한 헥사고날"이 아니라,
**WYSIWYG 에디터 도메인에 최적화된 실용적 아키텍처**이다.

점진적 개선(데이터 구조체 이동)으로 의존성 방향을 정리하면,
전면 전환 없이도 충분한 구조적 건전성을 확보할 수 있다.
