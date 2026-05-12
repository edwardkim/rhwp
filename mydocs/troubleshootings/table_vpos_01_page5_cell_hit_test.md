# table-vpos-01.hwp 5쪽 인라인 표 셀 클릭 진입 불가 — 진단 노트

> 본 노트는 `mydocs/plans/...` 수행 계획서 승인 후 Phase 2 진단 결과를 기록한 임시 메모.
> 정식 구현 계획서/스테이지 보고서는 별도. 이슈 채번 전이라 파일명에 이슈 번호 없음.
> 작성일: 2026-05-12
>
> **Issue #850 과의 관계 — 별개 이슈** (2026-05-12 pre-#717 commit 1c783a89 직접 검증 결과):
> - [Issue #850](https://github.com/edwardkim/rhwp/issues/850): v0.7.11 회귀 (Task #717 commit ef67efa1 직접 원인). 회귀 라인 [cursor_rect.rs:391-403](../../src/document_core/queries/cursor_rect.rs#L391-L403) (`_ => current_table_meta` → `_ => None`).
> - 본 사례: pre-#717 (commit 1c783a89) 에서도 c=2 column 4개 케이스 동일 FAIL. **장기간 잠재 결함**. 회귀 라인 [cursor_rect.rs:648-666](../../src/document_core/queries/cursor_rect.rs#L648-L666) (v0.5.0 부터 불변).
> - 두 이슈는 cellPath 길이 1 증상은 공유하지만 **메커니즘과 발생 시점이 다름**. 별개 fix 필요.
> - 부가 발견: Task #717 이 c=0 column 라벨 셀의 별개 회귀(잘못된 pi=30 으로 misroute) 는 fix 함. c=2 column 의 first-match 버그는 손대지 못함.

## 1. 재현 환경

- 파일: [samples/table-vpos-01.hwp](../../samples/table-vpos-01.hwp)
- 페이지: 5쪽 (global_idx=4, section=0)
- 증상 (사용자 확인): 5쪽 3개 표 전부 셀 안에 텍스트 커서가 안 들어감
- 한컴 정품: 정상 동작
- 본 조사: HWP5 경로(.hwp) 전용. HWPX 무관.

## 2. dump-pages / dump 결과 (확정)

페이지 5는 다음 6개 PageItem으로 구성됨:

```
=== 페이지 5 (global_idx=4, section=0, page_num=5) ===
  body_area: x=75.6 y=94.5 w=642.5 h=933.5
  단 0 (items=6, used=920.4px)
    Table          pi=30 ci=1  1x2  638.8x37.8px  wrap=TopAndBottom tac=true  vpos=0
    FullParagraph  pi=31  h=1.3                                       vpos=3284
    Table          pi=32 ci=0  1x1  638.8x53.9px  wrap=TopAndBottom tac=true  vpos=3444
    FullParagraph  pi=33  h=30.8                                      vpos=7932
    Shape          pi=33 ci=0  wrap=TopAndBottom tac=true             vpos=7932
    Table          pi=34 ci=0  1x1  638.8x778.8px wrap=TopAndBottom tac=true  vpos=10405
```

**핵심**: pi=30 host 문단의 `PageItem::FullParagraph` 가 **없음**. pi=32, pi=34도 동일(표 PageItem만 발행). pi=31, pi=33은 표 사이를 잇는 빈/Shape host 문단.

pi=30 조판부호:
```
--- 문단 0.30 --- cc=17, text_len=0, controls=2 [쪽나누기]
  ls[0]: ts=0, vpos=0, lh=2832, tag=0x00060000
  [0] 감추기: header=true, footer=false, ...
  [1] 표: 1행×2열, 셀=2, 쪽나눔=RowBreak (attr=0x00000006), ...
       treat_as_char=true, wrap=위아래, vert=문단(0=0.0mm), horz=문단(0=0.0mm)
```

- 페이지나눔은 **문단 레벨 플래그** (`[쪽나누기]`)로 표시. 별도 control 아님.
- ci=0 = 감추기 (머리말 숨김), ci=1 = 1x2 표.

## 3. 페이지 4 (정상 동작) 비교

```
=== 페이지 4 ===
    FullParagraph  pi=24 ...
    FullParagraph  pi=25 ...
    FullParagraph  pi=26 ...
    FullParagraph  pi=27 ...
    Table          pi=28 ci=0  4x6 ... tac=true  vpos=38580
    Table          pi=29 ci=0  1x3 ... tac=true  vpos=44406
```

pi=28/29 조판부호:
- `cc=9, text_len=0, controls=1` (표 단독)
- tac=true, wrap=위아래
- pi=28: 4x6, pi=29: 1x3 (그림 포함)

**관찰**: pi=28/29와 pi=32/34는 **문단 구조가 동일**. 둘 다 host paragraph가 `FullParagraph` PageItem으로 발행되지 않음. 그런데 pi=28/29는 정상 동작하고 pi=32/34는 실패한다고 사용자가 주장 → **단순히 "host PageItem 없음"만으로는 원인 설명 불가**.

페이지 4와 페이지 5의 결정적 차이:
- 페이지 5는 pi=30의 **`[쪽나누기]` 문단 플래그**로 시작 (강제 페이지 진입).
- 페이지 5의 첫 항목이 **vpos=0** (body 최상단)에서 표.
- 페이지 5는 같은 페이지의 **모든** 표 PageItem이 host FullParagraph 없이 등장 (pi=30/32/34 모두). 페이지 4는 pi=24~27 본문 텍스트가 PageItem으로 등장한 뒤 pi=28/29 표 PageItem만 등장 — 같은 페이지 안에 본문 TextRun 다수 존재.

## 4. SVG 디버그 오버레이 — 셀 bbox 좌표 (RED 테스트 입력 후보)

`cargo run -- export-svg samples/table-vpos-01.hwp -p 4 --debug-overlay` 산출:

| 라벨 | cell-clip id | x | y | w | h | 중심 (x, y) |
|---|---|---|---|---|---|---|
| pi=30 cell[0] "참고" | 5 | 75.6 | 94.5 | 76.2 | 37.8 | 113.7, 113.4 |
| pi=30 cell[1] "정부혁신…" | 9 | 151.7 | 94.5 | 562.6 | 37.8 | 433.0, 113.4 |
| pi=32 cell[0] "국민이…" | 20 | 77.4 | 137.0 | 638.8 | 53.9 | 396.8, 164.0 |
| pi=34 outer cell[0] (빈 여백 상단) | 33 | 77.4 | 229.9 | 638.8 | 782.6 | 396.8, 250.0 |
| pi=34 내부 11x3 r=0 c=2 "국민 주도…" | 61 | 177.6 | 298.0 | 529.9 | 45.1 | 442.5, 320.5 |

## 5. 코드 분석 — Table RenderNode 메타 전파 경로

### TableNode 메타 세팅 ([src/renderer/layout/table_layout.rs:354-360](../../src/renderer/layout/table_layout.rs#L354-L360))

```rust
RenderNodeType::Table(TableNode {
    ...
    section_index: Some(section_index),          // 항상 Some (인자)
    para_index: table_meta.map(|(pi, _)| pi),    // table_meta 가 Some 일 때만 Some
    control_index: table_meta.map(|(_, ci)| ci), // table_meta 가 Some 일 때만 Some
    ...
})
```

→ `table_meta: Option<(usize, usize)>` 가 **Some**으로 전달되어야 meta가 채워진다.

### 호출 측 ([src/renderer/layout.rs:2453-2473](../../src/renderer/layout.rs#L2453-L2473))

`layout_table_item` 의 `renders_outside_body` 분기/일반 분기 모두 `Some((para_index, control_index))` 로 호출:

```rust
let _table_y_end = self.layout_table(
    tree, &mut tmp_node, t,
    page_content.section_index, ...
    Some((para_index, control_index)),
    ...
);
```

→ pi=30/32/34 의 Table RenderNode 메타는 **정상 채워질 것으로 예상** (실측 필요).

### hit_test_native 의 메타 활용 ([src/document_core/queries/cursor_rect.rs:396-426](../../src/document_core/queries/cursor_rect.rs#L396-L426))

```rust
let table_meta = if let RenderNodeType::Table(ref tn) = node.node_type {
    match (tn.section_index, tn.para_index, tn.control_index) {
        (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
        _ => None,
    }
} else { current_table_meta };

if let RenderNodeType::TableCell(ref tc) = node.node_type {
    let (si, ppi, ci, has_meta) = table_meta
        .map(|(si, ppi, ci)| (si, ppi, ci, true))
        .unwrap_or((0, 0, 0, false));
    cell_bboxes.push(CellBboxInfo { ... has_meta, ... });
}
```

→ section/para/control 모두 Some이면 `has_meta=true`. L673의 셀 필터 통과.

### TAC inline 표 set_inline_shape_position 누락 ([src/renderer/layout.rs:2393-2421](../../src/renderer/layout.rs#L2393-L2421))

`layout_table_item` 의 TAC 분기에서 inline_shape_positions 미세팅 상태(host FullParagraph 없음)인 경우 x 좌표를 수동으로 계산하지만 **`set_inline_shape_position` 호출은 없음**:

```rust
} else if is_tac {
    // TAC 문단에 PageItem::FullParagraph 가 발행되지 않아
    // paragraph_layout 가 호출되지 않는 케이스(선행 공백만 있는 TAC 표 등):
    // composed.lines[0] 의 runs 에서 TAC 이전 텍스트 폭을 직접
    // 합산해 표 x 좌표에 반영한다. inline_shape_position 미세팅 상태에서
    // 기본값 col_area.x(body_left) 으로 붕괴되는 현상 방지.
    let leading = composed.get(para_index)
        .map(|c| compute_tac_leading_width(c, control_index, styles))
        .unwrap_or(0.0);
    let base_x = col_area.x + effective_margin + leading;
    ...
    Some(aligned_x)
}
```

코드 작성자가 이미 "FullParagraph 없는 TAC 표" 케이스를 명시적으로 인지하고 있고, x 좌표는 보강했지만 inline_shape_positions 등록은 보강하지 않음.

## 6. 현 시점 가설 재정렬

### 가설 B (Table RenderNode 메타 결손) — **유력성 하향**
- 코드 분석상 `layout_table_item` 두 분기 모두 `table_meta=Some(...)` 전달. 결손 가능성 낮음.
- 단, 1x1 wrapper unwrap 분기 ([table_layout.rs:151-207](../../src/renderer/layout/table_layout.rs#L151-L207))는 본 사례에 적용 안 됨 (pi=30 1x2, pi=32 text 있음, pi=34 paragraphs=2).

### 가설 A (inline_shape_positions 미등록) — **이론상 사실이나 hit-test 셀 경로(L671)가 대체로 매칭하면 무관**
- L2421 코드는 set 하지 않는다. 이 자체는 hit-test 인라인-Shape 경로(L592-641) 매칭 실패를 야기.
- 하지만 셀 bbox 매칭 경로(L671-762)가 `has_meta=true` 인 셀 후보를 가지면 셀로 진입 가능 → 인라인-Shape 경로 실패만으로는 셀 진입 차단 안 됨.
- 단, **bbox 후보가 클릭 위치를 포함하지 않으면** 셀 경로도 실패.

### 가설 G (신규) — **셀 bbox 좌표 좌표 자체가 클릭 위치를 포함하지 않을 가능성**
- TAC inline 표의 inline x 좌표 (L2393-2421 의 `aligned_x`) 와 실제 Table RenderNode bbox 좌표가 서로 다른 좌표계로 등록되어 hit-test 가 클릭 좌표를 표 bbox 안으로 인식하지 못할 수 있음.
- SVG 셀 bbox 좌표는 정상이지만 (cell-clip 정확), 그것이 **TableCell 노드 bbox**이고 hit-test 가 사용하는 좌표인지는 별도 검증 필요.

### 가설 H (신규) — **pi=30 의 `[쪽나누기]` 문단 플래그가 표 RenderNode 부모 노드 분기를 다르게 만들 가능성**
- 페이지 첫 paragraph 가 페이지나눔 플래그 + 감추기 control + TAC 표 ci=1 인 매우 특수 케이스.
- pagination 단계 또는 build_page_tree 단계에서 본 paragraph 의 표 control 이 별도 처리 경로를 타고 Table RenderNode 가 별도 부모(예: 페이지나눔 핸들러)에 부착되어 통상 page-tree-walk 에서 누락될 가능성.
- pi=32/34 는 동일 페이지에 있어 페이지 build 자체의 어떤 상태가 망가지면 같이 영향 받을 수 있음.

### 결론 (현 시점)
**가설 G/H 가 가장 유력**. 결정적 진단은 `hit_test_native` 의 실측 출력으로만 가능.

## 7. RED 회귀 테스트 실행 결과 (2026-05-12)

`tests/issue_table_vpos_01_page5_cell_hit_test.rs` 작성·실행:

```
running 5 tests
test page5_header_cell0_center_enters_cell ... ok
test page5_header_cell1_center_enters_cell ... ok
test page5_title_cell_center_enters_cell ... ok
test page5_big_inner_title_cell_returns_outer_meta ... ok
test page5_big_inner_11x3_cell_returns_nested_path ... FAILED

failures:
---- page5_big_inner_11x3_cell_returns_nested_path ----
deeply nested click must include full path, got [(0, 0, 1)],
hit={"cellIndex":0,"cellParaIndex":1,
     "cellPath":[{"cellIndex":0,"cellParaIndex":1,"controlIndex":0}],
     "charOffset":0,"controlIndex":0,
     "cursorRect":{"height":712.5,"pageIndex":4,"x":396.9,"y":298.0},
     "paragraphIndex":1,"parentParaIndex":34,"sectionIndex":0}
```

### 7.1 의미
- **4/5 PASS**: pi=30 header 두 셀, pi=32 title 셀, pi=34 외곽 1x1 안의 inner 1x1 title 셀 — 모두 hit-test 반환값이 정상 (cellPath 충분).
- **1/5 FAIL**: pi=34 외곽 1x1 안의 **inner 11x3 r=0,c=2 셀** 텍스트 영역 클릭 시 hit-test 가 **외곽 1x1 셀**(cellIndex=0, cellParaIndex=1)에서 멈춤. cellPath 길이가 1뿐이며 inner 11x3 entry 가 누락.

### 7.2 사용자 인식과의 차이
- 사용자는 "5쪽 3개 표 전부 클릭 안됨" 주장. 실측 결과 hit-test 단에서는 pi=30/32/34 외곽까지 정상 반환. pi=34 inner 11x3 만 path 누락.
- 사용자가 본 증상은 pi=34 inner 11x3 가 페이지 5의 시각적 대부분을 차지하므로 (cell-clip 영역 y=298~1011, 약 713px) "큰 표" 안 클릭은 모두 inner 11x3 영역. 결과적으로 외곽 셀에 커서가 들어가 inner 11x3 의 어느 셀도 편집 불가.
- pi=30/32 가 클릭 안된다는 인식은 시각적 추정일 가능성. **추가 사용자 확인 필요**.

### 7.3 ROOT CAUSE 확정 (디버그 출력 직접 캡처, 2026-05-12)

**임시 디버그 로그**(cursor_rect.rs 에 일시적으로 `eprintln!` 추가 후 revert) 결과:

```
[hit_test_native page=4 (442.5, 320.5)] all_cell_hits=[13, 17] hit_cell=Some((13, 0)) hit_body=None
  cell_hit[13] tid=Some(32) si=0 pi=1 cs=0 cc=0  bbox=(396.9, 298.0, 625.2 × 712.5)  ctx=[(0, 0, 1)]
  cell_hit[17] tid=Some(51) si=0 pi=0 cs=0 cc=19 bbox=(184.4, 310.5, 330.0 × 20.0)   ctx=[(0, 0, 1), (0, 2, 0)]
```

**해석**:
- runs[13]: 외곽 pi=34 셀 paragraph 1의 **빈 placeholder TextRun**.
  - char_count=0 (no text), bbox 높이 712.5 px = 외곽 셀 paragraph 1 영역 전체 (= 내부 11x3 표 host 영역).
  - cell_context path 길이 1 (외곽 entry 만).
  - table_id = 32 (외곽 표 RenderNode id).
- runs[17]: 내부 11x3 표 r=0,c=2 셀의 **실제 텍스트 TextRun** ("국민 주도 참여‧소통 거버넌스 구현").
  - bbox 정확히 텍스트 폭 (330×20).
  - cell_context path 길이 2 (외곽 + 내부 entry).
  - table_id = 51 (내부 11x3 RenderNode id, 외곽과 다름 — 정상).

**버그**: [cursor_rect.rs:648-666](../../src/document_core/queries/cursor_rect.rs#L648-L666) 의 "1. 정확한 bbox 히트 검사" 분기가 **첫 번째 매칭되는 cell-context TextRun**(runs[13]) 을 골라 early-return. 더 깊은(path 깊이 2, 작은 bbox) 내부 TextRun(runs[17]) 은 무시.

```rust
// 현 코드 — first-match 선택
for (i, run) in runs.iter().enumerate() {
    if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
        && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
    {
        ...
        if run.cell_context.is_some() {
            if hit_cell.is_none() {                  // ← 첫 매칭만 선택
                hit_cell = Some((i, run.char_start + char_offset));
            }
        }
        ...
    }
}
```

트리 순회는 depth-first 이므로 외곽 셀의 placeholder TextRun이 내부 셀의 텍스트 TextRun보다 먼저 매칭 → 외곽이 선택됨.

### 7.3.1 왜 다른 셀(pi=30/32, pi=34 외곽, inner 1x1 title, inner 11x3 r=0,c=0) 은 정상?

| 케이스 | click x | 외곽 placeholder x_range | placeholder 매칭? | 결과 |
|---|---|---|---|---|
| pi=30 header c0 (113.7, 113.4) | 113.7 | (pi=30 placeholder 별도 영역) | × | PASS |
| pi=30 header c1 (433.0, 113.4) | 433.0 | (pi=30 placeholder 별도 영역) | × | PASS |
| pi=32 title (396.8, 164.0) | 396.8 | (pi=32 placeholder 별도 영역) | × | PASS |
| pi=34 inner 1x1 title (396.8, 260.6) | 396.8 | (외곽 cell p[0] placeholder x_range 와 y 불일치) | × | PASS |
| pi=34 inner 11x3 r=0,c=0 (128, 380) | 128 | [396.9, 1022.1] (외곽 placeholder x_min=396.9) | × (x 밖) | PASS |
| **pi=34 inner 11x3 r=0,c=2 (442.5, 320.5)** | **442.5** | **[396.9, 1022.1]** | ○ | **FAIL** |
| **pi=34 inner 11x3 r=1,c=2 (442.5, 403)** | **442.5** | **[396.9, 1022.1]** | ○ | **FAIL** |

즉 click x 좌표가 외곽 placeholder의 x_range 안에 들어가는 inner cell click 만 본 버그가 발현. column c=2 (가장 오른쪽 큰 열) 의 모든 행이 영향. column c=0 (좌측 라벨 열) 은 placeholder x_range 밖이라 정상.

### 7.4 Task #717 와의 관계
- Task #717 (commit ef67efa1, 2026-05-09) 은 `cell_bboxes` 의 has_meta 보완 패스([cursor_rect.rs:520-546](../../src/document_core/queries/cursor_rect.rs#L520-L546)) 수정. 본 버그는 그 보완 패스에 도달하기 전 단계 (L643-666 의 first-match) 에서 발생 → Task #717 의 fix 가 본 케이스를 커버하지 못함.

### 7.4 Task #717 와의 관계
- Task #717 (commit ef67efa1, 2026-05-09) 의 회귀 테스트 [tests/issue_717_table_cell_hit_test.rs](../../tests/issue_717_table_cell_hit_test.rs) 는 `samples/exam_social.hwp` 의 1x1 중첩 셀 케이스 (cellParaIndex=0). cellPath 길이 2 정상 반환 확인.
- 본 사례는 1x1 (outer) → 11x3 (inner) 중첩이며 inner table 이 외곽 cell 의 **paragraphIndex=1** (두 번째 문단) 에 위치. 차이점:
  - cellParaIndex=0 vs cellParaIndex=1
  - 외곽 cell.paragraphs.len()=2 (Task #688 의 1x1 unwrap 가드 발동 안 함)
- 즉 Task #717 의 fix 가 cellParaIndex>0 또는 paragraphs.len()>1 인 케이스를 커버하지 못한 잔여 회귀 가능성.

### 7.5 수정 방향 (구현 계획서에서 확정)

핵심 수정 위치: [cursor_rect.rs:648-666](../../src/document_core/queries/cursor_rect.rs#L648-L666). 옵션:

**옵션 1 — `cell_context.path.len()` 우선 (deepest cell wins)**
```rust
// 모든 매칭 후 path 깊이가 가장 깊은 것 선택
let cell_hits: Vec<(usize, usize, usize)> = runs.iter().enumerate()
    .filter(|(_, r)| /* bbox contains click */)
    .filter(|(_, r)| r.cell_context.is_some())
    .map(|(i, r)| {
        let depth = r.cell_context.as_ref().unwrap().path.len();
        (i, char_offset_of(r), depth)
    })
    .collect();
let hit_cell = cell_hits.into_iter().max_by_key(|(_, _, depth)| *depth);
```
- 장점: 명확한 의미 ("가장 깊이 중첩된 셀이 클릭 주체")
- 단점: 동일 깊이에서 first-match 유지 → tie-break 룰 필요

**옵션 2 — bbox 면적 최소 우선 (smallest cell wins)**
```rust
let hit_cell = runs.iter().enumerate()
    .filter(|(_, r)| /* bbox contains click + cell_context */)
    .min_by_key(|(_, r)| (r.bbox_w * r.bbox_h * 1000.0) as i64);
```
- 장점: 셀 bbox 매칭 경로(L671-675)와 일관된 selection 룰
- 단점: 면적 비교가 본 의도(중첩 깊이)에 항상 맞지는 않음 (큰 inner cell vs 작은 outer 빈 paragraph 면적이 역전될 수 있음)

**옵션 3 — char_count=0 placeholder 제외 + first-match 유지**
```rust
if run.cell_context.is_some() && run.char_count > 0 {
    if hit_cell.is_none() {
        hit_cell = Some((i, run.char_start + char_offset));
    }
}
```
- 장점: 최소 변경, 본문 placeholder만 무시
- 단점: 합법적인 빈 셀(char_count=0)에 대한 hit도 함께 무시 → 빈 셀 진입 회귀 위험. Task #717 RED 케이스(셀 빈 영역 클릭) 회귀 가능.

**옵션 4 (추천) — path depth 우선, tie-break 로 작은 bbox**
```rust
let hit_cell = runs.iter().enumerate()
    .filter(|(_, r)| /* bbox contains click + cell_context */)
    .max_by_key(|(_, r)| {
        let depth = r.cell_context.as_ref().unwrap().path.len();
        let neg_area = -((r.bbox_w * r.bbox_h * 1000.0) as i64);  // 작을수록 우선
        (depth, neg_area)
    });
```
- 가장 안정적. 본 사례 inner runs[17] (depth 2, bbox 6600) 가 outer runs[13] (depth 1, bbox 445400) 을 이긴다.

### 7.6 검증 (구현 계획서 단계)
1. 본 노트에 첨부된 RED 테스트 [tests/issue_table_vpos_01_page5_cell_hit_test.rs](../../tests/issue_table_vpos_01_page5_cell_hit_test.rs) 의 `page5_big_inner_11x3_cell_returns_nested_path` 가 PASS.
2. 기존 회귀 [tests/issue_717_table_cell_hit_test.rs](../../tests/issue_717_table_cell_hit_test.rs), `tests/issue_630.rs`, `tests/issue_nested_table_border.rs` 등 PASS 유지.
3. `cargo test` 전체 PASS.
4. rhwp-studio E2E: 페이지 5 inner 11x3 의 각 셀 클릭 시 텍스트 커서가 해당 inner 셀 안에 진입.

### 7.7 다음 액션 (사용자 승인 필요)

### 옵션 1 — RED 회귀 테스트 작성 (수행 계획서 Step 4) ← **2026-05-12 완료**
- 신규 파일: `tests/issue_table_vpos_01_page5_cell_hit_test.rs`
- 패턴: [tests/issue_717_table_cell_hit_test.rs](../../tests/issue_717_table_cell_hit_test.rs) 복제 (`load_*`, `hit_json`, `assert_table_hit`, `path_tuples`)
- 케이스 (4개) — 4절 표에서 도출한 좌표 사용:
  - `page5_header_cell0_center`: hitTest(4, 113.7, 113.4) → `parentParaIndex=Some(30)`, `controlIndex=Some(1)`
  - `page5_header_cell1_center`: hitTest(4, 433.0, 113.4) → `parentParaIndex=Some(30)`, `controlIndex=Some(1)`, `cellIndex=1`
  - `page5_title_cell_center`: hitTest(4, 396.8, 164.0) → `parentParaIndex=Some(32)`, `controlIndex=Some(0)`
  - `page5_big_outer_top_blank`: hitTest(4, 396.8, 250.0) → `parentParaIndex=Some(34)`, `controlIndex=Some(0)`
  - (선택) `page5_big_nested_center`: hitTest(4, 442.5, 320.5) → `cellPath.len()==2`
- 본 테스트가 **현 시점 FAIL** 해야 함. PASS 면 좌표/케이스 재조정.

### 옵션 2 — 디버그 출력 1회성 추가 후 cargo run 으로 dump
- `hit_test_native` 진입부에 `eprintln!("…")` 임시 삽입 → 트리 구조 / cell_bboxes 내용 출력
- 결과 확인 후 디버그 코드 제거 (커밋 안 함)

### 옵션 3 — Phase 2 종료, 가설별 추가 진단 정지하고 가설 G/H 어느 쪽을 우선 검증할지 결정 후 구현 계획서로 진행

## 8. 참조 좌표

- [src/document_core/queries/cursor_rect.rs:319-470](../../src/document_core/queries/cursor_rect.rs#L319-L470) — `hit_test_native`, `collect_runs`, meta 전파
- [src/document_core/queries/cursor_rect.rs:520-546](../../src/document_core/queries/cursor_rect.rs#L520-L546) — TextRun 기반 cell meta 보완 (Task #717)
- [src/document_core/queries/cursor_rect.rs:592-641](../../src/document_core/queries/cursor_rect.rs#L592-L641) — inline_shape_positions 매칭
- [src/document_core/queries/cursor_rect.rs:671-762](../../src/document_core/queries/cursor_rect.rs#L671-L762) — 셀 bbox 매칭
- [src/renderer/layout.rs:2212-2474](../../src/renderer/layout.rs#L2212-L2474) — PageItem::Table 처리, `layout_table_item`
- [src/renderer/layout.rs:2393-2421](../../src/renderer/layout.rs#L2393-L2421) — TAC 표 inline_shape_position 미세팅 케이스 분기 (set 누락)
- [src/renderer/layout/table_layout.rs:127-360](../../src/renderer/layout/table_layout.rs#L127-L360) — `layout_table` 시그니처, TableNode 생성
- [src/renderer/layout/paragraph_layout.rs:88-587](../../src/renderer/layout/paragraph_layout.rs#L88-L587) — `layout_inline_table_paragraph` (FullParagraph 발행 케이스에서 set 호출)
- [src/renderer/pagination/engine.rs:1032-1944](../../src/renderer/pagination/engine.rs#L1032-L1944) — PageItem::Table emit 지점들
