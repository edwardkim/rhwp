# 한컴 웹기안기 대응 기능정의서

## 문서 정보

| 항목 | 내용 |
|------|------|
| 문서명 | 한컴 웹기안기 대응 기능정의서 |
| 타스크 | #43 |
| 작성일 | 2026-02-12 |
| 버전 | v3.0 (최종) |

---

## 1. 개요 및 마이그레이션 전략

### 1.1 목표

한컴 웹기안기(웹한글 기안기)를 사용하는 공공기관이 rhwp로 전환할 때 **기존 개발된 소스코드의 변경을 최소화**하는 것이 핵심 목표이다.

### 1.2 마이그레이션 원칙

| 원칙 | 설명 |
|------|------|
| 웹 편집기 UI 동일 구현 | 한컴 웹기안기와 동일한 편집기 UI/UX 제공 |
| 개발자 API 동일 구현 | HwpCtrl 호환 API (동일 메서드명, 파라미터, 동작) |

### 1.3 마이그레이션 시나리오

```
[기존 공공기관 코드]                    [마이그레이션 후]
HwpCtrl.Open("doc.hwp")          →    HwpCtrl.Open("doc.hwp")        // 동일
HwpCtrl.PutFieldText("name", …)  →    HwpCtrl.PutFieldText("name", …) // 동일
HwpCtrl.SaveAs("out.hwp", "HWP") →    HwpCtrl.SaveAs("out.hwp", "HWP") // 동일

변경점: <script src="webhwpctrl.js">  →  <script src="rhwp.js">  // 로더만 교체
```

### 1.4 호환 레이어 아키텍처

```
┌─────────────────────────────────────────────────────┐
│  기존 공공기관 JavaScript 코드 (변경 없음)              │
│  HwpCtrl.Open(), PutFieldText(), SaveAs() 등          │
├─────────────────────────────────────────────────────┤
│  rhwp 호환 레이어 (rhwp.js)                            │
│  HwpCtrl API → rhwp WASM API 변환                     │
│  ┌───────────┬───────────┬───────────┬────────────┐ │
│  │ 등급 A    │ 등급 B    │ 등급 C    │ 등급 X     │ │
│  │ 직접 매핑  │ 변환 매핑  │ 신규 구현  │ 빈 함수    │ │
│  └───────────┴───────────┴───────────┴────────────┘ │
├─────────────────────────────────────────────────────┤
│  rhwp WASM Core (Rust → WebAssembly)                 │
│  문서 파싱, 렌더링, 편집, 직렬화                         │
└─────────────────────────────────────────────────────┘
```

### 1.5 호환성 등급 정의

| 등급 | 의미 | 마이그레이션 영향 |
|------|------|-----------------|
| A (직접 매핑) | rhwp에 동등 기능 존재, 래퍼만 작성 | 소스 변경 불필요 |
| B (변환 매핑) | rhwp에 유사 기능, 파라미터 변환 필요 | 소스 변경 불필요 (래퍼가 변환) |
| C (신규 구현) | rhwp에 해당 기능 없음, 내부 구현 필요 | 소스 변경 불필요 (구현 후) |
| D (아키텍처 차이) | 서버 의존 등 구조적 차이 | 소스 일부 변경 필요 |
| X (불필요) | UI 전용, 서버 전용 등 | 빈 함수/스텁으로 처리 |

---

## 2. 웹기안기 HwpCtrl API 전수 분류표

### 2.1 API 전체 규모 요약

| 구성요소 | 규모 | 설명 |
|----------|------|------|
| HwpCtrl Properties | 18개 | 문서 상태/속성 조회 및 설정 |
| HwpCtrl Methods | 67개 | 문서 제어 핵심 메서드 |
| Action IDs | 312개 | Run()으로 실행하는 단위 액션 |
| ParameterSet Types | 50개 | 액션/메서드에 전달하는 구조화 데이터 |
| Action Object | Properties 2 + Methods 5 | 액션 실행 메커니즘 |
| CtrlCode Object | Properties 6 + Methods 1 | 문서 내 컨트롤 코드 |
| ParameterSet Object | Properties 3 + Methods 11 | 파라미터 집합 처리 |
| ParameterArray Object | Properties 2 + Methods 4 | 배열 형식 파라미터 |

### 2.2 HwpCtrl Properties (18개)

| # | Property | Type | 설명 | 카테고리 |
|---|----------|------|------|----------|
| 1 | CellShape | ParameterSet | 현재 셀의 속성 (읽기/쓰기) | 서식 |
| 2 | CharShape | ParameterSet | 현재 커서 위치의 글자 모양 (읽기/쓰기) | 서식 |
| 3 | CurFieldState | Integer | 현재 필드 상태 | 필드 |
| 4 | CurSelectedCtrl | CtrlCode | 현재 선택된 컨트롤 | 개체 |
| 5 | EditMode | Integer | 편집 모드 (0=읽기전용, 1=편집, 0x10=배포용) | 편집제어 |
| 6 | EngineProperties | ParameterSet | 엔진 환경 설정 | 환경설정 |
| 7 | HeadCtrl | CtrlCode | 첫 번째 컨트롤 코드 | 개체 |
| 8 | IsEmpty | Boolean | 문서가 비어있는지 여부 | 문서정보 |
| 9 | IsModified | Boolean | 문서가 수정되었는지 여부 | 문서정보 |
| 10 | LastCtrl | CtrlCode | 마지막 컨트롤 코드 | 개체 |
| 11 | PageCount | Integer | 전체 페이지 수 | 문서정보 |
| 12 | ParaShape | ParameterSet | 현재 커서 위치의 문단 모양 (읽기/쓰기) | 서식 |
| 13 | ParentCtrl | CtrlCode | 상위 컨트롤 코드 | 개체 |
| 14 | ReadOnlyMode | Boolean | 읽기 전용 모드 | 편집제어 |
| 15 | ScrollPosInfo | ParameterSet | 스크롤바 위치 정보 | 뷰 |
| 16 | SelectionMode | Integer | 선택 모드 (0=없음, 1=일반, 2=칸, 3=표, 4=개체) | 선택 |
| 17 | Version | String | 버전 정보 | 시스템 |
| 18 | ViewProperties | ParameterSet | 뷰 속성 (줌 종류, 비율) | 뷰 |

### 2.3 HwpCtrl Methods (67개)

#### 2.3.1 문서 관리 (8개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | Open | path, format, arg, callback, callbackUserData | void | 문서 열기 |
| 2 | OpenDocument | path, format, callback | void | 문서 열기 (간소화) |
| 3 | SaveAs | fileName, format, arg, callback, callbackUserData | void | 다른 이름으로 저장 |
| 4 | SaveDocument | fileName, format, callback | void | 문서 저장 (간소화) |
| 5 | Clear | option | void | 문서 닫기 (0=확인, 1=버림, 2=변경시저장, 3=무조건저장) |
| 6 | Insert | path, format, arg, callback, callbackUserData | void | 문서 끼워넣기 |
| 7 | InsertDocument | path, callback | void | 문서 끼워넣기 (간소화) |
| 8 | PrintDocument | - | void | 인쇄 |

#### 2.3.2 텍스트 입출력 (8개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | GetText | - | String | 텍스트 읽기 (InitScan 후) |
| 2 | GetTextBySet | text | ParameterSet | 텍스트 읽기 (ParameterSet 반환) |
| 3 | GetPageText | pageNo, option | String | 특정 페이지 텍스트 |
| 4 | GetTextFile | format, option, callback | String | 문서 전체를 특정 형식으로 |
| 5 | SetTextFile | data, format, option, callback, callbackUserData | void | 특정 형식 데이터를 문서에 설정 |
| 6 | InitScan | option, range, spara, spos, epara, epos | void | 텍스트 스캔 시작 |
| 7 | ReleaseScan | - | void | 텍스트 스캔 종료 |
| 8 | GetHeadingString | - | String | 문단번호/글머리표 문자열 |

#### 2.3.3 커서/위치 (9개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | GetPos | - | Object | 현재 커서 위치 (list, para, pos) |
| 2 | SetPos | list, para, pos | Boolean | 커서 위치 설정 |
| 3 | GetPosBySet | - | ParameterSet | 커서 위치 (ParameterSet) |
| 4 | SetPosBySet | pos | Boolean | 커서 위치 설정 (ParameterSet) |
| 5 | MovePos | moveID, para, pos | Boolean | 커서 이동 (28+ 이동 타입) |
| 6 | MoveToField | field, text, start, select | Boolean | 필드로 이동 |
| 7 | MoveToFieldEx | field, text, start, boolean, select | Boolean | 필드로 이동 (확장) |
| 8 | KeyIndicator | - | Object | 현재 위치 정보 (구역, 쪽, 단, 줄, 칸) |
| 9 | ShowCaret | show | void | 캐럿 표시/숨김 |

#### 2.3.4 선택/블록 (4개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | SelectText | spara, spos, epara, epos | void | 텍스트 선택 |
| 2 | GetSelectedPos | - | Object | 선택 영역 위치 |
| 3 | GetSelectedPosBySet | sset, eset | void | 선택 영역 위치 (ParameterSet) |
| 4 | GetMousePos | Xrelto, Yrelto | ParameterSet | 마우스 위치 |

#### 2.3.5 필드 관리 (10개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | CreateField | direction, memo, name | void | 필드 생성 |
| 2 | FieldExist | field | Boolean | 필드 존재 여부 |
| 3 | GetFieldList | number, option | String | 필드 목록 |
| 4 | GetFieldText | fieldlist | String | 필드 텍스트 읽기 |
| 5 | PutFieldText | fieldlist, textlist | void | 필드 텍스트 설정 |
| 6 | GetCurFieldName | option | String | 현재 필드 이름 |
| 7 | SetCurFieldName | fieldname, option, direction, memo | void | 현재 필드 이름 설정 |
| 8 | ModifyFieldProperties | field, remove, add | Integer | 필드 속성 수정 |
| 9 | RenameField | oldname, newname | void | 필드 이름 변경 |
| 10 | SetFieldViewOption | option | void | 필드 표시 옵션 설정 |

#### 2.3.6 이미지/객체 삽입 (4개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | InsertPicture | path, embedded, sizeoption, reverse, watermark, effect, width, height, callback | CtrlCode | 그림 삽입 |
| 2 | InsertBackgroundPicture | bordertype, path, embedded, filloption, watermark, effect, brightness, contrast, callback | void | 배경 그림 삽입 |
| 3 | InsertCtrl | ctrlid, initparam | CtrlCode | 컨트롤 삽입 (표 생성 등) |
| 4 | DeleteCtrl | ctrl | void | 컨트롤 삭제 |

#### 2.3.7 표 조회 (2개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | GetTableCellAddr | type | Integer | 셀 주소 (0=칸, 1=줄) |
| 2 | GetViewStatus | - | ParameterSet | 현재 뷰 상태 |

#### 2.3.8 페이지 이미지 (2개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | CreatePageImage | fileName, pgno, format | void | 페이지 이미지 생성 |
| 2 | CreatePageImageEx | fileName, pgno, resolution, depth, format, option, callback, callbackUserData | void | 페이지 이미지 생성 (확장) |

#### 2.3.9 액션 시스템 (5개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | CreateAction | actionID | Action | 액션 객체 생성 |
| 2 | CreateSet | setid | ParameterSet | ParameterSet 생성 |
| 3 | Run | actionID, callback, callbackUserData | Boolean | 액션 실행 |
| 4 | ReplaceAction | oldActionID, newActionID | void | 액션 대체 |
| 5 | LockCommand | actionID, lock | void | 액션 잠금 |

#### 2.3.10 편집 제어 (2개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | IsCommandLock | actionID | Boolean | 액션 잠금 여부 |
| 2 | AddEventListener | eventType, listener | void | 이벤트 리스너 등록 |

#### 2.3.11 UI 제어 (7개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | ShowToolBar | show | void | 도구 모음 표시 |
| 2 | ShowStatusBar | show | void | 상태 표시줄 표시 |
| 3 | ShowRibbon | show | void | 리본 메뉴 표시 |
| 4 | SetToolBar | option, ToolBarID | void | 도구 모음 설정 |
| 5 | ShowVerticalScroll | show | void | 세로 스크롤바 표시 |
| 6 | ShowHorizontalScroll | show | void | 가로 스크롤바 표시 |
| 7 | ShowCaret | show | void | 캐럿 표시 |

#### 2.3.12 유틸리티 (6개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | SolarToLunar | solarYear, solarMonth, solarDay | Object | 양력→음력 변환 |
| 2 | SolarToLunarBySet | solarYear, solarMonth, solarDay | ParameterSet | 양력→음력 변환 (Set) |
| 3 | LunarToSolar | lunarYear, lunarMonth, lunarDay, leap | Object | 음력→양력 변환 |
| 4 | LunarToSolarBySet | lunarYear, lunarMonth, lunarDay, leap | ParameterSet | 음력→양력 변환 (Set) |
| 5 | GetCtrlHorizontalOffset | ctrl, relTo | Integer | 컨트롤 가로 오프셋 |
| 6 | GetCtrlVerticalOffset | ctrl, relTo | Integer | 컨트롤 세로 오프셋 |

#### 2.3.13 맞춤법 (1개)

| # | Method | Parameters | Return | 설명 |
|---|--------|-----------|--------|------|
| 1 | IsSpellCheckCompleted | - | Boolean | 맞춤법 검사 완료 여부 |

### 2.4 Action Table (312개, 15개 카테고리)

Action은 `HwpCtrl.Run("ActionID")`로 실행하거나, `HwpCtrl.CreateAction("ActionID")`로 Action 객체를 생성하여 ParameterSet과 함께 실행한다.

**기호 설명**:
- `-` : ParameterSet 없음 (`HwpCtrl.Run()` 직접 호출 가능)
- `SetID` : 해당 ParameterSet 사용
- `SetID*` : 외부에서 ParameterSet을 반드시 만들어야 정상 동작

#### 2.4.1 커서이동 (51개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| MoveColumnBegin | - | 단의 시작으로 이동 |
| MoveColumnEnd | - | 단의 끝으로 이동 |
| MoveDocBegin | - | 문서 시작으로 이동 |
| MoveDocEnd | - | 문서 끝으로 이동 |
| MoveDown | - | 아래로 이동 |
| MoveLeft | - | 왼쪽으로 이동 |
| MoveLineBegin | - | 줄 시작으로 이동 |
| MoveLineDown | - | 한 줄 아래로 이동 |
| MoveLineEnd | - | 줄 끝으로 이동 |
| MoveLineUp | - | 한 줄 위로 이동 |
| MoveListBegin | - | 리스트 시작으로 이동 |
| MoveListEnd | - | 리스트 끝으로 이동 |
| MoveNextChar | - | 다음 글자로 이동 |
| MoveNextColumn | - | 다음 단으로 이동 |
| MoveNextParaBegin | - | 다음 문단 시작으로 이동 |
| MoveNextPos | - | 다음 위치로 이동 (서브리스트 포함) |
| MoveNextPosEx | - | 다음 위치로 이동 (머리말/꼬리말/각주/미주 포함) |
| MoveNextWord | - | 다음 단어로 이동 |
| MovePageBegin | - | 페이지 시작으로 이동 |
| MovePageDown | - | 다음 페이지 시작으로 이동 |
| MovePageEnd | - | 페이지 끝으로 이동 |
| MovePageUp | - | 이전 페이지 시작으로 이동 |
| MoveParaBegin | - | 문단 시작으로 이동 |
| MoveParaEnd | - | 문단 끝으로 이동 |
| MoveParentList | - | 상위 리스트로 이동 |
| MovePrevChar | - | 이전 글자로 이동 |
| MovePrevColumn | - | 이전 단으로 이동 |
| MovePrevParaBegin | - | 이전 문단 시작으로 이동 |
| MovePrevParaEnd | - | 이전 문단 끝으로 이동 |
| MovePrevPos | - | 이전 위치로 이동 (서브리스트 포함) |
| MovePrevPosEx | - | 이전 위치로 이동 (머리말/꼬리말/각주/미주 포함) |
| MovePrevWord | - | 이전 단어로 이동 |
| MoveRight | - | 오른쪽으로 이동 |
| MoveRootList | - | 루트 리스트로 이동 |
| MoveScrollDown | - | 아래로 스크롤 이동 |
| MoveScrollNext | - | 다음으로 스크롤 이동 |
| MoveScrollPrev | - | 이전으로 스크롤 이동 |
| MoveScrollUp | - | 위로 스크롤 이동 |
| MoveSectionDown | - | 다음 구역으로 이동 |
| MoveSectionUp | - | 이전 구역으로 이동 |
| MoveTopLevelBegin | - | 탑레벨 리스트 시작으로 이동 |
| MoveTopLevelEnd | - | 탑레벨 리스트 끝으로 이동 |
| MoveTopLevelList | - | 탑레벨 리스트로 이동 |
| MoveUp | - | 위로 이동 |
| MoveViewBegin | - | 현재 뷰 시작으로 이동 |
| MoveViewDown | - | 뷰 아래로 이동 (PgDn) |
| MoveViewEnd | - | 현재 뷰 끝으로 이동 |
| MoveViewUp | - | 뷰 위로 이동 (PgUp) |
| MoveWordBegin | - | 단어 시작으로 이동 |
| MoveWordEnd | - | 단어 끝으로 이동 |
| ReturnPrevPos | - | 직전 위치로 복귀 |

#### 2.4.2 선택확장 (36개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| Select | - | 선택 시작 (F3) |
| SelectAll | - | 모두 선택 |
| SelectColumn | - | 칸 블록 선택 (F4) |
| SelectCtrlFront | - | 개체 정방향 선택 |
| SelectCtrlReverse | - | 개체 역방향 선택 |
| MoveSelDocBegin | - | 셀렉션: 문서 처음 |
| MoveSelDocEnd | - | 셀렉션: 문서 끝 |
| MoveSelDown | - | 셀렉션: 아래로 |
| MoveSelLeft | - | 셀렉션: 왼쪽으로 |
| MoveSelLineBegin | - | 셀렉션: 줄 처음 |
| MoveSelLineDown | - | 셀렉션: 한줄 아래 |
| MoveSelLineEnd | - | 셀렉션: 줄 끝 |
| MoveSelLineUp | - | 셀렉션: 한줄 위 |
| MoveSelListBegin | - | 셀렉션: 리스트 처음 |
| MoveSelListEnd | - | 셀렉션: 리스트 끝 |
| MoveSelNextChar | - | 셀렉션: 다음 글자 |
| MoveSelNextParaBegin | - | 셀렉션: 다음 문단 처음 |
| MoveSelNextPos | - | 셀렉션: 다음 위치 |
| MoveSelNextWord | - | 셀렉션: 다음 단어 |
| MoveSelPageDown | - | 셀렉션: 페이지 다운 |
| MoveSelPageUp | - | 셀렉션: 페이지 업 |
| MoveSelParaBegin | - | 셀렉션: 문단 처음 |
| MoveSelParaEnd | - | 셀렉션: 문단 끝 |
| MoveSelPrevChar | - | 셀렉션: 이전 글자 |
| MoveSelPrevParaBegin | - | 셀렉션: 이전 문단 시작 |
| MoveSelPrevParaEnd | - | 셀렉션: 이전 문단 끝 |
| MoveSelPrevPos | - | 셀렉션: 이전 위치 |
| MoveSelPrevWord | - | 셀렉션: 이전 단어 |
| MoveSelRight | - | 셀렉션: 오른쪽으로 |
| MoveSelTopLevelBegin | - | 셀렉션: 처음 |
| MoveSelTopLevelEnd | - | 셀렉션: 끝 |
| MoveSelUp | - | 셀렉션: 위로 |
| MoveSelViewDown | - | 셀렉션: 뷰 아래 |
| MoveSelViewUp | - | 셀렉션: 뷰 위 |
| MoveSelWordBegin | - | 셀렉션: 단어 처음 |
| MoveSelWordEnd | - | 셀렉션: 단어 끝 |

#### 2.4.3 텍스트 편집 (29개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| BreakColDef | - | 단 정의 삽입 |
| BreakColumn | - | 단 나누기 |
| BreakLine | - | 줄 바꿈 (Line Break) |
| BreakPage | - | 쪽 나누기 |
| BreakPara | - | 문단 나누기 |
| BreakSection | - | 구역 나누기 |
| Delete | - | Delete 키 |
| DeleteBack | - | Backspace 키 |
| DeleteField | - | 누름틀/메모 지우기 |
| DeleteLine | - | 한줄 지우기 (Ctrl+Y) |
| DeleteLineEnd | - | 줄 끝까지 지우기 (Alt+Y) |
| DeleteWord | - | 단어 지우기 (Ctrl+T) |
| DeleteWordBack | - | 뒤로 단어 지우기 (Ctrl+BS) |
| InsertCpNo | - | 현재 쪽 번호 삽입 |
| InsertCpTpNo | - | 현재 쪽/전체 쪽 삽입 |
| InsertEndnote | - | 미주 입력 |
| InsertFieldTemplate | InsertFieldTemplate | 필드 입력 |
| InsertFile | InsertFile | 끼워 넣기 |
| InsertFixedWidthSpace | - | 고정폭 빈칸 삽입 |
| InsertFootnote | - | 각주 입력 |
| InsertHyperlink | Hyperlink | 하이퍼링크 만들기 |
| InsertLine | - | 선 넣기 |
| InsertNonBreakingSpace | - | 묶음 빈칸 삽입 |
| InsertPageNum | - | 쪽 번호 넣기 |
| InsertSpace | - | 공백 삽입 |
| InsertTab | - | 탭 삽입 |
| InsertText | InsertText | 텍스트 삽입 |
| InsertTpNo | - | 전체 쪽수 삽입 |
| InputCodeTable | CodeTable | 문자표 |

#### 2.4.4 글자 서식 (33개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| CharShape | CharShape | 글자 모양 (대화상자) |
| CharShapeBold | - | 진하게 (Alt+L) |
| CharShapeCenterline | - | 취소선 |
| CharShapeEmboss | - | 양각 |
| CharShapeEngrave | - | 음각 |
| CharShapeHeight | - | 글자 크기 (포커스 이동) |
| CharShapeHeightDecrease | - | 크기 작게 (Alt+Shift+R) |
| CharShapeHeightIncrease | - | 크기 크게 (Alt+Shift+E) |
| CharShapeItalic | - | 이탤릭 (Alt+Shift+I) |
| CharShapeNextFaceName | - | 다음 글꼴 (Alt+Shift+F) |
| CharShapeNormal | - | 보통 모양 (Alt+Shift+C) |
| CharShapeOutline | - | 외곽선 |
| CharShapePrevFaceName | - | 이전 글꼴 (Alt+Shift+G) |
| CharShapeShadow | - | 그림자 |
| CharShapeSpacing | - | 자간 (포커스 이동) |
| CharShapeSpacingDecrease | - | 자간 좁게 (Alt+Shift+N) |
| CharShapeSpacingIncrease | - | 자간 넓게 (Alt+Shift+W) |
| CharShapeSubscript | - | 아래첨자 (Alt+Shift+S) |
| CharShapeSuperscript | - | 위첨자 (Alt+Shift+P) |
| CharShapeSuperSubscript | - | 첨자 토글 (위→아래→보통) |
| CharShapeTextColorBlack | - | 글자색: 검정 |
| CharShapeTextColorBlue | - | 글자색: 파랑 |
| CharShapeTextColorBluish | - | 글자색: 청록 |
| CharShapeTextColorGreen | - | 글자색: 초록 |
| CharShapeTextColorRed | - | 글자색: 빨강 |
| CharShapeTextColorViolet | - | 글자색: 자주 |
| CharShapeTextColorWhite | - | 글자색: 흰색 |
| CharShapeTextColorYellow | - | 글자색: 노랑 |
| CharShapeUnderline | - | 밑줄 (Alt+Shift+U) |
| CharShapeWidth | - | 장평 (포커스 이동) |
| CharShapeWidthDecrease | - | 장평 좁게 (Alt+Shift+J) |
| CharShapeWidthIncrease | - | 장평 넓게 (Alt+Shift+K) |
| Hyperlink | HyperLink | 하이퍼링크 (삽입/수정) |

#### 2.4.5 문단 서식 (27개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| ParagraphShape | ParaShape | 문단 모양 (대화상자) |
| ParagraphShapeAlignCenter | - | 가운데 정렬 |
| ParagraphShapeAlignDistribute | - | 배분 정렬 |
| ParagraphShapeAlignDivision | - | 나눔 정렬 |
| ParagraphShapeAlignJustify | - | 양쪽 정렬 |
| ParagraphShapeAlignLeft | - | 왼쪽 정렬 |
| ParagraphShapeAlignRight | - | 오른쪽 정렬 |
| ParagraphShapeDecreaseLeftMargin | - | 왼쪽 여백 줄이기 |
| ParagraphShapeDecreaseLineSpacing | - | 줄 간격 줄이기 |
| ParagraphShapeDecreaseMargin | - | 양쪽 여백 줄이기 |
| ParagraphShapeDecreaseRightMargin | - | 오른쪽 여백 줄이기 |
| ParagraphShapeIncreaseLeftMargin | - | 왼쪽 여백 키우기 |
| ParagraphShapeIncreaseLineSpacing | - | 줄 간격 넓히기 |
| ParagraphShapeIncreaseMargin | - | 양쪽 여백 키우기 |
| ParagraphShapeIncreaseRightMargin | - | 오른쪽 여백 키우기 |
| ParagraphShapeIndentAtCaret | - | 첫 줄 내어쓰기 |
| ParagraphShapeIndentNegative | - | 첫 줄 한 글자 내어쓰기 |
| ParagraphShapeIndentPositive | - | 첫 줄 한 글자 들여쓰기 |
| ParagraphShapeProtect | - | 문단 보호 |
| ParagraphShapeWithNext | - | 다음 문단과 함께 |
| ParaNumberBullet | ParaShape | 문단번호/글머리표 |
| ParaNumberBulletLevelDown | - | 번호/글머리표 한 수준 아래 |
| ParaNumberBulletLevelUp | - | 번호/글머리표 한 수준 위 |
| PutBullet | - | 글머리표 달기 |
| PutNewParaNumber | ParaShape* | 문단번호 새 번호 시작 |
| PutOutlineNumber | - | 개요번호 달기 |
| PutParaNumber | - | 문단번호 달기 |

#### 2.4.6 표 조작 (50개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| TableCreate | TableCreation | 표 만들기 |
| TableAppendRow | - | 줄 추가 |
| TableInsertUpperRow | - | 위쪽 줄 삽입 |
| TableInsertLowerRow | - | 아래쪽 줄 삽입 |
| TableInsertLeftColumn | - | 왼쪽 칸 삽입 |
| TableInsertRightColumn | - | 오른쪽 칸 삽입 |
| TableInsertRowColumn | TableInsertLine | 줄/칸 삽입 |
| TableDeleteRow | - | 줄 지우기 |
| TableDeleteColumn | - | 칸 지우기 |
| TableDeleteCell | - | 셀 삭제 |
| TableDeleteRowColumn | TableDeleteLine | 줄/칸 지우기 |
| TableSubtractRow | - | 표 줄 삭제 |
| TableMergeCell | - | 셀 합치기 |
| TableSplitCell | TableSplitCell | 셀 나누기 |
| TableSplitCellCol2 | TableSplitCell | 셀 칸 나누기 |
| TableSplitCellRow2 | TableSplitCell | 셀 줄 나누기 |
| TableDistributeCellHeight | - | 셀 높이 같게 |
| TableDistributeCellWidth | - | 셀 너비 같게 |
| TableStringToTable | TableStrToTbl | 문자열을 표로 |
| TablePropertyDialog | ShapeObject | 표 고치기 |
| TableCellBlock | - | 셀 블록 |
| TableCellBlockCol | - | 셀 블록 (칸) |
| TableCellBlockRow | - | 셀 블록 (줄) |
| TableCellBlockExtend | - | 셀 블록 연장 (F5+F5) |
| TableCellBlockExtendAbs | - | 셀 블록 연장 (Shift+F5) |
| TableLeftCell | - | 셀 이동: 왼쪽 |
| TableRightCell | - | 셀 이동: 오른쪽 |
| TableRightCellAppend | - | 셀 이동: 오른쪽 이어서 |
| TableUpperCell | - | 셀 이동: 위 |
| TableLowerCell | - | 셀 이동: 아래 |
| TableColBegin | - | 셀 이동: 열 시작 |
| TableColEnd | - | 셀 이동: 열 끝 |
| TableColPageDown | - | 셀 이동: 페이지 다운 |
| TableColPageUp | - | 셀 이동: 페이지 업 |
| TableResizeDown | - | 셀 크기 변경: 아래 |
| TableResizeUp | - | 셀 크기 변경: 위 |
| TableResizeLeft | - | 셀 크기 변경: 왼쪽 |
| TableResizeRight | - | 셀 크기 변경: 오른쪽 |
| TableResizeCellDown | - | 셀 크기 변경: 셀 아래 |
| TableResizeCellUp | - | 셀 크기 변경: 셀 위 |
| TableResizeCellLeft | - | 셀 크기 변경: 셀 왼쪽 |
| TableResizeCellRight | - | 셀 크기 변경: 셀 오른쪽 |
| TableResizeExDown | - | 셀 크기 변경: 아래 (비블록) |
| TableResizeExUp | - | 셀 크기 변경: 위 (비블록) |
| TableResizeExLeft | - | 셀 크기 변경: 왼쪽 (비블록) |
| TableResizeExRight | - | 셀 크기 변경: 오른쪽 (비블록) |
| TableResizeLineDown | - | 셀 크기 변경: 선 아래 |
| TableResizeLineUp | - | 셀 크기 변경: 선 위 |
| TableResizeLineLeft | - | 셀 크기 변경: 선 왼쪽 |
| TableResizeLineRight | - | 셀 크기 변경: 선 오른쪽 |

#### 2.4.7 셀 서식 (6개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| CellBorder | CellBorderFill | 셀 테두리 |
| CellBorderFill | CellBorderFill | 셀 테두리/배경 |
| CellFill | CellBorderFill | 셀 배경 |
| CellZoneBorder | CellBorderFill | 셀 테두리 (여러 셀) |
| CellZoneBorderFill | CellBorderFill | 셀 테두리/배경 (여러 셀) |
| CellZoneFill | CellBorderFill | 셀 배경 (여러 셀) |

#### 2.4.8 검색/치환 (8개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| FindDlg | FindReplace | 찾기 대화상자 |
| ReplaceDlg | FindReplace | 찾아 바꾸기 대화상자 |
| ForwardFind | FindReplace* | 앞으로 찾기 |
| BackwardFind | FindReplace* | 뒤로 찾기 |
| RepeatFind | FindReplace* | 다시 찾기 |
| ReverseFind | FindReplace* | 거꾸로 찾기 |
| ExecReplace | FindReplace* | 바꾸기 실행 |
| AllReplace | FindReplace* | 모두 바꾸기 |

#### 2.4.9 개체 조작 (46개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| DrawObjCreatorArc | - | 호 그리기 |
| DrawObjCreatorEllipse | - | 원 그리기 |
| DrawObjCreatorLine | - | 선 그리기 |
| DrawObjCreatorRectangle | - | 사각형 그리기 |
| DrawObjCreatorTextBox | - | 글상자 |
| ModifyCtrl | - | 컨트롤 고치기 |
| ModifyFieldClickhere | InsertFieldTemplate | 누름틀 정보 고치기 |
| ModifyFillProperty | - | 채우기 속성 고치기 |
| ModifyHyperlink | HyperLink | 하이퍼링크 고치기 |
| ModifyLineProperty | - | 선/테두리 속성 고치기 |
| ModifyShapeObject | - | 개체 속성 고치기 |
| PictureInsertDialog | - | 그림 넣기 대화상자 |
| ShapeObjDialog | ShapeObject | 개체 환경설정 |
| ShapeObjAlignBottom | - | 아래로 정렬 |
| ShapeObjAlignCenter | - | 가운데 정렬 |
| ShapeObjAlignHeight | - | 높이 맞춤 |
| ShapeObjAlignHorzSpacing | - | 가로 균등 배분 |
| ShapeObjAlignLeft | - | 왼쪽 정렬 |
| ShapeObjAlignMiddle | - | 중간 정렬 |
| ShapeObjAlignRight | - | 오른쪽 정렬 |
| ShapeObjAlignSize | - | 폭/높이 맞춤 |
| ShapeObjAlignTop | - | 위로 정렬 |
| ShapeObjAlignVertSpacing | - | 세로 균등 배분 |
| ShapeObjAlignWidth | - | 폭 맞춤 |
| ShapeObjAttachCaption | - | 캡션 넣기 |
| ShapeObjAttachTextBox | - | 글상자로 만들기 |
| ShapeObjBringForward | - | 앞으로 |
| ShapeObjBringInFrontOfText | - | 글 앞으로 |
| ShapeObjBringToFront | - | 맨 앞으로 |
| ShapeObjCtrlSendBehindText | - | 글 뒤로 |
| ShapeObjDetachCaption | - | 캡션 없애기 |
| ShapeObjDetachTextBox | - | 글상자 속성 없애기 |
| ShapeObjHorzFlip | - | 좌우 뒤집기 |
| ShapeObjHorzFlipOrgState | - | 좌우 뒤집기 원상태 |
| ShapeObjLock | - | 개체 잠금 |
| ShapeObjMoveDown | - | 키로 아래 이동 |
| ShapeObjMoveLeft | - | 키로 왼쪽 이동 |
| ShapeObjMoveRight | - | 키로 오른쪽 이동 |
| ShapeObjMoveUp | - | 키로 위 이동 |
| ShapeObjNextObject | - | 다음 개체 이동 (Tab) |
| ShapeObjPrevObject | - | 이전 개체 이동 (Shift+Tab) |
| ShapeObjResizeDown | - | 크기 조절: 아래 |
| ShapeObjResizeLeft | - | 크기 조절: 왼쪽 |
| ShapeObjResizeRight | - | 크기 조절: 오른쪽 |
| ShapeObjResizeUp | - | 크기 조절: 위 |
| ShapeObjSendBack | - | 뒤로 |
| ShapeObjSendToBack | - | 맨 뒤로 |
| ShapeObjTableSelCell | - | 표에서 첫 셀 선택 |
| ShapeObjTextBoxEdit | - | 글상자 편집모드 진입 |
| ShapeObjUngroup | - | 그룹 풀기 |
| ShapeObjUnlockAll | - | 개체 잠금 해제 |
| ShapeObjVertFlip | - | 상하 뒤집기 |
| ShapeObjVertFlipOrgState | - | 상하 뒤집기 원상태 |

#### 2.4.10 문서 관리 (4개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| DocSummaryInfo | SummaryInfo | 문서 요약 정보 |
| DocumentInfo | DocumentInfo* | 문서 상세 정보 |
| FileSetSecurity | FileSetSecurity* | 배포용 문서 설정 |
| SpellingCheck | SpellingCheck | 맞춤법 검사 |

#### 2.4.11 페이지 설정 (3개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| PageSetup | SecDef | 편집 용지 설정 |
| PageNumPos | PageNumPos | 쪽 번호 위치 |
| PageHiding | PageHiding | 감추기 |

#### 2.4.12 머리말/꼬리말 (1개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| HeaderFooter | HeaderFooter | 머리말/꼬리말 |

#### 2.4.13 뷰 설정 (3개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| ViewZoomFitPage | - | 화면 확대: 페이지 맞춤 |
| ViewZoomFitWidth | - | 화면 확대: 폭 맞춤 |
| ViewZoomNormal | - | 화면 확대: 정상 |

#### 2.4.14 편집 제어 (10개)

| Action ID | SetID | 설명 |
|-----------|-------|------|
| Cancel | - | ESC (취소) |
| Close | - | 현재 리스트 닫기 |
| CloseEx | - | 현재 리스트 닫기 (확장, Shift+Esc) |
| Erase | - | 지우기 |
| Redo | - | 다시 실행 |
| Undo | - | 되살리기 |
| ToggleOverwrite | - | 삽입/덮어쓰기 전환 |
| Print | - | 인쇄 |
| ReplaceAction | - | 액션 대체 |
| Hyperlink | HyperLink | 하이퍼링크 |

#### 2.4.15 Action Table 카테고리별 통계

| 카테고리 | 수 | ParameterSet 필요 | 비고 |
|----------|----|--------------------|------|
| 커서이동 | 51 | 0 | 모두 Run() 직접 호출 |
| 선택확장 | 36 | 0 | 모두 Run() 직접 호출 |
| 텍스트편집 | 29 | 5 | InsertText, InsertFile, InsertFieldTemplate 등 |
| 글자서식 | 33 | 2 | CharShape, Hyperlink |
| 문단서식 | 27 | 3 | ParagraphShape, ParaNumberBullet 등 |
| 표조작 | 50 | 7 | TableCreate, TableSplitCell 등 |
| 셀서식 | 6 | 6 | 모두 CellBorderFill 사용 |
| 검색치환 | 8 | 8 | 모두 FindReplace 사용 |
| 개체조작 | 53 | 4 | ShapeObjDialog, ModifyHyperlink 등 |
| 문서관리 | 4 | 4 | 모두 ParameterSet 사용 |
| 페이지설정 | 3 | 3 | 모두 ParameterSet 사용 |
| 머리말/꼬리말 | 1 | 1 | HeaderFooter |
| 뷰설정 | 3 | 0 | 모두 Run() 직접 호출 |
| 편집제어 | 10 | 1 | 대부분 Run() 직접 호출 |
| **합계** | **314** | **44** | |

### 2.5 ParameterSet Types (50개)

#### 2.5.1 핵심 서식 관련 (7개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 1 | CharShape | 글자 모양 | 63 | - |
| 2 | ParaShape | 문단 모양 | 33 | - |
| 3 | BorderFill | 테두리/배경 | 27 | - |
| 4 | BorderFillExt | 테두리/배경 확장 | 33 | BorderFill |
| 5 | CellBorderFill | 셀 테두리/배경 | 38 | BorderFillExt |
| 6 | BulletShape | 글머리표 모양 | 11 | - |
| 7 | NumberingShape | 문단 번호 모양 | 50+ | - |

#### 2.5.2 표/셀 관련 (5개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 8 | Table | 표 | 11+ | ShapeObject |
| 9 | Cell | 셀 | 15+ | ListProperties |
| 10 | TableCreation | 표 생성 | 13 | - |
| 11 | TableSplitCell | 셀 나누기 | 5 | - |
| 12 | ListProperties | 서브 리스트 속성 | 7 | - |

#### 2.5.3 그리기 개체 관련 (9개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 13 | ShapeObject | 개체 공통 속성 | 40+ | - |
| 14 | DrawFillAttr | 채우기 속성 | 24 | - |
| 15 | DrawImageAttr | 그림 속성 | 17 | - |
| 16 | DrawLineAttr | 선 속성 | 12 | - |
| 17 | DrawLayout | 레이아웃 | 3 | - |
| 18 | DrawRotate | 회전 | 7 | - |
| 19 | DrawShadow | 그림자 | 5 | - |
| 20 | DrawShear | 기울이기 | 2 | - |
| 21 | DrawArcType | 호 테두리 모양 | 2 | - |

#### 2.5.4 페이지/구역 관련 (6개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 22 | SecDef | 구역 속성 | 32 | - |
| 23 | PageDef | 용지 설정 | 13 | - |
| 24 | PageBorderFill | 쪽 테두리/배경 | 35+ | BorderFill |
| 25 | PageHiding | 감추기 | 1 | - |
| 26 | PageNumPos | 쪽 번호 위치 | 6 | - |
| 27 | PageNumCtrl | 페이지 번호 | 1 | - |

#### 2.5.5 검색/치환 관련 (1개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 28 | FindReplace | 찾기/바꾸기 | 23 | - |

#### 2.5.6 머리말/꼬리말/각주 (3개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 29 | HeaderFooter | 머리말/꼬리말 | 3 | - |
| 30 | FootnoteShape | 각주 모양 | 17 | - |
| 31 | EndnoteShape | 미주 모양 | 17 | - |

#### 2.5.7 필드/문서마당 (1개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 32 | InsertFieldTemplate | 문서마당 정보 | 6 | - |

#### 2.5.8 문서 정보 (3개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 33 | SummaryInfo | 문서 요약 | 24 | - |
| 34 | DocumentInfo | 문서 상세 | 17 | - |
| 35 | ViewProperties | 뷰 속성 | 6 | - |

#### 2.5.9 보안/삽입 관련 (4개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 36 | FileSetSecurity | 배포용 문서 | 3 | - |
| 37 | InsertText | 텍스트 삽입 | 1 | - |
| 38 | InsertFile | 파일 삽입 | 7 | - |
| 39 | HyperLink | 하이퍼링크 | 5 | - |

#### 2.5.10 위치/단 관련 (2개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 40 | ListParaPos | 커서 위치 | 3 | - |
| 41 | ColDef | 단 정의 | 11 | - |

#### 2.5.11 기타 (9개)

| # | SetID | 설명 | 아이템 수 | 상속 |
|---|-------|------|----------|------|
| 42 | Caption | 캡션 속성 | 4 | - |
| 43 | CodeTable | 문자표 | 1 | - |
| 44 | CtrlData | 컨트롤 데이터 | 1 | - |
| 45 | EngineProperties | 환경 설정 | 7 | - |
| 46 | MemoShape | 메모 모양 | 7 | - |
| 47 | SpellingCheck | 맞춤법 | 2 | - |
| 48 | TabDef | 탭 정의 | 3 | - |
| 49 | TableDeleteLine | 줄/칸 삭제 | 1 | - |
| 50 | TableInsertLine | 줄/칸 삽입 | 2 | - |

#### 2.5.12 ParameterSet 상속 관계

```
BorderFill
├── BorderFillExt
│   └── CellBorderFill
└── PageBorderFill

ListProperties
└── Cell

ShapeObject
└── Table
```

### 2.6 Supporting Objects

#### 2.6.1 Action Object

| 구분 | 이름 | 설명 |
|------|------|------|
| Property | ActID | 액션 ID |
| Property | SetID | ParameterSet ID |
| Method | CreateSet() | ParameterSet 생성 |
| Method | GetDefault(set) | 기본값 설정 |
| Method | PopupDialog(set) | 대화상자 표시 |
| Method | Execute(set) | 실행 |
| Method | Run() | 간편 실행 |

**Action 실행 패턴**:
```javascript
// 패턴 1: Run() — ParameterSet 불필요한 액션
HwpCtrl.Run("CharShapeBold");

// 패턴 2: CreateAction + Execute — ParameterSet 필요한 액션
var act = HwpCtrl.CreateAction("CharShape");
var set = act.CreateSet();
act.GetDefault(set);
set.SetItem("TextColor", 0xFF0000);
act.Execute(set);
```

#### 2.6.2 CtrlCode Object

| 구분 | 이름 | 설명 |
|------|------|------|
| Property | CtrlCh | 컨트롤 문자 |
| Property | CtrlID | 컨트롤 ID (e.g., "tbl", "gso") |
| Property | Next | 다음 컨트롤 |
| Property | Prev | 이전 컨트롤 |
| Property | Properties | 컨트롤 속성 (ParameterSet) |
| Property | UserDesc | 사용자 설명 |
| Method | GetAnchorPos(relTo) | 앵커 위치 |

#### 2.6.3 ParameterSet Object

| 구분 | 이름 | 설명 |
|------|------|------|
| Property | Count | 아이템 수 |
| Property | IsSet | Set인지 여부 |
| Property | SetID | ParameterSet ID |
| Method | Clone() | 복제 |
| Method | CreateItemArray(id, count) | 아이템 배열 생성 |
| Method | CreateItemSet(id, setid) | 하위 ParameterSet 생성 |
| Method | GetInterSection(set) | 교집합 |
| Method | IsEquivalent(set) | 동등 비교 |
| Method | Item(id) | 아이템 값 읽기 |
| Method | ItemExist(id) | 아이템 존재 여부 |
| Method | Merge(set) | 병합 |
| Method | RemoveAll() | 전체 삭제 |
| Method | RemoveItem(id) | 아이템 삭제 |
| Method | SetItem(id, value) | 아이템 값 설정 |

#### 2.6.4 ParameterArray Object

| 구분 | 이름 | 설명 |
|------|------|------|
| Property | Count | 배열 크기 |
| Property | IsSet | Set인지 여부 |
| Method | Clone() | 복제 |
| Method | Copy(src) | 복사 |
| Method | Item(index) | 값 읽기 |
| Method | SetItem(index, value) | 값 설정 |

### 2.7 이벤트 시스템

| 이벤트 타입 | 값 | 설명 | 콜백 파라미터 |
|------------|-----|------|-------------|
| OnMouseLButtonDown | 0 | 왼쪽 마우스 다운 | x, y |
| OnMouseLButtonUp | 1 | 왼쪽 마우스 업 | x, y |
| OnScroll | 2 | 스크롤 | - |

```javascript
HwpCtrl.AddEventListener(0, function(x, y) {
    console.log('Click: ' + x + ', ' + y);
});
```

---

## 3. rhwp vs 웹기안기 호환성 매핑표

### 3.1 아키텍처 차이 요약

호환 레이어 설계에 앞서, 두 시스템의 근본적인 아키텍처 차이를 이해해야 한다.

| 관점 | 한컴 웹기안기 (HwpCtrl) | rhwp |
|------|------------------------|------|
| 편집 패러다임 | **커서 기반** — 커서를 이동하고 현재 위치에서 편집 | **좌표 기반** — section/para/offset 좌표로 직접 접근 |
| 동작 실행 | Action 시스템 (CreateAction → Execute) | 직접 메서드 호출 |
| 파라미터 전달 | ParameterSet (Item/SetItem 패턴) | JSON 문자열 |
| 서식 조회 | Property (CharShape, ParaShape) | getCharPropertiesAt() JSON 반환 |
| 서버 의존 | 서버에서 HWP 파일 관리, Open/SaveAs가 서버 경로 | WASM 로컬 처리, Uint8Array 입출력 |
| UI 제어 | ShowToolBar, ShowRibbon 등 내장 UI | UI 없음 (개발자가 자유 구현) |

**호환 레이어의 핵심 과제**: JavaScript 레이어에서 커서 상태를 관리하고, HwpCtrl의 커서 기반 API를 rhwp의 좌표 기반 API로 변환한다.

```
HwpCtrl.MovePos(2)                    // 문서 시작으로 이동
HwpCtrl.Run("CharShapeBold")          // 현재 위치에 Bold 적용

↓ 호환 레이어 변환 ↓

cursorState = { sec: 0, para: 0, pos: 0 }  // JS에서 커서 상태 관리
rhwp.applyCharFormat(0, 0, startOff, endOff, '{"bold":true}')  // 좌표로 변환
```

### 3.2 rhwp 현재 WASM API (50개 + HwpViewer 8개)

| # | 카테고리 | API 이름 (JS) | 설명 |
|---|---------|--------------|------|
| 1 | 문서 로딩 | new(data) | HWP 바이트 로드 |
| 2 | 문서 로딩 | createEmpty() | 빈 문서 생성 |
| 3 | 렌더링 | renderPageSvg(pageNum) | SVG 렌더링 |
| 4 | 렌더링 | renderPageHtml(pageNum) | HTML 렌더링 |
| 5 | 렌더링 | renderPageCanvas(pageNum) | Canvas 명령 수 반환 |
| 6 | 렌더링 | renderPageToCanvas(pageNum, canvas) | Canvas 직접 렌더링 |
| 7 | 문서 정보 | pageCount() | 페이지 수 |
| 8 | 문서 정보 | getDocumentInfo() | 문서 정보 JSON |
| 9 | 문서 정보 | getPageInfo(pageNum) | 페이지 정보 JSON |
| 10 | 문서 정보 | getPageTextLayout(pageNum) | 텍스트 레이아웃 JSON |
| 11 | 문서 정보 | getPageControlLayout(pageNum) | 컨트롤 레이아웃 JSON |
| 12 | 표시 설정 | setDpi(dpi) / getDpi() | DPI 설정/조회 |
| 13 | 표시 설정 | setFallbackFont(path) / getFallbackFont() | 대체 폰트 설정/조회 |
| 14 | 표시 설정 | setShowParagraphMarks(enabled) | 문단부호 표시 |
| 15 | 텍스트 편집 | insertText(sec, para, offset, text) | 텍스트 삽입 |
| 16 | 텍스트 편집 | deleteText(sec, para, offset, count) | 텍스트 삭제 |
| 17 | 텍스트 편집 | insertTextInCell(...) | 셀 텍스트 삽입 |
| 18 | 텍스트 편집 | deleteTextInCell(...) | 셀 텍스트 삭제 |
| 19 | 텍스트 편집 | splitParagraph(sec, para, offset) | 문단 분할 (Enter) |
| 20 | 텍스트 편집 | mergeParagraph(sec, para) | 문단 병합 (BS at start) |
| 21 | 표 구조 | insertTableRow(sec, para, ctrl, row, below) | 행 삽입 |
| 22 | 표 구조 | insertTableColumn(sec, para, ctrl, col, right) | 열 삽입 |
| 23 | 표 구조 | mergeTableCells(sec, para, ctrl, sr, sc, er, ec) | 셀 병합 |
| 24 | 표 구조 | splitTableCell(sec, para, ctrl, row, col) | 셀 분할 |
| 25 | 서식 조회 | getCharPropertiesAt(sec, para, offset) | 글자 속성 JSON |
| 26 | 서식 조회 | getCellCharPropertiesAt(...) | 셀 글자 속성 JSON |
| 27 | 서식 조회 | getParaPropertiesAt(sec, para) | 문단 속성 JSON |
| 28 | 서식 조회 | getCellParaPropertiesAt(...) | 셀 문단 속성 JSON |
| 29 | 서식 적용 | findOrCreateFontId(name) | 폰트 ID 조회/생성 |
| 30 | 서식 적용 | applyCharFormat(sec, para, start, end, json) | 글자 서식 적용 |
| 31 | 서식 적용 | applyCharFormatInCell(...) | 셀 글자 서식 적용 |
| 32 | 서식 적용 | applyParaFormat(sec, para, json) | 문단 서식 적용 |
| 33 | 서식 적용 | applyParaFormatInCell(...) | 셀 문단 서식 적용 |
| 34 | 클립보드 | hasInternalClipboard() | 클립보드 존재 여부 |
| 35 | 클립보드 | getClipboardText() | 클립보드 텍스트 |
| 36 | 클립보드 | clearClipboard() | 클립보드 초기화 |
| 37 | 클립보드 | copySelection(sec, sPara, sOff, ePara, eOff) | 선택 복사 |
| 38 | 클립보드 | copySelectionInCell(...) | 셀 선택 복사 |
| 39 | 클립보드 | copyControl(sec, para, ctrl) | 컨트롤 복사 |
| 40 | 클립보드 | pasteInternal(sec, para, offset) | 내부 붙여넣기 |
| 41 | 클립보드 | pasteInternalInCell(...) | 셀 내부 붙여넣기 |
| 42 | HTML 변환 | exportSelectionHtml(sec, sPara, sOff, ePara, eOff) | HTML 내보내기 |
| 43 | HTML 변환 | exportSelectionInCellHtml(...) | 셀 HTML 내보내기 |
| 44 | HTML 변환 | exportControlHtml(sec, para, ctrl) | 컨트롤 HTML |
| 45 | HTML 변환 | pasteHtml(sec, para, offset, html) | HTML 붙여넣기 |
| 46 | HTML 변환 | pasteHtmlInCell(...) | 셀 HTML 붙여넣기 |
| 47 | 문서 내보내기 | exportHwp() | HWP 바이너리 내보내기 |
| 48 | 문서 내보내기 | convertToEditable() | 배포용→편집 변환 |
| 49 | 뷰어 | HwpViewer.new(document) | 뷰어 생성 |
| 50 | 뷰어 | HwpViewer.updateViewport(sx, sy, w, h) | 뷰포트 갱신 |
| 51 | 뷰어 | HwpViewer.setZoom(zoom) | 줌 설정 |
| 52 | 뷰어 | HwpViewer.visiblePages() | 가시 페이지 |
| 53 | 뷰어 | HwpViewer.pendingTaskCount() | 대기 작업 수 |
| 54 | 뷰어 | HwpViewer.pageCount() | 페이지 수 |
| 55 | 뷰어 | HwpViewer.renderPageSvg(pageNum) | SVG 렌더링 |
| 56 | 뷰어 | HwpViewer.renderPageHtml(pageNum) | HTML 렌더링 |

### 3.3 HwpCtrl Properties 호환성 매핑 (18개)

| # | Property | 등급 | rhwp 매핑 | 호환 레이어 구현 |
|---|----------|------|----------|----------------|
| 1 | PageCount | A | pageCount() | 직접 매핑 |
| 2 | CharShape | B | getCharPropertiesAt() | JSON→ParameterSet 변환 래퍼 |
| 3 | ParaShape | B | getParaPropertiesAt() | JSON→ParameterSet 변환 래퍼 |
| 4 | CellShape | B | getCellCharPropertiesAt() + getCellParaPropertiesAt() | 합성 래퍼 |
| 5 | IsEmpty | B | getDocumentInfo() | 결과 파싱으로 판단 |
| 6 | Version | B | getDocumentInfo() | version 필드 추출 |
| 7 | ViewProperties | B | HwpViewer.setZoom() | 줌 관련 ParameterSet 변환 |
| 8 | EditMode | B | convertToEditable() | 편집/읽기전용/배포용 상태 래퍼 관리 |
| 9 | ReadOnlyMode | B | convertToEditable() | EditMode와 연동 |
| 10 | ScrollPosInfo | B | HwpViewer.updateViewport() | 스크롤 위치 변환 |
| 11 | CurFieldState | C | - | 필드 시스템 신규 구현 필요 |
| 12 | CurSelectedCtrl | C | - | 커서/선택 시스템 신규 구현 필요 |
| 13 | HeadCtrl | C | - | 컨트롤 순회 API 신규 구현 필요 |
| 14 | IsModified | C | - | 문서 변경 추적 신규 구현 필요 |
| 15 | LastCtrl | C | - | 컨트롤 순회 API 신규 구현 필요 |
| 16 | ParentCtrl | C | - | 컨트롤 순회 API 신규 구현 필요 |
| 17 | SelectionMode | C | - | 선택 모드 관리 신규 구현 필요 |
| 18 | EngineProperties | X | - | rhwp 자체 설정 체계, 스텁 처리 |

**Properties 등급 분포**: A=1, B=9, C=7, X=1

### 3.4 HwpCtrl Methods 호환성 매핑 (67개)

#### 3.4.1 등급 A — 직접 매핑 (4개)

| HwpCtrl Method | rhwp API | 호환 레이어 |
|---------------|----------|-----------|
| Open(path, format, ...) | new(data) | fetch(path)→bytes→new(bytes) 래퍼 |
| OpenDocument(path, ...) | new(data) | Open과 동일 |
| SaveAs(fileName, ...) | exportHwp() | exportHwp()→download(bytes) 래퍼 |
| SaveDocument(fileName, ...) | exportHwp() | SaveAs와 동일 |

#### 3.4.2 등급 B — 변환 매핑 (17개)

| HwpCtrl Method | rhwp API | 변환 내용 |
|---------------|----------|----------|
| Clear(option) | createEmpty() | 옵션에 따라 저장 후 초기화 |
| GetPageText(pageNo, option) | getPageTextLayout(pageNum) | 레이아웃 JSON에서 텍스트만 추출 |
| GetTextFile("HWP", ...) | exportHwp() | bytes→Base64 변환 |
| SetTextFile(data, "HWP", ...) | new(data) | Base64→bytes→새 문서 로드 |
| InsertCtrl("tbl", tbset) | insertTableRow/Column 등 | 표 생성 시 행열 삽입 조합 |
| GetTableCellAddr(type) | getPageControlLayout() | 레이아웃 JSON에서 셀 좌표 추출 |
| GetViewStatus() | HwpViewer.updateViewport() | 뷰포트 정보 변환 |
| CreatePageImage(file, pgno, fmt) | renderPageSvg/Canvas() | SVG/Canvas→이미지 변환 |
| CreatePageImageEx(...) | renderPageToCanvas() | Canvas에 렌더링 후 toBlob() |
| GetCtrlHorizontalOffset(ctrl, relTo) | getPageControlLayout() | 레이아웃 JSON에서 좌표 추출 |
| GetCtrlVerticalOffset(ctrl, relTo) | getPageControlLayout() | 레이아웃 JSON에서 좌표 추출 |
| InsertPicture(path, ...) | pasteHtml() | `<img>` HTML을 통한 삽입 (부분적) |
| PrintDocument() | renderPageToCanvas() | Canvas 렌더링 후 window.print() |
| KeyIndicator() | getDocumentInfo() + 커서상태 | 현재 위치 정보 합성 |
| SelectText(spara, spos, epara, epos) | copySelection() | 좌표 기반 선택으로 변환 |
| GetSelectedPos() | (커서 상태) | JS 레이어의 선택 상태 반환 |
| GetSelectedPosBySet(sset, eset) | (커서 상태) | ParameterSet 형식으로 변환 |

#### 3.4.3 등급 C — 신규 구현 필요 (30개)

| HwpCtrl Method | 미구현 분야 | 구현 필요사항 |
|---------------|-----------|-------------|
| GetText() | 텍스트 스캔 | InitScan/GetText/ReleaseScan 스캔 메커니즘 구현 |
| GetTextBySet(text) | 텍스트 스캔 | ParameterSet 반환 방식 |
| InitScan(...) | 텍스트 스캔 | 범위 기반 텍스트 순회 |
| ReleaseScan() | 텍스트 스캔 | 스캔 리소스 해제 |
| GetHeadingString() | 문단번호 | 현재 문단의 번호/글머리표 문자열 |
| GetPos() / SetPos() | 커서 시스템 | 커서 위치 관리 |
| GetPosBySet() / SetPosBySet() | 커서 시스템 | ParameterSet 기반 커서 |
| MovePos(moveID, ...) | 커서 시스템 | 28+ 이동 타입 구현 |
| MoveToField(field, ...) | 필드 시스템 | 필드 탐색 및 이동 |
| MoveToFieldEx(field, ...) | 필드 시스템 | 확장 필드 이동 |
| GetMousePos(Xrelto, Yrelto) | 이벤트 시스템 | 마우스 좌표→문서 좌표 변환 |
| CreateField(direction, memo, name) | 필드 시스템 | 필드(누름틀) 생성 |
| FieldExist(field) | 필드 시스템 | 필드 존재 확인 |
| GetFieldList(number, option) | 필드 시스템 | 필드 목록 조회 |
| GetFieldText(fieldlist) | 필드 시스템 | 필드 텍스트 읽기 |
| PutFieldText(fieldlist, textlist) | 필드 시스템 | 필드 텍스트 설정 |
| GetCurFieldName(option) | 필드 시스템 | 현재 필드 이름 |
| SetCurFieldName(fieldname, ...) | 필드 시스템 | 필드 이름 설정 |
| ModifyFieldProperties(field, ...) | 필드 시스템 | 필드 속성 수정 |
| RenameField(oldname, newname) | 필드 시스템 | 필드 이름 변경 |
| SetFieldViewOption(option) | 필드 시스템 | 필드 표시 옵션 |
| InsertBackgroundPicture(...) | 이미지 | 배경 이미지 삽입 |
| DeleteCtrl(ctrl) | 컨트롤 관리 | 컨트롤 삭제 |
| Insert(path, ...) | 문서 삽입 | 다른 문서 끼워넣기 |
| InsertDocument(path, ...) | 문서 삽입 | 간소화 버전 |
| CreateAction(actionID) | Action 시스템 | Action 객체 생성 |
| CreateSet(setid) | Action 시스템 | ParameterSet 생성 |
| Run(actionID, ...) | Action 시스템 | 액션 실행 |
| ReplaceAction(old, new) | Action 시스템 | 액션 대체 |
| AddEventListener(type, listener) | 이벤트 시스템 | 마우스/스크롤 이벤트 |

#### 3.4.4 등급 D — 아키텍처 차이 (2개)

| HwpCtrl Method | 차이점 | 마이그레이션 영향 |
|---------------|--------|----------------|
| LockCommand(actionID, lock) | Action 시스템 의존 | Action 시스템 전체 설계 필요 |
| IsCommandLock(actionID) | Action 시스템 의존 | 위와 동일 |

#### 3.4.5 등급 X — 스텁 처리 (14개)

| HwpCtrl Method | 사유 | 처리 |
|---------------|------|------|
| ShowToolBar(show) | rhwp 자체 UI | 빈 함수 |
| ShowStatusBar(show) | rhwp 자체 UI | 빈 함수 |
| ShowRibbon(show) | rhwp 자체 UI | 빈 함수 |
| SetToolBar(option, id) | rhwp 자체 UI | 빈 함수 |
| ShowVerticalScroll(show) | rhwp 자체 UI | 빈 함수 |
| ShowHorizontalScroll(show) | rhwp 자체 UI | 빈 함수 |
| ShowCaret(show) | JS에서 캐럿 직접 관리 | 빈 함수 |
| SetFieldViewOption(option) | 필드 표시 옵션 | 빈 함수 |
| SolarToLunar(y, m, d) | 달력 변환 (문서와 무관) | JS 라이브러리로 대체 |
| SolarToLunarBySet(y, m, d) | 달력 변환 | JS 라이브러리로 대체 |
| LunarToSolar(y, m, d, leap) | 달력 변환 | JS 라이브러리로 대체 |
| LunarToSolarBySet(y, m, d, leap) | 달력 변환 | JS 라이브러리로 대체 |
| IsSpellCheckCompleted() | 맞춤법 미지원 | false 반환 |
| GetTextFile("JSON"/"HTML"/...) | 비-HWP 형식 | 지원 형식만 처리, 나머지 빈 반환 |

**Methods 등급 분포**: A=4, B=17, C=30, D=2, X=14 (합계 67)

### 3.5 Action Table 호환성 매핑 (314개)

#### 3.5.1 카테고리별 등급 분석

| 카테고리 | 수 | 등급 | rhwp 매핑 | 호환 레이어 구현 방안 |
|----------|-----|------|----------|---------------------|
| 커서이동 | 51 | C→B | - | JS 커서 상태 관리 + getPageTextLayout()로 좌표 계산. 호환 레이어에서 커서 위치를 추적하여 MovePos의 28+ 타입을 모두 JS로 구현 |
| 선택확장 | 36 | C→B | copySelection() | JS에서 선택 범위를 추적하고, 결과를 copySelection/applyCharFormat 등에 전달 |
| 텍스트편집 | 29 | B/C | insertText(), deleteText(), splitParagraph(), mergeParagraph(), pasteHtml() | 기본 편집(삽입/삭제/분할/병합) 매핑 가능. 각주/미주/필드 삽입은 C등급 |
| 글자서식 | 33 | B/C | applyCharFormat() | CharShape 대화상자는 C. 개별 속성(Bold, Italic 등)은 applyCharFormat()으로 B 매핑. 색상 단축키(8개)도 B |
| 문단서식 | 27 | B/C | applyParaFormat() | ParagraphShape 대화상자는 C. 정렬(6종), 여백/간격 조절은 applyParaFormat()으로 B 매핑 |
| 표조작 | 50 | B/C | insertTableRow(), insertTableColumn(), mergeTableCells(), splitTableCell() | 행열 삽입/삭제/병합/분할은 B. 셀 이동/크기 변경/셀 블록은 C |
| 셀서식 | 6 | C | - | 셀 테두리/배경 설정 API 신규 구현 필요 |
| 검색치환 | 8 | C | - | 검색/치환 엔진 전체 신규 구현 필요 |
| 개체조작 | 53 | C | - | 그리기 개체 생성/편집/정렬/순서 전체 신규 구현 |
| 문서관리 | 4 | B/C | getDocumentInfo(), exportHwp() | SummaryInfo, DocumentInfo는 B. FileSetSecurity는 C |
| 페이지설정 | 3 | C | - | 편집 용지, 쪽 번호, 감추기 설정 신규 구현 필요 |
| 머리말꼬리말 | 1 | C | - | 머리말/꼬리말 편집 신규 구현 필요 |
| 뷰설정 | 3 | B | HwpViewer.setZoom() | 줌 관련 3종 매핑 가능 |
| 편집제어 | 10 | C | - | Undo/Redo 신규 구현 필요. Cancel, Close는 JS 이벤트 |

#### 3.5.2 Action별 상세 등급

**등급 B — 변환 매핑 가능 (약 75개)**

| 분류 | Action IDs | rhwp 매핑 |
|------|-----------|----------|
| 텍스트 삽입/삭제 | InsertText, InsertSpace, InsertTab, InsertNonBreakingSpace, InsertFixedWidthSpace, Delete, DeleteBack, DeleteWord, DeleteWordBack, DeleteLine, DeleteLineEnd, BreakPara, BreakLine | insertText(), deleteText(), splitParagraph() |
| 글자 서식 | CharShapeBold, CharShapeItalic, CharShapeUnderline, CharShapeCenterline, CharShapeEmboss, CharShapeEngrave, CharShapeOutline, CharShapeShadow, CharShapeSubscript, CharShapeSuperscript, CharShapeSuperSubscript, CharShapeNormal, CharShapeHeightIncrease, CharShapeHeightDecrease, CharShapeSpacingIncrease, CharShapeSpacingDecrease, CharShapeWidthIncrease, CharShapeWidthDecrease, CharShapeTextColorBlack~Yellow (8) | applyCharFormat(json) |
| 문단 서식 | ParagraphShapeAlignLeft/Center/Right/Justify/Distribute/Division (6), ParagraphShapeIncreaseLeftMargin 등 여백 (8), ParagraphShapeIndentPositive/Negative/AtCaret (3), ParagraphShapeIncreaseLineSpacing/Decrease (2) | applyParaFormat(json) |
| 표 구조 | TableInsertUpperRow, TableInsertLowerRow, TableInsertLeftColumn, TableInsertRightColumn, TableAppendRow, TableDeleteRow, TableDeleteColumn, TableMergeCell, TableSplitCell/Col2/Row2 (3) | insertTableRow/Column(), mergeTableCells(), splitTableCell() |
| 뷰 | ViewZoomFitPage, ViewZoomFitWidth, ViewZoomNormal | HwpViewer.setZoom() |

**등급 C — 신규 구현 필요 (약 220개)**

| 분류 | 수 | 신규 구현 내용 |
|------|-----|-------------|
| 커서이동 전체 | 51 | 커서 상태 관리 시스템 + 문서 구조 기반 위치 계산 |
| 선택확장 전체 | 36 | 선택 범위 관리 시스템 |
| 검색치환 전체 | 8 | 텍스트 검색 엔진 + UI |
| 개체조작 전체 | 53 | 그리기 개체 시스템 |
| 셀서식 전체 | 6 | 셀 테두리/배경 API |
| 필드/문서마당 | 3 | InsertFieldTemplate, ModifyFieldClickhere, DeleteField |
| 각주/미주/페이지 | 7 | InsertFootnote, InsertEndnote, BreakPage, BreakSection, BreakColumn, BreakColDef, PageSetup 등 |
| 편집제어 | 7 | Undo, Redo, Cancel, Close, CloseEx, ToggleOverwrite, Erase |
| 문단번호 | 5 | ParaNumberBullet, PutBullet, PutParaNumber, PutOutlineNumber, PutNewParaNumber |
| 대화상자 | 7 | CharShape, ParagraphShape, FindDlg, ReplaceDlg, PictureInsertDialog, TablePropertyDialog, ShapeObjDialog 등 |
| 기타 | ~40 | 표 셀 이동/크기 변경, 하이퍼링크, 문서관리 등 |

**등급 X — 스텁 처리 (약 19개)**

| Action ID | 사유 |
|-----------|------|
| Print | 브라우저 인쇄 API로 대체 |
| SpellingCheck | 맞춤법 기능 미지원 |
| InputCodeTable | 문자표 (OS IME로 대체) |
| DrawObjCreatorArc/Ellipse/Line/Rectangle/TextBox (5) | 그리기 도구 (P2) |
| ShapeObjLock/UnlockAll (2) | 개체 잠금 (P2) |
| DocSummaryInfo | 문서 정보 대화상자 (UI 전용) |
| 기타 UI 관련 액션 | 빈 함수 처리 |

### 3.6 ParameterSet 호환성 매핑 (50개)

호환 레이어에서 ParameterSet은 **JavaScript 객체**로 구현한다. `CreateSet()`, `Item()`, `SetItem()`, `ItemExist()` 등의 메서드를 가진 래퍼 클래스를 제공한다.

```javascript
// 호환 레이어의 ParameterSet 구현 예시
class RhwpParameterSet {
    constructor(setId) { this._setId = setId; this._items = {}; }
    get SetID() { return this._setId; }
    Item(id) { return this._items[id]; }
    SetItem(id, value) { this._items[id] = value; }
    ItemExist(id) { return id in this._items; }
    // ...
}
```

| # | SetID | 등급 | rhwp 매핑 | 비고 |
|---|-------|------|----------|------|
| 1 | CharShape | B | applyCharFormat(json) | 63개 아이템 → JSON 속성 변환 |
| 2 | ParaShape | B | applyParaFormat(json) | 33개 아이템 → JSON 속성 변환 |
| 3 | BorderFill | C | - | 테두리/배경 설정 API 필요 |
| 4 | BorderFillExt | C | - | BorderFill 확장 |
| 5 | CellBorderFill | C | - | 셀 테두리/배경 |
| 6 | BulletShape | C | - | 글머리표 |
| 7 | NumberingShape | C | - | 문단번호 |
| 8 | Table | B | getPageControlLayout() | 표 속성 조회 (부분) |
| 9 | Cell | B | getCellCharPropertiesAt() 등 | 셀 속성 조회 (부분) |
| 10 | TableCreation | B | insertTableRow/Column 조합 | 표 생성 (변환) |
| 11 | TableSplitCell | B | splitTableCell() | 셀 나누기 |
| 12 | ListProperties | C | - | 서브 리스트 |
| 13 | ShapeObject | C | - | 개체 공통 속성 |
| 14 | DrawFillAttr | C | - | 채우기 |
| 15 | DrawImageAttr | C | - | 그림 속성 |
| 16 | DrawLineAttr | C | - | 선 속성 |
| 17 | DrawLayout | C | - | 개체 레이아웃 |
| 18 | DrawRotate | C | - | 개체 회전 |
| 19 | DrawShadow | C | - | 개체 그림자 |
| 20 | DrawShear | C | - | 개체 기울이기 |
| 21 | DrawArcType | C | - | 호 유형 |
| 22 | SecDef | C | - | 구역 속성 |
| 23 | PageDef | C | - | 용지 설정 |
| 24 | PageBorderFill | C | - | 쪽 테두리 |
| 25 | PageHiding | C | - | 감추기 |
| 26 | PageNumPos | C | - | 쪽 번호 위치 |
| 27 | PageNumCtrl | C | - | 페이지 번호 |
| 28 | FindReplace | C | - | 찾기/바꾸기 |
| 29 | HeaderFooter | C | - | 머리말/꼬리말 |
| 30 | FootnoteShape | C | - | 각주 모양 |
| 31 | EndnoteShape | C | - | 미주 모양 |
| 32 | InsertFieldTemplate | C | - | 필드 정보 |
| 33 | SummaryInfo | B | getDocumentInfo() | 문서 요약 (부분) |
| 34 | DocumentInfo | B | getDocumentInfo() | 문서 상세 (부분) |
| 35 | ViewProperties | B | HwpViewer.setZoom() | 뷰 속성 |
| 36 | FileSetSecurity | C | convertToEditable() | 배포용 문서 (부분) |
| 37 | InsertText | B | insertText() | 텍스트 삽입 |
| 38 | InsertFile | C | - | 파일 삽입 |
| 39 | HyperLink | C | - | 하이퍼링크 |
| 40 | ListParaPos | B | (커서 상태) | 위치 정보 래퍼 |
| 41 | ColDef | C | - | 단 정의 |
| 42 | Caption | C | - | 캡션 |
| 43 | CodeTable | X | - | 문자표 (OS IME 대체) |
| 44 | CtrlData | C | - | 컨트롤 데이터 |
| 45 | EngineProperties | X | - | 엔진 설정 (rhwp 자체) |
| 46 | MemoShape | C | - | 메모 |
| 47 | SpellingCheck | X | - | 맞춤법 (미지원) |
| 48 | TabDef | C | - | 탭 정의 |
| 49 | TableDeleteLine | B | (행/열 삭제 조합) | 표 줄/칸 삭제 |
| 50 | TableInsertLine | B | insertTableRow/Column() | 표 줄/칸 삽입 |

**ParameterSet 등급 분포**: B=14, C=33, X=3

### 3.7 전체 호환성 요약

#### 3.7.1 등급별 통계

| 구성요소 | A | B | C | D | X | 합계 |
|----------|---|---|---|---|---|------|
| Properties | 1 | 9 | 7 | 0 | 1 | 18 |
| Methods | 4 | 17 | 30 | 2 | 14 | 67 |
| Actions | 0 | ~75 | ~220 | 0 | ~19 | 314 |
| ParameterSets | 0 | 14 | 33 | 0 | 3 | 50 |
| **합계** | **5** | **~115** | **~290** | **2** | **~37** | **449** |

#### 3.7.2 등급별 비율

```
A (직접 매핑)      ██                           1%   — 래퍼만 작성
B (변환 매핑)      ██████████████               26%  — 파라미터 변환 래퍼
C (신규 구현)      ██████████████████████████████████████  65%  — WASM 코어 구현 필요
D (아키텍처 차이)  █                             0.4% — 구조적 재설계
X (스텁 처리)      █████                         8%   — 빈 함수
```

#### 3.7.3 신규 구현 필요 기능 (등급 C) 분류

| 기능 영역 | 관련 API 수 | 핵심 구현 내용 |
|----------|-----------|-------------|
| **커서/위치 시스템** | 51+9+36 = 96 | 커서 상태 관리, 문서 구조 기반 위치 계산, 28+ 이동 타입 |
| **필드 시스템** | 10+3 = 13 | 누름틀/필드 생성, 조회, 수정, 탐색 |
| **Action/ParameterSet** | 5+14 = 19 | CreateAction/CreateSet/Run 패턴 JS 구현 |
| **검색/치환** | 8+1 = 9 | 텍스트 검색 엔진, 정규식, 서식 검색 |
| **개체 조작** | 53+9 = 62 | 그리기 개체, 이미지 삽입, 정렬, Z순서, 크기조절 |
| **Undo/Redo** | 2 | 명령 히스토리 시스템 |
| **페이지 설정** | 3+6 = 9 | 용지/구역/머리말/꼬리말/각주/미주 |
| **셀 서식** | 6+3 = 9 | 셀 테두리, 배경, 대각선 |
| **이벤트 시스템** | 1+3 = 4 | 마우스, 스크롤 이벤트 |
| **문서 삽입** | 2+1 = 3 | HWP 파일 끼워넣기 |
| **기타** | ~64 | 문단번호, 하이퍼링크, 표 셀 이동, 표 크기 변경 등 |

---

## 4. 미구현 기능 우선순위 (P0/P1/P2)

### 4.1 우선순위 기준

| 등급 | 기준 | 마이그레이션 영향 |
|------|------|-----------------|
| **P0 (필수)** | 공공기관 기안 시스템 핵심. 이것 없이는 마이그레이션 불가 | 80% 이상의 기존 코드가 의존 |
| **P1 (중요)** | 문서 편집 고급 기능. 대부분의 활용 시나리오에서 필요 | 50%~80% 코드가 의존 |
| **P2 (선택)** | 부가 기능. 없어도 기본 마이그레이션 가능 | 20% 미만 코드가 의존 |
| **X (스텁)** | 빈 함수로 처리 가능. 기능 없어도 오류 발생 안함 | 호출은 되나 결과 무시 가능 |

### 4.2 P0 — 필수 (마이그레이션 핵심)

| # | 기능 영역 | 관련 API | 구현 복잡도 | 사유 |
|---|----------|---------|-----------|------|
| 1 | **필드 시스템** | CreateField, FieldExist, GetFieldList, GetFieldText, PutFieldText, MoveToField, GetCurFieldName, SetCurFieldName, ModifyFieldProperties, RenameField, InsertFieldTemplate, ModifyFieldClickhere, DeleteField | 높음 | 공공기관 기안문의 **핵심 패턴**: Open→PutFieldText→SaveAs. 필드 없이는 마이그레이션 불가 |
| 2 | **커서/위치 시스템** | GetPos, SetPos, MovePos (28타입), MoveToField, GetPosBySet, SetPosBySet, 커서이동 Action 51개 | 높음 | 모든 편집 API가 커서 위치에 의존. 호환 레이어의 기반 인프라 |
| 3 | **Action/ParameterSet** | CreateAction, CreateSet, Run, Action.Execute, ParameterSet.Item/SetItem | 중간 | 서식 적용, 표 생성, 검색 등 대부분의 고급 기능이 Action 패턴 사용 |
| 4 | **Undo/Redo** | Undo, Redo | 높음 | 편집기 기본 기능. 사용자 경험에 필수 |
| 5 | **검색/치환** | ForwardFind, BackwardFind, AllReplace, FindDlg, ReplaceDlg, RepeatFind, ReverseFind, ExecReplace | 중간 | 기안문 템플릿 처리 및 일괄 수정에 활용 |
| 6 | **텍스트 스캔** | InitScan, GetText, GetTextBySet, ReleaseScan | 중간 | 문서 내용 일괄 읽기에 사용 |
| 7 | **선택/블록** | 선택확장 Action 36개, SelectText, GetSelectedPos | 중간 | 서식 적용 및 복사/붙여넣기의 전제 |

### 4.3 P1 — 중요 (편집기 고급 기능)

| # | 기능 영역 | 관련 API | 구현 복잡도 | 사유 |
|---|----------|---------|-----------|------|
| 1 | **이미지 삽입** | InsertPicture, InsertBackgroundPicture, PictureInsertDialog | 중간 | 기안문에 로고/직인 삽입 |
| 2 | **머리말/꼬리말** | HeaderFooter Action, HeaderFooter ParameterSet | 중간 | 공문서 양식에 필수 |
| 3 | **페이지 설정** | PageSetup (SecDef), PageNumPos, PageHiding | 중간 | 문서 레이아웃 |
| 4 | **셀 서식** | CellBorder/Fill/Zone 6개 Action, CellBorderFill ParameterSet | 중간 | 표 서식 고급 기능 |
| 5 | **각주/미주** | InsertFootnote, InsertEndnote, FootnoteShape | 중간 | 학술/법률 문서 |
| 6 | **문서 삽입** | Insert, InsertDocument, InsertFile | 중간 | 문서 합치기 |
| 7 | **하이퍼링크** | InsertHyperlink, ModifyHyperlink, Hyperlink | 낮음 | 링크 삽입/수정 |
| 8 | **컨트롤 관리** | InsertCtrl, DeleteCtrl, HeadCtrl, LastCtrl, ParentCtrl, CurSelectedCtrl | 중간 | 표/이미지 등 컨트롤 조작 |
| 9 | **배포용 문서** | FileSetSecurity, EditMode (배포 관련) | 중간 | 공문서 배포 |
| 10 | **이벤트 시스템** | AddEventListener (마우스, 스크롤) | 낮음 | 대화형 편집 |

### 4.4 P2 — 선택 (부가 기능)

| # | 기능 영역 | 관련 API | 구현 복잡도 | 사유 |
|---|----------|---------|-----------|------|
| 1 | 그리기 도구 | DrawObjCreator* (5), ShapeObj* (40+) | 높음 | 도형 그리기 기능 |
| 2 | 문단번호/글머리표 | ParaNumberBullet, PutBullet, PutParaNumber, PutOutlineNumber | 중간 | 자동 번호매기기 |
| 3 | 문자표 | InputCodeTable | 낮음 | OS IME로 대체 가능 |
| 4 | 단 정의 | BreakColDef, BreakColumn, ColDef | 중간 | 다단 레이아웃 |
| 5 | 달력 변환 | SolarToLunar/LunarToSolar 4개 | 낮음 | JS 라이브러리 대체 |
| 6 | 표 크기 조절 | TableResize* (20+) | 중간 | 마우스 드래그 크기 조절 |
| 7 | 문서 정보 | DocSummaryInfo, DocumentInfo | 낮음 | 메타데이터 대화상자 |

### 4.5 X — 스텁 처리 (빈 함수)

| 기능 | 관련 API | 처리 |
|------|---------|------|
| UI 제어 | ShowToolBar, ShowStatusBar, ShowRibbon, SetToolBar, ShowVerticalScroll, ShowHorizontalScroll, ShowCaret | `function() {}` |
| 맞춤법 | SpellingCheck, IsSpellCheckCompleted | `false` 반환 |
| 엔진 설정 | EngineProperties | 빈 ParameterSet |

---

## 5. 마이그레이션 시나리오별 필요 API

### 5.1 시나리오 정의

공공기관의 한컴 웹기안기 활용 패턴을 4가지 시나리오로 분류한다.

### 5.2 시나리오 1: 기안문 자동 생성 (가장 빈번)

**설명**: 서버에서 HWP 템플릿을 열고, 필드에 데이터를 넣고, 저장/다운로드

```javascript
// 전형적인 공공기관 기안문 생성 코드
HwpCtrl.Open(templatePath, "HWP", "");
HwpCtrl.PutFieldText("기안자", "홍길동");
HwpCtrl.PutFieldText("기안일자", "2026-02-12");
HwpCtrl.PutFieldText("제목", "업무 협조전");
HwpCtrl.PutFieldText("본문", longText);
HwpCtrl.SaveAs("output.hwp", "HWP", "download:true");
```

**필요 API (12개)**:

| API | 우선순위 | rhwp 등급 |
|-----|---------|----------|
| Open | P0 | A |
| SaveAs | P0 | A |
| PutFieldText | P0 | C |
| GetFieldText | P0 | C |
| FieldExist | P0 | C |
| GetFieldList | P0 | C |
| MoveToField | P0 | C |
| CreateField | P0 | C |
| Clear | P0 | B |
| Run("InsertText") | P0 | C (Action 시스템) |
| CharShape (property) | P1 | B |
| PageCount | P0 | A |

**마이그레이션 가능 여부**: 필드 시스템(C등급) 구현 후 가능. **핵심 블로커: 필드 시스템**

### 5.3 시나리오 2: 문서 뷰어 (단순)

**설명**: HWP 문서를 웹에서 읽기 전용으로 표시

```javascript
HwpCtrl.Open(docPath, "HWP", "");
var pageCount = HwpCtrl.PageCount;
for (var i = 0; i < pageCount; i++) {
    HwpCtrl.CreatePageImage("page" + i + ".png", i, "png");
}
```

**필요 API (5개)**:

| API | 우선순위 | rhwp 등급 |
|-----|---------|----------|
| Open | P0 | A |
| PageCount | P0 | A |
| CreatePageImage | P1 | B |
| CreatePageImageEx | P1 | B |
| GetPageText | P1 | B |

**마이그레이션 가능 여부**: **즉시 가능**. 모든 API가 A 또는 B 등급

### 5.4 시나리오 3: 양식 편집 (중간)

**설명**: 사용자가 웹 편집기에서 기안문을 직접 편집

```javascript
HwpCtrl.Open(templatePath, "HWP", "");
// 필드 이동 후 편집
HwpCtrl.MoveToField("본문", true, true, true);
// 서식 적용
HwpCtrl.Run("CharShapeBold");
HwpCtrl.Run("ParagraphShapeAlignCenter");
// 표 삽입
var act = HwpCtrl.CreateAction("TableCreate");
var set = act.CreateSet();
set.SetItem("Rows", 3); set.SetItem("Cols", 4);
HwpCtrl.InsertCtrl("tbl", set);
// 이미지 삽입
HwpCtrl.InsertPicture(imgPath, true, 0, false, false, 0, 0, 0);
HwpCtrl.SaveAs("edited.hwp", "HWP", "download:true");
```

**필요 API (25개)**:

| API | 우선순위 | rhwp 등급 |
|-----|---------|----------|
| Open, SaveAs, Clear | P0 | A/B |
| 필드 시스템 10개 | P0 | C |
| MovePos, GetPos, SetPos | P0 | C |
| 커서이동 Actions | P0 | C |
| CharShape Actions (Bold, Italic 등) | P0 | B |
| ParagraphShape Actions (정렬 등) | P0 | B |
| CreateAction, CreateSet, Run | P0 | C |
| InsertCtrl, TableCreate | P1 | B/C |
| InsertPicture | P1 | C |
| Undo, Redo | P0 | C |
| SelectText, GetSelectedPos | P0 | C |

**마이그레이션 가능 여부**: 필드+커서+Action+Undo 구현 후 가능. **핵심 블로커: 커서/필드/Action 시스템**

### 5.5 시나리오 4: 완전 편집기 (드문)

**설명**: 한컴 웹기안기의 모든 기능을 대체하는 완전한 웹 편집기

**필요 API**: 전체 449개 (스텁 제외)

**마이그레이션 가능 여부**: 모든 P0+P1+P2 구현 필요. 장기 목표

### 5.6 시나리오별 커버리지 현황

| 시나리오 | 필요 API | 현재 매핑 가능 | 신규 구현 필요 | 커버리지 |
|----------|---------|-------------|-------------|---------|
| 문서 뷰어 | 5 | 5 (A+B) | 0 | **100%** |
| 기안문 자동 생성 | 12 | 3 (A) | 9 (C) | **25%** |
| 양식 편집 | 25 | 8 (A+B) | 17 (C) | **32%** |
| 완전 편집기 | 449 | ~120 (A+B) | ~290 (C) | **27%** |

---

## 6. rhwp 고유 강점 (웹기안기에 없는 기능)

### 6.1 아키텍처 강점

| 강점 | rhwp | 한컴 웹기안기 | 마이그레이션 가치 |
|------|------|------------|----------------|
| **서버 비의존** | WASM 로컬 처리, 서버 불필요 | 서버 필수 (HNCG 파서 서버) | 인프라 비용 절감, 오프라인 가능 |
| **오픈소스** | 소스 공개, 커스터마이징 자유 | 폐쇄형 상용 제품 | 벤더 종속 탈피 |
| **AI 통합 최적화** | 좌표 기반 API → AI 함수 호출 적합 | 커서 기반 → AI 부적합 | 문서 자동 생성/분석에 유리 |
| **다중 렌더링** | SVG, HTML, Canvas 동시 지원 | Canvas 단일 | 용도별 최적 렌더링 선택 |
| **경량 배포** | WASM 바이너리 단독 배포 | 서버 클러스터 필요 | 설치/운영 간편 |

### 6.2 기능 강점

| 기능 | rhwp | 한컴 웹기안기 |
|------|------|------------|
| HTML 클립보드 | exportSelectionHtml, pasteHtml (서식 유지 변환) | 제한적 |
| 좌표 직접 편집 | section/para/offset 직접 접근 | 커서 이동 필수 |
| 셀 단위 API | insertTextInCell, applyCharFormatInCell 등 셀 전용 API | 커서를 셀로 이동 후 편집 |
| 배포용→편집 변환 | convertToEditable() 원클릭 | 암호 입력 필요 |
| 페이지 텍스트 레이아웃 | getPageTextLayout() 글자별 좌표 | 없음 |
| 컨트롤 레이아웃 | getPageControlLayout() 표/이미지 좌표 | 없음 |

### 6.3 전략적 포지셔닝

```
한컴 웹기안기                          rhwp
┌──────────────────┐              ┌──────────────────┐
│ "사람이 UI로 편집"  │              │ "AI가 API로 제어"   │
│                  │              │                  │
│ 커서 기반 편집      │              │ 좌표 기반 직접 접근   │
│ 서버 의존          │              │ WASM 로컬 처리      │
│ 폐쇄형 상용        │              │ 오픈소스 자유        │
│ UI 제어 API 풍부   │              │ 데이터 처리 API 풍부  │
└──────────────────┘              └──────────────────┘
        ↓ 호환 레이어 ↓                    ↓ 고유 강점 ↓
  기존 코드 무변경 마이그레이션        AI 문서 자동화 신규 시장
```

---

## 7. 호환 레이어 구현 로드맵

### 7.1 Phase 구성

전체 구현을 4개 Phase로 나누어 점진적으로 마이그레이션 커버리지를 확대한다.

### 7.2 Phase 1: 기안문 자동 생성 지원 (최우선)

**목표**: 시나리오 1 (기안문 자동 생성) 100% 커버리지
**타깃 커버리지**: 25% → 100% (시나리오 1 기준)

| 구현 항목 | 관련 API | 예상 규모 |
|----------|---------|----------|
| 호환 레이어 기본 프레임워크 | HwpCtrl 객체, ParameterSet 클래스 | JS 래퍼 |
| 파일 I/O 래퍼 | Open, SaveAs, Clear | JS 래퍼 (fetch+new+exportHwp) |
| **필드 시스템 구현** | CreateField, FieldExist, GetFieldList, GetFieldText, PutFieldText, MoveToField, GetCurFieldName, SetCurFieldName, ModifyFieldProperties, RenameField | WASM 코어 + JS 래퍼 |
| 텍스트 스캔 기본 | InitScan, GetText, ReleaseScan | WASM 코어 |
| 등급 X 스텁 | 모든 UI 제어, 맞춤법 등 | JS 빈 함수 |

**완료 시 가능한 코드**:
```javascript
// rhwp.js 로드 후 기존 코드 그대로 실행
HwpCtrl.Open("template.hwp", "HWP", "", function() {
    HwpCtrl.PutFieldText("기안자", "홍길동");
    HwpCtrl.PutFieldText("제목", "업무 협조전");
    HwpCtrl.SaveAs("output.hwp", "HWP", "download:true");
});
```

### 7.3 Phase 2: 기본 편집기 지원

**목표**: 시나리오 3 (양식 편집) 기본 커버리지
**타깃 커버리지**: 60% (시나리오 3 기준)

| 구현 항목 | 관련 API | 예상 규모 |
|----------|---------|----------|
| **커서 시스템** | GetPos, SetPos, MovePos, 커서이동 Actions | JS 커서 상태 관리 + WASM 위치 계산 |
| **선택 시스템** | SelectText, GetSelectedPos, 선택확장 Actions | JS 선택 범위 관리 |
| **Action 시스템** | CreateAction, CreateSet, Run, Execute | JS ParameterSet + Action 래퍼 |
| 글자 서식 Actions | CharShapeBold~CharShapeWidthDecrease (30+) | applyCharFormat 래핑 |
| 문단 서식 Actions | ParagraphShapeAlign* 등 (20+) | applyParaFormat 래핑 |
| **Undo/Redo** | Undo, Redo | WASM 명령 히스토리 |
| **검색/치환** | ForwardFind, BackwardFind, AllReplace 등 | WASM 검색 엔진 |

### 7.4 Phase 3: 고급 편집기 기능

**목표**: 시나리오 3 (양식 편집) 90%+ 커버리지

| 구현 항목 | 관련 API | 예상 규모 |
|----------|---------|----------|
| 이미지 삽입 | InsertPicture, InsertBackgroundPicture | WASM 이미지 핸들링 |
| 머리말/꼬리말 | HeaderFooter | WASM 편집 |
| 페이지 설정 | PageSetup (SecDef), PageNumPos | WASM 구역 설정 |
| 셀 서식 | CellBorder/Fill 6개 | WASM 셀 속성 |
| 각주/미주 | InsertFootnote, InsertEndnote | WASM 편집 |
| 하이퍼링크 | InsertHyperlink, ModifyHyperlink | WASM 링크 관리 |
| 컨트롤 관리 | InsertCtrl, DeleteCtrl, 컨트롤 순회 | WASM 컨트롤 API |
| 배포용 문서 | FileSetSecurity | WASM 보안 설정 |
| 이벤트 시스템 | AddEventListener | JS 이벤트 |

### 7.5 Phase 4: 완전 호환

**목표**: 시나리오 4 (완전 편집기) 95%+ 커버리지

| 구현 항목 | 관련 API | 예상 규모 |
|----------|---------|----------|
| 그리기 도구 | DrawObjCreator* 5개 + ShapeObj* 40+ | WASM 도형 엔진 |
| 문단번호/글머리표 | ParaNumberBullet 등 | WASM 번호 시스템 |
| 표 고급 기능 | TableResize* 20+, TableStringToTable 등 | WASM 표 엔진 확장 |
| 단 정의 | BreakColDef, BreakColumn, ColDef | WASM 다단 |
| 문서 합치기 | Insert, InsertDocument | WASM 문서 병합 |

### 7.6 Phase별 커버리지 목표

```
Phase 1 (기안문)    ████████                    시나리오1: 100%, 전체: ~30%
Phase 2 (기본편집)  ████████████████            시나리오3: 60%,  전체: ~50%
Phase 3 (고급편집)  ████████████████████████    시나리오3: 90%,  전체: ~75%
Phase 4 (완전호환)  ██████████████████████████████  시나리오4: 95%, 전체: ~95%
```

### 7.7 Phase 1 상세 구현 계획

Phase 1이 가장 중요하므로 구체적인 구현 계획을 제시한다.

#### 7.7.1 호환 레이어 JS 프레임워크

```javascript
// rhwp_compat.js — HwpCtrl 호환 래퍼
class HwpCtrl {
    constructor(wasmDocument) {
        this._doc = wasmDocument;
        this._cursor = { sec: 0, para: 0, pos: 0 };
        this._fields = new Map();  // 필드 캐시
    }

    // 등급 A: 직접 매핑
    get PageCount() { return this._doc.pageCount(); }

    Open(path, format, arg, callback) {
        fetch(path)
            .then(r => r.arrayBuffer())
            .then(buf => {
                this._doc = new HwpDocument(new Uint8Array(buf));
                this._scanFields();  // 필드 인덱스 구축
                if (callback) callback(true);
            });
    }

    SaveAs(fileName, format, arg, callback) {
        const bytes = this._doc.exportHwp();
        // arg에서 "download:true" 파싱
        if (arg && arg.includes("download:true")) {
            this._downloadBlob(bytes, fileName);
        }
        if (callback) callback(true);
    }

    // 등급 C: 필드 시스템 (WASM 코어 구현 필요)
    PutFieldText(fieldlist, textlist) { /* WASM 필드 API 호출 */ }
    GetFieldText(fieldlist) { /* WASM 필드 API 호출 */ }
    FieldExist(field) { /* WASM 필드 탐색 */ }

    // 등급 X: 스텁
    ShowToolBar(show) {}
    ShowRibbon(show) {}
    IsSpellCheckCompleted() { return false; }
}
```

#### 7.7.2 WASM 코어 필드 API 추가 (Rust)

```rust
// 추가 필요한 WASM API (src/wasm_api.rs)
#[wasm_bindgen]
impl HwpDocument {
    pub fn find_field(&self, name: &str) -> Option<String>;      // 필드 위치 JSON
    pub fn get_field_text(&self, name: &str) -> Option<String>;  // 필드 텍스트
    pub fn put_field_text(&mut self, name: &str, text: &str) -> Result<String, JsValue>;
    pub fn get_field_list(&self, option: u32) -> String;         // 필드 목록 JSON
    pub fn field_exist(&self, name: &str) -> bool;
    pub fn create_field(&mut self, sec: usize, para: usize, offset: usize,
                        name: &str, direction: &str) -> Result<String, JsValue>;
}
```

---

## 부록: 참조 문서 목록

| 문서 | 경로 |
|------|------|
| HwpCtrl API v2.4 | `mydocs/manual/hwpctl/hwpctl_API_v2.4.md` |
| Action Table v1.1 | `mydocs/manual/hwpctl/hwpctl_Action_Table__v1.1.md` |
| ParameterSet v1.2 | `mydocs/manual/hwpctl/hwpctl_ParameterSetID_Item_v1.2.md` |
| 한컴 개발자 포털 | https://developer.hancom.com/webhwp |
| 웹기안기 예제 | https://webhwpctrl-example.cloud.hancom.com/webhwp-example |
| webhwp 구현 분석 | `mydocs/feedback/webhwp_anal_001.md` |
| 아키텍처 비교 | `mydocs/tech/webhwp/parsing_comparison.md` |
| 프로젝트 비전 | `mydocs/tech/archive/project_vision.md` |
| 개발 로드맵 | `mydocs/tech/archive/dev_roadmap_20260324.md` |
