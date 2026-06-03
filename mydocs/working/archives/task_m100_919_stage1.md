# Task M100-919 Stage 1 정밀 진단 보고서

- 이슈: #919
- 브랜치: `local/task919`
- 일시: 2026-05-21

## 1. 진단 데이터

### 1.1 `samples/table-in-tbox.hwp` 구조 (`rhwp dump`)

페이지 1 body_area: x=75.6, y=75.6, w=642.5, h=1009.1

paragraph 0 (구역 0 첫 문단):
- controls[0]: SectionDef
- controls[1]: ColumnDef
- **controls[2]: Shape (사각형, 글상자)** — `treat_as_char=true, wrap=InFrontOfText`
  - 크기 166.3mm × 243.1mm = **628.5px × 919.1px** (96 dpi)
  - 위치: vpos=0
  - 글상자 내부 19개 paragraph (p[0]~p[18]) + 안 표 (p[2], p[6])

페이지 1 dump-pages:
```
단 0 (items=2, used=986.7px)
  FullParagraph  pi=0  h=976.3 (sb=0.0 lines=976.3 sa=0.0)
  Shape          pi=0 ci=2  wrap=InFrontOfText tac=true  vpos=0
```

### 1.2 글상자 실제 좌표 (debug-overlay SVG 추출)

- **글상자 외곽 (검정 테두리)**: x=75.6, y=75.6, w=**628.5**, h=**976.3** → right=**704.1**, bottom=**1051.8**
- 글상자 안 큰 표 (pi=6): x=79.4~700.3, y=159.7~841.3
- 글상자 안 paragraph 18 (마지막): y=**1024.1** — body_area bottom (1084.7) 안

### 1.3 hit_test_native 시나리오 (9 케이스 정밀)

| 시나리오 | 좌표 | 결과 (요약) | 분석 |
|---------|------|------------|------|
| **A** 글상자 top edge | (400, 76) | `pi=0, charOffset=1` (본문) ❌ | 경계선 미인식 |
| **B** 글상자 위쪽 외부 | (400, 50) | `pi=0, charOffset=0` (본문) | 외부 정상 |
| **G** 글상자 right edge | (703, 400) | `pi=0, charOffset=1` (본문) ❌ | 경계선 미인식 |
| **H** 글상자 아래 외부 | (400, 1100) | `pi=18, isTextBox=true, cellPath` ✅ | 글상자 안 paragraph 정상 |
| **C** 안표 셀 내부 | (100, 180) | `pi=0, charOffset=0` (본문) ❌ | 빈 셀 미인식 |
| **D** 안표 빈영역 | (400, 500) | `pi=0, charOffset=1` (본문) ❌ | 빈 영역 미인식 |
| **E** 안표 위 빈영역 | (400, 150) | `pi=0, charOffset=1` (본문) ❌ | 글상자 내부 빈 영역 미인식 |
| **F** 안표 아래 본문 | (400, 900) | `pi=0, charOffset=1` (본문) ❌ | 글상자 내부 본문 미인식 |
| **I** 글상자 좌측 외부 | (50, 400) | `pi=0, cellPath, isTextBox=true` ✅ | 글상자 안 표 셀 정상 |

## 2. 결함 본질 정확화

### 2.1 시나리오 분류

1. **글상자 안 텍스트 위 클릭** (H, I 등) → 정상 hit (`isTextBox=true + cellPath`)
2. **글상자 안 빈 영역 클릭** (B, C, D, E, F) → 본문 paragraph 0 으로 fall-through ❌
3. **글상자 외부 경계선 클릭** (A, G) → 본문 paragraph 0 으로 fall-through. studio 에서 `isShapeBorderClick` 패턴으로 처리해야 함

### 2.2 코드 갭 정확 식별

`src/document_core/queries/cursor_rect.rs` 의 `hit_test_native`:

#### Gap-1: `collect_runs` 가 `RenderNodeType::TextBox` 노드 미수집 (line 564~593)

`TableCell` 노드는 `cell_bboxes` 수집 — `clicked_cell` 선택에 사용. **`TextBox` 노드는 미수집** → 글상자 안 빈 영역 클릭 시 매칭 안 됨.

```rust
// 현재 (line 565):
if let RenderNodeType::TableCell(ref tc) = node.node_type { ... cell_bboxes.push(...); }
// 부재:
// if let RenderNodeType::TextBox = node.node_type { ... textbox_bboxes.push(...); }
```

#### Gap-2: `wasm_api.rs` 에 `getShapeBBox` API 부재

`getTableBBox(sec, ppi, ci)` 만 존재 — 글상자 대응 API 신규 필요.

#### Gap-3: rhwp-studio `input-handler-mouse.ts` 의 글상자 1차 클릭 흐름

현재 글상자 1차 클릭 → `pictureObjectSelection` (글상자 객체 선택) 으로 무조건 진입. 한컴 UX 정합:
- 외부 경계선 (tolerance 5px) 만 객체 선택
- 내부 (텍스트 위 + 빈 영역) 는 즉시 cursor 진입

#### Gap-4: rhwp-studio `input-handler.ts` 에 `isShapeBorderClick` 부재

`isTableBorderClick` 일반화 헬퍼 신규 필요.

### 2.3 부수 발견 — 글상자 안 텍스트 좌표 정밀도 (본 task 범위 외)

H 케이스 (400, 1100) 가 글상자 안 pi=18 으로 매핑되나, pi=18 의 SVG 좌표는 y=1024.1 — body_area bottom (1084.7) 안. 본 task 와 무관 (글상자 안 텍스트는 정상 hit 됨).

## 3. Stage 2 구현 계획서 작성 진행

본 진단 결과로 Stage 2 구현 계획서 (Stage 2.1~2.5) 를 작성한다.

핵심 변경:
- **Native (Rust)**: `cursor_rect.rs collect_runs` 에 `RenderNodeType::TextBox` 수집 + `clicked_cell` 선택에 textbox 우선 + `wasm_api.rs getShapeBBox` 신규
- **Studio (TS)**: `isShapeBorderClick` + `findShapeByOuterClick` 헬퍼 + `input-handler-mouse.ts` 클릭 흐름 정정 (외부 경계선만 객체 선택, 내부는 즉시 cursor 진입)
- **회귀 가드**: `tests/issue_919_textbox_hit_test.rs` 4 케이스
- **이미 구현 (변경 없음)**: Esc 패턴 (글상자 편집 → 객체 선택, `input-handler-keyboard.ts:937`)
