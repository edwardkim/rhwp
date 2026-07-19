# 타스크 43 수행계획서: 한컴 웹기안기 대응 기능정의서 작성

## 배경

한컴의 웹기안기(웹한글 기안기)는 프론트엔드 개발자가 JavaScript API(HwpCtrl)로 HWP 문서를 프로그래밍 방식으로 제어할 수 있는 제품이다. rhwp가 동등한 JavaScript API를 제공하면 한컴의 독과점 시장에 도전할 수 있다.

현재 rhwp는 WASM 기반 약 49개의 API를 보유하고 있으나, 웹기안기의 HwpCtrl API와 체계적으로 대비한 기능정의서가 없다.

## 목표

한컴 웹기안기 HwpCtrl API를 전수 분석하고, rhwp의 현재 API와 1:1 대비하여 Gap을 식별한 **기능정의서**를 작성한다. 이 문서는 향후 개발 우선순위 결정의 기초 자료가 된다.

## 분석 대상

### 한컴 웹기안기 HwpCtrl API

한컴 개발자 포털(developer.hancom.com/webhwp) 및 로컬 문서에서 확인된 API 전체 구성:

| 구성요소 | 설명 | 규모 |
|----------|------|------|
| HwpCtrl | 메인 문서 제어 인터페이스 | Properties 11 + Methods 53 = 64 |
| Action | 작업 실행 메커니즘 (ActID + SetID) | Properties 2 + Methods 5 = 7 |
| CtrlCode | 제어 코드 관리 | Properties 5 |
| ParameterSet | 파라미터 집합 처리 | Properties 3 + Methods 11 = 14 |
| ParameterArray | 배열 형식 파라미터 관리 | Methods 2 |
| Action Table | 실행 가능한 단위 액션 | 200+ 액션 ID |
| ParameterSet Table | 데이터 구조체 정의 | 30+ 세트 타입 |

로컬 참조 문서:
- `mydocs/manual/hwpctl/hwpctl_API_v2.4.md` — HwpCtrl API 전체 문서
- `mydocs/manual/hwpctl/hwpctl_Action_Table__v1.1.md` — 200+ Action ID 전체 목록
- `mydocs/manual/hwpctl/hwpctl_ParameterSetID_Item_v1.2.md` — 30+ ParameterSet 정의

### rhwp 현재 WASM API (49개 메서드)

| 카테고리 | 수 | 주요 메서드 |
|----------|---|-----------|
| 문서 로딩 | 2 | new, createEmpty |
| 렌더링 | 4 | renderPageSvg, renderPageHtml, renderPageCanvas, renderPageToCanvas |
| 문서 정보 | 5 | pageCount, getDocumentInfo, getPageInfo, getPageTextLayout, getPageControlLayout |
| 표시 설정 | 4 | setDpi, getDpi, setFallbackFont, setShowParagraphMarks |
| 텍스트 편집 | 4 | insertText, deleteText, splitParagraph, mergeParagraph |
| 표 구조 편집 | 4 | insertTableRow, insertTableColumn, mergeTableCells, splitTableCell |
| 셀 텍스트 편집 | 2 | insertTextInCell, deleteTextInCell |
| 서식 조회 | 4 | getCharPropertiesAt, getParaPropertiesAt + 셀 변형 |
| 서식 적용 | 5 | applyCharFormat, applyParaFormat + 셀 변형, findOrCreateFontId |
| 내부 클립보드 | 7 | copySelection, pasteInternal, copyControl + 셀 변형 등 |
| HTML 클립보드 | 6 | exportSelectionHtml, pasteHtml, exportControlHtml + 셀 변형 |
| 문서 내보내기 | 2 | exportHwp, convertToEditable |

## 산출물

| 문서 | 경로 | 설명 |
|------|------|------|
| 수행계획서 | `mydocs/plans/task_43.md` | 본 문서 |
| 구현 계획서 | `mydocs/plans/task_43_impl.md` | 단계별 진행 계획 |
| 기능정의서 | `mydocs/plans/task_43_feature_def.md` | 최종 산출물 |

## 작업 방식

이 타스크는 **문서 작성 타스크**이다. 코드 변경 없음.

1. 한컴 개발자 포털의 HwpCtrl API 문서를 상세 분석
2. rhwp의 현재 WASM API를 전수 조사 (src/wasm_api.rs)
3. 1:1 대비표 작성 및 Gap 식별
4. 우선순위 분류 및 기능정의서 완성

## 참조 자료

| 문서 | 경로 | 내용 |
|------|------|------|
| HwpCtrl API v2.4 | `mydocs/manual/hwpctl/hwpctl_API_v2.4.md` | HwpCtrl Properties/Methods, Action, ParameterSet 전체 문서 |
| Action Table v1.1 | `mydocs/manual/hwpctl/hwpctl_Action_Table__v1.1.md` | 200+ Action ID 전체 목록 및 예제 |
| ParameterSet v1.2 | `mydocs/manual/hwpctl/hwpctl_ParameterSetID_Item_v1.2.md` | 30+ ParameterSet 타입 및 아이템 정의 |
| 한컴 개발자 포털 | https://developer.hancom.com/webhwp | 웹기안기 공식 문서 |
| 웹기안기 예제 | https://webhwpctrl-example.cloud.hancom.com/webhwp-example | API 활용 예제 (React SPA) |
| webhwp 구현 분석 | `mydocs/feedback/webhwp_anal_001.md` | 한컴 웹HWP JS 번들 역공학 분석 |
| 아키텍처 비교 | `mydocs/tech/webhwp/parsing_comparison.md` | 한컴 webhwp vs rhwp 비교 |
| 프로젝트 비전 | `mydocs/tech/archive/project_vision.md` | 전략 방향 |
| 개발 로드맵 | `mydocs/tech/archive/dev_roadmap_20260324.md` | 제품화 로드맵 |
