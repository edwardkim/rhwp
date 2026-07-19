/**
 * [Task #2327] 문서 변경 WasmBridge 메서드의 단일 권위 목록(레지스트리).
 *
 * studio 의 undo 는 executeOperation → CommandHistory 경유가 전제이지만 강제
 * 장치가 없어 기록이 옵트인이다 — 미기록 뮤테이션은 ① 해당 편집 undo 불가
 * ② redo 스택 미무효화 ③ 스냅샷 undo 의 전체 문서 복원에 동반 파괴, 3중
 * 위험을 만든다 (#2027/#2037/#2053/#2077 재발 계급).
 *
 * 이 레지스트리는 tests/mutation-routing-guard.test.ts 가 파싱해 저작 시점에
 * ① 브리지 신규/rename 뮤테이터의 분류 누락(양방향 드리프트)과 ② 뮤테이션
 * 표면 원장(BASELINE)의 증가를 차단한다. 브리지에 문서 변경 메서드를 추가하면
 * 두 목록 중 하나에 반드시 분류해야 한다.
 *
 * (초기 설계의 DEV 런타임 opDepth 가드는 kind:'record' 계약(드래그/이동/표
 * nudge — 뮤테이션을 record 전에 직접 적용)을 구분하지 못해 일상 편집마다
 * 오탐하고 warnedMethods 소진으로 진짜 미라우팅까지 침묵시켜, 저작 시점 소스
 * 가드만 남겼다. 자세한 근거: PR #2329 리뷰 스레드.)
 */

/** 문서 IR(직렬화 결과)을 바꾸는 WasmBridge 공개 메서드 전수. */
export const MUTATING_METHODS: readonly string[] = [
  // 쪽/구역/다단
  'setPageDef', 'setSectionDef', 'setSectionDefAll', 'setPageBorderFill', 'setColumnDef',
  // 본문 텍스트/문단
  'insertText', 'deleteText', 'deleteRange', 'splitParagraph', 'mergeParagraph',
  'insertPageBreak', 'insertColumnBreak', 'insertNewNumber', 'setNumberingRestart',
  // 셀 텍스트/문단
  'insertTextInCell', 'insertTextInCellDeferredPagination', 'deleteTextInCell',
  'deleteRangeInCell', 'insertTextInCellByPath', 'deleteTextInCellByPath',
  'splitParagraphInCell', 'mergeParagraphInCell', 'splitParagraphInCellByPath',
  'mergeParagraphInCellByPath',
  // 표 구조/속성
  'createTable', 'createTableEx', 'deleteTableControl', 'insertTableRow',
  'insertTableColumn', 'deleteTableRow', 'deleteTableColumn', 'mergeTableCells',
  'splitTableCell', 'splitTableCellInto', 'splitTableCellsInRange', 'resizeTableCells',
  'moveTableOffset', 'setTableProperties', 'setCellProperties', 'setCellZoneProperties',
  'pasteTableCellsTransposed', 'transposeTableCellsInPlace', 'pasteTableCellsTransposedAsTable',
  'evaluateTableFormula',
  // 그림/도형/수식 개체
  'insertPicture', 'assignPictureImage', 'setPictureProperties',
  'setHeaderFooterPictureProperties', 'setCellPicturePropertiesByPath',
  'setCellShapePropertiesByPath', 'deletePictureControl', 'deleteCellPictureControlByPath',
  'createShapeControl', 'setShapeProperties', 'deleteShapeControl', 'changeShapeZOrder',
  'groupShapes', 'ungroupShape', 'moveLineEndpoint', 'updateConnectorsInSection',
  'insertEquation', 'setEquationProperties', 'setNoteEquationProperties', 'deleteEquationControl',
  // 각주/미주
  'insertFootnote', 'insertEndnote', 'deleteFootnote', 'applyEndnoteShape',
  'insertTextInFootnote', 'deleteTextInFootnote', 'splitParagraphInFootnote',
  'mergeParagraphInFootnote', 'applyParaFormatInFootnote',
  // 붙여넣기
  'pasteInternal', 'pasteInternalInCell', 'pasteInternalInCellByPath', 'pasteControl',
  'pasteHtml', 'pasteHtmlInCell', 'pasteHtmlInCellByPath',
  // 글자/문단 모양
  'applyCharFormat', 'setCharShapeId', 'applyCharFormatInCell', 'setCharShapeIdInCell',
  'applyParaFormat', 'setParaShapeId', 'applyParaFormatInCell', 'setCellParaShapeId',
  // 스타일/번호 정의 (DocInfo 변이 포함)
  'updateStyle', 'updateStyleShapes', 'createStyle', 'deleteStyle', 'applyStyle',
  'applyCellStyle', 'createNumbering', 'ensureDefaultNumbering', 'ensureDefaultBullet',
  'findOrCreateFontId', 'findOrCreateFontIdForLang',
  // 머리말/꼬리말
  'createHeaderFooter', 'deleteHeaderFooter', 'toggleHideHeaderFooter',
  'insertTextInHeaderFooter', 'deleteTextInHeaderFooter', 'splitParagraphInHeaderFooter',
  'mergeParagraphInHeaderFooter', 'applyParaFormatInHf', 'insertFieldInHf', 'applyHfTemplate',
  // 필드/양식/찾아바꾸기/책갈피
  'setFieldValue', 'setFieldValueByName', 'removeFieldAt', 'insertClickHereField',
  'updateClickHereProps', 'setFormValue', 'setFormValueInCell',
  'replaceText', 'replaceOne', 'replaceAll',
  'addBookmark', 'deleteBookmark', 'renameBookmark',
  // lineseg 재계산 (#177 — 저장 lineseg 를 실제로 변경)
  'reflowLinesegs',
];

/**
 * 변이형 동사로 시작하지만 문서 IR 을 바꾸지 않는 메서드 — 드리프트 검사의
 * 명시 분류. (세션/렌더 상태, 캐럿 탐색, 수명주기)
 */
export const EXCLUDED_NON_DOCUMENT: readonly string[] = [
  'createNewDocument', // 수명주기 — 히스토리는 deactivate 에서 별도 초기화
  'clearLayerResourceCache', // 렌더 캐시
  'setShowParagraphMarks', 'setShowControlCodes', 'setShowTransparentBorders', // 표시 토글
  'setClipEnabled', // 렌더 클립 옵션
  'setActiveField', 'clearActiveField', // 편집 세션 상태 (직렬화 비대상)
  'moveVertical', 'moveVerticalByPath', // 캐럿 세로 탐색 (조회)
  'ensureParagraphStableIds', // 런타임 추적 id 부여
];

