# Task 291 구현설계서: 바탕쪽 렌더링

## 1. 목표

`samples/hwpctl_Action_Table__v1.1.hwp`의 바탕쪽을 올바르게 렌더링한다.
바탕쪽은 본문 아래 **최하위 레이어**에서 렌더링되며,
구역 내 모든 페이지에 동일하게 표시된다.

## 2. 바탕쪽 개념

- 구역(Section) 단위로 설정
- 적용 범위: 양쪽(Both) / 홀수(Odd) / 짝수(Even) / 임의(Custom)
- 콘텐츠: Shape, Picture, Table, 텍스트 등 자유 배치
- 본문 뒤(최하위 레이어)에 렌더링
- `hide_master_page` 플래그 또는 PageHide 컨트롤로 감추기 가능

## 3. 바이너리 구조 분석

### 대상 파일의 extra_child_records (SectionDef 하위)

```
[0]  tag=72(CTRL_HEADER) level=2 size=34   ← 바탕쪽 컨트롤 헤더
[1]  tag=66(LIST_HEADER) level=2 size=24   ← 바탕쪽 문단 리스트 (Both)
[2]  tag=67(PARA_HEADER) level=3 size=50   ↓ 바탕쪽 문단
[3]  tag=68(PARA_TEXT)   level=3 size=8
[4]  tag=69(PARA_CHAR_SHAPE) level=3
[5]  tag=71(CTRL_HEADER) level=3 size=60   ← Shape ctrl[0] (한컴 로고)
[6]  tag=76(SHAPE_COMP)  level=4
[7-13] ...                                 ← Shape 하위 + Picture ctrl[1]
[14] tag=71(CTRL_HEADER) level=3 size=196  ← Picture ctrl[2] or Shape
[15-19] ...                                ← 나머지 Shape/Picture 하위
```

### 파싱 현황

| 항목 | 현재 상태 | 비고 |
|------|----------|------|
| LIST_HEADER(tag=66) level=2 | ✅ 파싱 | Both 바탕쪽 1개 |
| CTRL_HEADER(tag=72) level=2 | ❌ 무시 | 바탕쪽 컨트롤 헤더 |
| "임의:N" 바탕쪽 | ❌ 미지원 | 추가 조사 필요 |

## 4. 렌더링 레이어 구조

```
┌──────────────────────────────┐
│ Layer 5: 글 앞으로 (Shape)     │ ← 최상위
│ Layer 4: 본문 텍스트 + 표      │
│ Layer 3: 머리말/꼬리말          │
│ Layer 2: 쪽 테두리/배경         │
│ Layer 1: 바탕쪽               │ ← 최하위 (본문 뒤)
└──────────────────────────────┘
```

현재 `build_page_tree` 호출 순서:
1. `build_page_background` (배경색) → Layer 2
2. `build_page_borders` (테두리선) → Layer 2
3. `build_master_page` (바탕쪽) → **Layer 1** ✅ 올바른 위치
4. `build_header/footer` (머리말/꼬리말) → Layer 3
5. `build_single_column` (본문) → Layer 4
6. Shape pass (`render_shapes_on_page`) → Layer 5

## 5. 구현 단계

### 5-1단계: 발견된 렌더링 버그 수정

**문제 1**: `build_master_page`에서 `Control::Picture`를 `layout_shape`로 전달하지만,
`layout_shape`는 `Control::Shape`만 처리하고 Picture는 무시 (즉시 return)

**수정**: Picture를 `compute_object_position` + `layout_picture`로 별도 렌더링

**파일**: `src/renderer/layout.rs` (build_master_page)

### 5-2단계: 바탕쪽 렌더 노드 캐시

**설계 원칙**: 바탕쪽은 구역 내 모든 페이지에 동일하므로 **1회 빌드 → 캐시 → 참조**

#### 5-2-1. 생성 시점

```
DocumentCore::paginate()
  ↓
  pagination 완료 후
  ↓
  build_master_page_cache()  ← 각 구역의 바탕쪽 렌더 노드를 미리 빌드
  ↓
  master_page_cache에 저장
```

#### 5-2-2. 캐시 자료구조

```rust
// DocumentCore에 추가
struct MasterPageCacheEntry {
    render_node: RenderNode,     // 바탕쪽 렌더 노드 (MasterPage 타입, children 포함)
    page_width: f64,             // 생성 시점의 페이지 크기 (검증용)
    page_height: f64,
}

master_page_cache: HashMap<(usize, usize), MasterPageCacheEntry>
// 키: (section_index, master_page_index)
```

#### 5-2-3. 캐시 빌드 (paginate 후)

```rust
fn build_master_page_cache(&mut self) {
    self.master_page_cache.clear();
    for (sec_idx, section) in self.document.sections.iter().enumerate() {
        if section.section_def.hide_master_page { continue; }
        for (mp_idx, mp) in section.section_def.master_pages.iter().enumerate() {
            let layout = PageLayoutInfo::from_page_def(&section.section_def.page_def, ...);
            let composed = compose_master_page_paragraphs(mp);
            let render_node = self.layout_engine.build_master_page_node(
                mp, &layout, &composed, &self.styles, &self.bin_data_content, sec_idx,
            );
            self.master_page_cache.insert(
                (sec_idx, mp_idx),
                MasterPageCacheEntry { render_node, page_width: layout.page_width, page_height: layout.page_height }
            );
        }
    }
}
```

#### 5-2-4. 페이지 렌더링 시 참조

```rust
// build_page_tree 내부
fn insert_cached_master_page(tree: &mut PageRenderTree, cache: &HashMap<...>, key: (usize, usize)) {
    if let Some(entry) = cache.get(&key) {
        // RenderNode를 clone하여 삽입 (node ID 재할당 필요)
        let cloned = clone_render_node_with_new_ids(tree, &entry.render_node);
        tree.root.children.push(cloned);
    }
}
```

#### 5-2-5. 노드 ID 충돌 방지

캐시된 RenderNode를 clone할 때 기존 페이지의 node ID와 충돌하지 않도록
`tree.next_id()`로 새 ID를 할당하는 deep clone 함수가 필요:

```rust
fn clone_render_node_with_new_ids(tree: &mut PageRenderTree, node: &RenderNode) -> RenderNode {
    let new_id = tree.next_id();
    let mut cloned = RenderNode::new(new_id, node.node_type.clone(), node.bbox.clone());
    for child in &node.children {
        cloned.children.push(clone_render_node_with_new_ids(tree, child));
    }
    cloned
}
```

#### 5-2-6. 캐시 무효화

| 이벤트 | 동작 |
|--------|------|
| `paginate()` 호출 | `master_page_cache.clear()` |
| 문서 로드 | paginate() → 자동 재빌드 |
| 구역 설정 변경 | paginate() → 자동 재빌드 |

### 5-3단계: Canvas 렌더러 확인

- `rhwp-studio/src/view/canvas-renderer.ts`에서 RenderNode 재귀 렌더링 확인
- MasterPage 노드 타입이 children 순회에 포함되는지 검증
- 필요시 타입 매칭 추가

### 5-4단계: "임의:N" 바탕쪽 파싱 (추가 조사 후)

- CTRL_HEADER(tag=72) level=2의 역할 해석
- 한컴에서 "임의:1"이 실제로 어떻게 저장되는지 바이너리 분석
- `HeaderFooterApply::Custom(u16)` 타입 추가 검토
- **이 단계는 현재 파일의 "양쪽" 바탕쪽 렌더링 완성 후 진행**

## 6. 수정 대상 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/layout.rs` | build_master_page에서 Picture 별도 렌더링 |
| `src/document_core/mod.rs` | master_page_cache 필드 추가 |
| `src/document_core/queries/rendering.rs` | paginate() 후 캐시 빌드, build_page_tree에서 캐시 참조 |
| `src/renderer/render_tree.rs` | clone_render_node_with_new_ids 함수 |
| `rhwp-studio/src/view/canvas-renderer.ts` | MasterPage 노드 타입 처리 확인 |

## 7. 검증

- `samples/hwpctl_Action_Table__v1.1.hwp` 16페이지
  - 모든 페이지: 바탕쪽 요소 (한컴 로고, 파란선, HANCOM 로고) 렌더링
  - SVG: 바탕쪽 요소가 본문 아래 레이어에 출력
  - WASM: Canvas에서 동일하게 렌더링
- 기존 HWP 파일 회귀: `cargo test` 716+ 통과
- 바탕쪽 없는 문서: 영향 없음 확인
