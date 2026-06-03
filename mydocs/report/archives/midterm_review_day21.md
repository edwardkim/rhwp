# rhwp 프로젝트 중간 점검 보고서 (21일차)

## 문서 정보

| 항목 | 내용 |
|------|------|
| 문서명 | 기능명세 대비 구현 현황 조견표 |
| 기준 기능명세 | task_43_feature_def.md (v3.0, 2026-02-12) |
| 점검일 | 2026-02-26 (프로젝트 21일차) |
| 기능명세 작성일 | 2026-02-12 (프로젝트 8일차) |
| 경과 | 13일 (타스크 43 이후) |

---

## 1. 전체 요약

### 1.1 WASM API 성장

| 시점 | WASM API 수 | 비고 |
|------|-----------|------|
| 기능명세 작성 시 (Day 8) | 56개 | task_43_feature_def.md §3.2 기준 |
| 현재 (Day 21) | 125개 | +69개 (123% 증가) |

### 1.2 등급별 구현 현황 종합

기능명세의 449개 API 항목 대비 현재 구현 상태:

| 등급 | 명세 수 | 구현 상태 | 설명 |
|------|--------|----------|------|
| A (직접 매핑) | 5 | **5/5 (100%)** | 래퍼만 작성하면 됨 → 이미 WASM API 존재 |
| B (변환 매핑) | ~115 | **~95/115 (83%)** | 대부분의 변환 매핑 가능한 API가 WASM에 구현됨 |
| C (신규 구현) | ~290 | **~105/290 (36%)** | 가장 많은 진척. 커서·선택·편집·표·도형 등 |
| D (아키텍처 차이) | 2 | **0/2 (0%)** | Action 시스템 의존, 아직 미착수 |
| X (스텁 처리) | ~37 | **대응 불필요** | 빈 함수로 처리 예정 |
| **합계** | **449** | **~205/412 (50%)** | X 제외 실질 커버리지 |

### 1.3 시나리오별 커버리지 변화

| 시나리오 | Day 8 | Day 21 | 변화 | 비고 |
|---------|-------|--------|------|------|
| 문서 뷰어 | 100% | **100%** | ±0 | 이미 완전 지원 |
| 기안문 자동 생성 | 25% | **25%** | ±0 | 핵심 블로커: 필드 시스템 미구현 |
| 양식 편집 | 32% | **68%** | +36% | 커서·서식·표·선택·Undo 대폭 진척 |
| 완전 편집기 | 27% | **50%** | +23% | WASM 코어 대폭 확장 |

---

## 2. HwpCtrl Properties 조견표 (18개)

| # | Property | 등급 | rhwp 구현 | 상태 | 비고 |
|---|----------|------|----------|------|------|
| 1 | PageCount | A | `pageCount()` | **완료** | |
| 2 | CharShape | B | `getCharPropertiesAt()` | **완료** | JSON 반환, ParameterSet 래퍼 필요 |
| 3 | ParaShape | B | `getParaPropertiesAt()` | **완료** | JSON 반환, ParameterSet 래퍼 필요 |
| 4 | CellShape | B | `getCellCharPropertiesAt()` + `getCellParaPropertiesAt()` | **완료** | 합성 래퍼 필요 |
| 5 | IsEmpty | B | `getDocumentInfo()` | **완료** | 결과 파싱으로 판단 가능 |
| 6 | Version | B | `getDocumentInfo()` | **완료** | version 필드 추출 |
| 7 | ViewProperties | B | `HwpViewer.setZoom()` | **완료** | |
| 8 | EditMode | B | `convertToEditable()` | **부분** | 편집/읽기전용 전환 가능, 배포용 모드 전환 추가 필요 |
| 9 | ReadOnlyMode | B | `convertToEditable()` | **부분** | EditMode와 연동 |
| 10 | ScrollPosInfo | B | `HwpViewer.updateViewport()` | **완료** | |
| 11 | CurFieldState | C | - | 미구현 | 필드 시스템 필요 |
| 12 | CurSelectedCtrl | C | Studio: 표/그림 선택 모드 | **부분** | Studio UI에서 선택 동작, WASM API 래퍼 미구현 |
| 13 | HeadCtrl | C | `findNextEditableControl()` | **부분** | DFS 순회로 컨트롤 탐색 가능 |
| 14 | IsModified | C | - | 미구현 | 문서 변경 추적 필요 |
| 15 | LastCtrl | C | `findNextEditableControl()` | **부분** | 역방향 DFS 순회 |
| 16 | ParentCtrl | C | 네비게이션 컨텍스트에 부모 정보 존재 | **부분** | NavContextEntry에 부모 컨트롤 정보 |
| 17 | SelectionMode | C | Studio: F5 셀 선택, 개체 선택 | **부분** | 3종 선택 모드 UI 구현 |
| 18 | EngineProperties | X | - | 스텁 | rhwp 자체 설정 체계 |

**Properties 진척률**: 완료 8 + 부분 6 + 미구현 2 + 스텁 1 = **82% (완료+부분)**

---

## 3. HwpCtrl Methods 조견표 (67개)

### 3.1 문서 관리 (8개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | Open | A | `new(data)` | **완료** |
| 2 | OpenDocument | A | `new(data)` | **완료** |
| 3 | SaveAs | A | `exportHwp()` | **완료** |
| 4 | SaveDocument | A | `exportHwp()` | **완료** |
| 5 | Clear | B | `createBlankDocument()` | **완료** |
| 6 | Insert | C | - | 미구현 |
| 7 | InsertDocument | C | - | 미구현 |
| 8 | PrintDocument | B | `renderPageToCanvas()` → `window.print()` | **부분** |

### 3.2 텍스트 입출력 (8개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | GetText | C | - | 미구현 |
| 2 | GetTextBySet | C | - | 미구현 |
| 3 | GetPageText | B | `getPageTextLayout()` | **완료** |
| 4 | GetTextFile | B | `exportHwp()` (HWP만) | **부분** |
| 5 | SetTextFile | B | `new(data)` (HWP만) | **부분** |
| 6 | InitScan | C | - | 미구현 |
| 7 | ReleaseScan | C | - | 미구현 |
| 8 | GetHeadingString | C | - | 미구현 |

### 3.3 커서/위치 (9개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | GetPos | C | `getCaretPosition()` | **부분** |
| 2 | SetPos | C | `hitTest()` + 커서 상태 | **부분** |
| 3 | GetPosBySet | C | - | 미구현 |
| 4 | SetPosBySet | C | - | 미구현 |
| 5 | MovePos | C | `navigateNextEditable()` + `moveVertical()` | **부분** |
| 6 | MoveToField | C | - | 미구현 |
| 7 | MoveToFieldEx | C | - | 미구현 |
| 8 | KeyIndicator | B | `getDocumentInfo()` + 커서 상태 | **부분** |
| 9 | ShowCaret | X | - | 스텁 |

### 3.4 선택/블록 (4개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | SelectText | B | `getSelectionRects()` + `copySelection()` | **부분** |
| 2 | GetSelectedPos | B | Studio 커서 상태 | **부분** |
| 3 | GetSelectedPosBySet | B | - | 미구현 |
| 4 | GetMousePos | C | `hitTest()` | **완료** |

### 3.5 필드 관리 (10개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1-10 | CreateField ~ SetFieldViewOption | C | - | **전체 미구현** |

### 3.6 이미지/객체 삽입 (4개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | InsertPicture | B→C | `insertPicture()` | **완료** |
| 2 | InsertBackgroundPicture | C | - | 미구현 |
| 3 | InsertCtrl | B | `createTable()`, `createShapeControl()`, `insertPicture()` | **완료** |
| 4 | DeleteCtrl | C | `deleteTableControl()`, `deletePictureControl()`, `deleteShapeControl()` | **완료** |

### 3.7 표 조회 (2개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | GetTableCellAddr | B | `getCellInfo()`, `getCellInfoByPath()` | **완료** |
| 2 | GetViewStatus | B | `HwpViewer.updateViewport()` | **완료** |

### 3.8 페이지 이미지 (2개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | CreatePageImage | B | `renderPageSvg()`, `renderPageCanvas()` | **완료** |
| 2 | CreatePageImageEx | B | `renderPageToCanvas()` | **완료** |

### 3.9 액션 시스템 (5개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1-5 | CreateAction ~ LockCommand | C/D | - | **전체 미구현** |

### 3.10 편집 제어 (2개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | IsCommandLock | D | - | 미구현 |
| 2 | AddEventListener | C | Studio EventBus | **부분** |

### 3.11 UI 제어 (7개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1-7 | ShowToolBar ~ ShowCaret | X | - | **전체 스텁** |

### 3.12 유틸리티 (6개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1-4 | Solar/LunarTo* | X | - | JS 라이브러리 대체 |
| 5 | GetCtrlHorizontalOffset | B | `getPageControlLayout()` | **완료** |
| 6 | GetCtrlVerticalOffset | B | `getPageControlLayout()` | **완료** |

### 3.13 맞춤법 (1개)

| # | Method | 등급 | rhwp 구현 | 상태 |
|---|--------|------|----------|------|
| 1 | IsSpellCheckCompleted | X | - | 스텁 |

### Methods 진척 요약

| 등급 | 수 | 완료 | 부분 | 미구현 | 스텁 |
|------|---|------|------|--------|------|
| A | 4 | 4 | - | - | - |
| B | 17 | 10 | 5 | 1 | 1 |
| C | 30 | 4 | 3 | 23 | - |
| D | 2 | - | - | 2 | - |
| X | 14 | - | - | - | 14 |
| **합계** | **67** | **18** | **8** | **26** | **15** |

**Methods 진척률**: 완료 18 + 부분 8 = **39% (실질), X 제외 시 49%**

---

## 4. Action Table 조견표 (314개, 카테고리별)

### 4.1 커서이동 (51개) — C등급

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| MoveLeft/Right (좌우) | 2 | `navigateNextEditable(delta=±1)` | **완료** |
| MoveUp/Down (상하) | 2 | `moveVertical()` + `moveVerticalByPath()` | **완료** |
| MoveLineBegin/End | 2 | `getLineInfo()` → 줄 시작/끝 | **완료** |
| MoveLineUp/Down | 2 | `moveVertical()` | **완료** |
| MoveNextChar/PrevChar | 2 | `navigateNextEditable()` | **완료** |
| MoveNextWord/PrevWord | 2 | Studio `cursor.ts` 단어 이동 | **완료** |
| MoveParaBegin/End | 2 | Studio `cursor.ts` 문단 이동 | **완료** |
| MoveNextPos/PrevPos | 2 | `navigateNextEditable()` (서브리스트 포함) | **완료** |
| MoveDocBegin/End | 2 | Studio 커서 처리 | **완료** |
| MovePageUp/Down | 2 | Studio 스크롤 + 커서 | **부분** |
| MoveListBegin/End | 2 | `navigateNextEditable()` 컨텍스트 | **부분** |
| MoveColumnBegin/End | 2 | 다단 커서 네비게이션 (task 166) | **완료** |
| MoveNextColumn/PrevColumn | 2 | 다단 경계 이동 (task 166) | **완료** |
| MoveSectionUp/Down | 2 | - | 미구현 |
| MoveScrollUp/Down/Next/Prev | 4 | Studio 스크롤 | **완료** |
| MoveViewBegin/End/Up/Down | 4 | Studio 뷰포트 | **부분** |
| MoveWordBegin/End | 2 | Studio `cursor.ts` | **완료** |
| MoveTopLevel*/MoveParentList/MoveRootList | 5 | `navigateNextEditable()` | **부분** |
| MoveNextParaBegin/PrevParaBegin/PrevParaEnd | 3 | Studio `cursor.ts` | **완료** |
| MoveNextPosEx/PrevPosEx | 2 | - | 미구현 |
| ReturnPrevPos | 1 | - | 미구현 |

**커서이동 구현률**: 약 **37/51 (73%)**

### 4.2 선택확장 (36개) — C등급

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| Select (F3) | 1 | Studio `cursor.ts` Shift 선택 | **완료** |
| SelectAll | 1 | `performSelectAll()` | **완료** |
| SelectColumn (F4) | 1 | - | 미구현 |
| SelectCtrlFront/Reverse | 2 | - | 미구현 |
| MoveSelLeft/Right/Up/Down | 4 | Studio Shift+화살표 | **완료** |
| MoveSelLineBegin/End/Up/Down | 4 | Studio Shift+Home/End/위/아래 | **완료** |
| MoveSelNextChar/PrevChar | 2 | Studio Shift+좌/우 | **완료** |
| MoveSelNextWord/PrevWord | 2 | Studio Ctrl+Shift+좌/우 | **완료** |
| MoveSelDocBegin/End | 2 | Studio Ctrl+Shift+Home/End | **완료** |
| MoveSelParaBegin/End | 2 | Studio 선택 확장 | **부분** |
| MoveSelNextParaBegin/PrevParaBegin/PrevParaEnd | 3 | - | 미구현 |
| MoveSelNextPos/PrevPos | 2 | - | 미구현 |
| MoveSelListBegin/End | 2 | - | 미구현 |
| MoveSelTopLevelBegin/End | 2 | - | 미구현 |
| MoveSelPageUp/Down | 2 | - | 미구현 |
| MoveSelViewUp/Down | 2 | - | 미구현 |
| MoveSelWordBegin/End | 2 | - | 미구현 |

**선택확장 구현률**: 약 **19/36 (53%)**

### 4.3 텍스트 편집 (29개) — B/C 혼합

| Action | SetID | rhwp 구현 | 상태 |
|--------|-------|----------|------|
| InsertText | InsertText | `insertText()` / `insertTextInCell()` | **완료** |
| InsertSpace | - | `insertText(" ")` | **완료** |
| InsertTab | - | `insertText("\t")` | **완료** |
| InsertNonBreakingSpace | - | `insertText()` 특수문자 | **완료** |
| InsertFixedWidthSpace | - | `insertText()` 특수문자 | **완료** |
| Delete | - | `deleteText()` / `deleteTextInCell()` | **완료** |
| DeleteBack | - | `deleteText()` + `mergeParagraph()` | **완료** |
| DeleteWord | - | Studio 단어 삭제 | **완료** |
| DeleteWordBack | - | Studio 역방향 단어 삭제 | **완료** |
| DeleteLine | - | - | 미구현 |
| DeleteLineEnd | - | - | 미구현 |
| BreakPara | - | `splitParagraph()` / `splitParagraphInCell()` | **완료** |
| BreakLine | - | - | 미구현 |
| BreakPage | - | - | 미구현 |
| BreakSection | - | - | 미구현 |
| BreakColumn | - | - | 미구현 |
| BreakColDef | - | - | 미구현 |
| DeleteField | - | - | 미구현 |
| InsertCpNo / InsertCpTpNo / InsertTpNo | - | - | 미구현 |
| InsertPageNum | - | - | 미구현 |
| InsertEndnote / InsertFootnote | - | - | 미구현 |
| InsertFieldTemplate | SetID | - | 미구현 |
| InsertFile | SetID | - | 미구현 |
| InsertHyperlink | SetID | - | 미구현 |
| InsertLine | - | - | 미구현 |
| InputCodeTable | SetID | - | 스텁 |

**텍스트 편집 구현률**: **11/29 (38%)**

### 4.4 글자 서식 (33개) — B/C 혼합

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| CharShape 대화상자 | 1 | Studio `char-shape-dialog.ts` (3탭) | **완료** |
| Bold/Italic/Underline | 3 | `applyCharFormat()` + 단축키 | **완료** |
| Strikethrough(Centerline) | 1 | `applyCharFormat()` + 스타일바 | **완료** |
| Emboss/Engrave | 2 | `applyCharFormat()` | **완료** |
| Outline/Shadow | 2 | `applyCharFormat()` | **완료** |
| Superscript/Subscript/Toggle | 3 | `applyCharFormat()` | **완료** |
| Normal (서식 초기화) | 1 | `applyCharFormat()` | **부분** |
| HeightIncrease/Decrease | 2 | Studio 스타일바 크기 ±1 | **완료** |
| SpacingIncrease/Decrease | 2 | `applyCharFormat()` | **완료** |
| WidthIncrease/Decrease | 2 | `applyCharFormat()` | **완료** |
| NextFaceName/PrevFaceName | 2 | Studio 글꼴 드롭다운 | **부분** |
| TextColor 8색 | 8 | Studio 컬러피커 | **완료** |
| Height/Spacing/Width 포커스 | 3 | Studio 스타일바 | **부분** |
| Hyperlink | 1 | - | 미구현 |

**글자 서식 구현률**: **28/33 (85%)**

### 4.5 문단 서식 (27개) — B/C 혼합

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| ParagraphShape 대화상자 | 1 | Studio `para-shape-dialog.ts` (4탭) | **완료** |
| 정렬 6종 (Left~Division) | 6 | `applyParaFormat()` + 스타일바 4종 | **완료** |
| 왼쪽 여백 증가/감소 | 2 | `applyParaFormat()` | **완료** |
| 오른쪽 여백 증가/감소 | 2 | `applyParaFormat()` | **완료** |
| 양쪽 여백 증가/감소 | 2 | `applyParaFormat()` | **완료** |
| 줄간격 증가/감소 | 2 | `applyParaFormat()` | **완료** |
| 들여쓰기 3종 | 3 | `applyParaFormat()` | **완료** |
| ParagraphShapeProtect | 1 | - | 미구현 |
| ParagraphShapeWithNext | 1 | - | 미구현 |
| 문단번호/글머리표 7종 | 7 | - | 미구현 |

**문단 서식 구현률**: **19/27 (70%)**

### 4.6 표 조작 (50개) — B/C 혼합

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| TableCreate | 1 | `createTable()` + Studio 격자 다이얼로그 | **완료** |
| 행 삽입/삭제 (3+2) | 5 | `insertTableRow()`, `deleteTableRow()` | **완료** |
| 열 삽입/삭제 (2+2) | 4 | `insertTableColumn()`, `deleteTableColumn()` | **완료** |
| TableMergeCell | 1 | `mergeTableCells()` | **완료** |
| TableSplitCell 3종 | 3 | `splitTableCell()`, `splitTableCellInto()` | **완료** |
| TableDistributeWidth/Height | 2 | - | 미구현 |
| TableStringToTable | 1 | - | 미구현 |
| TablePropertyDialog | 1 | Studio `table-cell-props-dialog.ts` (6탭) | **완료** |
| TableCellBlock 5종 | 5 | Studio F5 셀 선택 모드 | **완료** |
| 셀 이동 8종 (Left~ColEnd) | 8 | Studio Tab/Shift+Tab + 화살표 | **완료** |
| TableResize 16종 | 16 | `resizeTableCells()` (부분) | **부분** |
| TableSubtractRow | 1 | `deleteTableRow()` | **완료** |
| TableDeleteCell | 1 | - | 미구현 |
| TableInsertRowColumn | 1 | `insertTableRow/Column` 조합 | **완료** |
| TableDeleteRowColumn | 1 | - | 미구현 |

**표 조작 구현률**: **33/50 (66%)**

### 4.7 셀 서식 (6개) — C등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| CellBorder | Studio 표/셀 속성 대화상자 테두리 탭 | **완료** |
| CellFill | Studio 표/셀 속성 대화상자 배경 탭 | **완료** |
| CellBorderFill | Studio 표/셀 속성 대화상자 | **완료** |
| CellZoneBorder | `setCellProperties()` (범위) | **부분** |
| CellZoneFill | `setCellProperties()` (범위) | **부분** |
| CellZoneBorderFill | - | **부분** |

**셀 서식 구현률**: **3/6 완료 + 3/6 부분 = 100% (부분 포함)**

### 4.8 검색/치환 (8개) — C등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| FindDlg ~ AllReplace (전체) | - | **전체 미구현** |

**검색/치환 구현률**: **0/8 (0%)**

### 4.9 개체 조작 (46개) — C등급

| 하위 기능 | 수 | rhwp 구현 | 상태 |
|----------|---|----------|------|
| DrawObjCreatorTextBox | 1 | `createShapeControl()` | **완료** |
| DrawObjCreator 4종 (Arc/Ellipse/Line/Rectangle) | 4 | - | 미구현 |
| PictureInsertDialog | 1 | Studio 그림 삽입 다이얼로그 | **완료** |
| ModifyCtrl / ModifyShapeObject | 2 | `setShapeProperties()`, `setPictureProperties()` | **완료** |
| ShapeObjDialog | 1 | Studio `picture-props-dialog.ts` | **완료** |
| ShapeObjBringToFront/Forward/SendBack/ToBack | 4 | `changeShapeZOrder()` | **완료** |
| ShapeObjBringInFrontOfText/SendBehindText | 2 | - | 미구현 |
| ShapeObjHorzFlip/VertFlip + 원상태 | 4 | Studio 회전/대칭 (task 165) | **완료** |
| ShapeObjMove 4방향 | 4 | - | 미구현 |
| ShapeObjResize 4방향 | 4 | - | 미구현 |
| ShapeObjNextObject/PrevObject | 2 | - | 미구현 |
| ShapeObjTableSelCell | 1 | Studio 표 클릭 → 셀 진입 | **완료** |
| ShapeObjTextBoxEdit | 1 | Studio 글상자 클릭 → 편집 | **완료** |
| ShapeObjAttachCaption/DetachCaption | 2 | - | 미구현 |
| ShapeObjAttachTextBox/DetachTextBox | 2 | - | 미구현 |
| ShapeObjLock/UnlockAll | 2 | - | 미구현 |
| ShapeObjUngroup | 1 | - | 미구현 |
| ShapeObjAlign 10종 | 10 | - | 미구현 |
| ModifyFill/LineProperty | 2 | `setShapeProperties()` (부분) | **부분** |
| ModifyFieldClickhere | 1 | - | 미구현 |
| ModifyHyperlink | 1 | - | 미구현 |

**개체 조작 구현률**: 약 **16/46 (35%)**

### 4.10 문서관리 (4개) — B/C 혼합

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| DocSummaryInfo | `getDocumentInfo()` | **부분** |
| DocumentInfo | `getDocumentInfo()` | **부분** |
| FileSetSecurity | `convertToEditable()` | **부분** |
| SpellingCheck | - | 스텁 |

### 4.11 페이지 설정 (3개) — C등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| PageSetup | `setPageDef()` + Studio `page-setup-dialog.ts` | **완료** |
| PageNumPos | - | 미구현 |
| PageHiding | - | 미구현 |

### 4.12 머리말/꼬리말 (1개) — C등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| HeaderFooter | 렌더링만 지원, 편집 미구현 | 미구현 |

### 4.13 뷰 설정 (3개) — B등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| ViewZoomFitPage | Studio 뷰 메뉴 | **완료** |
| ViewZoomFitWidth | Studio 뷰 메뉴 | **완료** |
| ViewZoomNormal | Studio 뷰 메뉴 | **완료** |

### 4.14 편집 제어 (10개) — C등급

| Action | rhwp 구현 | 상태 |
|--------|----------|------|
| Undo | Studio Ctrl+Z | **완료** |
| Redo | Studio Ctrl+Shift+Z / Ctrl+Y | **완료** |
| Cancel (ESC) | Studio ESC 처리 | **완료** |
| Close / CloseEx | - | 미구현 |
| Erase | `deleteRange()` / `deleteRangeInCell()` | **완료** |
| ToggleOverwrite | - | 미구현 |
| Print | Studio 인쇄 (부분) | **부분** |
| ReplaceAction | - | 미구현 |
| Hyperlink | - | 미구현 |

### Actions 진척 요약

| 카테고리 | 수 | 완료+부분 | 구현률 |
|---------|---|----------|--------|
| 커서이동 | 51 | 37 | 73% |
| 선택확장 | 36 | 19 | 53% |
| 텍스트편집 | 29 | 11 | 38% |
| 글자서식 | 33 | 28 | 85% |
| 문단서식 | 27 | 19 | 70% |
| 표조작 | 50 | 33 | 66% |
| 셀서식 | 6 | 6 | 100% |
| 검색치환 | 8 | 0 | 0% |
| 개체조작 | 46 | 16 | 35% |
| 문서관리 | 4 | 3 | 75% |
| 페이지설정 | 3 | 1 | 33% |
| 머리말꼬리말 | 1 | 0 | 0% |
| 뷰설정 | 3 | 3 | 100% |
| 편집제어 | 10 | 5 | 50% |
| **합계** | **307** | **181** | **59%** |

> ※ 314개 중 스텁 처리 대상 ~7개 제외 = 307개 실질 대상

---

## 5. ParameterSet 조견표 (50개)

### 5.1 등급 B — 매핑 가능 (14개)

| # | SetID | rhwp 구현 | 상태 |
|---|-------|----------|------|
| 1 | CharShape | `applyCharFormat(json)` — 63항목 중 주요 20+ 매핑 | **완료** |
| 2 | ParaShape | `applyParaFormat(json)` — 33항목 중 주요 15+ 매핑 | **완료** |
| 3 | Table | `getTableProperties()` / `setTableProperties()` | **완료** |
| 4 | Cell | `getCellProperties()` / `setCellProperties()` | **완료** |
| 5 | TableCreation | `createTable()` | **완료** |
| 6 | TableSplitCell | `splitTableCell()` / `splitTableCellInto()` | **완료** |
| 7 | SummaryInfo | `getDocumentInfo()` (부분) | **부분** |
| 8 | DocumentInfo | `getDocumentInfo()` (부분) | **부분** |
| 9 | ViewProperties | `HwpViewer.setZoom()` | **완료** |
| 10 | InsertText | `insertText()` | **완료** |
| 11 | ListParaPos | 커서 상태 (sec, para, pos) | **완료** |
| 12 | TableDeleteLine | `deleteTableRow/Column()` | **완료** |
| 13 | TableInsertLine | `insertTableRow/Column()` | **완료** |
| 14 | FileSetSecurity | `convertToEditable()` (부분) | **부분** |

**B등급 구현률**: 완료 11 + 부분 3 = **100% (부분 포함)**

### 5.2 등급 C — 신규 구현 필요 (33개)

| # | SetID | rhwp 구현 | 상태 | 비고 |
|---|-------|----------|------|------|
| 1 | BorderFill | 렌더링 파싱 완료, 편집 API 있음 | **부분** | 셀 속성 다이얼로그에서 사용 |
| 2 | BorderFillExt | 렌더링 파싱 완료 | **부분** | |
| 3 | CellBorderFill | `setCellProperties()` | **부분** | |
| 4 | ShapeObject | `getShapeProperties()` / `setShapeProperties()` | **부분** | 주요 속성만 |
| 5 | DrawImageAttr | `getPictureProperties()` / `setPictureProperties()` | **부분** | |
| 6 | DrawLineAttr | `setShapeProperties()` (부분) | **부분** | |
| 7 | DrawRotate | 회전 렌더링 + UI (task 165) | **부분** | |
| 8 | DrawShadow | 그림자 렌더링 | **부분** | 읽기만 |
| 9 | SecDef | `setPageDef()` | **부분** | 주요 필드만 |
| 10 | PageDef | `getPageDef()` / `setPageDef()` | **완료** | |
| 11 | BulletShape | - | 미구현 | |
| 12 | NumberingShape | - | 미구현 | |
| 13 | ListProperties | - | 미구현 | |
| 14 | DrawFillAttr | 채우기 렌더링 (task 158) | **부분** | 6종 패턴 채우기 |
| 15 | DrawLayout | - | 미구현 | |
| 16 | DrawShear | - | 미구현 | |
| 17 | DrawArcType | 호 렌더링 (task 165) | **부분** | |
| 18 | PageBorderFill | - | 미구현 | |
| 19 | PageHiding | - | 미구현 | |
| 20 | PageNumPos | - | 미구현 | |
| 21 | PageNumCtrl | - | 미구현 | |
| 22 | FindReplace | - | 미구현 | |
| 23 | HeaderFooter | 렌더링만 | 미구현 | |
| 24 | FootnoteShape | 렌더링만 | 미구현 | |
| 25 | EndnoteShape | 렌더링만 | 미구현 | |
| 26 | InsertFieldTemplate | - | 미구현 | |
| 27 | HyperLink | - | 미구현 | |
| 28 | ColDef | 다단 렌더링 + 편집 (task 166) | **부분** | |
| 29 | Caption | - | 미구현 | |
| 30 | CtrlData | - | 미구현 | |
| 31 | MemoShape | - | 미구현 | |
| 32 | TabDef | 탭 정의 렌더링 완료 | **부분** | |
| 33 | InsertFile | - | 미구현 | |

**C등급 구현률**: 완료 1 + 부분 13 + 미구현 19 = **42% (완료+부분)**

### 5.3 등급 X — 스텁 (3개)

| # | SetID | 상태 |
|---|-------|------|
| 1 | CodeTable | 스텁 (OS IME 대체) |
| 2 | EngineProperties | 스텁 |
| 3 | SpellingCheck | 스텁 |

---

## 6. 기능명세에 없는 rhwp 독자 구현

기능명세(task 43)에서 예측하지 못했거나 rhwp 고유 강점으로 추가 구현된 기능:

| # | 기능 | WASM API | 비고 |
|---|------|---------|------|
| 1 | **중첩 표 경로 기반 API** | `*ByPath()` 시리즈 7개 | 한컴 API에 없는 중첩 표 직접 접근 |
| 2 | **HTML 양방향 변환** | `exportSelectionHtml`, `pasteHtml` 등 5개 | 서식 유지 클립보드 |
| 3 | **컨트롤 복사/붙여넣기** | `copyControl`, `pasteControl`, `clipboardHasControl` | 표/그림 개체 단위 복사 |
| 4 | **배치 모드** | `beginBatch`, `endBatch`, `getEventLog` | 대량 편집 최적화 |
| 5 | **WebCanvas 렌더링** | `renderPageToCanvas()` 직접 렌더링 | 브라우저 Canvas 2D 직접 |
| 6 | **다단 편집** | 칼럼 추적 커서, 다단 경계 이동, 다단 선택 | task 166 |
| 7 | **글상자 오버플로우 네비게이션** | DFS 오버플로우 연결 | task 159 |
| 8 | **도형 회전/대칭 렌더링** | SVG/Canvas/WebCanvas 모두 지원 | task 165 |
| 9 | **패턴 채우기** | 6종 SVG/Canvas/WebCanvas 렌더링 | task 158 |
| 10 | **4중 렌더링 백엔드** | SVG, HTML, Canvas 명령, WebCanvas 직접 | 기능명세 당시 3종 → 4종 |
| 11 | **셀 분할 고급** | `splitTableCellInto(N×M)`, `splitTableCellsInRange` | 범위 분할 |
| 12 | **표 이동/크기 조절** | `moveTableOffset`, `resizeTableCells` | 대화형 편집 |
| 13 | **빈 문서 생성** | `createBlankDocument()` | 템플릿 기반 |
| 14 | **투명 테두리 토글** | `setShowTransparentBorders()` | 편집 보조 |
| 15 | **진단 도구** | `measureWidthDiagnostic()` | 줄 폭 측정 진단 |

---

## 7. 핵심 미구현 영역 분석

### 7.1 P0 미구현 항목 (마이그레이션 블로커)

| # | 영역 | API 수 | 영향 | 선행 조건 |
|---|------|--------|------|----------|
| 1 | **필드 시스템** | 13개 | 기안문 자동 생성 불가 | WASM 코어 필드 파싱/편집 |
| 2 | **Action/ParameterSet 프레임워크** | 5+14개 | 고급 서식 적용 불가 | JS 호환 레이어 |
| 3 | **텍스트 스캔** | 4개 | 문서 일괄 읽기 불가 | WASM 코어 스캔 API |

### 7.2 완료율이 높은 영역 (강점)

| # | 영역 | 구현률 | 비고 |
|---|------|--------|------|
| 1 | 글자 서식 | 85% | 대화상자 + 단축키 + 서식바 |
| 2 | 커서 이동 | 73% | DFS + 다단 + 오버플로우 |
| 3 | 문단 서식 | 70% | 대화상자 + 정렬 + 여백 |
| 4 | 표 조작 | 66% | 생성/삽입/삭제/병합/분할 + 대화상자 |
| 5 | 셀 서식 | 100% | 테두리/배경 대화상자 |
| 6 | 뷰 설정 | 100% | 줌 전체 |

### 7.3 구현률이 낮은 영역 (약점)

| # | 영역 | 구현률 | 비고 |
|---|------|--------|------|
| 1 | 검색/치환 | 0% | 전체 미구현 |
| 2 | 머리말/꼬리말 편집 | 0% | 렌더링만 지원 |
| 3 | 필드 시스템 | 0% | P0 블로커 |
| 4 | 개체 그리기 도구 | 0% | P2 |
| 5 | 텍스트 편집 고급 | 38% | 각주/미주/필드/페이지나누기 미구현 |

---

## 8. Phase 로드맵 대비 진척

기능명세 §7의 4-Phase 로드맵 대비:

| Phase | 목표 | 진척 | 상세 |
|-------|------|------|------|
| Phase 1: 기안문 자동 생성 | 필드 시스템 + 호환 래퍼 | **10%** | 호환 래퍼 미착수, 필드 시스템 미착수 |
| Phase 2: 기본 편집기 | 커서+선택+Action+Undo+검색 | **65%** | 커서·선택·Undo 완료, Action·검색 미착수 |
| Phase 3: 고급 편집기 | 이미지+머리말+페이지설정+셀서식 | **45%** | 이미지·셀서식·페이지설정 완료, 머리말·각주 미착수 |
| Phase 4: 완전 호환 | 그리기+문단번호+표고급+다단 | **25%** | 다단 완료, 나머지 미착수 |

> **주의**: Phase 순서와 실제 구현 순서가 다름. 실제로는 Phase 2~4를 병행하면서 편집기 코어를 우선 강화하는 전략을 취함. Phase 1 (호환 레이어)는 아직 미착수.

---

## 9. 결론

### 9.1 21일간 성과

- **WASM API**: 56개 → 125개 (+123%)
- **렌더링**: 3종 → 4종 백엔드 (WebCanvas 추가)
- **편집**: 표 생성·삽입·삭제·병합·분할, 그림/글상자 삽입·삭제, 회전/대칭, 패턴 채우기
- **네비게이션**: DFS 순회, 다단 경계, 오버플로우 연결, 중첩 표 경로 기반
- **UI**: 12개 대화상자, 45+ 기능 메뉴, 19개 단축키, 4종 컨텍스트 메뉴
- **전체 커버리지**: 449개 중 ~205개 대응 (50%, X 제외)

### 9.2 향후 우선순위

| 순위 | 항목 | 사유 |
|------|------|------|
| 1 | **필드 시스템** (P0) | 기안문 자동 생성 = 공공기관 핵심 시나리오 |
| 2 | **호환 레이어 프레임워크** (P0) | HwpCtrl API 래퍼 = 마이그레이션 기반 |
| 3 | **검색/치환** (P0) | 편집기 기본 기능, 0% |
| 4 | **머리말/꼬리말 편집** (P1) | 공문서 양식 필수 |
| 5 | **Action/ParameterSet** (P0) | 고급 서식 적용의 기반 |

### 9.3 시나리오별 전망

```
문서 뷰어       ████████████████████  100% — 완전 지원
기안문 자동생성  █████                  25% — 필드 시스템 구현 시 100% 달성
양식 편집       █████████████▌         68% — 커서/서식/표 대폭 진척
완전 편집기     ██████████             50% — WASM 코어 확장 중
```
