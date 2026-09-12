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
  'setPageDef', 'setPageMargin', 'setSectionDef', 'setSectionDefAll', 'setPageBorderFill', 'setColumnDef',
  // 본문 텍스트/문단
  'insertText', 'replaceBodyTextLocal', 'deleteText', 'deleteRange', 'splitParagraph', 'mergeParagraph',
  'insertPageBreak', 'insertColumnBreak', 'insertNewNumber', 'setNumberingRestart',
  // 셀 텍스트/문단
  'insertTextInCell', 'insertTextInCellDeferredPagination', 'deleteTextInCell',
  'deleteTextInCellDeferredPagination', 'replaceTextInCellDeferredPagination',
  'deleteRangeInCell', 'insertTextInCellByPath', 'deleteTextInCellByPath', 'deleteRangeInCellByPath',
  'splitParagraphInCell', 'mergeParagraphInCell', 'splitParagraphInCellByPath',
  'mergeParagraphInCellByPath',
  // 표 구조/속성
  'createTable', 'createTableEx', 'deleteTableControl', 'insertTableRow', 'splitTable', 'mergeTableWithNext',
  'insertTableColumn', 'deleteTableRow', 'deleteTableColumn', 'mergeTableCells',
  'splitTableCell', 'splitTableCellInto', 'splitTableCellsInRange', 'resizeTableCells',
  'moveTableOffset', 'setTableProperties', 'setCellProperties', 'setCellZoneProperties',
  'applyCellBorderFillIds', 'removeBorderFillTails',
  'pasteTableCellsTransposed', 'transposeTableCellsInPlace', 'pasteTableCellsTransposedAsTable',
  'evaluateTableFormula',
  // 그림/도형/수식 개체
  'insertPicture', 'assignPictureImage', 'setPictureProperties',
  'setHeaderFooterPictureProperties', 'setCellPicturePropertiesByPath',
  'setCellShapePropertiesByPath', 'deletePictureControl', 'deleteCellPictureControlByPath',
  'createShapeControl', 'setShapeProperties', 'deleteShapeControl', 'changeShapeZOrder',
  'applyShapeZOrderPairs', // [#5769 후속] z 절대 대입 — SetZOrderCommand 의 undo/redo 경로
  'groupShapes', 'ungroupShape', 'moveLineEndpoint', 'updateConnectorsInSection',
  'insertEquation', 'setEquationProperties', 'setNoteEquationProperties', 'deleteEquationControl',
  // 차트 데이터 (#4694) — bin_data_content 슬롯 바이트 변이 (IR 무변경이지만 직렬화 결과가 바뀐다)
  'setChartData', 'setChartDataByIndex',
  // 각주/미주
  'insertFootnote', 'insertEndnote', 'deleteFootnote', 'applyEndnoteShape',
  'insertTextInFootnote', 'deleteTextInFootnote', 'splitParagraphInFootnote',
  'mergeParagraphInFootnote', 'applyParaFormatInFootnote',
  // 붙여넣기
  'pasteInternal', 'pasteInternalInCell', 'pasteInternalInCellByPath', 'pasteControl',
  'pasteHtml', 'pasteHtmlInCell', 'pasteHtmlInCellByPath', 'pasteHwpJson',
  // 글자/문단 모양
  'applyCharFormat', 'setCharShapeId', 'applyCharFormatInCell', 'applyCharFormatInCellByPath',
  'setCharShapeIdInCell', 'setCharShapeIdInCellByPath',
  'setCharShapeRuns', 'setCharShapeRunsInCellByPath',
  'applyParaFormat', 'setParaShapeId', 'applyParaFormatInCell', 'setCellParaShapeId',
  // 스타일/번호 정의 (DocInfo 변이 포함)
  'updateStyle', 'updateStyleShapes', 'createStyle', 'deleteStyle', 'applyStyle',
  'applyCellStyle', 'createNumbering', 'ensureDefaultNumbering', 'ensureDefaultBullet',
  'findOrCreateFontId', 'findOrCreateFontIdForLang',
  // 머리말/꼬리말
  'createHeaderFooter', 'deleteHeaderFooter', 'toggleHideHeaderFooter',
  'insertTextInHeaderFooter', 'deleteTextInHeaderFooter', 'replaceRangeInHeaderFooter',
  'splitParagraphInHeaderFooter', 'mergeParagraphInHeaderFooter',
  'applyCharFormatInHeaderFooter', 'applyParaFormatInHf', 'insertFieldInHf', 'applyHfTemplate',
  // 필드/양식/찾아바꾸기/책갈피
  'insertHyperlink', 'updateHyperlink', 'replaceHyperlinkText', 'removeHyperlink',
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
  // [#5769] 삭제 조각(fragment) API — capture·discard 는 IR 비변경(캡처·저장소 정리).
  // restoreDeleteFragment 는 IR 을 되살리는 변이지만 배선이 CommandHistory.undo 단일
  // 경로로 고정돼 있어(SnapshotCommand.undo 의 restoreSnapshot 과 같은 취급)
  // executeOperation 라우팅 강제 대상에서 제외한다. 히스토리 밖 직접 호출 금지.
  'captureDeleteRange', 'restoreDeleteFragment', 'discardDeleteFragment',
  // [#5769 Stage 4] 구역 raw 저널 API — capture·discard 는 IR 비변경.
  // restoreSectionRaw 는 passthrough 를 되살리지만 SetSectionPropsCommand.undo 단일
  // 경로로 고정돼 있어(restoreDeleteFragment 와 같은 취급) 제외한다. 히스토리 밖 직접 호출 금지.
  'captureSectionRaw', 'restoreSectionRaw', 'discardSectionRaw',
  // [#7002] 스냅샷 API — 위 두 저널의 선례이면서 정작 분류가 빠져 있었다
  // (이 파일의 restoreDeleteFragment 주석이 SnapshotCommand.undo 의 restoreSnapshot 을
  // 근거로 인용한다). save·discard 는 저장소 적재·해제라 IR 비변경이고,
  // restoreSnapshot 은 CommandHistory.undo 단일 경로로 고정돼 같은 취급이다.
  'saveSnapshot', 'restoreSnapshot', 'discardSnapshot',
  // [#7002] 그림 변환 저널 API — capture·discard 는 IR 비변경.
  // swapPictureTransform 은 IR 을 바꾸지만 두 경로 다 히스토리 항목을 만들지 않는다:
  // PictureTransformResizeCommand.execute/undo(Command 안) 와
  // PictureResizeJournal.cancel(드래그 취소·실패의 원상 복귀 — 되돌릴 편집이 없다).
  // 위 두 저널과 달리 '단일 경로 고정' 이 아니므로 근거를 따로 적는다.
  'capturePictureTransform', 'swapPictureTransform', 'discardPictureTransform',
  // [#7002] 지연 쪽나눔 진행 API — 조판은 파생 상태다. 문서 IR 을 바꾸지 않는다.
  'beginDeferredPagination', 'stepDeferredPagination',
  'flushDeferredPagination', 'cancelDeferredPagination',
  // [#7002] 직렬화 산출 — 저장 어댑터는 `prepare_hwp_export_snapshot`(document.rs:1598)
  // 이 뜬 **사본**에 적용되고 live 문서는 그대로다. 세 경로의 `&mut self` 는 실제로
  // 필요 없는 것이며(좁히는 것은 별건), 지금도 문서를 바꾸지 않는다.
  'exportHwp', 'exportHwpVerify', 'exportHwpWithPassword',
  // [#7002 · #4180] 저장 직전 캐럿 스탬프. doc_properties.caret_* 와 DocInfo raw_stream 을
  // surgical update 하므로 **저장 바이트를 바꾼다**. 그럼에도 제외인 이유는 편집이 아니라
  // 저장 흐름(onBeforeExport)의 일부이기 때문이다 — 되돌릴 사용자 편집이 없다.
  'setCaretPosition',
  // [#7002] 내부 클립보드 적재 — 문서는 읽기만 하고 복사본을 코어 버퍼에 담는다.
  'copySelection', 'copySelectionInCell', 'copySelectionInCellByPath',
  'copySelectionInHeaderFooter', 'copyControl', 'copyTableCellsTransposed',
  // [#7002] 조회 — 본문은 읽기뿐인데 `*_mut` 접근자를 거치느라 `&mut self` 가 됐다.
  'getCellCharPropertiesAtByPath', 'getCharShapeRunsInCellByPath',
];
