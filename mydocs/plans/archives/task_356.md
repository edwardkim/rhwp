# Task 356: hwpctl 호환 레이어 설계

## 목표

한컴 웹기안기의 `hwpctl` JavaScript API와 호환되는 래퍼 레이어를 설계한다.
웹개발자가 기존 hwpctl 코드를 **변경 없이** rhwp로 전환할 수 있게 한다.

## hwpctl 3축 구조

```
┌─────────────────────────────────────────────────────┐
│  HwpCtrl API (메서드 ~30개)                          │
│  Open, Save, CreateAction, InsertCtrl, Run, ...     │
├─────────────────────────────────────────────────────┤
│  Action (312개)                                      │
│  ├─ 핵심 Action (43개) — ParameterSet 사용           │
│  │   TableCreate, CharShape, ParaShape, ...         │
│  └─ 단순 Action (269개) — ParameterSet 없음          │
│      MoveLeft, Copy, Paste, CharShapeBold, ...      │
├─────────────────────────────────────────────────────┤
│  ParameterSet (40개, Item 530개)                     │
│  TableCreation, CharShape, ParaShape, ShapeObject,  │
│  BorderFill, Cell, PageDef, HeaderFooter, ...       │
└─────────────────────────────────────────────────────┘
```

## 아키텍처

```
웹개발자 JavaScript (기존 hwpctl 코드)
    │
    ▼
┌─────────────────────────────────────┐
│  hwpctl 호환 래퍼 (TypeScript)       │
│  rhwp-studio/src/hwpctl/            │
│                                     │
│  HwpCtrl                            │
│  ├─ CreateAction(id) → Action       │
│  │     ├─ CreateSet() → ParamSet    │
│  │     ├─ GetDefault(set)           │
│  │     ├─ Execute(set)              │
│  │     └─ Run()                     │
│  ├─ InsertCtrl(type, set)           │
│  ├─ Run(actionId)                   │
│  └─ ...                            │
│                                     │
│  Action → rhwp API 변환 엔진         │
│  ParameterSet → JSON 변환            │
└──────────┬──────────────────────────┘
           │
           ▼
┌─────────────────────────────────────┐
│  rhwp WASM API (228개 메서드)        │
│  HwpDocument                        │
└─────────────────────────────────────┘
```

## 핵심 실행 패턴

```javascript
// 패턴 1: Action + ParameterSet
var act = HwpCtrl.CreateAction("TableCreate");
var set = act.CreateSet();
act.GetDefault(set);
set.SetItem("Rows", 5);
set.SetItem("Cols", 3);
act.Execute(set);

// 패턴 2: 단순 Run
HwpCtrl.Run("CharShapeBold");

// 패턴 3: InsertCtrl
var set = HwpCtrl.CreateSet("TableCreation");
set.SetItem("Rows", 5);
HwpCtrl.InsertCtrl("tbl", set);
```

## 분할정복 구현 계획

총 312개 Action을 5개씩 묶어 단계별로 정복한다.

### Wave 1: 프레임워크 + 표 생성 (Task 357)

**프레임워크**: HwpCtrl, Action, ParameterSet 기본 클래스
**Action 5개**:

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 1 | TableCreate | TableCreation | `create_table()` | 표 만들기 |
| 2 | InsertText | InsertText | `insert_text()` | 텍스트 삽입 |
| 3 | BreakPara | — | `split_paragraph()` | 문단 나누기 |
| 4 | BreakPage | — | 쪽 나누기 | |
| 5 | BreakColumn | — | 단 나누기 | |

### Wave 2: 서식 (Task 358)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 6 | CharShape | CharShape | `apply_char_format()` | 글자 모양 |
| 7 | ParagraphShape | ParaShape | `apply_para_format()` | 문단 모양 |
| 8 | CharShapeBold | — | bold 토글 | 진하게 |
| 9 | CharShapeItalic | — | italic 토글 | 기울임 |
| 10 | CharShapeUnderline | — | underline 토글 | 밑줄 |

### Wave 3: 표 편집 (Task 359)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 11 | TableInsertRowColumn | TableInsertLine | `insert_table_row/column()` | 줄/칸 삽입 |
| 12 | TableDeleteRowColumn | TableDeleteLine | `delete_table_row/column()` | 줄/칸 삭제 |
| 13 | TableSplitCell | TableSplitCell | `split_table_cell()` | 셀 나누기 |
| 14 | CellBorderFill | CellBorderFill | `apply_cell_style()` | 셀 테두리/배경 |
| 15 | TablePropertyDialog | ShapeObject | 표 속성 | 표 고치기 |

### Wave 4: 이동/선택 (Task 360)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 16 | MoveLeft | — | 커서 좌 | |
| 17 | MoveRight | — | 커서 우 | |
| 18 | MoveUp | — | 커서 상 | |
| 19 | MoveDown | — | 커서 하 | |
| 20 | SelectAll | — | 전체 선택 | |

### Wave 5: 클립보드 + 실행취소 (Task 361)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 21 | Copy | — | `copy_selection()` | 복사 |
| 22 | Cut | — | 잘라내기 | |
| 23 | Paste | — | `paste_internal()` | 붙여넣기 |
| 24 | Undo | — | 미지원 (stub) | 실행취소 |
| 25 | Redo | — | 미지원 (stub) | 다시실행 |

### Wave 6: 용지/구역/머리말 (Task 362)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 26 | PageSetup | SecDef | `set_page_def()` | 편집 용지 |
| 27 | HeaderFooter | HeaderFooter | `insert_header/footer()` | 머리말/꼬리말 |
| 28 | BreakSection | — | 구역 나누기 | |
| 29 | BreakColDef | — | 단 정의 삽입 | |
| 30 | PageNumPos | PageNumPos | 쪽 번호 | |

### Wave 7: 찾기/바꾸기 (Task 363)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 31 | FindDlg | FindReplace | 미지원 | 찾기 |
| 32 | ReplaceDlg | FindReplace | 미지원 | 바꾸기 |
| 33 | ForwardFind | FindReplace* | 미지원 | 앞으로 찾기 |
| 34 | AllReplace | FindReplace* | 미지원 | 모두 바꾸기 |
| 35 | Hyperlink | HyperLink | 미지원 | 하이퍼링크 |

### Wave 8: 셀 서식 (Task 364)

| # | Action | ParameterSet | rhwp API | 설명 |
|---|--------|-------------|----------|------|
| 36 | CellBorder | CellBorderFill | `apply_cell_style()` | 셀 테두리 |
| 37 | CellFill | CellBorderFill | `apply_cell_style()` | 셀 배경 |
| 38 | CellZoneBorder | CellBorderFill | 영역 테두리 | |
| 39 | CellZoneBorderFill | CellBorderFill | 영역 테두리/배경 | |
| 40 | CellZoneFill | CellBorderFill | 영역 배경 | |

### Wave 9: 글자 서식 단축키 (Task 365)

| # | Action | 설명 |
|---|--------|------|
| 41~45 | CharShapeHeight/Increase/Decrease, CharShapeSpacing/Increase | 크기/자간 |

### Wave 10: 글자 서식 단축키 2 (Task 366)

| # | Action | 설명 |
|---|--------|------|
| 46~50 | CharShapeSuperscript/Subscript/Normal/Outline/Shadow | 첨자/외곽/그림자 |

### Wave 11~: 나머지 단순 Action (Task 367~)

| Wave | Action 수 | 주요 내용 |
|------|----------|----------|
| 11 | 5 | 이동 (Home/End/PageUp/PageDown/DocStart) |
| 12 | 5 | 이동 (DocEnd/WordLeft/WordRight/ParaUp/ParaDown) |
| 13 | 5 | 선택 이동 (SelectLeft/Right/Up/Down/All) |
| 14 | 5 | 삭제 (Delete/Backspace/DeleteWord/DeleteLine/DeleteBack) |
| ... | 5 | ... |
| ~60+ | 나머지 | 기타 |

> Wave 1~6 (30개 Action) 완료 시 핵심 기능 90% 커버.
> Wave 7~10 (20개) 완료 시 서식 기능 완전.
> Wave 11+ (260개) 단순 키보드/이동 동작 — 점진적 추가.

## 파일 구조

```
rhwp-studio/src/hwpctl/
├── index.ts              # HwpCtrl 클래스 + createHwpCtrl()
├── action.ts             # Action 클래스
├── parameter-set.ts      # ParameterSet 클래스
├── action-registry.ts    # 312개 Action 등록 테이블
├── actions/
│   ├── table.ts          # TableCreate, TableInsert/Delete, TableSplit
│   ├── text.ts           # InsertText, BreakPara/Page/Column
│   ├── format.ts         # CharShape, ParaShape, CharShapeBold...
│   ├── navigate.ts       # Move*, Select*
│   ├── clipboard.ts      # Copy, Cut, Paste, Undo, Redo
│   ├── page.ts           # PageSetup, HeaderFooter, PageNumPos
│   └── cell.ts           # CellBorder/Fill/Zone*
├── mappings/
│   ├── table-creation.ts # TableCreation Set → create_table 변환
│   ├── char-shape.ts     # CharShape Set → apply_char_format 변환
│   ├── para-shape.ts     # ParaShape Set → apply_para_format 변환
│   ├── shape-object.ts   # ShapeObject Set → CommonObjAttr 변환
│   ├── page-def.ts       # SecDef Set → set_page_def 변환
│   └── border-fill.ts    # BorderFill Set → 변환
└── types.ts              # PIT_UI1, PIT_I4 등 hwpctl 타입
```

## 구현 현황 추적표

### 진행률 요약

| 구분 | 전체 | 구현 | 비율 |
|------|------|------|------|
| **Wave 1~6 (핵심)** | 30 | 30 | 100% |
| **Wave 7~10 (서식)** | 20 | 0 | 0% |
| **Wave 11+ (단순)** | 262 | 0 | 0% |
| HwpCtrl API 메서드 | 30 | 6 | 20% |
| ParameterSet | 40 | 2 | 5% |
| **총 Action** | **312** | **30** | **10%** |

### Wave별 상태

| Wave | Task | Action 수 | 상태 | 비고 |
|------|------|----------|------|------|
| 1 | 358 | 5 | 완료 | TableCreate/InsertText/BreakPara/Page/Column |
| 2 | 359 | 5 | 완료 | CharShape/ParaShape/Bold/Italic/Underline |
| 3 | 360 | 5 | 완료 | TableInsert/Delete/Split/CellBorderFill/TableProperty |
| 4 | 361 | 5 | 완료 | MoveLeft/Right/Up/Down/SelectAll |
| 5 | 362 | 5 | 완료 | Copy/Cut/Paste/Undo(stub)/Redo(stub) |
| 6 | 363 | 5 | 완료 | PageSetup/HeaderFooter/BreakSection(stub)/ColDef(stub)/PageNum(stub) |
| 7 | 363 | 5 | 미착수 | 찾기/바꾸기 |
| 8 | 364 | 5 | 미착수 | 셀 서식 |
| 9 | 365 | 5 | 미착수 | 글자 단축키 1 |
| 10 | 366 | 5 | 미착수 | 글자 단축키 2 |
| 11+ | 367+ | 262 | 미착수 | 단순 Action |

> 이 표는 각 Wave 완료 시 갱신한다.

## 제약 사항

1. **ActiveX 미지원**: `Open(path)` → `Open(blob)` (Blob/ArrayBuffer 입력)
2. **비동기 초기화**: `createHwpCtrl()` → 이후 동기 사용
3. **미구현 graceful**: 미지원 Action 호출 시 `console.warn` + 무시
4. **이벤트**: `addEventListener(eventType, callback)` — CustomEvent 매핑
5. **서버 API**: `GetTextFile`, `SaveAs` 등 서버 연동은 콜백 기반

## 사용 예시

```javascript
import { createHwpCtrl } from '@rhwp/hwpctl';

// 초기화 (1회)
const HwpCtrl = await createHwpCtrl({
    wasmUrl: '/pkg/rhwp_bg.wasm',
    container: document.getElementById('editor'),
});

// 이후 기존 hwpctl 코드 그대로 사용
var act = HwpCtrl.CreateAction("TableCreate");
var set = act.CreateSet();
act.GetDefault(set);
set.SetItem("Rows", 5);
set.SetItem("Cols", 3);
act.Execute(set);
```
