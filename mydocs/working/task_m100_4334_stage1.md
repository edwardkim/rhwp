---
kind: investigation
status: completed
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-08-11
---

# Task #4334 Stage 1 — `stableIndex` 를 `next_id()` 카운터 의존에서 문서 경로로

이슈 [#4334](https://github.com/edwardkim/rhwp/issues/4334): `LayoutFrame::next_id`
(`render_tree.rs`)가 단조 카운터로 `NodeId`(u32)를 발급한다 — 159개 발급 지점, 트리 안 위치가
아니라 방문 순서를 인코딩할 뿐이다. 원 이슈는 `NodeId` 자체(전 노드 정체성)를 구조적 이름으로
바꾸는 안을 제안했다. 분해 1단계인 이 작업은 그중 **`paper_node_sort_key`(같은 plane/zOrder 안
tie-break)** 하나만 다룬다 — `RenderNode.id`/`NodeId` 는 이번에 건드리지 않았다.

## 무엇을 바꿨나

```rust
// before — src/renderer/layout.rs, paper_node_sort_key
let (z_order, stable_index) = layer
    .map(|layer| (layer.z_order, layer.stable_index))
    .unwrap_or((0, node.id));            // ← layer 없는 inline 노드는 원본 카운터
(Self::render_layer_plane(layer), z_order, stable_index)   // (u8, i32, u32)

// after
let z_order = layer.map(|layer| layer.z_order).unwrap_or(0);
let doc_path = doc_path_for_node(node).unwrap_or_default();
(Self::render_layer_plane(layer), z_order, doc_path)       // (u8, i32, DocPath)
```

`RenderLayerInfo.stable_index` 는 `object_stable_index(para, ctrl)` 의 패킹된 u32(문단 1이면 벌써
65536)였고, layer 없는 inline 노드는 `next_id()` 카운터(보통 수십~수백)였다. 두 갈래가 서로 다른
수 공간인데 TS 는 하나의 수로 비교했다(`ka[2] > kb[2]`). 이제 layer 유무와 무관하게 `DocPath`
(`[section, para, ...cell 경로, control]` 정수 배열) 하나의 좌표계를 쓰고 사전식으로 비교한다 —
Rust 는 `Vec<u32>` 의 기본 `Ord`, TS 는 새 `compareLexArrays`.

`InlineShapeKey`(`render_tree.rs`)/`CellContext.path`(`layout.rs`)와 같은 좌표계를 재사용했다 —
새 이름공간을 만들지 않았다.

## 네 방향 중 셋을 폐기했다 — 두 방향은 커밋으로도 남기지 않았다

### A. 트리 위치 기반 `StructuralPath` + 고정폭 해시 — 폐기, 코드도 남기지 않음

첫 실험은 루트에서 각 노드까지의 **부모 기준 child 인덱스 체인**을 정체성으로 쓰고, `NodeId = u32`
폭을 유지하려고 FNV-1a 로 접는 안이었다. 실측(`samples/hwpspec-w.hwp`,
`samples/issue2006/1790387_prep_final_report.hwpx`, `samples/issue1921/59043_regulatory_analysis.hwp`,
문서당 최대 60페이지):

```
pages=97  total_nodes=18389  max_nodes_per_page=886
u32_hash_collisions=0  u64_hash_collisions=0
```

충돌은 0건이었지만 **0 보장은 아니다** — 886 노드/페이지의 u32 birthday bound ≈ 9×10⁻⁵. 트리 정의상
자동으로 보장되던 유일성을 확률로 바꾸는 거래라 폐기했다. 게다가 트리 위치는 out-of-flow 개체
재배치 때문에 문서 순서와 어긋난다(아래 B 참고).

**커밋을 남기지 않은 이유**: `StructuralPath`/`collect_structural_paths`/`hash_structural_path_u32`
/`hash_structural_path_u64` 를 채택안은 한 번도 호출하지 않는다. 그대로 두면 테스트만 부르는
`pub(crate)` 죽은 코드가 프로덕션에 남는다. 측정값은 위에 남기고 코드는 버렸다.

### B. `collect_controls` 의 DFS 순회 서수 — 폐기, 코드도 남기지 않음

두 번째 안은 `stableIndex` 를 `collect_controls`(rendering.rs)의 pre-order DFS 방문 순서로
통일하는 것이었다. 이 안이 성립하려면 **현재도** `node.id`(발급 순서)가 그 DFS 순서와 일치해야
한다. 97페이지·18,389노드를 훑어 위반 1건을 찾았다:

```
samples/issue2006/1790387_prep_final_report.hwpx page=0
prev(id=76, type=TextRun) -> node(id=6, type=Table)
```

out-of-flow(layer 있는) 개체는 별도 패스에서 일찍 id 를 받지만 최종 트리에서는 Body 서브트리
**뒤에** 형제로 재배치된다(`root.children == [PageBg, Header, Body, Table, Footer]`). 이 Table 은
`object_stable_index(para=0, ctrl=2) = 2` 로 문서 맨 앞을 올바로 반영하는데, DFS 위치 서수를 쓰면
Body 의 자손 ~85개보다 뒤로 밀려 의미가 뒤집힌다 — 하필 stableIndex 가 필요한 바로 그 out-of-flow
개체에서 깨진다. **STOP.**

**커밋을 남기지 않은 이유**: 그 선행 확인 테스트는 `violations.len() == 1` 을 고정했다. 채택안이
더 이상 의존하지 않는 성질(`next_id()` 발급 순서 vs DFS 순서)을 실제 문서 3종에 대해 못박는
단언이라, 무관한 레이아웃 변경이 이 수를 흔들면 다음 작업자는 원인 조사 대신 숫자만 고쳐
통과시키게 된다. 관측은 위 블록에 남기고 테스트는 버렸다.

### C. 스칼라 `object_stable_index(para, control)` 공간을 inline 노드까지 확장 — 폐기

layered 노드가 이미 쓰는 패킹 스칼라를 inline 노드까지 그대로 넓히는 안. 233페이지 실측에서 세
가지가 드러났다.

| 관측 | 수치 |
|---|---|
| `collect_controls` 가 TS 로 내보내는 Table/Equation/Image | 1,680 |
| 그중 para/control 인덱스가 **없는** 노드 | 42 |
| 그중 `cell_index`/`cell_context` 를 갖는 노드 | 391 (23%) |
| `sort_paper_render_nodes` 대상 중 layer=None | 56% |

- 문서 위치를 못 채우는 노드가 42개 있고, 이들은 **오늘 이미** raw `node.id` 폴백에 전적으로
  의존한다.
- 23% 가 셀 안에 있는데 16/16 비트 패킹은 `cell_path` 를 담지 못한다 — 서로 다른 셀의 동일
  `(para, control)` 이 충돌한다.

즉 스칼라를 무엇으로 채울지의 문제가 아니라 **스칼라 자체가 부족**했다. **STOP.**

이 방향의 선행 확인 테스트(`issue_4334_stage3_document_position_coverage_precheck`)는 **남겼다** —
아래 "구현" 이 만든 잔여(24)를 고정하는 래칫으로 역할이 바뀌었기 때문이다.

### D. 문서 경로 정수 배열 — 채택

`(section, para)` + 셀 중첩 축(`(control, cell, cell_para)` 반복) + `control` 을 정수 배열로 만들고
사전식으로 비교한다. `InlineShapeKey`/`CellContext.path` 와 같은 좌표계라 새로 발명한 게 없고,
트리 정의상 충돌이 없으며(해시가 아니다), 셀 축과 합성 노드를 모두 담는다.

## 42개 host 전수 조사 — 세 플러밍 결손, 잔여 24

C 에서 실측한 42개(para/control 없는 Table/Equation/Image)를 전수 조사한 결과 **host 없는 노드는
하나도 없었다** — 전부 "host 는 있지만 노드 필드로 안 이어져 있던" 플러밍 결손이다.

1. **TAC(text-as-char) 중첩 표** — `src/renderer/layout/table_cell_content.rs`
   (`layout_embedded_table`)가 `enclosing_ctx`(호스트 경로: section/para/parent_path/control_index)
   를 **갖고 있으면서도** `TableNode.section_index/para_index/control_index` 를 전부 `None` 으로
   버렸다.
2. **바탕쪽(master page) `Control::Picture`** — `src/renderer/layout.rs` 의 `build_master_page` 가
   `Control::Table`/`Shape` 분기는 이미 바탕쪽 로컬 `(pi, ci)` 를 넘기는데 `Control::Picture`
   분기만 `layout_picture` 에 `None, None` 을 넘겼다.
3. **재귀 중첩 표 3곳**(`table_layout.rs` 2곳, `table_partial.rs` 1곳) — `enclosing_cell_ctx`
   (`nested_ctx`)는 넘기면서 `table_meta` 는 `None`. 셀 경로의 마지막 두 항목에서
   `(cell_para_index, control_index)` 를 유도해 채웠다 — 이 유도식은 `CellContext::nested_table_meta`
   로 한 번만 쓴다(`table_partial.rs` 의 기존 `layout_partial_table_item` 패턴과 같은 식).

세 원인을 모두 고친 뒤 재측정: **42 → 24**. 남은 24는 전부 Image, 원인 하나로 수렴한다 —
`render_cell_background`(`src/renderer/layout/table_layout.rs`, 표 셀 배경/무늬 이미지 채우기)는
문서 Control 이 아니라 셀의 border-fill 스타일에서 파생된 순수 장식이라 그 함수 자체가
section/para/control/cell_context 를 매개변수로 받지 않는다. "이 이미지의 독립된 문서 위치"라는
질문 자체가 성립하지 않는 카테고리 — 구조적으로 host 가 없는 유일한 잔여였다.

## 구현

- `src/renderer/render_tree.rs`: `DocPath = Vec<u32>` + `doc_path_for_node(&RenderNode) ->
  Option<DocPath>`. Table/Image 는 `cell_context`(다단계 경로)를, Rectangle/Line/Ellipse/Path/
  Equation 은 기존 단일 레벨(`cell_index`/`cell_para_index`/`outer_table_control_index`, Task
  #1138/#1151 패턴)을 반영. `TableNode` 에 `cell_context: Option<CellContext>` 필드 신설.
- `src/renderer/layout.rs`: `paper_node_sort_key` 반환 타입 `(u8,i32,u32)` → `(u8,i32,DocPath)`,
  `unwrap_or((0, node.id))` 폴백 제거. `sort_paper_render_nodes`(내부 페인트 정렬)와
  `collect_controls`(TS 노출) 둘 다 같은 함수를 거치므로 자동으로 함께 갱신됐다 —
  `task1197_paper_nodes_sort_by_plane_z_order_and_stable_index` 로 내부 정렬 의미가 안 깨졌음을
  확인해 분리하지 않았다. `build_master_page` 의 Picture 분기 pi/ci 스레딩(결손 2),
  `CellContext::nested_table_meta` 신설(결손 3).
- `table_layout.rs`/`table_partial.rs`/`table_cell_content.rs`: 결손 1·3 수정.
- `src/document_core/queries/rendering.rs`: `collect_controls` 가 `"stableIndex"` 를 JSON
  **배열**로 방출. 필드명은 유지했다 — 같은 저장소 안 Rust↔TS 전용 계약이고 소비자가
  `input-handler-picture.ts` 하나뿐이라 하위호환 층이 필요 없다.
- TS(`rhwp-studio/src/engine/input-handler-picture.ts`, `core/types.ts`): `controlTopKey`/
  `isAboveControl` 을 새 `compareLexArrays`(배열 사전식 비교)로. `ControlLayoutItem.stableIndex`
  를 `number[]` 로. `LayerInfo.stableIndex: number` 는 그대로 뒀다(별개 계약, 아래 참고).

## 새 정렬 계약의 성질

**전순서인가.** `Vec<u32>` 의 `Ord` 는 어떤 두 값에도 정의되는 전순서다. 따라서 `(u8, i32, DocPath)`
비교는 `paper_node_sort_key` 에 닿는 모든 노드 타입에 대해 총체적(total)이다 — 비교 불능 쌍은 없다.

**조상이 자손의 접두사일 때.** 경로 길이는 항상 `3k+3`(`[section, para]` + 셀 3원소 × k +
`control`)이라, 한 경로가 다른 경로의 진접두사가 되는 경우는 **한쪽이 다른 쪽을 담고 있을 때**
뿐이다: 표 자신이 `[0,0,2]`, 그 셀 안 개체가 `[0,0,2,5,1,0]`. 사전식 비교는 짧은 쪽(조상)을 작다고
보고, 정렬키에서 "작다"는 "먼저 그린다 = 아래"다. 담는 표가 담긴 개체보다 아래인 것이 렌더
순서상 옳으므로 **의도한 관계**다.

**동률(tie).** 순서 자체는 전순서지만 서로 다른 노드가 같은 경로를 받을 수는 있다. 두 경우다.

1. `doc_path_for_node` 가 `None` 인 노드 — 호출부가 빈 경로로 폴백하므로 서로 전부 동률이다.
   빈 배열은 사전식 최솟값이라 같은 plane/zOrder 안에서 **항상 맨 아래**다.
2. `doc_path_single_cell_level` 을 쓰는 타입(Rectangle/Line/Ellipse/Path/Equation)의 2중 이상
   중첩 — 그 필드들이 애초에 단일 레벨 근사라 바깥 레벨이 구분되지 않는다(#4334 이전부터의 근사).

두 경우 모두 `sort_paper_render_nodes` 의 `sort_by_key` 가 std 안정 정렬이므로 동률 노드는 삽입
순서를 그대로 유지한다. 즉 "무엇이 이들을 정렬하는가"의 답은 **아무것도 정렬하지 않고 삽입 순서가
남는다**이다.

**24개 잔여 Image 의 실제 영향.** 이들은 `render_cell_background` 가 `cell_node.children` 에 직접
붙이는 셀 장식이라 `paper_images`(페이지 레벨 out-of-flow 목록)에 들어가지 않는다 —
`sort_paper_render_nodes` 는 이들을 보지 않는다. 이들이 나가는 경로는 `collect_controls` → TS
히트테스트 하나뿐이고, TS `isAboveControl` 은 동률에서 `false` 를 돌려주므로 먼저 emit 된
(DFS 순서) 쪽이 최상단으로 남는다. 종전에는 `node.id` 가 이들에게 전순서를 줬으므로 이 부분이
행동 변화다. 다만 (a) 서로 다른 셀의 배경은 공간적으로 겹치지 않고, (b) 빈 경로는 사전식 최솟값
이라 같은 plane/zOrder 의 위치 있는 컨트롤에게 **항상 진다** — "셀 배경은 셀 내용보다 아래"라는
올바른 답이므로, 종전 `node.id` 순서보다 오히려 정확해진다.

## 재앵커한 통합 테스트 2개

`cargo test --lib` 만으로는 안 보이는 실패였다 — 둘 다 `tests/` 통합 테스트, `--profile
release-test --tests` 로 처음 발견했다.

- `tests/issue_1486_hwpx_partial_tac_table.rs`(`collect_issue_1486_tables`) — 원래
  `para_index.is_none() && control_index.is_none()` 로 "9쪽 상단 TAC 중첩 표"를 찾았는데, 결손 1
  수정으로 그 인덱스가 채워지자 후보 0개로 실패했다. 실패 자체가 수정이 실제로 이 표에 적용됐다는
  증거였다. 모델 레벨(`document_core::DocumentCore`) 추적으로 실제 값을 확인해
  `section_index==Some(0) && para_index==Some(21) && control_index==Some(0)`(section 0 →
  paragraph 74 의 1×1 래퍼 표 → 그 셀 안 29개 문단 중 21번째가 담은 3행 2열 중첩 표)를 기존 기하
  조건에 **추가**했다 — 기하 조건은 지우지 않았다. #1486 본 검증(TAC 중첩 표가 본문 좌/우 경계를
  벗어나지 않는가)은 같은 표를 찾아 같은 두 bbox 단언을 그대로 돌리므로 무손상.
- `tests/issue_rowbreak_chart_overlap.rs`(`first_nested_table_bbox`) — 같은 결함 패턴, HWP/HWPX 두
  변형 테스트가 함께 실패했다. 이번엔 특정 인덱스를 하드코딩하지 않고 `cell_context.is_some()` 로
  바꿨다 — 이 함수는 HWP/HWPX 양쪽에서 재사용되는데 같은 의미 내용도 포맷별 내부 문단 인덱스가
  다를 수 있어 값 고정이 부적절했기 때문이다. `cell_context.is_some()` 이 원래 `is_none()` 이
  대신 쓰던 신호("이 표가 중첩돼 있는가?")의 진짜 의미다.

전수 조사(`grep -rn '\.para_index\.is_none()\|\.control_index\.is_none()\|\.section_index\.
is_none()\|\.cell_context\.is_none()' tests/ src/`)로 같은 패턴이 더 있는지 저장소 전체를 훑었다 —
위 두 곳 외 나머지는 전부 `TextRunNode.cell_context`/`ImageNode.cell_context` 를 "본문(비-셀)
run/image 만" 필터링하는 기존 프로덕션 로직이었다(#4334 가 손대지 않은 필드라 무관).

## 안 한 것 — 별도 이슈로 분리 예정

- **159곳 `next_id()` 치환.** `RenderNode.id`/`NodeId` 자체도 손대지 않았다.
- **`RenderLayerInfo.stable_index` 필드 자체** — `paper_node_sort_key` 는 이제 이 값을 안 읽지만
  필드는 남겨뒀다. 다른 소비자가 둘 있다: `src/renderer/svg.rs` 의 `node_z_sort_key`(SVG 렌더 시
  자식 정렬용, 별개의 세 번째 sort-key 구현 — `layer=None` 이면 0 폴백, `node.id` 는 안 읽으므로
  이번 이슈의 결함과 무관하지만 여전히 패킹된 u32 를 쓴다. 이제 `paper_node_sort_key` 와 세 번째
  키의 **의미가 갈라졌다** — 두 정렬을 하나로 합치는 건 후속 단계다), `src/paint/json.rs`
  (`RenderLayerInfo` 자체의 JSON 직렬화, `paint/layer_tree.rs` 의 페인트 트리 계약 —
  `get_page_control_layout_native` 의 controls JSON 과는 다른 별개 출력).
- **`render_cell_background` 의 24개 잔여 Image** — 문서 경로 매개변수를 새로 뚫으려면 호출부
  전체 개정이 필요해 범위 밖으로 남겼다. `doc_path_for_node` 는 이 경우 빈 경로로 결정적으로
  폴백한다(node.id 는 안 읽음). 영향은 위 "새 정렬 계약의 성질" 참고.
- **`doc_path_for_node` 가 다루지 않는 `TextLine`** — 종이 레벨(`paper_images`)로 올라오는
  `RenderNodeType::TextLine` 이 실제로 있다(아래 시각 증적의 `samples/aift.hwp` 페이지 24).
  이 타입은 `doc_path_for_node` 의 match 어느 팔에도 없어 빈 경로를 받고, 같은 plane·zOrder
  안에서 항상 맨 아래로 간다. 이번 라운드에서 관측된 유일한 정렬 변화의 원인이며 그 사례에서는
  시각 영향이 없었다(겹치지 않음). `TextLineNode` 는 `section_index`/`para_index` 는 갖고 있으나
  `control_index` 가 없다(줄은 컨트롤이 아니다) — `[section, para]` 2원소 경로를 주는 설계가
  자연스럽지만, 그러면 같은 문단의 컨트롤(`[section, para, control]`)보다 항상 앞(=아래)이라는
  **새 정렬 규칙**이 생긴다. 의미 결정이 필요한 별도 작업이라 여기서는 손대지 않았다.

## 검증 게이트 — 실측

기준 base `upstream/devel` (rebase 후 `git rev-list --count HEAD..upstream/devel == 0`).

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, warning 0 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | exit 0 · FAILED 0 · `test result: ok` 블록 519개 · 통과 5,684개 |
| Native Skia `--features native-skia skia --lib` | exit 0 · 58 passed / 0 failed |
| Native Skia `--test issue_2225_missing_picture_placeholder` | exit 0 · 2 passed / 0 failed |
| Native Skia `--test render_p37_direct_pdf_export` | exit 0 · 4 passed / 0 failed |
| `wasm-pack build --target web --out-dir pkg` | exit 0 |
| rhwp-studio `npx tsc --noEmit` | exit 0 · 에러 0 |
| rhwp-studio `npm test` | exit 0 · tests 836 · pass 835 · fail 0 · skip 1 |

#4334 전용 테스트 4개는 모두 통과했다.

- `renderer::layout::tests::issue_4334_paper_node_sort_key_no_longer_depends_on_node_id`
- `renderer::layout::tests::task1197_paper_nodes_sort_by_plane_z_order_and_stable_index`
- `renderer::layout::integration_tests::tests::issue_4334_stage1_textbox_under_image_top_object_pin`
- `renderer::layout::integration_tests::tests::issue_4334_stage3_document_position_coverage_precheck`

`issue_4334_stage3_document_position_coverage_precheck` 를 `--nocapture` 로 다시 돌려 얻은 실측값
(`cargo test --profile release-test --lib issue_4334 -- --nocapture`, 10개 fixture · 문서당 최대
40페이지):

```
issue_4334_stage3: pages=233 paper(layer=Some/None)=44/56 controls(total/missing_doc_pos/with_cell)=1680/24/391
```

- `controls total=1680` — TS 로 나가는 Table/Equation/Image
- `missing_doc_pos=24` — 문서 위치를 못 만드는 노드(전부 `render_cell_background` 장식 Image).
  세 플러밍 결손을 고치기 전 값은 42였다.
- `with_cell=391` (23%) — 셀 안에 있는 노드. 셀 축이 필요하다는 근거.
- `paper(layer=Some/None)=44/56` — `sort_paper_render_nodes` 대상 중 layer 유무 비율(참고값,
  이번 라운드가 손대지 않은 축).

## 시각 증적

`upstream/devel`(8ea92cdad, "before")과 이 브랜치 HEAD("after")에서 각각
`cargo build --profile release-test --features native-skia --bin rhwp` 로 바이너리를 만들어 같은
문서를 렌더하고 비교했다. 산출물은 CONTRIBUTING 규약대로 저장소에 커밋하지 않고 저장소 밖에 둔다:
`~/Desktop/rhwp_4334_visual/{before,after}/`.

### 전수 대조 — 10개 문서 738페이지, 차이 1페이지

`export-svg` 로 전 페이지를 뽑아 `diff -rq` 로 대조했다.

| 문서 | 페이지 | 다른 파일 |
|---|---|---|
| `samples/textbox-under-image.hwp` | 1 | 0 |
| `samples/issue2006/1790387_prep_final_report.hwpx` | 143 | 0 |
| `samples/issue1921/59043_regulatory_analysis.hwp` | 37 | 0 |
| `samples/20250130-hongbo.hwp` | 4 | 0 |
| `samples/aift.hwp` | 74 | **1** (`aift_025.svg`) |
| `samples/21_언어_기출_편집가능본.hwp` | 15 | 0 |
| `samples/task2093/1192000_hydrogen_policy_research.hwp` | 16 | 0 |
| `samples/2025 행정업무운영 편람(최종).hwp` | 393 | 0 |
| `samples/3-09월_교육_통합_2023.hwpx` | 20 | 0 |
| `samples/2022년 국립국어원 업무계획.hwp` | 35 | 0 |
| **합계** | **738** | **1** |

### 유일한 차이 — `samples/aift.hwp` 페이지 24 (0-based)

`export-render-tree -p 24` 로 페이지 루트 자식 순서를 비교하면 원인이 그대로 보인다.

```
before: [PageBg, Header, Body, Group(y=152.4, 이미지 2개), TextLine(y=522.5), Footer]
after:  [PageBg, Header, Body, TextLine(y=522.5), Group(y=152.4, 이미지 2개), Footer]
```

`paper_images` 에 out-of-flow `Group`(wrap=Square, 이미지 2개) 과 종이 레벨로 올라온 `TextLine`
둘이 들어 있고 **plane·zOrder 가 같아 세 번째 키가 결정한다**. 종전에는 `node.id` — out-of-flow
개체가 별도 패스에서 먼저 id 를 받으므로 Group 이 앞이었다. 지금은 `TextLine` 이
`doc_path_for_node` 미지원 타입이라 빈 경로(사전식 최솟값)를 받아 먼저 가고, 문서 위치가 있는
Group 이 뒤(=위)로 간다. 이 문서에서 관측된 "빈 경로는 항상 맨 아래" 규칙의 실사례다.

plane 과 zOrder 는 손대지 않았으므로 BehindText↔InFrontOfText 가 뒤집히는 일은 구조적으로
불가능하다 — 바뀔 수 있는 건 같은 plane·zOrder 안의 순서뿐이다.

### 래스터 결과 — 변화 없음

SVG 요소 순서는 바뀌었지만 **픽셀은 동일**하다. 옮겨간 두 이미지의 bbox 는 y 152.4~511.1 이고,
이들이 건너뛴 30개 `<text>` 는 전부 y=536.07 한 줄이라 공간적으로 겹치지 않는다.

```
$ md5 -q before/aift_p24.png after/aift_p24.png
f6518ca21f4cdf1b2490d54bdf24f920
f6518ca21f4cdf1b2490d54bdf24f920
```

`samples/textbox-under-image.hwp`(글상자가 이미지 위에 겹치는 한컴 권위 샘플)의 PNG 도 40,853
바이트로 before/after 동일하다. 이 fixture 의 tie-break 결과 자체는
`issue_4334_stage1_textbox_under_image_top_object_pin` 이 `stableIndex` 값까지(`글상자 [0,0,2]` >
`이미지 [0,0,3]` 이 아니라 plane 3 > 2 로 결정) Rust 쪽에서 고정한다.

**결론: 겹침 개체가 있는 738페이지에서 시각 회귀 0건, 정렬 순서 변화 1건이며 그 1건은 이 이슈가
의도한 방향(카운터 → 문서 위치)의 변화다.**


## 후속 이슈 (2026-08-11)

이 작업에서 발견했지만 범위 밖이라 손대지 않은 것을 전부 이슈로 분리했다.

- **[#4521](https://github.com/edwardkim/rhwp/issues/4521)** — `doc_path_for_node` 에
  `TextLine` arm 이 없어 paper 층위 줄이 빈 경로를 받고 언제나 가라앉는다. 738쪽 중
  유일하게 달라진 `samples/aift.hwp` 24쪽의 원인이다(래스터는 동일). 2원소 경로를 주면
  같은 문단 컨트롤 경로의 진접두사가 되어 "줄은 컨트롤 아래"라는 새 의미 규칙이 생기므로
  배관이 아니라 결정이 필요하다. `walk_controls` 래칫이 쓰는 근사 판정도 함께 걸었다.
- **[#4522](https://github.com/edwardkim/rhwp/issues/4522)** — `svg.rs:253`
  `node_z_sort_key` 가 세 번째 정렬 키 구현으로 남아 있고, 이번 변경으로 `paper_node_sort_key`
  와 **의미가 갈라졌다**(패킹 u32 대 문서 경로). 종전에는 중복이되 일치했다.
- **[#4523](https://github.com/edwardkim/rhwp/issues/4523)** — `doc_path_single_cell_level`
  이 같은 타입 `Option<usize>` 6개를 받고(호출부 5곳은 전부 올바름을 확인),
  `unwrap_or(0)` 근사가 이중 중첩 Rectangle/Line/Ellipse/Path/Equation 의 경로를 뭉갤 수
  있다. `node.id` 폴백을 없앴으므로 그 경우 동률을 깨는 것이 없다.
- **[#4524](https://github.com/edwardkim/rhwp/issues/4524)** — `controlTopKey(ctrl: any)`
  가 `stableIndex: number[]` 계약을 무력화한다. 오늘 두 계약이 섞이는 경로는 없지만,
  섞이면 던지지 않고 조용히 전부 동률이 된다.
