/* tslint:disable */
/* eslint-disable */

/**
 * 한 번의 문서 내보내기 결과.
 *
 * 바이트와 content-loss 보고서가 같은 객체에 있어 다른 저장의 상태와 섞이지 않는다.
 * `takeBytes()`는 Rust 결과의 바이트 소유권을 한 번만 소비하며, 보고서는 그 전후 어느
 * 순서로든 읽을 수 있다. 바이트를 두 번 꺼내는 것은 명시적 오류다.
 */
export class DocumentExport {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 이번 산출물의 content-loss 보고서(JSON). `takeBytes()` 뒤에도 읽을 수 있다.
     */
    contentLoss(): string;
    /**
     * 아직 JS로 옮기지 않은 바이트를 소유하는지 반환한다.
     */
    hasBytes(): boolean;
    /**
     * 산출 바이트 소유권을 한 번 꺼낸다.
     */
    takeBytes(): Uint8Array;
}

/**
 * WASM에서 사용할 HWP 문서 래퍼
 *
 * 도메인 로직은 `DocumentCore`에 구현되어 있으며,
 * `Deref`/`DerefMut`를 통해 투명하게 접근한다.
 */
export class HwpDocument {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 책갈피 추가
     */
    addBookmark(sec: number, para: number, char_offset: number, name: string): string;
    /**
     * [#5959] 셀/zone border_fill_id 직접 대입 (undo·redo 전용).
     *
     * 스타일 테이블을 건드리지 않고 execute 의 변경 기록을 되돌린다.
     * json: `{"cells":[{"cellIdx":0,"id":3}],"zones":[{"startRow":..,"startCol":..,
     * "endRow":..,"endCol":..,"id":5}]}` — zone `id` 가 null 이면 그 범위의 zone 을
     * 제거한다. 반환: JSON `{"ok":true}`
     */
    applyCellBorderFillIds(section_idx: number, parent_para_idx: number, control_idx: number, json: string): string;
    /**
     * 스타일을 적용한다 (셀 내 문단).
     */
    applyCellStyle(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, style_id: number): string;
    /**
     * 글자 서식을 적용한다 (본문 문단).
     */
    applyCharFormat(sec_idx: number, para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    /**
     * 커서 좌표(list/para/pos)로 글자 서식을 건다 — 웹한글컨트롤 `Run("CharShape*")`.
     *
     * `endPos` 가 문단 길이를 넘으면 끝까지로 자른다. `pos` 는 코드 유닛이다.
     */
    applyCharFormatAtCursor(list_id: number, para_in_list: number, start_pos: number, end_pos: number, props_json: string): string;
    /**
     * 글자 서식을 적용한다 (셀 내 문단).
     */
    applyCharFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    applyCharFormatInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, start_offset: number, end_offset: number, props_json: string): string;
    /**
     * `applyCharFormatInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * startOffset, endOffset, props: object }`. `props` 는 글자 서식 JSON 객체(positional
     * 의 props_json 과 동일). positional 과 동일 동작.
     */
    applyCharFormatInCellEx(options_json: string): string;
    /**
     * 머리말/꼬리말 선택 범위에 글자 서식을 적용한다.
     */
    applyCharFormatInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, start_hf_para_idx: number, start_char_offset: number, end_hf_para_idx: number, end_char_offset: number, props_json: string): string;
    /**
     * 미주 모양을 적용한다.
     */
    applyEndnoteShape(section_idx: number, props_json: string): string;
    /**
     * 머리말/꼬리말 마당(템플릿)을 적용한다.
     */
    applyHfTemplate(section_idx: number, is_header: boolean, apply_to: number, template_id: number): string;
    applyParaFormat(sec_idx: number, para_idx: number, props_json: string): string;
    /**
     * 커서 좌표(list/para)로 문단 서식을 건다 — 웹한글컨트롤 `Run("ParagraphShape*")`.
     */
    applyParaFormatAtCursor(list_id: number, para_in_list: number, props_json: string): string;
    /**
     * 문단 서식을 적용한다 (셀 내 문단).
     */
    applyParaFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, props_json: string): string;
    /**
     * 각주/미주 내부 문단 속성 적용
     */
    applyParaFormatInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, props_json: string): string;
    /**
     * 머리말/꼬리말 문단에 문단 서식을 적용한다.
     */
    applyParaFormatInHf(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, props_json: string): string;
    /**
     * Shape z 순서 절대 대입(#5769 후속) — 속성쌍 커맨드의 undo/redo 경로.
     * pairs_json: `[{"ppi":N,"ci":N,"z":N},...]`
     * 반환: JSON `{"ok":true,"applied":N}`
     */
    applyShapeZOrderPairs(section_idx: number, pairs_json: string): string;
    /**
     * 스타일을 적용한다 (본문 문단).
     */
    applyStyle(sec_idx: number, para_idx: number, style_id: number): string;
    /**
     * Options JSON: {operation:{action,request},dryRun?:boolean}.
     * Preview prepares detached changes; it never mutates or saves the document.
     */
    applyTemplateOperation(options_json: string): string;
    /**
     * [Task #2230] 기존 Picture 컨트롤에 이미지를 지정한다 — 그림 미지정
     * placeholder(missing image 컨트롤)의 편집 뷰 그림 삽입.
     *
     * `cell_path_json` 이 빈 문자열 또는 `"[]"` 면 본문 문단의 컨트롤,
     * 그 외에는 셀/글상자 안 문단의 컨트롤을 대상으로 한다. 개체 틀 크기는
     * 유지되고(한컴 placeholder 는 틀에 그림을 맞춤) BinData 등록 규칙은
     * insertPicture 와 공유한다.
     *
     * 반환: `{"ok":true,"binDataId":<N>}`
     */
    assignPictureImage(section_idx: number, parent_para_idx: number, cell_path_json: string, control_idx: number, image_data: Uint8Array, natural_width_px: number, natural_height_px: number, extension: string): string;
    /**
     * 개체에 캡션을 붙인다 — 웹한글컨트롤 `Run("ShapeObjAttachCaption")`.
     */
    attachCaptionAt(para_in_list: number, control_index: number): string;
    /**
     * Batch 모드를 시작한다. 이후 Command 호출 시 paginate()를 건너뛴다.
     */
    beginBatch(): string;
    /**
     * 대형 표 continuation shadow job을 시작한다. 공개 페이지는 완료 전까지 유지된다.
     */
    beginDeferredPagination(fragment_budget: number): string;
    /**
     * 나누기 — 웹한글컨트롤 `Run("BreakPage"·"BreakColumn"·"BreakColDef"·"BreakSection")`.
     *
     * `kind` 는 `page`·`column`·`colDef`·`section`.
     */
    breakAtCursor(list_id: number, para_in_list: number, pos: number, kind: string): string;
    cancelDeferredPagination(): boolean;
    canvasMetricsActive(): boolean;
    /**
     * 삭제 직전 문단 범위 원본을 조각으로 보관한다 (#5769).
     *
     * 반드시 `deleteRangeNative` 호출 **전**에 불린다. 반환 조각 ID 는
     * `restoreDeleteFragment`/`discardDeleteFragment` 에 쓴다.
     */
    captureDeleteRange(section_idx: number, start_para: number, end_para: number): number;
    /**
     * 그림 리사이즈 전에 원본 변환만 보관한다.
     */
    capturePictureTransform(target_json: string): number;
    /**
     * 속성 변경 직전 구역 raw 스트림+봉인을 보관한다 (#5769 Stage 4).
     *
     * 반드시 `setSectionDef` 호출 **전**에 불린다. 반환 ID 는
     * `restoreSectionRaw`/`discardSectionRaw` 에 쓴다.
     */
    captureSectionRaw(section_idx: number): number;
    /**
     * Shape z-order 변경
     * operation: "front" | "back" | "forward" | "backward"
     * 반환: `{"ok":true,"zOrder":N,"moves":[{"ppi","ci","before","after"}...]}`
     * [#5769 후속] moves 는 실제로 대입된 (대상+교환 이웃) before/after 쌍이다 —
     * SetZOrderCommand 가 undo/redo 절대 복원 쌍으로 소비한다.
     */
    changeShapeZOrder(section_idx: number, parent_para_idx: number, control_idx: number, operation: string): string;
    /**
     * 활성 필드를 해제한다 (안내문 다시 표시).
     */
    clearActiveField(): void;
    /**
     * 내부 클립보드를 초기화한다.
     */
    clearClipboard(): void;
    /**
     * exact slot 하나의 명시 variable-font instance 요청을 제거한다.
     * 없는 요청의 clear는 멱등이며 다른 slot의 요청을 건드리지 않는다.
     */
    clearExactFontInstance(options_json: string): string;
    /**
     * 셀 블록이 덮은 칸들의 글을 비운다 — `Run("TableDeleteCell")`. 규약은 merge 와 같다.
     */
    clearTableCellsAtCursor(list_id: number, end_row: number, end_col: number): string;
    /**
     * 내부 클립보드에 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
     */
    clipboardHasControl(): boolean;
    collectCanvasMetricRequests(document: number, fonts: number): string;
    /**
     * 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다.
     *
     * 반환값: JSON `{"ok":true,"converted":true}` 또는 `{"ok":true,"converted":false}`
     */
    convertToEditable(): string;
    /**
     * 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
     *
     * [Task #1161] `cell_path_json` 이 빈 문자열/`"[]"` 면 본문, 그 외에는 셀/글상자
     * 경로(`[{"controlIndex","cellIndex","cellParaIndex"}, ...]`)의 컨트롤을 복사한다.
     */
    copyControl(section_idx: number, para_idx: number, cell_path_json: string, control_idx: number): string;
    /**
     * 선택 영역을 내부 클립보드에 복사한다.
     *
     * 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
     */
    copySelection(section_idx: number, start_para_idx: number, start_char_offset: number, end_para_idx: number, end_char_offset: number): string;
    /**
     * 표 셀 내부 선택 영역을 내부 클립보드에 복사한다.
     */
    copySelectionInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * 전체 cellPath가 가리키는 중첩 셀의 선택 영역을 내부 클립보드에 복사한다(#4272).
     */
    copySelectionInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * `copySelectionInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     */
    copySelectionInCellEx(options_json: string): string;
    /**
     * 머리말/꼬리말 선택 범위를 내부 클립보드에 복사한다.
     */
    copySelectionInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, start_hf_para_idx: number, start_char_offset: number, end_hf_para_idx: number, end_char_offset: number): string;
    /**
     * 선택 영역을 논리적 오프셋(`insertTextLogical` 과 같은 축)으로 받아 내부 클립보드에 복사한다.
     *
     * 각주·글자처럼 취급 개체 바로 뒤에서 시작한 선택은 그 개체를 담지 않는다 (#7444).
     * 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
     */
    copySelectionLogical(section_idx: number, start_para_idx: number, start_logical_offset: number, end_para_idx: number, end_logical_offset: number): string;
    /**
     * 선택된 표 셀 범위를 행/열 바꿈 복사용 내부 버퍼에 저장한다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     */
    copyTableCellsTransposed(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number): string;
    /**
     * 내장 템플릿에서 빈 문서를 생성한다.
     *
     * saved/blank2010.hwp를 WASM 바이너리에 포함하여 유효한 HWP 문서를 즉시 생성.
     * DocInfo raw_stream이 온전하므로 FIX-4 워크어라운드와 호환됨.
     */
    createBlankDocument(): string;
    /**
     * 빈 문서 생성 (테스트/미리보기용)
     *
     * 기본 A4 구역 1개 + 빈 문단 1개를 포함한다. 구역 0개 문서는 모든
     * 편집/조회 API가 "구역 인덱스 0 범위 초과"로 실패해 사용 불가하므로
     * 생성 직후 바로 편집 가능한 최소 구조를 보장한다 (#1386).
     *
     * 여기서 만든 문단은 **구역 정의·단 정의를 안 진다** — 실제 HWP 문서는 예외 없이 그
     * 둘을 첫 문단에 지므로 이 문서는 그 점에서 실물과 다르다. 한글 호환이 필요한 자리
     * (`Clear`)는 번들 템플릿을 쓰는 [`create_blank_document`](Self::create_blank_document)
     * 를 쓴다. 여기에 그 둘을 넣으면 `char_shapes` 자리가 16칸씩 밀려 기존 호출부가 깨진다.
     */
    static createEmpty(): HwpDocument;
    /**
     * 머리말/꼬리말 생성 (빈 문단 1개 포함)
     *
     * 반환: JSON `{"ok":true,"kind":"header/footer","applyTo":N,...}`
     */
    createHeaderFooter(section_idx: number, is_header: boolean, apply_to: number): string;
    /**
     * JSON으로 지정된 번호 형식으로 Numbering 정의를 생성한다.
     *
     * json: {"levelFormats":["^1.","^2)",...],"numberFormats":[0,8,...],"startNumber":1}
     * 반환값: Numbering ID (1-based)
     */
    createNumbering(json: string): number;
    /**
     * 커서 위치에 글상자(Rectangle + TextBox)를 삽입한다.
     *
     * json: `{"sectionIdx":N,"paraIdx":N,"charOffset":N,"width":N,"height":N,
     *         "horzOffset":N,"vertOffset":N,"treatAsChar":bool,"textWrap":"Square"}`
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     */
    createShapeControl(json: string): string;
    /**
     * 새 스타일을 생성한다.
     *
     * json: {"name":"...", "englishName":"...", "type":0, "nextStyleId":0}
     * 반환값: 새 스타일 ID (0-based)
     */
    createStyle(json: string): number;
    /**
     * 커서 위치에 새 표를 삽입한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     */
    createTable(section_idx: number, para_idx: number, char_offset: number, row_count: number, col_count: number): string;
    /**
     * 커서 위치에 표를 삽입한다 (확장, JSON 옵션).
     *
     * options JSON: { sectionIdx, paraIdx, charOffset, rowCount, colCount,
     *                 treatAsChar?: bool, colWidths?: [u32, ...] }
     */
    createTableEx(options_json: string): string;
    /**
     * 커서 좌표(list/para/pos)로 글자를 지운다 — 웹한글컨트롤 `Run("Delete*")`.
     *
     * `pos` 는 코드 유닛이고, 빈 범위면 아무 일도 하지 않는다.
     */
    deleteAtCursor(list_id: number, para_in_list: number, start_pos: number, end_pos: number): string;
    /**
     * 책갈피 삭제
     */
    deleteBookmark(sec: number, para: number, ctrl_idx: number): string;
    /**
     * [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (by_path).
     */
    deleteCellPictureControlByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number): string;
    /**
     * [#6771] 표 셀/글상자 내부 **표** 삭제 (by_path).
     *
     * 셀 안 1×1 안내 상자처럼 본문 리스트 밖에 있는 표를 지운다 — `deleteControlAt` 은
     * 본문만, `deleteTableControl` 은 `(구역, 문단, 컨트롤)` 만 다뤄 짚지 못하던 자리다.
     */
    deleteCellTableControlByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number): string;
    /**
     * 컨트롤 하나를 지운다 — 웹한글컨트롤 `DeleteCtrl`.
     */
    deleteControlAt(list_id: number, para_in_list: number, control_index: number): string;
    /**
     * 수식 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    deleteEquationControl(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 본문 각주 컨트롤을 삭제한다.
     */
    deleteFootnote(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 머리말/꼬리말을 삭제한다 (컨트롤 자체 제거).
     */
    deleteHeaderFooter(section_idx: number, is_header: boolean, apply_to: number): string;
    deleteParagraph(section_idx: number, para_idx: number): string;
    /**
     * 그림 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    deletePictureControl(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 본문 선택 영역을 삭제한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
     */
    deleteRange(section_idx: number, start_para_idx: number, start_char_offset: number, end_para_idx: number, end_char_offset: number): string;
    /**
     * 셀 내 선택 영역을 삭제한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
     */
    deleteRangeInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    deleteRangeInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, start_para: number, start_offset: number, end_para: number, end_offset: number): string;
    /**
     * `deleteRangeInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     */
    deleteRangeInCellEx(options_json: string): string;
    /**
     * Shape(글상자) 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    deleteShapeControl(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 스타일을 삭제한다.
     *
     * 바탕글(ID 0)은 삭제할 수 없다.
     * 삭제된 스타일을 사용 중인 문단은 바탕글(ID 0)로 변경된다.
     */
    deleteStyle(style_id: number): boolean;
    /**
     * 표에서 열을 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     */
    deleteTableColumn(section_idx: number, parent_para_idx: number, control_idx: number, col_idx: number): string;
    /**
     * 표 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    deleteTableControl(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 표에서 행을 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     */
    deleteTableRow(section_idx: number, parent_para_idx: number, control_idx: number, row_idx: number): string;
    /**
     * 문단에서 텍스트를 삭제한다.
     *
     * 삭제 후 구역을 재구성하고 재페이지네이션한다.
     * 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
     */
    deleteText(section_idx: number, para_idx: number, char_offset: number, count: number): string;
    /**
     * 표 셀 내부 문단에서 텍스트를 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
     */
    deleteTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    deleteTextInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, count: number): string;
    /**
     * 표 셀 내부 문단에서 텍스트를 삭제하되 전체 페이지네이션은 호출자가 지연한다.
     *
     * 결과 JSON은 `charOffset`과 상대 cell-flow 변화 신호 `cellFlowChanged`를 포함한다.
     */
    deleteTextInCellDeferredPagination(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    /**
     * `deleteTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, count }`. positional 과 동일 동작.
     */
    deleteTextInCellEx(options_json: string): string;
    /**
     * 각주 내 텍스트를 삭제한다.
     */
    deleteTextInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, char_offset: number, count: number): string;
    /**
     * 머리말/꼬리말 내 텍스트 삭제
     *
     * 반환: JSON `{"ok":true,"charOffset":<offset>}`
     */
    deleteTextInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, count: number): string;
    /**
     * 개체에서 캡션을 뗀다 — 웹한글컨트롤 `Run("ShapeObjDetachCaption")`.
     */
    detachCaptionAt(para_in_list: number, control_index: number): string;
    /**
     * 삭제 조각을 제거하여 메모리를 해제한다 — 히스토리 축출·클리어 시 스냅샷
     * `discardSnapshot` 과 짝으로 호출한다(#5769).
     */
    discardDeleteFragment(id: number): void;
    discardPictureTransform(id: number): void;
    /**
     * 구역 raw 캡처를 제거하여 메모리를 해제한다 — 히스토리 축출·클리어 계약 (#5769 Stage 4).
     */
    discardSectionRaw(id: number): void;
    /**
     * 지정 ID의 스냅샷을 제거하여 메모리를 해제한다.
     */
    discardSnapshot(id: number): void;
    /**
     * Batch 모드를 종료하고 누적된 이벤트를 반환한다.
     */
    endBatch(): string;
    /**
     * 특정 문자의 글머리표 정의가 없으면 생성한다.
     *
     * 반환값: Bullet ID (1-based)
     */
    ensureDefaultBullet(bullet_char_str: string): number;
    /**
     * 문서에 기본 문단 번호 정의가 없으면 생성한다.
     *
     * 반환값: Numbering ID (1-based)
     */
    ensureDefaultNumbering(): number;
    /**
     * 표 셀에서 계산식을 실행한다.
     *
     * formula: "=SUM(A1:A5)", "=A1+B2*3" 등
     * write_result: true이면 결과를 셀에 기록
     */
    evaluateTableFormula(section_idx: number, parent_para_idx: number, control_idx: number, target_row: number, target_col: number, formula: string, write_result: boolean): string;
    /**
     * `evaluateTableFormula` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, targetRow, targetCol,
     * formula: string, writeResult? }`. positional 과 동일 동작.
     */
    evaluateTableFormulaEx(options_json: string): string;
    /**
     * 컨트롤 객체를 HTML 문자열로 변환한다.
     */
    exportControlHtml(section_idx: number, para_idx: number, cell_path_json: string, control_idx: number): string;
    /**
     * HML 원본의 공통 IR을 HWPML 2.91 XML로 직렬화하여 반환한다.
     */
    exportHml(): Uint8Array;
    /**
     * 문서를 HWP 바이너리로 내보낸다.
     *
     * Document IR을 HWP 5.0 CFB 바이너리로 직렬화하여 반환한다.
     * HWPX 출처 문서는 `export_hwp_with_adapter` 를 통해 HWPX→HWP IR 매핑 어댑터를
     * 자동 적용하여 한컴 호환성과 자기 재로드 페이지 보존을 보장한다 (#178).
     * HWP 출처는 어댑터가 no-op 이므로 기존 동작과 동일.
     */
    exportHwp(): Uint8Array;
    /**
     * 어댑터 적용 + HWP 직렬화 + 자기 재로드 검증을 수행하고 결과를 JSON 으로 반환한다 (#178).
     *
     * 반환 JSON:
     * ```json
     * {
     *   "bytesLen": 678912,
     *   "pageCountBefore": 9,
     *   "pageCountAfter": 9,
     *   "recovered": true
     * }
     * ```
     *
     * 본 함수는 검증 메타데이터만 반환하며 bytes 자체는 별도 호출 (`exportHwp`) 로 받아야 한다.
     * 검증과 실제 사용을 분리하여 호출자가 결과에 따라 다른 동작을 취할 수 있도록 한다.
     */
    exportHwpVerify(): string;
    /**
     * 문서를 HWP5 EncryptVersion 4 비밀번호 문서로 내보낸다.
     *
     * browser UI는 암호를 저장하지 않고 저장 시점에만 전달한다. HWPX 출처 문서는 일반
     * HWP 저장과 동일하게 HWPX-to-HWP adapter를 먼저 적용한다.
     */
    exportHwpWithPassword(password: string): Uint8Array;
    /**
     * 비밀번호 HWP 바이트 + 내용 손실 보고 (#4430).
     */
    exportHwpWithPasswordAndReport(password: string): DocumentExport;
    /**
     * HWP 바이트와 이번 산출물의 내용 손실을 같은 결과로 반환한다 (#4430).
     *
     * 명시적 Studio 저장은 이 API를 사용한다. 기존 `exportHwp()`는 호환성을 위해
     * byte-only로 유지되며, autosave/embed/history/compare/hwpctl/digest 등 별도
     * 소비자는 아직 보고서를 받지 않는다.
     */
    exportHwpWithReport(): DocumentExport;
    /**
     * Document IR을 HWPX(ZIP+XML)로 직렬화하여 반환한다.
     */
    exportHwpx(): Uint8Array;
    /**
     * 문서를 ODF AES-256-CBC/PBKDF2 비밀번호 보호 HWPX로 내보낸다.
     */
    exportHwpxWithPassword(password: string): Uint8Array;
    /**
     * 비밀번호 HWPX 바이트 + 내용 손실 보고 (#4430).
     */
    exportHwpxWithPasswordAndReport(password: string): DocumentExport;
    /**
     * HWPX 바이트와 이번 산출물의 내용 손실을 같은 결과로 반환한다 (#4430).
     */
    exportHwpxWithReport(): DocumentExport;
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (본문).
     */
    exportSelectionHtml(section_idx: number, start_para_idx: number, start_char_offset: number, end_para_idx: number, end_char_offset: number): string;
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
     */
    exportSelectionInCellHtml(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * 전체 cellPath가 가리키는 중첩 셀 선택을 HTML로 변환한다(#4272).
     */
    exportSelectionInCellHtmlByPath(section_idx: number, parent_para_idx: number, path_json: string, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * `exportSelectionInCellHtml` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     */
    exportSelectionInCellHtmlEx(options_json: string): string;
    /**
     * 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다 (F11 키).
     */
    findNearestControlBackward(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 현재 위치 이후의 가장 가까운 선택 가능 컨트롤을 찾는다 (Shift+F11).
     */
    findNearestControlForward(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다.
     * delta=+1(앞), delta=-1(뒤). ctrl_idx=-1이면 본문 텍스트에서 출발.
     */
    findNextEditableControl(section_idx: number, para_idx: number, ctrl_idx: number, delta: number): string;
    /**
     * 글꼴 이름으로 font_id를 조회하거나 새로 생성한다.
     *
     * 한글(0번) 카테고리에서 이름 검색 → 없으면 7개 전체 카테고리에 신규 등록.
     * 반환값: font_id (u16), 실패 시 -1
     */
    findOrCreateFontId(name: string): number;
    /**
     * 특정 언어 카테고리에서 글꼴 이름으로 ID를 찾거나 등록한다.
     */
    findOrCreateFontIdForLang(lang: number, name: string): number;
    /**
     * 지연된 페이지네이션을 동기 barrier로 flush하고 최신 페이지 수를 반환한다.
     */
    flushDeferredPagination(): string;
    /**
     * [#4709] SVG 메트릭 face 주석 부착 여부를 반환한다.
     */
    getAnnotateMetricFont(): boolean;
    /**
     * 문서 내 모든 책갈피 목록 반환
     */
    getBookmarks(): string;
    /**
     * 문서에 정의된 글머리표(Bullet) 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, char }, ...]
     * id는 1-based (ParaShape.numbering_id와 동일)
     */
    getBulletList(): string;
    /**
     * 문서 전체의 bounded CanvasKit direct replay capability를 compact JSON으로 반환한다.
     */
    getCanvasKitDocumentPreflight(mode: string, profile: string): string;
    /**
     * CanvasKit direct replay 정책 진단을 JSON 문자열로 반환한다.
     *
     * `mode` 는 `"default"` 또는 `"compat"` 를 받는다. 빈 문자열은 `"default"` 로 처리한다.
     * 현재 두 mode 모두 hidden Canvas2D overlay 없이 direct replay required 정책을 따른다.
     * `compat` 는 API/URL 호환성과 이후 보수적인 direct replay 튜닝을 위해 남겨 둔 선택지다.
     */
    getCanvasKitReplayPlan(page_num: number, mode: string): string;
    getCanvasKitReplayPlanWithProfile(page_num: number, mode: string, profile: string): string;
    getCanvasPageLayerTree(page: number, profile: string, omit_font_bytes: boolean): string;
    /**
     * 문서에 저장된 캐럿 위치를 반환한다 (문서 로딩 시 캐럿 자동 배치용).
     *
     * 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
     */
    getCaretPosition(): string;
    /**
     * 캐럿이 설 수 있는 자리들 — 한 글자 이동(`MoveNextChar` 류)이 딛는 눈금.
     */
    getCaretStops(list_id: number, para_in_list: number): string;
    /**
     * 셀 내부 문단의 글자 속성을 조회한다.
     */
    getCellCharPropertiesAt(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
    getCellCharPropertiesAtByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): string;
    /**
     * 표 셀의 행/열/병합 정보를 반환한다.
     *
     * 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
     */
    getCellInfo(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): string;
    /**
     * 경로 기반 셀 정보 조회 (중첩 표용).
     *
     * 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
     */
    getCellInfoByPath(section_idx: number, parent_para_idx: number, path_json: string): string;
    /**
     * 셀 고유 속성을 조회한다.
     *
     * cellzone overlay를 합성하지 않고 셀 자체의 borderFill만 반환한다.
     */
    getCellOwnProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): string;
    /**
     * 셀 내부 문단의 문단 속성을 조회한다.
     */
    getCellParaPropertiesAt(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 표 셀 내 문단 수를 반환한다.
     */
    getCellParagraphCount(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): number;
    /**
     * 경로 기반: 셀/글상자 내 문단 수를 반환한다 (중첩 표/글상자 지원).
     */
    getCellParagraphCountByPath(section_idx: number, parent_para_idx: number, path_json: string): number;
    /**
     * 표 셀 내 문단의 글자 수를 반환한다.
     */
    getCellParagraphLength(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): number;
    /**
     * 경로 기반: 셀 내 문단의 글자 수를 반환한다 (중첩 표 지원).
     */
    getCellParagraphLengthByPath(section_idx: number, parent_para_idx: number, path_json: string): number;
    /**
     * [Task #1151 v4] 표 셀 내 Picture 속성 조회 (by_path). Shape 패턴 정합.
     */
    getCellPicturePropertiesByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number): string;
    getCellProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): string;
    /**
     * 셀 속성을 조회한다.
     *
     * 반환: JSON `{width, height, paddingLeft, paddingRight, paddingTop, paddingBottom, applyInnerMargin, verticalAlign, textDirection, isHeader, cellProtect, fieldName, editableInForm, ...borderFill}`
     */
    getCellPropertiesByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, cell_idx: number): string;
    /**
     * [Task #1138] 표 셀 내 Shape(글상자/사각형/도형) 속성 조회 (by_path).
     */
    getCellShapePropertiesByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number): string;
    /**
     * 커서가 든 셀의 모양 — 웹한글컨트롤 `CellShape` 파라미터셋.
     */
    getCellShapeSet(list_id: number): string;
    /**
     * 셀 내부 문단의 스타일을 조회한다.
     */
    getCellStyleAt(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 표 셀의 텍스트 방향을 반환한다 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움).
     */
    getCellTextDirection(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): number;
    /**
     * 스트림 자리를 글자 번호로 옮긴다 — 글자 번호를 받는 코어 API 에 넘길 때 쓴다.
     */
    getCharIndexAtStreamPos(list_id: number, para_in_list: number, pos: number): string;
    /**
     * 캐럿 위치의 글자 속성을 조회한다.
     *
     * 반환값: JSON 객체 (fontFamily, fontSize, bold, italic, underline, strikethrough, textColor 등)
     */
    getCharPropertiesAt(sec_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 머리말/꼬리말 캐럿 위치의 글자 속성을 조회한다.
     */
    getCharPropertiesInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number): string;
    /**
     * 문자 offset 범위의 모양 구간 목록을 조회한다.
     */
    getCharShapeRuns(sec: number, para: number, start: number, end: number): string;
    getCharShapeRunsInCellByPath(sec: number, para: number, path_json: string, start: number, end: number): string;
    /**
     * 커서 자리의 글자 모양 — 웹한글컨트롤 `CharShape` 파라미터셋 값(§8.2.2).
     *
     * 항목 이름과 단위는 한글 것이다(`Height` 는 HWPUNIT, `AlignType` 은 코드값).
     */
    getCharShapeSet(list_id: number, para_in_list: number, pos: number): string;
    /**
     * [#4694] 본문 직속 차트의 숫자 데이터를 조회한다 (3인자 주소).
     *
     * 컨테이너(글상자·표 셀·머리말) 안 차트는 이 주소로 표현할 수 없다 —
     * `getChartDataByIndex` 를 쓴다.
     */
    getChartData(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * [#4694] 문서 순번(0-based)으로 차트 데이터를 조회한다 — 정본 주소.
     */
    getChartDataByIndex(index: number): string;
    /**
     * 누름틀 필드의 속성을 조회한다.
     *
     * 반환: JSON `{"ok":true,"guide":"안내문","memo":"메모","name":"이름","editable":true}`
     */
    getClickHereProps(field_id: number): string;
    /**
     * 내부 클립보드의 플레인 텍스트를 반환한다.
     */
    getClipboardText(): string;
    /**
     * 현재 구역의 다단 설정을 JSON으로 반환한다.
     */
    getColumnDef(section_idx: number): string;
    /**
     * 컨트롤의 이미지 바이너리 데이터를 반환한다 (Uint8Array).
     */
    getControlImageData(section_idx: number, para_idx: number, cell_path_json: string, control_idx: number): Uint8Array;
    /**
     * 컨트롤의 이미지 MIME 타입을 반환한다.
     */
    getControlImageMime(section_idx: number, para_idx: number, cell_path_json: string, control_idx: number): string;
    /**
     * 문단 내 컨트롤의 텍스트 위치 배열을 반환한다.
     */
    getControlTextPositions(section_idx: number, para_idx: number): string;
    /**
     * 문서가 담은 컨트롤 사슬 — `HeadCtrl`·`LastCtrl` 과 `Next`·`Prev` 가 딛는다.
     */
    getControls(): string;
    /**
     * 커서가 든 필드의 상태 — 웹한글컨트롤 `CurFieldState`.
     */
    getCurFieldState(list_id: number, para_in_list: number, pos: number): number;
    /**
     * 한글 커서 좌표계(`list`/`para`/`pos`)를 쓰는 데 필요한 문서 사실.
     *
     * 리스트 표와 루트 리스트의 시작·끝 위치를 함께 준다. 자세한 계약은
     * `DocumentCore::get_cursor_model_json`.
     */
    getCursorModel(): string;
    /**
     * 커서 위치의 픽셀 좌표를 반환한다.
     *
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     */
    getCursorRect(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 경로 기반 커서 좌표 조회 (중첩 표용).
     *
     * path_json: `[{"controlIndex":N,"cellIndex":N,"cellParaIndex":N}, ...]`
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     */
    getCursorRectByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): string;
    /**
     * [#2021] 경로 기반 커서 좌표 조회 + 페이지 힌트 — 직전 캐럿 페이지를 전달하면
     * 해당 페이지(±1)를 먼저 탐색해, 거대 표 문서에서 캐시 무효화 직후의 선형 페이지
     * 재빌드 비용을 피한다. 힌트가 틀려도 종전 전체 탐색으로 fallback (좌표 불변).
     */
    getCursorRectByPathNear(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, hint_page: number): string;
    /**
     * 표 셀 내부 커서 위치의 픽셀 좌표를 반환한다.
     *
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     */
    getCursorRectInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
    /**
     * 각주 내 커서 렉트 계산
     */
    getCursorRectInFootnote(page_num: number, footnote_index: number, fn_para_idx: number, char_offset: number): string;
    /**
     * 머리말/꼬리말 내 커서 위치의 픽셀 좌표를 반환한다.
     *
     * `preview_page_hint`: 편집 정의를 투영할 대표 페이지 힌트. Studio는 구역의 첫 페이지를
     * 전달한다. 음수이면 호환 경로로 실제 적용 페이지를 앞에서부터 찾는다.
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     */
    getCursorRectInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, preview_page_hint: number): string;
    /**
     * 각주/미주 내부 커서 렉트 계산
     */
    getCursorRectInNote(section_idx: number, para_idx: number, control_idx: number, note_para_idx: number, char_offset: number): string;
    /**
     * 줄 경계 offset을 특정 시각 줄 기준으로 해석한 커서 좌표를 반환한다.
     *
     * `at_end=false`이면 lineIndex 줄의 시작, `at_end=true`이면 lineIndex 줄의 끝을 반환한다.
     * soft-wrap 경계에서는 같은 charOffset이 이전 줄 끝과 다음 줄 시작을 동시에 뜻할 수 있어
     * Home/End가 이 API로 시각 줄 affinity를 명시한다.
     */
    getCursorRectOnLine(section_idx: number, para_idx: number, line_index: number, at_end: boolean, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 문서 정보를 JSON 문자열로 반환한다.
     */
    getDocumentInfo(): string;
    /**
     * 현재 DPI를 반환한다.
     */
    getDpi(): number;
    /**
     * 미주 모양을 조회한다.
     */
    getEndnoteShape(section_idx: number): string;
    /**
     * 수식 컨트롤의 속성을 조회한다.
     *
     * 반환: JSON `{ script, fontSize, color, baseline, fontName }`
     */
    getEquationProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 현재 이벤트 로그를 JSON으로 반환한다.
     */
    getEventLog(): string;
    /**
     * [Task #741 후속] 외부 file path 그림 영역 영역 영역 영역 basename 목록 영역 반환.
     *
     * HWP3 파일 영역 image 영역 영역 절대 경로 영역 저장 영역. WASM 환경 영역 영역 file
     * system access 부재 영역, JS 영역 영역 영역 영역 fetch 영역 영역 영역 file 영역 load
     * 영역 후 `injectExternalImage` 영역 영역 영역 inject 영역.
     *
     * 반환: JSON 배열 `["oracle.gif", "rdb02.gif", ...]` (중복 제거)
     */
    getExternalImageBasenames(): string;
    /**
     * [Task #1142] 외부 file path 그림 reference 목록을 구조화된 JSON 배열로 반환한다.
     *
     * 반환: JSON 배열 `[{ key, binDataId, originalPath, basename, extension, loaded }, ...]`
     */
    getExternalImageReferences(): string;
    /**
     * 현재 대체 폰트 경로를 반환한다.
     */
    getFallbackFont(): string;
    /**
     * 커서 위치의 필드 범위 정보를 조회한다 (본문 문단).
     *
     * 반환: `{inField, fieldId?, startCharIdx?, endCharIdx?, isGuide?, guideName?, editableInForm?}`
     */
    getFieldInfoAt(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * path 기반: 중첩 표 셀의 필드 범위 정보를 조회한다.
     */
    getFieldInfoAtByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): string;
    /**
     * 커서 위치의 필드 범위 정보를 조회한다 (셀/글상자 내 문단).
     */
    getFieldInfoAtInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, is_textbox: boolean): string;
    /**
     * `getFieldInfoAtInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
     */
    getFieldInfoAtInCellEx(options_json: string): string;
    /**
     * 문서 내 모든 필드 목록을 JSON 배열로 반환한다.
     *
     * 반환: `[{fieldId, fieldType, name, guide, command, value, location}]`
     */
    getFieldList(): string;
    /**
     * field_id로 필드 값을 조회한다.
     *
     * 반환: `{ok, value}`
     */
    getFieldValue(field_id: number): string;
    /**
     * 필드 이름으로 값을 조회한다.
     *
     * 반환: `{ok, fieldId, value}`
     */
    getFieldValueByName(name: string): string;
    /**
     * 페이지의 bounded layout font decision trace를 JSON으로 반환한다 (#4961).
     */
    getFontDecisionTrace(page_num: number, options_json: string): string;
    /**
     * 본문 커서 위치의 각주 마커를 조회한다.
     *
     * direction: "backward" 또는 "forward"
     */
    getFootnoteAtCursor(section_idx: number, para_idx: number, char_offset: number, direction: string): string;
    /**
     * 각주 정보를 조회한다.
     */
    getFootnoteInfo(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 페이지 좌표에서 양식 개체를 찾는다.
     *
     * 반환: `{found, sec, para, ci, formType, name, value, caption, text, bbox}`
     */
    getFormObjectAt(page_num: number, x: number, y: number): string;
    /**
     * 양식 개체 상세 정보를 반환한다 (properties 포함).
     *
     * 반환: `{ok, formType, name, value, text, caption, enabled, width, height, foreColor, backColor, properties}`
     */
    getFormObjectInfo(sec: number, para: number, ci: number): string;
    /**
     * 양식 개체 값을 조회한다.
     *
     * 반환: `{ok, formType, name, value, text, caption, enabled}`
     */
    getFormValue(sec: number, para: number, ci: number): string;
    /**
     * 머리말/꼬리말 조회
     *
     * 반환: JSON `{"ok":true,"exists":true/false,...}`
     */
    getHeaderFooter(section_idx: number, is_header: boolean, apply_to: number): string;
    /**
     * 이 쪽에서 머리말/꼬리말을 편집할 때 대상이 되는 (구역, applyTo) 를 반환한다.
     *
     * 좌표 없이 쪽만으로 묻는 경로(툴바 `머리말`/`꼬리말`)용 — 히트테스트와 같은 답을 쓴다.
     * 반환: JSON `{"ok":true,"sectionIndex":N,"applyTo":N}`
     */
    getHeaderFooterEditTarget(page_num: number, is_header: boolean): string;
    /**
     * 문서 전체의 머리말/꼬리말 목록을 반환한다.
     */
    getHeaderFooterList(current_section_idx: number, current_is_header: boolean, current_apply_to: number): string;
    /**
     * 머리말/꼬리말 문단 정보 조회
     *
     * 반환: JSON `{"ok":true,"paraCount":N,"charCount":N,"text":"..."}`
     */
    getHeaderFooterParaInfo(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number): string;
    /**
     * [Task #825] 머리말/꼬리말 안 그림의 속성 조회.
     * path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
     *       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
     */
    getHeaderFooterPictureProperties(section_idx: number, outer_para_idx: number, outer_control_idx: number, inner_para_idx: number, inner_control_idx: number): string;
    /**
     * 머리말/꼬리말 정의가 속한 구역의 대표 편집 페이지(구역 첫 페이지)를 반환한다.
     */
    getHeaderFooterPreviewPage(section_idx: number): string;
    /**
     * HML 열기 메타데이터와 손실 진단을 JSON으로 반환한다.
     * 다른 입력 포맷에서는 `null`을 반환한다.
     */
    getHmlOpenMetadata(): string;
    /**
     * HML 저장 가능 여부와 모든 차단 진단을 canonical JSON DTO로 반환한다.
     */
    getHmlSaveState(): string;
    /**
     * 표시 문자는 Unicode scalar 축이다. 대상 누락 시 본문으로 폴백하지 않는다.
     */
    getHyperlinkContext(target_json: string): string;
    /**
     * 문단 내 줄 정보를 반환한다 (커서 수직 이동/Home/End용).
     *
     * 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
     */
    getLineInfo(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 표 셀 내 문단의 줄 정보를 반환한다.
     *
     * 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
     */
    getLineInfoInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
    /**
     * 줄이 시작하는 자리들 — `MoveLineBegin`·`MoveLineEnd` 가 딛는 값(코드 유닛).
     */
    getLineStarts(list_id: number, para_in_list: number): string;
    /**
     * 문단의 논리적 길이를 반환한다 (텍스트 문자 + 인라인 컨트롤 수).
     */
    getLogicalLength(section_idx: number, para_idx: number): number;
    /**
     * 각주/미주 편집 모드 진입 대상 조회
     */
    getNoteEditInfo(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 각주/미주 내부 수식 컨트롤의 속성을 조회한다.
     */
    getNoteEquationProperties(kind: string, section_idx: number, parent_para_idx: number, note_control_idx: number, note_para_idx: number, inner_control_idx: number): string;
    /**
     * 문서에 정의된 문단 번호(Numbering) 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, levelFormats: [...] }, ...]
     * id는 1-based (ParaShape.numbering_id와 동일)
     */
    getNumberingList(): string;
    /**
     * 개체 사이를 도는 차례(쪽·z) — 웹한글컨트롤 `Run("ShapeObjNext/PrevObject")` 용.
     */
    getObjectCycle(): string;
    /**
     * 본문에 놓인 개체 목록 — `Run("ShapeObjNextObject")` 따위가 딛는다.
     */
    getObjects(): string;
    /**
     * 문단 모양의 개요 번호만 탐색 정보로 반환한다.
     *
     * 일반 문단의 `1.` 같은 텍스트는 분석하지 않는다.
     */
    getOutlineNavigation(): string;
    /**
     * 구역의 쪽 테두리/배경 설정을 JSON으로 반환한다.
     */
    getPageBorderFill(section_idx: number): string;
    /**
     * 쪽마다 캐럿이 설 수 있는 첫 자리 — 웹한글컨트롤 `Run("MovePage*")` 용.
     */
    getPageCaretStarts(): string;
    /**
     * 컨트롤(표, 이미지 등) 레이아웃 정보를 반환한다.
     */
    getPageControlLayout(page_num: number): string;
    /**
     * 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환한다.
     */
    getPageDef(section_idx: number): string;
    /**
     * 본문(flow) 그림의 배치 정보만 작은 JSON 으로 반환한다 (Task #3315).
     *
     * 전체 레이어 트리를 받아 flow 그림을 걸러내던 studio 경로를 대체한다. 바이트는 빠져
     * 있고 `sourceImageKey` 로 `getSourceImageBytes` 를 부르면 된다.
     */
    getPageFlowImageOps(page_num: number): string;
    /**
     * 페이지의 각주 참조 정보
     */
    getPageFootnoteInfo(page_num: number, footnote_index: number): string;
    /**
     * 감추기 조회
     */
    getPageHide(sec: number, para: number): string;
    /**
     * 페이지 정보를 JSON 문자열로 반환한다.
     */
    getPageInfo(page_num: number): string;
    /**
     * 페이지 레이어 트리를 JSON 문자열로 반환한다.
     *
     * screen profile 기본값이므로 `getPageLayerTreeWithProfile` 로 위임한다 — 같은 핫패치
     * 경계를 지나야 한다. PageRenderer 가 좁은 질의를 못 쓸 때 되돌아오는 경로다.
     */
    getPageLayerTree(page_num: number): string;
    /**
     * Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
     */
    getPageLayerTreeV2Compat(page: number): string;
    getPageLayerTreeV2CompatWithProfile(page: number, profile: string): string;
    /**
     * Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
     */
    getPageLayerTreeV2StrictGlyphOutline(page: number): string;
    getPageLayerTreeV2StrictGlyphOutlineWithProfile(page: number, profile: string): string;
    /**
     * Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
     */
    getPageLayerTreeV2StrictGlyphRun(page: number): string;
    getPageLayerTreeV2StrictGlyphRunWithProfile(page: number, profile: string): string;
    /**
     * Object adapter with owned binary resources. No WASM-memory views escape.
     */
    getPageLayerTreeValue(page: number): any;
    /**
     * Object adapter with owned binary resources. No WASM-memory views escape.
     */
    getPageLayerTreeValueV2Compat(page: number): any;
    getPageLayerTreeValueV2CompatWithProfile(page: number, profile: string): any;
    /**
     * Only content-addressed keys supplied by the caller omit an array slot.
     */
    getPageLayerTreeValueV2CompatWithProfileAndResourceKeys(page: number, profile: string, known_image_keys: any, known_svg_keys: any): any;
    /**
     * Object adapter with owned binary resources. No WASM-memory views escape.
     */
    getPageLayerTreeValueV2StrictGlyphOutline(page: number): any;
    getPageLayerTreeValueV2StrictGlyphOutlineWithProfile(page: number, profile: string): any;
    /**
     * Only content-addressed keys supplied by the caller omit an array slot.
     */
    getPageLayerTreeValueV2StrictGlyphOutlineWithProfileAndResourceKeys(page: number, profile: string, known_image_keys: any, known_svg_keys: any): any;
    /**
     * Object adapter with owned binary resources. No WASM-memory views escape.
     */
    getPageLayerTreeValueV2StrictGlyphRun(page: number): any;
    getPageLayerTreeValueV2StrictGlyphRunWithProfile(page: number, profile: string): any;
    /**
     * Only content-addressed keys supplied by the caller omit an array slot.
     */
    getPageLayerTreeValueV2StrictGlyphRunWithProfileAndResourceKeys(page: number, profile: string, known_image_keys: any, known_svg_keys: any): any;
    getPageLayerTreeValueWithProfile(page: number, profile: string): any;
    /**
     * Only content-addressed keys supplied by the caller omit an array slot.
     * Optional byte-omission flags preserve the current source-image/font key
     * APIs; omitted flags retain the original four-argument inline contract.
     */
    getPageLayerTreeValueWithProfileAndResourceKeys(page: number, profile: string, known_image_keys: any, known_svg_keys: any, omit_font_bytes?: boolean | null, omit_image_bytes?: boolean | null): any;
    /**
     * 페이지 레이어 트리를 profile 별로 반환한다.
     *
     * [Task #3315] `omit_image_bytes` 를 `true` 로 주면 `sourceImageKey`를 낼 수 있는 그림만
     * base64를 생략하고, 바이트는 `getSourceImageBytes(key)`로 따로 받는다. 키 없는 합성 그림은
     * 소비자가 되찾을 방법이 없으므로 같은 `byKey` 요청에서도 인라인 base64를 유지한다.
     * [Task #4969] 네 번째 `omit_font_bytes`를 `true`로 주면 exact font metadata/key는
     * 유지하고 `resources.fontBlobs` payload만 생략한다. 바이트는
     * `getSourceFontBytes(key)`로 현재 document generation에서 따로 받는다.
     * 인자를 생략하면(`undefined`) 그림 payload는 inline으로 유지하지만, schema minor 21과
     * 최상위 `imageBytes:"inline"` 메타데이터가 있으므로 JSON 전체의 byte identity는 보장하지 않는다.
     */
    getPageLayerTreeWithProfile(page_num: number, profile: string, omit_image_bytes?: boolean | null, omit_font_bytes?: boolean | null): string;
    /**
     * 위치에 해당하는 글로벌 쪽 번호 반환
     */
    getPageOfPosition(section_idx: number, para_idx: number): string;
    /**
     * 페이지 overlay 이미지 정보만 JSON 문자열로 반환한다.
     */
    getPageOverlayImages(page_num: number): string;
    /**
     * 페이지 렌더 트리를 JSON 문자열로 반환한다.
     */
    getPageRenderTree(page_num: number): string;
    /**
     * 페이지가 그리는 그림들의 신원 키만 작은 JSON 으로 반환한다 (Task #3315).
     */
    getPageSourceImageKeys(page_num: number): string;
    /**
     * 쪽 하나의 글 — 웹한글컨트롤 `GetPageText`.
     */
    getPageText(page_index: number): string;
    /**
     * 특정 페이지의 텍스트 레이아웃 정보를 JSON 문자열로 반환한다.
     *
     * 각 TextRun의 위치, 텍스트, 글자별 X 좌표 경계값을 포함한다.
     */
    getPageTextLayout(page_num: number): string;
    /**
     * 문단 하나의 캐럿 경계 — `MoveParaBegin`·`MoveParaEnd`·`MoveListBegin/End` 가 딛는 값.
     */
    getParaBounds(list_id: number, para_in_list: number): string;
    /**
     * 캐럿 위치의 문단 속성을 조회한다.
     *
     * 반환값: JSON 객체 (alignment, lineSpacing, marginLeft, marginRight, indent 등)
     */
    getParaPropertiesAt(sec_idx: number, para_idx: number): string;
    /**
     * 각주/미주 내부 문단 속성 조회
     */
    getParaPropertiesInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number): string;
    /**
     * 머리말/꼬리말 문단의 문단 속성을 조회한다.
     */
    getParaPropertiesInHf(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number): string;
    /**
     * 커서 자리의 문단 모양 — 웹한글컨트롤 `ParaShape` 파라미터셋 값(§8.2.11).
     */
    getParaShapeSet(list_id: number, para_in_list: number): string;
    /**
     * 구역 내 문단 수를 반환한다.
     */
    getParagraphCount(section_idx: number): number;
    /**
     * 문단의 글자 수(char 개수)를 반환한다.
     */
    getParagraphLength(section_idx: number, para_idx: number): number;
    /**
     * 가져온 PDF의 원본 쪽/출력 구역 대응·경고·임시 문단 정책을 JSON으로 반환한다.
     * 편집 후 현재 쪽수가 아닌 가져오기 시점의 진단이다. 다른 형식은 `null`이다.
     */
    getPdfImportDiagnostics(): string;
    /**
     * Worker가 입력을 복사하기 전에 제품의 실제 요청 한도를 조회한다.
     */
    static getPdfImportPolicy(): string;
    /**
     * 그림 컨트롤의 속성을 조회한다.
     *
     * 반환: JSON `{ width, height, treatAsChar, ... }`
     */
    getPictureProperties(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 글로벌 쪽 번호에 해당하는 첫 문단 위치 반환
     */
    getPositionOfPage(global_page: number): string;
    /**
     * 문서 글을 한글 스캔 차례로 — `InitScan`·`GetText`·`ReleaseScan` 이 쓴다.
     */
    getScanItems(): string;
    /**
     * 구역(Section) 수를 반환한다.
     */
    getSectionCount(): number;
    /**
     * 구역 정의(SectionDef)를 JSON으로 반환한다.
     */
    getSectionDef(section_idx: number): string;
    /**
     * 구역마다 첫 본문 문단 번호 — `MoveSectionUp`·`MoveSectionDown` 이 딛는다.
     */
    getSectionStarts(): string;
    /**
     * 본문 선택 영역의 줄별 사각형을 반환한다.
     *
     * 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
     */
    getSelectionRects(section_idx: number, start_para_idx: number, start_char_offset: number, end_para_idx: number, end_char_offset: number): string;
    /**
     * 셀 내 선택 영역의 줄별 사각형을 반환한다.
     *
     * 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
     */
    getSelectionRectsInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * 전체 cellPath로 중첩 셀 선택 영역의 줄별 사각형을 반환한다(#4272).
     *
     * `path_json`의 마지막 엔트리는 선택 대상 셀을 지정하며, 시작·끝 문단 인덱스는
     * 별도 인자로 받아 여러 문단 선택도 같은 컨테이너 경로에서 처리한다.
     */
    getSelectionRectsInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
    /**
     * `getSelectionRectsInCellByPath`의 page hint options 변형(#4272).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, path, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset, startPageHint?, endPageHint? }`.
     * `path`는 cellPath JSON 문자열이다.
     */
    getSelectionRectsInCellByPathEx(options_json: string): string;
    /**
     * `getSelectionRectsInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset, startPageHint?, endPageHint? }`.
     * page hint가 누락되거나 유효하지 않으면 positional 과 동일한 전체 탐색을 사용한다.
     */
    getSelectionRectsInCellEx(options_json: string): string;
    /**
     * 각주/미주 내부 선택 영역의 줄별 사각형을 반환한다.
     */
    getSelectionRectsInFootnote(page_num: number, footnote_index: number, start_fn_para_idx: number, start_char_offset: number, end_fn_para_idx: number, end_char_offset: number): string;
    /**
     * 요청한 한 페이지의 머리말/꼬리말 선택 사각형을 반환한다.
     */
    getSelectionRectsInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, page_num: number, start_hf_para_idx: number, start_char_offset: number, end_hf_para_idx: number, end_char_offset: number): string;
    /**
     * [Task #919] 글상자/도형 컨트롤의 페이지 좌표 바운딩박스를 반환한다.
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     * studio 의 `isShapeBorderClick` 헬퍼에서 외곽 경계선 클릭 판별에 사용.
     */
    getShapeBBox(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * Shape(글상자) 속성을 조회한다.
     *
     * 반환: JSON `{ width, height, treatAsChar, tbMarginLeft, ... }`
     */
    getShapeProperties(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 조판부호 표시 여부를 반환한다.
     */
    getShowControlCodes(): boolean;
    /**
     * 문단부호(¶) 표시 여부를 반환한다.
     */
    getShowParagraphMarks(): boolean;
    /**
     * 투명선 표시 여부를 반환한다.
     */
    getShowTransparentBorders(): boolean;
    /**
     * Portable font resource key로 exact source bytes를 Uint8Array로 반환한다 (Task #4969).
     *
     * `getPageLayerTreeWithProfile(page, profile, imageMode, true)`의 opt-in 경로다.
     * key가 현재 document generation의 registry source와 정확히 일치하지 않으면 던진다.
     */
    getSourceFontBytes(key: string): Uint8Array;
    /**
     * 원본 파일 형식을 반환한다 ("hwp", "hwpx", "hml", 또는 "pdf").
     */
    getSourceFormat(): string;
    /**
     * 그림 신원 키로 바이트를 Uint8Array 로 반환한다 (Task #3315).
     *
     * `getPageLayerTreeWithProfile(page, profile, true)` 로 base64 를 생략했을 때 바이트를
     * 받는 경로다. mime 은 레이어 트리의 그림 op 이 계속 싣고 있으므로 여기서 되풀이하지
     * 않는다.
     *
     * 키를 풀 수 없으면 던진다 — 세대가 바뀐 낡은 키이거나 없는 그림이다. 호출부는 잡아서
     * 레이어 트리를 다시 받는 쪽으로 되돌아가면 된다.
     */
    getSourceImageBytes(key: string): Uint8Array;
    /**
     * 문서에 저장된 캐럿 위치를 **원본 값 그대로** 돌려준다.
     *
     * 한글은 문서를 열면 이 자리에 캐럿을 놓는다(`GetPos` 첫 답과 일치). studio 의
     * `getCaretPosition` 은 이 값을 구역/문단으로 해석하지만, 여기서는 해석하지 않는다 —
     * `list` 는 구역 번호가 아니라 리스트 아이디다.
     */
    getStoredCaret(): string;
    /**
     * 문서 구조(개요/조문) 트리를 JSON으로 반환 (사이드바 목차 네비게이션용)
     *
     * `mode`: `"auto"` | `"outline"` | `"clause"` (인식 불가 시 `auto`).
     */
    getStructure(mode: string): string;
    /**
     * 특정 문단의 스타일을 조회한다.
     *
     * 반환값: JSON { id, name }
     */
    getStyleAt(sec_idx: number, para_idx: number): string;
    /**
     * 특정 스타일의 CharShape/ParaShape 속성을 상세 조회한다.
     *
     * 반환값: JSON { charProps: {...}, paraProps: {...} }
     */
    getStyleDetail(style_id: number): string;
    /**
     * 문서에 정의된 스타일 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, name, englishName, type, paraShapeId, charShapeId }, ...]
     */
    getStyleList(): string;
    /**
     * 표 전체의 바운딩박스를 반환한다.
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     */
    getTableBBox(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 지정 page 에 배치된 표 fragment 의 바운딩박스를 반환한다 (#2400).
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     */
    getTableBBoxAtPage(section_idx: number, parent_para_idx: number, control_idx: number, page_idx: number): string;
    /**
     * 표의 모든 셀 bbox를 반환한다 (F5 셀 선택 모드용).
     *
     * 반환: JSON `[{cellIdx, row, col, rowSpan, colSpan, pageIndex, x, y, w, h}, ...]`
     */
    getTableCellBboxes(section_idx: number, parent_para_idx: number, control_idx: number, page_hint?: number | null): string;
    /**
     * 지정 페이지의 셀 경계만 조회한다. 전체 셀 선택 API의 범위는 유지한다.
     */
    getTableCellBboxesAtPage(section_idx: number, parent_para_idx: number, control_idx: number, page: number): string;
    /**
     * 경로 기반 표 셀 바운딩박스 조회 (중첩 표용).
     *
     * 반환: JSON 배열 `[{"cellIdx":N,"row":N,"col":N,...,"x":F,"y":F,"w":F,"h":F}, ...]`
     */
    getTableCellBboxesByPath(section_idx: number, parent_para_idx: number, path_json: string): string;
    /**
     * 표의 행/열/셀 수를 반환한다.
     *
     * 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
     */
    getTableDimensions(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 경로 기반 표 차원 조회 (중첩 표용).
     *
     * 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
     */
    getTableDimensionsByPath(section_idx: number, parent_para_idx: number, path_json: string): string;
    /**
     * 표 속성을 조회한다.
     *
     * 반환: JSON `{cellSpacing, paddingLeft, paddingRight, paddingTop, paddingBottom, pageBreak, repeatHeader}`
     */
    getTableProperties(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 문단에 텍스트박스가 있는 Shape 컨트롤이 있으면 해당 control_index를 반환한다.
     * 없으면 -1을 반환한다.
     */
    getTextBoxControlIndex(section_idx: number, para_idx: number): number;
    /**
     * 문서 글 전체 — `GetTextFile("TEXT")`. CP949 수치 참조를 적용한 JSON 문자열이다.
     */
    getTextFileText(): string;
    /**
     * 문서 글 전체 — `GetTextFile("UNICODE")`. 원문 Unicode JSON 문자열이다.
     */
    getTextFileUnicode(): string;
    /**
     * 표 셀 내 문단에서 텍스트 부분 문자열을 반환한다.
     */
    getTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    getTextInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, count: number): string;
    /**
     * `getTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, count }`. positional 과 동일 동작.
     */
    getTextInCellEx(options_json: string): string;
    /**
     * 문단에서 텍스트 부분 문자열을 반환한다 (Undo용 텍스트 보존).
     */
    getTextRange(section_idx: number, para_idx: number, char_offset: number, count: number): string;
    /**
     * HWPX 비표준 감지 경고를 JSON 문자열로 반환한다 (#177).
     *
     * ## 반환 형식
     *
     * ```json
     * {
     *   "count": 3,
     *   "summary": {
     *     "lineseg 배열이 비어있음": 1,
     *     "lineseg 가 미계산 상태 (line_height=0)": 2
     *   },
     *   "warnings": [
     *     {
     *       "section": 0,
     *       "paragraph": 5,
     *       "kind": "LinesegArrayEmpty",
     *       "cell": null
     *     },
     *     {
     *       "section": 0,
     *       "paragraph": 10,
     *       "kind": "LinesegUncomputed",
     *       "cell": {"ctrl": 0, "row": 0, "col": 1, "innerPara": 0}
     *     }
     *   ]
     * }
     * ```
     */
    getValidationWarnings(): string;
    /**
     * 지금 단어의 끝 — `MoveWordEnd` 가 가는 자리(다음 공백 글자의 자리).
     */
    getWordEnd(list_id: number, para_in_list: number, pos: number): string;
    /**
     * 단어가 시작하는 자리들 — `MoveNextWord` 류가 딛는 눈금(코드 유닛).
     */
    getWordStarts(list_id: number, para_in_list: number): string;
    /**
     * 선택된 개체들을 하나의 GroupShape로 묶는다.
     * json: `{"sectionIdx":N, "targets":[{"paraIdx":N,"controlIdx":N},...]}`
     * 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
     */
    groupShapes(json: string): string;
    /**
     * 내부 클립보드에 데이터가 있는지 확인한다.
     */
    hasInternalClipboard(): boolean;
    /**
     * 행/열 바꿈 복사 버퍼 보유 여부를 반환한다.
     */
    hasTableTransposeClipboard(): boolean;
    /**
     * 페이지 좌표에서 문서 위치를 찾는다.
     *
     * 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
     */
    hitTest(page_num: number, x: number, y: number): string;
    /**
     * 본문 인라인 각주 마커 히트테스트
     */
    hitTestBodyFootnoteMarker(page_num: number, x: number, y: number): string;
    /**
     * 각주 영역 히트테스트
     */
    hitTestFootnote(page_num: number, x: number, y: number): string;
    /**
     * 페이지 좌표가 머리말/꼬리말 영역에 해당하는지 판별한다.
     *
     * 반환: JSON `{"hit":true/false,"isHeader":bool,"sectionIndex":N,"applyTo":N}`
     */
    hitTestHeaderFooter(page_num: number, x: number, y: number): string;
    /**
     * 각주 내부 텍스트 히트테스트
     */
    hitTestInFootnote(page_num: number, x: number, y: number): string;
    /**
     * 머리말/꼬리말 내부 텍스트 히트테스트.
     *
     * 편집 모드에서 클릭한 좌표의 문단·문자 위치를 반환.
     * 반환: JSON `{"hit":true,"paraIndex":N,"charOffset":N,"cursorRect":{...}}`
     */
    hitTestInHeaderFooter(page_num: number, is_header: boolean, x: number, y: number): string;
    /**
     * 대표 편집 페이지에서 명시한 HF target으로 내부 텍스트를 히트테스트한다.
     */
    hitTestInHeaderFooterTarget(page_num: number, section_idx: number, is_header: boolean, apply_to: number, x: number, y: number): string;
    /**
     * Import from a distinct, read-only document handle; no source bytes in JSON.
     * Options JSON: {request:{sourceSection,sourceStart,sourceEnd,targetSection,insertBefore,count,limits?},dryRun?:boolean}.
     */
    importParagraphBlock(source: HwpDocument, options_json: string): string;
    /**
     * PDF 전체 또는 선택 쪽을 공통 IR로 가져온다 (기본 포함 `pdf-import`).
     * options.sourcePages는 0-based 증가 배열이며 생략하면 전체 쪽이다.
     * 동기 실행이므로 브라우저에서 취소/시간 제한이 필요하면 Worker에서 호출한다.
     */
    static importPdf(data: Uint8Array, options: any): HwpDocument;
    /**
     * [Task #741 후속] 외부 file path 그림 영역 영역 binary data 영역 inject.
     *
     * JS 영역 영역 영역 fetch 영역 영역 영역 file 영역 load 영역 후 본 메서드 영역 호출 영역
     * IR 영역 영역 영역 image binary 영역 영역 → renderer 영역 영역 표시.
     *
     * `basename`: 영역 영역 file 영역 영역 (예: "oracle.gif")
     * `data`: 영역 영역 binary 영역
     * `display_path`: dialog 영역 영역 영역 영역 표시 영역 영역 path. 빈 문자열 ("") 영역
     *                 영역 영역 fallback 영역 영역 `/samples/<basename>` 영역 사용. 한컴 viewer
     *                 정합 영역 영역 OS 영역 절대 경로 영역 영역 (예: "/Users/.../samples/rdb02.gif")
     */
    injectExternalImage(basename: string, data: Uint8Array, display_path: string): number;
    /**
     * [Task #1143] `getExternalImageReferences()` 의 key로 외부 이미지 bytes를 주입한다.
     *
     * 지원 key: `binData:<bin_data_id>`.
     * 잘못된 key, 존재하지 않는 key, 이미 loaded 상태인 reference는 0을 반환한다.
     */
    injectExternalImageByKey(key: string, data: Uint8Array, display_path: string): number;
    /**
     * 자동 번호 끼우기 — `InsertPageNum`·`InsertCpNo`·`InsertTpNo`. `kind` 는
     * `page`·`current`·`total`.
     */
    insertAutoNumberAtCursor(list_id: number, para_in_list: number, pos: number, kind: string): string;
    /**
     * 현재 본문 위치에 ClickHere 누름틀 필드를 삽입한다.
     */
    insertClickHereField(section_idx: number, para_idx: number, char_offset: number, guide: string, memo: string, name: string, editable: boolean): string;
    /**
     * 한글 커서 좌표(list/para/pos)에 누름틀을 넣는다 — 웹한글컨트롤 `CreateField`.
     *
     * `pos` 는 코드 유닛이다(확장 컨트롤 하나가 8칸). 글자 번호를 받는
     * `insertClickHereField` 와 좌표계가 다르다.
     */
    insertClickHereFieldAtCursor(list_id: number, para_in_list: number, pos: number, guide: string, memo: string, name: string, editable: boolean): string;
    /**
     * 현재 중첩 표 cellPath 위치에 ClickHere 누름틀 필드를 삽입한다.
     */
    insertClickHereFieldByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, guide: string, memo: string, name: string, editable: boolean): string;
    /**
     * `insertClickHereFieldByPath` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, path: string, charOffset?, guide?,
     * memo?, name?, editable? }`. `path` 는 cell_path JSON 문자열. positional 과 동일 동작.
     */
    insertClickHereFieldByPathEx(options_json: string): string;
    /**
     * `insertClickHereField` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, paraIdx, charOffset?, guide?, memo?, name?, editable? }`.
     * positional 과 동일 동작.
     */
    insertClickHereFieldEx(options_json: string): string;
    /**
     * 현재 셀/글상자 위치에 ClickHere 누름틀 필드를 삽입한다.
     */
    insertClickHereFieldInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, is_textbox: boolean, guide: string, memo: string, name: string, editable: boolean): string;
    /**
     * `insertClickHereFieldInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox?, guide?, memo?, name?, editable? }`. positional 과 동일 동작.
     */
    insertClickHereFieldInCellEx(options_json: string): string;
    /**
     * 단 나누기 삽입 (Ctrl+Shift+Enter)
     */
    insertColumnBreak(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 미주를 삽입한다.
     */
    insertEndnote(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 수식을 삽입한다.
     */
    insertEquation(section_idx: number, para_idx: number, char_offset: number, script: string, font_size: number, color: number): string;
    /**
     * 머리말/꼬리말 문단에 필드 마커를 삽입한다.
     */
    insertFieldInHf(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, field_type: number): string;
    /**
     * 각주를 삽입한다.
     */
    insertFootnote(section_idx: number, para_idx: number, char_offset: number): string;
    insertHyperlinkEx(options_json: string): number;
    /**
     * 새 번호 지정 컨트롤 삽입 (쪽 > 새 번호로 시작)
     */
    insertNewNumber(section_idx: number, para_idx: number, char_offset: number, start_num: number): string;
    /**
     * 강제 쪽 나누기 삽입 (Ctrl+Enter)
     */
    insertPageBreak(section_idx: number, para_idx: number, char_offset: number): string;
    insertParagraph(section_idx: number, para_idx: number): string;
    /**
     * 커서 위치에 그림을 삽입한다.
     *
     * image_data: 이미지 바이너리 데이터 (PNG/JPG/GIF/BMP 등)
     * width, height: HWPUNIT 단위 크기
     * extension: 파일 확장자 (jpg, png 등)
     *
     * 반환:
     * - 본문 inline: `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     * - 셀 floating (#1151): `{"ok":true,"paraIdx":<table_para>,"controlIdx":<new_sibling_idx>}`
     *
     * `cell_path_json` 이 빈 문자열 또는 `"[]"` 면 본문 inline 삽입. 그 외에는
     * 표 셀 영역에 floating picture (한컴 정합) 로 삽입한다.
     * 예: `[{"controlIndex":0,"cellIndex":2,"cellParaIndex":0}]`
     * [Task #1151 v8 결함 C] `paper_offset_x_hu / paper_offset_y_hu` 는 사용자가 셀 안에
     * 클릭/드래그한 위치 (paper-relative HU). studio 의 finishImagePlacement 가 drag 좌표를
     * 변환하여 전달. JS 측에서 `undefined` 전달 시 (또는 음수) wasm 이 셀 좌상단을 default 사용
     * — 기존 동작 호환.
     */
    insertPicture(section_idx: number, para_idx: number, char_offset: number, cell_path_json: string, image_data: Uint8Array, width: number, height: number, natural_width_px: number, natural_height_px: number, extension: string, description: string, paper_offset_x_hu?: number | null, paper_offset_y_hu?: number | null): string;
    /**
     * 커서 위치에 그림을 삽입한다 (확장, options object — #1413).
     *
     * positional `insertPicture` 와 동일 동작의 얇은 어댑터. 이미지 바이너리는 별도
     * `image_data` 인자(Uint8Array)로 받고, 나머지는 JSON options 로 받는다. 필드 추가/
     * 순서 변경 시 호출부 영향이 작다.
     *
     * options JSON 키 (positional 과 동일 의미, camelCase):
     * `{ sectionIdx, paraIdx, charOffset?, cellPath?: string, width, height,
     *    naturalWidthPx, naturalHeightPx, extension?, description?,
     *    paperOffsetXHu?: number|null, paperOffsetYHu?: number|null }`
     * - `cellPath` 는 cell_path_json 문자열(빈 문자열/`"[]"` 이면 본문 inline).
     * - 반환값은 `insertPicture` 와 동일.
     */
    insertPictureEx(options_json: string, image_data: Uint8Array): string;
    /**
     * 표에 열을 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     */
    insertTableColumn(section_idx: number, parent_para_idx: number, control_idx: number, col_idx: number, right: boolean): string;
    /**
     * 표에 행을 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     */
    insertTableRow(section_idx: number, parent_para_idx: number, control_idx: number, row_idx: number, below: boolean): string;
    /**
     * 문단에 텍스트를 삽입한다.
     *
     * 삽입 후 구역을 재구성하고 재페이지네이션한다.
     * 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
     */
    insertText(section_idx: number, para_idx: number, char_offset: number, text: string): string;
    /**
     * 커서 좌표(list/para/pos)에 글자를 끼운다 — 웹한글컨트롤 `Run("Insert*Space")`.
     */
    insertTextAtCursor(list_id: number, para_in_list: number, pos: number, text: string): string;
    /**
     * 표 셀 내부 문단에 텍스트를 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
     */
    insertTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, text: string): string;
    insertTextInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, text: string): string;
    /**
     * 표 셀 내부 문단에 텍스트를 삽입하되 전체 페이지네이션은 호출자가 지연한다.
     *
     * Studio의 page-local 단일 입력처럼 현재 페이지를 먼저 갱신하고 idle 시점에
     * 전체 페이지네이션을 한 번만 수행하는 경로에서 사용한다.
     * 결과 JSON은 `charOffset`과 상대 cell-flow 변화 신호 `cellFlowChanged`를 포함한다.
     */
    insertTextInCellDeferredPagination(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, text: string): string;
    /**
     * `insertTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, text: string }`. positional 과 동일 동작.
     */
    insertTextInCellEx(options_json: string): string;
    /**
     * 각주 내 텍스트를 삽입한다.
     */
    insertTextInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, char_offset: number, text: string): string;
    /**
     * 머리말/꼬리말 내 텍스트 삽입
     *
     * 반환: JSON `{"ok":true,"charOffset":<new_offset>}`
     */
    insertTextInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, text: string): string;
    /**
     * 논리적 오프셋으로 텍스트를 삽입한다.
     *
     * logical_offset: 텍스트 문자 + 인라인 컨트롤을 각각 1로 세는 위치.
     * 예: "abc[표]XYZ" → a(0) b(1) c(2) [표](3) X(4) Y(5) Z(6)
     * logical_offset=4이면 표 뒤의 X 앞에 삽입.
     * 반환값: JSON `{"ok":true,"logicalOffset":<new_logical_offset>}`
     */
    insertTextLogical(section_idx: number, para_idx: number, logical_offset: number, text: string): string;
    /**
     * 아무 내용도 없는 빈 문서인가 — 웹한글컨트롤 `IsEmpty`(§8.2.7).
     */
    isEmptyDocument(): boolean;
    /**
     * [#4694] 문서의 모든 차트를 문서 순서로 열거한다.
     *
     * 반환: JSON `[{ index, section, paragraph, control, container?, zipPart?, nestedCopy? }]`
     * studio 는 이 목록을 선택 컨트롤과 대조해 정본 주소(문서 순번)를 얻는다.
     */
    listCharts(): string;
    /**
     * 논리적 오프셋 → 텍스트 오프셋 변환.
     */
    logicalToTextOffset(section_idx: number, para_idx: number, logical_offset: number): number;
    /**
     * 문단별 줄 폭 측정 진단 (WASM)
     */
    measureWidthDiagnostic(section_idx: number, para_idx: number): string;
    /**
     * 현재 문단을 이전 문단에 병합한다 (Backspace at start).
     *
     * para_idx의 텍스트가 para_idx-1에 결합되고 para_idx는 삭제된다.
     * 반환값: JSON `{"ok":true,"paraIdx":<merged_para_idx>,"charOffset":<merge_point>}`
     */
    mergeParagraph(section_idx: number, para_idx: number): string;
    /**
     * 셀 내부 문단을 이전 문단에 병합한다 (셀 내 Backspace at start).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<prev_idx>,"charOffset":<merge_point>}`
     */
    mergeParagraphInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    mergeParagraphInCellByPath(section_idx: number, parent_para_idx: number, path_json: string): string;
    /**
     * 각주 내 문단을 병합한다 (Backspace at start).
     */
    mergeParagraphInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number): string;
    /**
     * 머리말/꼬리말 내 문단 병합 (Backspace at start)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<prev_idx>,"charOffset":<merge_point>}`
     */
    mergeParagraphInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number): string;
    /**
     * 표의 셀을 병합한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     */
    mergeTableCells(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number): string;
    /**
     * `mergeTableCells` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
     * endRow, endCol }`. positional 과 동일 동작.
     */
    mergeTableCellsEx(options_json: string): string;
    /**
     * 현재 표에 다음 표를 이어 붙인다 (한컴 [표-표 붙이기]).
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>}`
     */
    mergeTableWithNext(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * 개체를 한 걸음 옮긴다 — 웹한글컨트롤 `ShapeObjMove*`(걸음 56 HWPUNIT).
     */
    moveControlAt(para_in_list: number, control_index: number, dx: number, dy: number): string;
    /**
     * 직선 끝점 이동 (글로벌 HWPUNIT 좌표)
     */
    moveLineEndpoint(sec: number, para: number, ci: number, sx: number, sy: number, ex: number, ey: number): string;
    /**
     * `moveLineEndpoint` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, para, ci, sx, sy, ex, ey }` (좌표는 i32). positional 과 동일 동작.
     */
    moveLineEndpointEx(options_json: string): string;
    /**
     * 표의 위치 오프셋(vertical_offset, horizontal_offset)을 이동한다.
     *
     * delta_h, delta_v: HWPUNIT 단위 이동량 (양수=오른쪽/아래, 음수=왼쪽/위)
     * 반환: JSON `{"ok":true}`
     */
    moveTableOffset(section_idx: number, parent_para_idx: number, control_idx: number, delta_h: number, delta_v: number): string;
    /**
     * 수직 커서 이동 (ArrowUp/Down) — 단일 호출로 줄/문단/표/구역 경계를 모두 처리한다.
     *
     * delta: -1=위, +1=아래
     * preferred_x: 이전 반환값의 preferredX (최초 이동 시 -1.0 전달)
     * 셀 컨텍스트: 본문이면 모두 0xFFFFFFFF 전달
     *
     * 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
     */
    moveVertical(section_idx: number, para_idx: number, char_offset: number, delta: number, preferred_x: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 경로 기반 수직 커서 이동 (중첩 표용).
     *
     * 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
     */
    moveVerticalByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, delta: number, preferred_x: number): string;
    /**
     * `moveVertical` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, paraIdx, charOffset?, delta, preferredX,
     * parentParaIdx?, controlIdx?, cellIdx?, cellParaIdx? }`. cell 컨텍스트 키가 모두
     * 생략되면 본문 이동(parentParaIdx=MAX 동작과 동일). positional 과 동일 동작.
     */
    moveVerticalEx(options_json: string): string;
    /**
     * 페이지 단위로 이전/다음 머리말·꼬리말로 이동한다.
     *
     * 반환: JSON `{"ok":true,"pageIndex":N,"sectionIdx":N,"isHeader":bool,"applyTo":N}`
     * 또는 더 이상 이동할 페이지가 없으면 `{"ok":false}`
     */
    navigateHeaderFooterByPage(current_page: number, is_header: boolean, direction: number): string;
    /**
     * 문서 트리 DFS 기반 다음/이전 편집 가능 위치를 반환한다.
     * context_json: NavContextEntry 배열의 JSON (빈 배열 "[]" = body)
     */
    navigateNextEditable(sec: number, para: number, char_offset: number, delta: number, context_json: string): string;
    /**
     * HWP 파일 바이트를 로드하여 문서 객체를 생성한다.
     */
    constructor(data: Uint8Array);
    /**
     * 비밀번호로 보호된 HWP/HWPX 파일을 비밀번호와 함께 로드한다.
     *
     * HWP5 EncryptVersion 4, 압축 HWP3와 ODF AES-256-CBC HWPX를 지원한다.
     * 구버전/비압축 HWP3 암호화와 DRM은 지원하지 않는다.
     * 비밀번호가 틀린 경우 JS 측에서 잡을 수 있도록 에러 메시지에
     * "비밀번호가 일치하지 않"이 포함된 `JsValue` 를 반환한다.
     * 암호화되지 않은 일반 문서에 비밀번호를 전달해도 정상 로드된다.
     */
    static openWithPassword(data: Uint8Array, password: string): HwpDocument;
    /**
     * 총 페이지 수를 반환한다.
     */
    pageCount(): number;
    /**
     * 페이지에 각주 영역이 있는지 빠르게 확인 (hitTestFootnote fast-reject).
     * 페이지네이션 메타데이터만 조회하므로 render tree build가 필요 없다 (#2428).
     */
    pageHasFootnoteFootholds(page_num: number): boolean;
    /**
     * 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
     */
    pasteControl(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * HTML 문자열을 파싱하여 캐럿 위치에 삽입한다 (본문).
     *
     * `char_offset` 과 반환 `charOffset` 은 논리 위치(`insertTextLogical` 과 같은 축)다.
     */
    pasteHtml(section_idx: number, para_idx: number, char_offset: number, html: string): string;
    /**
     * HTML 문자열을 파싱하여 셀 내부 캐럿 위치에 삽입한다.
     */
    pasteHtmlInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, html: string): string;
    /**
     * HTML 문자열을 파싱하여 cellPath가 가리키는 중첩 표 셀에 삽입한다.
     */
    pasteHtmlInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, html: string): string;
    /**
     * `pasteHtmlInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, html: string }`. positional 과 동일 동작.
     */
    pasteHtmlInCellEx(options_json: string): string;
    /**
     * 한글 클립보드 문서모델(hwpjson)을 캐럿 위치에 삽입한다 (본문).
     *
     * 한글은 Ctrl+C 시 클립보드 HTML 끝 주석에 문서 모델 전체를 싣는다. HTML 에는 없는
     * 글꼴 등록·문단모양·쪽 설정·셀 속성·그림 원본이 여기 있어, 이 경로라야 원본과 같은
     * 조판이 나온다. 실패하면 호출한 쪽이 종전 `pasteHtml` 로 되돌아가면 된다.
     */
    pasteHwpJson(section_idx: number, para_idx: number, char_offset: number, json: string): string;
    /**
     * 내부 클립보드의 내용을 캐럿 위치에 붙여넣는다 (본문 문단).
     *
     * 반환값: JSON `{"ok":true,"paraIdx":<idx>,"charOffset":<offset>}`
     */
    pasteInternal(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 내부 클립보드의 내용을 표 셀 내부에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
     */
    pasteInternalInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
    /**
     * 내부 클립보드의 내용을 cellPath가 가리키는 중첩 표 셀에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
     */
    pasteInternalInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): string;
    /**
     * 행/열 바꿈 복사 버퍼를 대상 시작 셀부터 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     */
    pasteTableCellsTransposed(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number): string;
    /**
     * 행/열 바꿈 복사 버퍼를 커서 위치에 새 표로 생성해 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"paraIdx":N,"controlIdx":N,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     */
    pasteTableCellsTransposedAsTable(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 한/글 5.x/97 OLE 수식을 편집 가능한 native equation으로 변환한다.
     */
    promoteOleEquation(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 사용자 명시 요청에 의한 lineseg 전체 reflow (#177).
     *
     * `reflow_zero_height_paragraphs` 의 자동 경로와 달리, "빈 line_segs + text 존재"
     * 케이스까지 포함해 재계산한다. 반환값은 실제로 reflow 된 문단 개수.
     *
     * 호출 이후 렌더 캐시·페이지네이션이 갱신되므로 즉시 렌더링하면 보정된 결과가 보인다.
     */
    reflowLinesegs(): number;
    registerCanvasMetricReplies(document: number, fonts: number, revision: number, json: string): boolean;
    /**
     * Browser/host font selection이 확정한 face bytes를 exact layout slot에 등록한다.
     * family 이름 재탐색 없이 `(charShapeId, languageIndex)`에 직접 결합한다.
     */
    registerExactFontSource(char_shape_id: number, language_index: number, font_bytes: Uint8Array, face_index: number): string;
    /**
     * 실제 FontFace의 원본 HFT 프로그램을 paint 전용으로 등록한다. 재조판하지 않는다.
     */
    registerHftOutlineFont(family: string, bytes: Uint8Array, face_index: number): boolean;
    /**
     * [#5959] 이번 apply 가 push 한 BorderFill 꼬리 항목을 절단한다.
     *
     * json: `{"fromLen":12,"dirtyWas":false}` — `from_len` 위 항목을 모두 잘라내고,
     * 원래 길이로 돌아왔으면 dirty 플래그를 `dirtyWas` 로 원복한다.
     * 반환: JSON `{"ok":true,"discarded":N,"fullyDiscarded":bool}`
     */
    removeBorderFillTails(json: string): string;
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (본문 문단).
     */
    removeFieldAt(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (셀/글상자 내 문단).
     */
    removeFieldAtInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, is_textbox: boolean): string;
    /**
     * `removeFieldAtInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
     */
    removeFieldAtInCellEx(options_json: string): string;
    removeHyperlinkEx(options_json: string): void;
    /**
     * 책갈피 이름 변경
     */
    renameBookmark(sec: number, para: number, ctrl_idx: number, new_name: string): string;
    /**
     * 필드 이름을 바꾼다 — 누름틀과 셀 필드를 모두 다룬다.
     *
     * `updateClickHereProps` 는 누름틀 전용이라 셀 필드에서 `{"ok":false}` 를 돌려준다.
     * 웹한글컨트롤 `RenameField`(§8.3.36)의 계약은 두 갈래를 가리지 않는다.
     *
     * 반환: JSON `{"ok":true,"renamed":N}` / `{"ok":false,"renamed":0}`
     */
    renameField(oldname: string, newname: string): string;
    /**
     * 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
     *
     * 반환: 완전한 `<svg>` 문자열
     */
    renderEquationPreview(script: string, font_size_hwpunit: number, color: number): string;
    /**
     * 구역 첫 페이지에 요청한 머리말/꼬리말 정의를 가상 투영해 Canvas 2D로 렌더링한다.
     *
     * 일반 page tree cache와 pagination active target은 바꾸지 않는다. Studio는 결과 canvas를
     * 머리말/꼬리말 밴드에만 clip해 편집 중 비인쇄 overlay로 사용한다.
     */
    renderHeaderFooterEditPreviewToCanvas(page_num: number, section_idx: number, is_header: boolean, apply_to: number, canvas: HTMLCanvasElement, scale: number): void;
    /**
     * 특정 페이지를 Canvas 명령 수로 반환한다.
     */
    renderPageCanvas(page_num: number): number;
    renderPageCanvasLegacy(page_num: number): number;
    /**
     * 특정 페이지를 HTML 문자열로 렌더링한다.
     */
    renderPageHtml(page_num: number): string;
    /**
     * [#3137 Stage 4] 기존 Canvas의 page-space 일부만 다시 재생한다.
     *
     * Canvas 크기와 나머지 픽셀은 유지한다. 호출 조건이나 크기가 맞지 않으면 오류를
     * 반환하며 Studio는 기존 full-page repaint로 폴백한다.
     */
    renderPagePatchToCanvasFilteredWithProfile(page_num: number, canvas: HTMLCanvasElement, scale: number, layer_kind: string, profile: string, x: number, y: number, width: number, height: number): void;
    /**
     * 특정 페이지를 SVG 문자열로 렌더링한다.
     */
    renderPageSvg(page_num: number): string;
    /**
     * 명시적인 출력 profile로 특정 페이지를 SVG 문자열로 렌더링한다.
     */
    renderPageSvgWithProfile(page_num: number, profile: string): string;
    /**
     * 특정 페이지를 Canvas 2D에 직접 렌더링한다.
     *
     * WASM 환경에서만 사용 가능하다. Canvas 크기는 페이지 크기 × scale로 설정된다.
     * scale이 0 이하이면 1.0으로 처리한다 (하위호환).
     */
    renderPageToCanvas(page_num: number, canvas: HTMLCanvasElement, scale: number): void;
    /**
     * 다층 레이어 필터를 적용한 Canvas 렌더링 (Task #516, Stage 5.2).
     *
     * `layer_kind`:
     * - `"all"` → 모든 PaintOp 렌더 (기본 `renderPageToCanvas` 와 동일)
     * - `"background"` → page background layer
     * - `"flow"` → 본문 layer (BehindText / InFrontOfText plane 제외)
     * - `"flow-dynamic"` → 본문 layer 중 Image/RawSvg 제외
     * - `"flow-static"` → page background + 본문 Image/RawSvg layer
     * - `"behind"` → BehindText overlay layer
     * - `"front"` → InFrontOfText overlay layer
     *
     * 본문 Canvas 와 overlay 컨테이너를 분리하는 다층 layer 아키텍처에서 사용.
     */
    renderPageToCanvasFiltered(page_num: number, canvas: HTMLCanvasElement, scale: number, layer_kind: string): void;
    renderPageToCanvasFilteredWithProfile(page_num: number, canvas: HTMLCanvasElement, scale: number, layer_kind: string, profile: string): void;
    /**
     * 특정 페이지를 기존 PageRenderTree 경로로 Canvas 2D에 직접 렌더링한다.
     */
    renderPageToCanvasLegacy(page_num: number, canvas: HTMLCanvasElement, scale: number): void;
    /**
     * 전체 치환
     */
    replaceAll(query: string, new_text: string, case_sensitive: boolean): string;
    replaceBodyTextLocal(section_idx: number, para_idx: number, char_offset: number, delete_count: number, text: string): string;
    replaceHyperlinkTextEx(options_json: string): boolean;
    /**
     * 단일 치환 (검색어 기반) — 첫 번째 매치만 교체
     */
    replaceOne(query: string, new_text: string, case_sensitive: boolean): string;
    /**
     * 머리말/꼬리말 선택 범위를 평문으로 원자 치환한다.
     */
    replaceRangeInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, start_hf_para_idx: number, start_char_offset: number, end_hf_para_idx: number, end_char_offset: number, replacement_text: string): string;
    /**
     * 텍스트 치환 (단일)
     */
    replaceText(sec: number, para: number, char_offset: number, length: number, new_text: string): string;
    /**
     * 표 셀 내부의 짧은 IME 조합 문자열을 원자적으로 교체하고 전체 페이지네이션은 지연한다.
     */
    replaceTextInCellDeferredPagination(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, delete_count: number, text: string): string;
    /**
     * 개체 크기를 한 걸음 바꾼다 — 웹한글컨트롤 `ShapeObjResize*`(걸음 283 HWPUNIT).
     */
    resizeControlAt(para_in_list: number, control_index: number, d_width: number, d_height: number): string;
    /**
     * 여러 셀의 width/height를 한 번에 조절한다 (배치).
     *
     * json: `[{"cellIdx":0,"widthDelta":150},{"cellIdx":2,"heightDelta":-100}]`
     * 반환: JSON `{"ok":true}`
     */
    resizeTableCells(section_idx: number, parent_para_idx: number, control_idx: number, json: string): string;
    /**
     * [#7189] 중첩 표의 셀 크기를 셀 경로로 조절한다 (배치).
     *
     * `cell_path_json`: `[{"controlIndex":0,"cellIndex":0,"cellParaIndex":9},...]`
     * 마지막 항목이 조절할 표를 가리킨다. 깊이 1 이면 평면 API 와 같은 경로로 처리한다.
     */
    resizeTableCellsByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, json: string): string;
    /**
     * 삭제 조각을 원래 자리에 되돌려 끼운다 — 삭제의 참 역연산 (#5769).
     *
     * 스냅샷 복원과 달리 문서 전체가 아니라 삭제 범위+꼬리 줄 좌표만 되돌린다.
     * 성공 시 조각은 소비(제거)된다.
     */
    restoreDeleteFragment(id: number): string;
    /**
     * 캡처한 구역 raw 를 되돌린다 — old 속성 재적용(재무효화) **뒤** 에 불린다 (#5769 Stage 4).
     */
    restoreSectionRaw(id: number): string;
    /**
     * 지정 ID의 스냅샷으로 Document를 복원한다.
     */
    restoreSnapshot(id: number): string;
    /**
     * Document 스냅샷을 저장하고 ID를 반환한다.
     */
    saveSnapshot(): number;
    /**
     * 문서 전체 검색 (모든 매치 반환)
     */
    searchAllText(query: string, case_sensitive: boolean, include_cells: boolean): string;
    /**
     * 문서 텍스트 검색
     *
     * [#3865] `include_cells` 를 참으로 주면 표 셀 안의 일반 텍스트 매치도 돌려준다. 그 경우
     * 결과에 `cellContext`(parentPara·ctrlIdx·cellIdx·cellPara)가 실리므로, 호출자는
     * 그 좌표로 커서를 옮길 수 있어야 한다. 생략하면 종전대로 본문만 본다.
     */
    searchText(query: string, from_sec: number, from_para: number, from_char: number, forward: boolean, case_sensitive: boolean, include_cells?: boolean | null): string;
    selectCanvasMetrics(enabled: boolean): boolean;
    /**
     * 활성 필드를 설정한다 (본문 문단 — 안내문 숨김용).
     */
    setActiveField(section_idx: number, para_idx: number, char_offset: number): boolean;
    /**
     * path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
     */
    setActiveFieldByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): boolean;
    /**
     * 활성 필드를 설정한다 (셀/글상자 내 문단 — 안내문 숨김용).
     * 변경이 발생하면 true를 반환한다.
     */
    setActiveFieldInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, is_textbox: boolean): boolean;
    /**
     * `setActiveFieldInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(bool 반환).
     */
    setActiveFieldInCellEx(options_json: string): boolean;
    /**
     * [#4709] SVG 출력에 배치 메트릭 face 주석을 붙일지 설정한다 (기본 꺼짐).
     *
     * 켜면 `renderPageSvg` 계열 출력의 각 `<text>`에 `data-metric-font`,
     * 루트 `<svg>`에 `data-rhwp-metric-fonts`(쉼표 구분 목록)가 붙는다.
     * 임베드 호스트가 해당 폰트 설치 여부 확인·대체 폰트 자간 보정에 쓴다.
     * 배치(레이아웃)에는 영향이 없는 뷰 전용 주석이다.
     */
    setAnnotateMetricFont(enabled: boolean): void;
    /**
     * [#4180] 저장 직전 UI 캐럿을 문서 캐럿 메타데이터에 반영한다
     * (한컴 의미론: 저장 시점 캐럿). 범위 밖 위치는 무시 — 저장을 막지 않는다.
     */
    setCaretPosition(section_idx: number, para_idx: number, char_offset: number): void;
    /**
     * 셀 내 문단의 paraShapeId를 직접 설정한다.
     */
    setCellParaShapeId(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, para_shape_id: number): string;
    /**
     * [Task #1151 v4] 표 셀 내 Picture 속성 변경 (by_path). Shape 패턴 정합.
     */
    setCellPicturePropertiesByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number, props_json: string): string;
    /**
     * 셀 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true,"changes":[{cellIdx,beforeId,afterId}...],
     * "borderFillLenBefore":N,"docInfoDirtyBefore":bool}` — changes 는 [#5959]
     * borderFillId 전환 기록(target+이웃)이다.
     */
    setCellProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, json: string): string;
    /**
     * [Task #1138] 표 셀 내 Shape 속성 변경 (by_path).
     */
    setCellShapePropertiesByPath(section_idx: number, parent_para_idx: number, cell_path_json: string, inner_control_idx: number, props_json: string): string;
    /**
     * 선택 영역을 하나의 셀처럼 취급하는 cellzone 테두리/배경 속성을 적용한다.
     *
     * 반환: JSON `{"ok":true,"startRow":...,"borderFillId":...,"zoneBeforeId":...,
     * "borderFillLenBefore":...,"docInfoDirtyBefore":...}`
     */
    setCellZoneProperties(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number, json: string): string;
    /**
     * 글자 서식 ID를 직접 복원한다 (본문 문단).
     */
    setCharShapeId(sec_idx: number, para_idx: number, start_offset: number, end_offset: number, char_shape_id: number): string;
    /**
     * 글자 서식 ID를 직접 복원한다 (셀 내 문단).
     */
    setCharShapeIdInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, start_offset: number, end_offset: number, char_shape_id: number): string;
    setCharShapeIdInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, start_offset: number, end_offset: number, char_shape_id: number): string;
    /**
     * `setCharShapeIdInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * startOffset, endOffset, charShapeId }`. positional 과 동일 동작.
     */
    setCharShapeIdInCellEx(options_json: string): string;
    /**
     * 구간 목록 전체를 검사한 뒤 본문 모양을 복원한다.
     */
    setCharShapeRuns(sec: number, para: number, start: number, end: number, runs_json: string): string;
    setCharShapeRunsInCellByPath(sec: number, para: number, path_json: string, start: number, end: number, runs_json: string): string;
    /**
     * [#4694] 본문 직속 차트의 숫자 데이터를 바꾼다 (3인자 주소).
     *
     * 반환: `{ok, chart, changedCount, changed[], wrote[]}` 또는 `{ok:false, invalid[]}`.
     */
    setChartData(section_idx: number, parent_para_idx: number, control_idx: number, edits_json: string): string;
    /**
     * [#4694] 문서 순번(0-based)으로 차트 데이터를 바꾼다 — 정본 주소.
     */
    setChartDataByIndex(index: number, edits_json: string): string;
    setClipEnabled(enabled: boolean): void;
    /**
     * 다단 설정 변경
     * column_type: 0=일반, 1=배분, 2=평행
     * same_width: 0=다른 너비, 1=같은 너비
     */
    setColumnDef(section_idx: number, column_count: number, column_type: number, same_width: number, spacing_hu: number): string;
    /**
     * 개체를 뒤집는다 — 웹한글컨트롤 `Run("ShapeObjHorzFlip")` 계열.
     */
    setControlFlipAt(para_in_list: number, control_index: number, vertical: boolean, org_state: boolean): string;
    /**
     * 개체의 잠금을 켜고 끈다 — 웹한글컨트롤 `ShapeObjLock`·`ShapeObjUnlockAll`.
     *
     * 문단·컨트롤 번호에 `u32::MAX` 를 주면 "모두"라는 뜻이다(모두 풀기가 쓴다).
     */
    setControlLock(para_in_list: number, control_index: number, locked: boolean): string;
    /**
     * 개체의 앞뒤 순서를 바꾼다 — 웹한글컨트롤 `Run("ShapeObjBringToFront")` 계열.
     *
     * `mode` 는 `front`·`back`·`forward`·`backward`·`inFrontOfText`·`behindText`.
     */
    setControlZOrderAt(para_in_list: number, control_index: number, mode: string): string;
    /**
     * DPI를 설정한다.
     */
    setDpi(dpi: number): void;
    /**
     * 수식 컨트롤의 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    setEquationProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, props_json: string): string;
    /**
     * 이미 등록된 exact font source slot에 명시 variable-font instance 요청을 설정한다.
     * JSON DTO의 검증·canonicalization·mutation 권위는 native command 한 곳에만 둔다.
     */
    setExactFontInstance(options_json: string): string;
    /**
     * 대체 폰트 경로를 설정한다.
     */
    setFallbackFont(path: string): void;
    /**
     * field_id로 필드 값을 설정한다.
     *
     * 반환: `{ok, fieldId, oldValue, newValue}`
     */
    setFieldValue(field_id: number, value: string): string;
    /**
     * 필드 이름으로 값을 설정한다.
     *
     * 반환: `{ok, fieldId, oldValue, newValue}`
     */
    setFieldValueByName(name: string, value: string): string;
    /**
     * 파일 이름을 설정한다 (머리말/꼬리말 필드 치환용).
     */
    setFileName(name: string): void;
    /**
     * Set an explicit layout/paint font environment. None restores the default.
     */
    setFontEnvironment(json?: string | null): boolean;
    /**
     * 양식 개체 값을 설정한다.
     *
     * value_json: `{"value":1}` 또는 `{"text":"입력값"}`
     * 반환: `{ok}`
     */
    setFormValue(sec: number, para: number, ci: number, value_json: string): string;
    /**
     * 셀 내부 양식 개체 값을 설정한다.
     *
     * table_para: 표를 포함한 최상위 문단 인덱스
     * table_ci: 표 컨트롤 인덱스
     * cell_idx: 셀 인덱스
     * cell_para: 셀 내 문단 인덱스
     * form_ci: 셀 내 양식 컨트롤 인덱스
     * value_json: `{"value":1}` 또는 `{"text":"입력값"}`
     * 반환: `{ok}`
     */
    setFormValueInCell(sec: number, table_para: number, table_ci: number, cell_idx: number, cell_para: number, form_ci: number, value_json: string): string;
    /**
     * `setFormValueInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, tablePara, tableCi, cellIdx, cellPara, formCi, value: object }`.
     * positional 과 동일 동작.
     */
    setFormValueInCellEx(options_json: string): string;
    /**
     * Native와 같은 조판 세대를 선택하고 공통 core에서 재페이지네이션한다.
     */
    setHangul2024Compat(enabled: boolean): void;
    /**
     * [Task #825] 머리말/꼬리말 안 그림 속성 변경.
     */
    setHeaderFooterPictureProperties(section_idx: number, outer_para_idx: number, outer_control_idx: number, inner_para_idx: number, inner_control_idx: number, props_json: string): string;
    /**
     * 각주/미주 내부 수식 컨트롤의 속성을 변경한다.
     */
    setNoteEquationProperties(kind: string, section_idx: number, parent_para_idx: number, note_control_idx: number, note_para_idx: number, inner_control_idx: number, props_json: string): string;
    /**
     * `setNoteEquationProperties` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ kind, sectionIdx, parentParaIdx, noteControlIdx, noteParaIdx,
     * innerControlIdx, props: object }`. positional 과 동일 동작.
     */
    setNoteEquationPropertiesEx(options_json: string): string;
    /**
     * 문단 서식을 적용한다 (본문 문단).
     * 문단 번호 시작 방식 설정
     */
    setNumberingRestart(section_idx: number, para_idx: number, mode: number, start_num: number): string;
    /**
     * 구역의 쪽 테두리/배경 설정을 변경하고 재페이지네이션한다.
     */
    setPageBorderFill(section_idx: number, json: string): string;
    /**
     * 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션한다.
     */
    setPageDef(section_idx: number, json: string): string;
    /**
     * 감추기 설정
     */
    setPageHide(sec: number, para: number, hide_header: boolean, hide_footer: boolean, hide_master: boolean, hide_border: boolean, hide_fill: boolean, hide_page_num: boolean): string;
    /**
     * `setPageHide` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, para, hideHeader?, hideFooter?, hideMaster?, hideBorder?,
     * hideFill?, hidePageNum? }`. positional 과 동일 동작.
     */
    setPageHideEx(options_json: string): string;
    /**
     * 문단의 paraShapeId를 직접 설정한다.
     */
    setParaShapeId(sec_idx: number, para_idx: number, para_shape_id: number): string;
    /**
     * 그림 컨트롤의 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    setPictureProperties(section_idx: number, parent_para_idx: number, control_idx: number, props_json: string): string;
    /**
     * 구역 정의(SectionDef)를 변경하고 재페이지네이션한다.
     */
    setSectionDef(section_idx: number, json: string): string;
    /**
     * 모든 구역의 SectionDef를 일괄 변경하고 재페이지네이션한다.
     */
    setSectionDefAll(json: string): string;
    /**
     * Shape(글상자) 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    setShapeProperties(section_idx: number, parent_para_idx: number, control_idx: number, props_json: string): string;
    /**
     * 조판부호 표시 여부를 설정한다 (개체 마커 + 문단부호 포함).
     */
    setShowControlCodes(enabled: boolean): void;
    /**
     * 문단부호(¶) 표시 여부를 설정한다.
     */
    setShowParagraphMarks(enabled: boolean): void;
    /**
     * 투명선 표시 여부를 설정한다.
     */
    setShowTransparentBorders(enabled: boolean): void;
    /**
     * 표 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    setTableProperties(section_idx: number, parent_para_idx: number, control_idx: number, json: string): string;
    /**
     * 개체에 글상자를 붙이거나 뗀다 — 웹한글컨트롤 `Run("ShapeObjAttach/DetachTextBox")`.
     */
    setTextBoxAt(para_in_list: number, control_index: number, attach: boolean): string;
    /**
     * 디버그 오버레이 표시 여부를 설정한다.
     */
    set_debug_overlay(enabled: boolean): void;
    /**
     * LINE_SEG vpos-reset 강제 분리 적용 여부를 설정한다.
     * 변경 시 페이지네이션 결과가 달라지므로 모든 섹션을 재페이지네이션한다.
     */
    set_respect_vpos_reset(enabled: boolean): void;
    /**
     * undo 스냅샷 저장소의 축출 상한. studio 예산의 유일한 출처다 (#7002 후속).
     *
     * studio 는 이 값에서 예산(`상한 - 2`)을 계산한다. 상수를 양쪽에 두면 순 Rust
     * 변경에서 frontend 레인이 skip 되어 드리프트가 CI 를 통과했다 — 값을 내보내
     * 사본을 없앤다.
     */
    snapshotCapacity(): number;
    /**
     * 커서 자리에서 문단을 가른다 — 웹한글컨트롤 `Run("BreakPara")`.
     */
    splitParaAtCursor(list_id: number, para_in_list: number, pos: number): string;
    /**
     * 캐럿 위치에서 문단을 분할한다 (Enter 키).
     *
     * char_offset 이후의 텍스트가 새 문단으로 이동한다.
     * 반환값: JSON `{"ok":true,"paraIdx":<new_para_idx>,"charOffset":0}`
     */
    splitParagraph(section_idx: number, para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    /**
     * 셀 내부 문단을 분할한다 (셀 내 Enter 키).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<new_idx>,"charOffset":0}`
     *
     * `removed_para_meta` 는 병합 undo 가 되돌려주는 값이다 — 본문 `splitParagraph`
     * 와 같은 규약이다 (Task #2342).
     */
    splitParagraphInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    splitParagraphInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, removed_para_meta?: string | null): string;
    /**
     * 각주 내 문단을 분할한다 (Enter).
     */
    splitParagraphInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    /**
     * 머리말/꼬리말 내 문단 분할 (Enter 키)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<new_idx>,"charOffset":0}`
     */
    splitParagraphInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    /**
     * 표를 지정 행에서 두 개로 나눈다 (한컴 [표-표 나누기]).
     *
     * 반환값: JSON `{"ok":true,"frontRows":<N>,"backParaIdx":<P>}`
     */
    splitTable(section_idx: number, parent_para_idx: number, control_idx: number, at_row: number): string;
    /**
     * 병합된 셀을 나눈다 (split).
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     */
    splitTableCell(section_idx: number, parent_para_idx: number, control_idx: number, row: number, col: number): string;
    /**
     * 셀을 N줄 × M칸으로 분할한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     */
    splitTableCellInto(section_idx: number, parent_para_idx: number, control_idx: number, row: number, col: number, n_rows: number, m_cols: number, equal_row_height: boolean, merge_first: boolean): string;
    /**
     * `splitTableCellInto` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, row, col, nRows, mCols,
     * equalRowHeight?, mergeFirst? }`. positional 과 동일 동작.
     */
    splitTableCellIntoEx(options_json: string): string;
    /**
     * 범위 내 셀들을 각각 N줄 × M칸으로 분할한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     */
    splitTableCellsInRange(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number, n_rows: number, m_cols: number, equal_row_height: boolean): string;
    /**
     * `splitTableCellsInRange` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
     * endRow, endCol, nRows, mCols, equalRowHeight? }`. positional 과 동일 동작.
     */
    splitTableCellsInRangeEx(options_json: string): string;
    /**
     * 대형 표 continuation을 fragment budget만큼 전진한다.
     */
    stepDeferredPagination(fragment_budget: number): string;
    /**
     * 현재 WASM 빌드에서 PDF 입력을 사용할 수 있는지 문서 생성 전에 조회한다.
     */
    static supportsPdfImport(): boolean;
    /**
     * 저장 상태와 현재 상태를 교환한다. 같은 ID로 Undo/Redo를 수행한다.
     */
    swapPictureTransform(id: number): void;
    /**
     * 커서가 든 셀을 기준으로 표를 고친다 — 웹한글컨트롤 `Run("TableInsert*"·"TableDelete*")`.
     *
     * `op` 는 `insertRowAbove`·`insertRowBelow`·`insertColLeft`·`insertColRight`·
     * `deleteRow`·`deleteCol`.
     */
    tableEditAtCursor(list_id: number, op: string): string;
    /**
     * 커서가 든 셀에서 `(endRow, endCol)` 까지를 하나로 합친다 — `Run("TableMergeCell")`.
     */
    tableMergeAtCursor(list_id: number, end_row: number, end_col: number): string;
    /**
     * 텍스트 오프셋 → 논리적 오프셋 변환.
     */
    textToLogicalOffset(section_idx: number, para_idx: number, text_offset: number): number;
    /**
     * 머리말/꼬리말 감추기를 토글한다 (현재 쪽만).
     *
     * 반환: JSON `{"hidden":true/false}` — 토글 후 상태
     */
    toggleHideHeaderFooter(page_index: number, is_header: boolean): string;
    /**
     * 선택된 전체 표를 제자리에서 전치한다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     */
    transposeTableCellsInPlace(section_idx: number, parent_para_idx: number, control_idx: number): string;
    /**
     * GroupShape를 풀어 자식 개체들을 개별로 복원한다.
     */
    ungroupShape(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 누름틀 필드의 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}` 또는 `{"ok":false}`
     */
    updateClickHereProps(field_id: number, guide: string, memo: string, name: string, editable: boolean): string;
    /**
     * 구역 내 모든 연결선의 좌표를 연결된 도형 위치에 맞게 갱신한다.
     */
    updateConnectorsInSection(section_idx: number): void;
    updateHyperlinkEx(options_json: string): boolean;
    /**
     * 스타일의 메타 정보(이름/영문이름/nextStyleId)를 수정한다.
     *
     * json: {"name":"...", "englishName":"...", "nextStyleId":0}
     */
    updateStyle(style_id: number, json: string): boolean;
    /**
     * 스타일의 CharShape/ParaShape를 수정한다.
     *
     * charMods/paraMods는 기존 parse_char_shape_mods/parse_para_shape_mods와 동일한 JSON 형식
     */
    updateStyleShapes(style_id: number, char_mods_json: string, para_mods_json: string): boolean;
}

/**
 * WASM 뷰어 컨트롤러 (뷰포트 관리 + 스케줄링)
 */
export class HwpViewer {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 뷰어 생성
     */
    constructor(document: HwpDocument);
    /**
     * 총 페이지 수
     */
    pageCount(): number;
    /**
     * 대기 중인 렌더링 작업 수
     */
    pendingTaskCount(): number;
    /**
     * 특정 페이지 HTML 렌더링
     */
    renderPageHtml(page_num: number): string;
    /**
     * 특정 페이지 SVG 렌더링
     */
    renderPageSvg(page_num: number): string;
    /**
     * 명시적인 출력 profile로 특정 페이지 SVG 렌더링
     */
    renderPageSvgWithProfile(page_num: number, profile: string): string;
    /**
     * [#4709] SVG 출력에 배치 메트릭 face 주석을 붙일지 설정한다 (기본 꺼짐).
     */
    setAnnotateMetricFont(enabled: boolean): void;
    /**
     * 줌 변경
     */
    setZoom(zoom: number): void;
    /**
     * 뷰포트 업데이트 (스크롤/리사이즈 시 호출)
     */
    updateViewport(scroll_x: number, scroll_y: number, width: number, height: number): void;
    /**
     * 현재 보이는 페이지 목록 반환
     */
    visiblePages(): Uint32Array;
}

/**
 * HWP 파일에서 썸네일 이미지만 경량 추출 (전체 파싱 없이)
 *
 * 반환: JSON `{ "format": "png"|"gif", "base64": "...", "width": N, "height": N }`
 * PrvImage가 없으면 `null` 반환
 */
export function extractThumbnail(data: Uint8Array): any;

/**
 * WASM panic hook 초기화 (한 번만 실행)
 */
export function init_panic_hook(): void;

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly version: () => [number, number];
    readonly init_panic_hook: () => void;
    readonly __wbg_documentexport_free: (a: number, b: number) => void;
    readonly __wbg_hwpdocument_free: (a: number, b: number) => void;
    readonly __wbg_hwpviewer_free: (a: number, b: number) => void;
    readonly documentexport_contentLoss: (a: number) => [number, number];
    readonly documentexport_hasBytes: (a: number) => number;
    readonly documentexport_takeBytes: (a: number) => [number, number, number, number];
    readonly extractThumbnail: (a: number, b: number) => any;
    readonly hwpdocument_addBookmark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_applyCellBorderFillIds: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_applyCellStyle: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormat: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_applyEndnoteShape: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_applyHfTemplate: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormat: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatAtCursor: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatInHf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyShapeZOrderPairs: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_applyStyle: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_applyTemplateOperation: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_assignPictureImage: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number, number];
    readonly hwpdocument_attachCaptionAt: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_beginBatch: (a: number) => [number, number, number, number];
    readonly hwpdocument_beginDeferredPagination: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_breakAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_cancelDeferredPagination: (a: number) => number;
    readonly hwpdocument_canvasMetricsActive: (a: number) => number;
    readonly hwpdocument_captureDeleteRange: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_capturePictureTransform: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_captureSectionRaw: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_changeShapeZOrder: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_clearActiveField: (a: number) => void;
    readonly hwpdocument_clearClipboard: (a: number) => void;
    readonly hwpdocument_clearExactFontInstance: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_clearTableCellsAtCursor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_clipboardHasControl: (a: number) => number;
    readonly hwpdocument_collectCanvasMetricRequests: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_convertToEditable: (a: number) => [number, number, number, number];
    readonly hwpdocument_copyControl: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_copySelection: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionLogical: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_copyTableCellsTransposed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_createBlankDocument: (a: number) => [number, number, number, number];
    readonly hwpdocument_createEmpty: () => number;
    readonly hwpdocument_createHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_createNumbering: (a: number, b: number, c: number) => number;
    readonly hwpdocument_createShapeControl: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_createStyle: (a: number, b: number, c: number) => number;
    readonly hwpdocument_createTable: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_createTableEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deleteAtCursor: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteBookmark: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteCellPictureControlByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_deleteCellTableControlByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_deleteControlAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteEquationControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deletePictureControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteRange: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_deleteRangeInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_deleteRangeInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_deleteRangeInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deleteShapeControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteStyle: (a: number, b: number) => number;
    readonly hwpdocument_deleteTableColumn: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteTableControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteTableRow: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteText: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCellDeferredPagination: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_detachCaptionAt: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_discardDeleteFragment: (a: number, b: number) => void;
    readonly hwpdocument_discardPictureTransform: (a: number, b: number) => void;
    readonly hwpdocument_discardSectionRaw: (a: number, b: number) => void;
    readonly hwpdocument_discardSnapshot: (a: number, b: number) => void;
    readonly hwpdocument_endBatch: (a: number) => [number, number, number, number];
    readonly hwpdocument_ensureDefaultBullet: (a: number, b: number, c: number) => number;
    readonly hwpdocument_ensureDefaultNumbering: (a: number) => number;
    readonly hwpdocument_evaluateTableFormula: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_evaluateTableFormulaEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_exportControlHtml: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_exportHml: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwp: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpVerify: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpWithPassword: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpWithPasswordAndReport: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_exportHwpWithReport: (a: number) => [number, number, number];
    readonly hwpdocument_exportHwpx: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpxWithPassword: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpxWithPasswordAndReport: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_exportHwpxWithReport: (a: number) => [number, number, number];
    readonly hwpdocument_exportSelectionHtml: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_exportSelectionInCellHtml: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_exportSelectionInCellHtmlByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_exportSelectionInCellHtmlEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_findNearestControlBackward: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_findNearestControlForward: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_findNextEditableControl: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly hwpdocument_findOrCreateFontId: (a: number, b: number, c: number) => number;
    readonly hwpdocument_findOrCreateFontIdForLang: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_flushDeferredPagination: (a: number) => [number, number, number, number];
    readonly hwpdocument_getAnnotateMetricFont: (a: number) => number;
    readonly hwpdocument_getBookmarks: (a: number) => [number, number, number, number];
    readonly hwpdocument_getBulletList: (a: number) => [number, number];
    readonly hwpdocument_getCanvasKitDocumentPreflight: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCanvasKitReplayPlan: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCanvasKitReplayPlanWithProfile: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCanvasPageLayerTree: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCaretPosition: (a: number) => [number, number, number, number];
    readonly hwpdocument_getCaretStops: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getCellCharPropertiesAt: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCellCharPropertiesAtByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellInfo: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellInfoByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellOwnProperties: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellParaPropertiesAt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellParagraphCount: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphCountByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphLength: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphLengthByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellPicturePropertiesByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellProperties: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellPropertiesByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellShapePropertiesByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellShapeSet: (a: number, b: number) => [number, number];
    readonly hwpdocument_getCellStyleAt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_getCellTextDirection: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCharIndexAtStreamPos: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCharPropertiesAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCharPropertiesInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCharShapeRuns: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCharShapeRunsInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCharShapeSet: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_getChartData: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getChartDataByIndex: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getClickHereProps: (a: number, b: number) => [number, number];
    readonly hwpdocument_getClipboardText: (a: number) => [number, number];
    readonly hwpdocument_getColumnDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getControlImageData: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getControlImageMime: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getControlTextPositions: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getControls: (a: number) => [number, number];
    readonly hwpdocument_getCurFieldState: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_getCursorModel: (a: number) => [number, number];
    readonly hwpdocument_getCursorRect: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectByPathNear: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInFootnote: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInNote: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectOnLine: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_getDocumentInfo: (a: number) => [number, number];
    readonly hwpdocument_getDpi: (a: number) => number;
    readonly hwpdocument_getEndnoteShape: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getEventLog: (a: number) => [number, number];
    readonly hwpdocument_getExternalImageBasenames: (a: number) => [number, number];
    readonly hwpdocument_getExternalImageReferences: (a: number) => [number, number];
    readonly hwpdocument_getFallbackFont: (a: number) => [number, number];
    readonly hwpdocument_getFieldInfoAt: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_getFieldInfoAtByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_getFieldInfoAtInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly hwpdocument_getFieldInfoAtInCellEx: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getFieldList: (a: number) => [number, number];
    readonly hwpdocument_getFieldValue: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getFieldValueByName: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getFontDecisionTrace: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFootnoteAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getFootnoteInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormObjectAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormObjectInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormValue: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterEditTarget: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterList: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterParaInfo: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterPreviewPage: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getHmlOpenMetadata: (a: number) => [number, number];
    readonly hwpdocument_getHmlSaveState: (a: number) => [number, number];
    readonly hwpdocument_getHyperlinkContext: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getLineInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getLineInfoInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getLineStarts: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getLogicalLength: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_getNoteEditInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getNoteEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_getNumberingList: (a: number) => [number, number];
    readonly hwpdocument_getObjectCycle: (a: number) => [number, number, number, number];
    readonly hwpdocument_getObjects: (a: number) => [number, number];
    readonly hwpdocument_getOutlineNavigation: (a: number) => [number, number, number, number];
    readonly hwpdocument_getPageBorderFill: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageCaretStarts: (a: number) => [number, number, number, number];
    readonly hwpdocument_getPageControlLayout: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageFlowImageOps: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageFootnoteInfo: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageHide: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageInfo: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTree: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2Compat: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2CompatWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2StrictGlyphOutline: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2StrictGlyphOutlineWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2StrictGlyphRun: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeV2StrictGlyphRunWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTreeValue: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2Compat: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2CompatWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2CompatWithProfileAndResourceKeys: (a: number, b: number, c: number, d: number, e: any, f: any) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphOutline: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphOutlineWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphOutlineWithProfileAndResourceKeys: (a: number, b: number, c: number, d: number, e: any, f: any) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphRun: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphRunWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueV2StrictGlyphRunWithProfileAndResourceKeys: (a: number, b: number, c: number, d: number, e: any, f: any) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeValueWithProfileAndResourceKeys: (a: number, b: number, c: number, d: number, e: any, f: any, g: number, h: number) => [number, number, number];
    readonly hwpdocument_getPageLayerTreeWithProfile: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getPageOfPosition: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageOverlayImages: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageRenderTree: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageSourceImageKeys: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageText: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageTextLayout: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getParaBounds: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getParaPropertiesAt: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getParaPropertiesInFootnote: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getParaPropertiesInHf: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getParaShapeSet: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getParagraphCount: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getParagraphLength: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_getPdfImportDiagnostics: (a: number) => [number, number];
    readonly hwpdocument_getPdfImportPolicy: () => [number, number];
    readonly hwpdocument_getPictureProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getPositionOfPage: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getScanItems: (a: number) => [number, number];
    readonly hwpdocument_getSectionCount: (a: number) => number;
    readonly hwpdocument_getSectionDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getSectionStarts: (a: number) => [number, number];
    readonly hwpdocument_getSelectionRects: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInCellByPathEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_getShapeBBox: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getShapeProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getShowControlCodes: (a: number) => number;
    readonly hwpdocument_getShowParagraphMarks: (a: number) => number;
    readonly hwpdocument_getShowTransparentBorders: (a: number) => number;
    readonly hwpdocument_getSourceFontBytes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getSourceFormat: (a: number) => [number, number];
    readonly hwpdocument_getSourceImageBytes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getStoredCaret: (a: number) => [number, number];
    readonly hwpdocument_getStructure: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getStyleAt: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getStyleDetail: (a: number, b: number) => [number, number];
    readonly hwpdocument_getStyleList: (a: number) => [number, number];
    readonly hwpdocument_getTableBBox: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTableBBoxAtPage: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableCellBboxes: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableCellBboxesAtPage: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableCellBboxesByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableDimensions: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTableDimensionsByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTextBoxControlIndex: (a: number, b: number, c: number) => number;
    readonly hwpdocument_getTextFileText: (a: number) => [number, number];
    readonly hwpdocument_getTextFileUnicode: (a: number) => [number, number];
    readonly hwpdocument_getTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_getTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getTextInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getTextRange: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getValidationWarnings: (a: number) => [number, number];
    readonly hwpdocument_getWordEnd: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_getWordStarts: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_groupShapes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_hasInternalClipboard: (a: number) => number;
    readonly hwpdocument_hasTableTransposeClipboard: (a: number) => number;
    readonly hwpdocument_hitTest: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestBodyFootnoteMarker: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestInFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestInHeaderFooter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_hitTestInHeaderFooterTarget: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_importParagraphBlock: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_importPdf: (a: number, b: number, c: any) => [number, number, number];
    readonly hwpdocument_injectExternalImage: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly hwpdocument_injectExternalImageByKey: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly hwpdocument_insertAutoNumberAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereField: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldByPathEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => [number, number, number, number];
    readonly hwpdocument_insertClickHereFieldInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertColumnBreak: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertEndnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertEquation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertFieldInHf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_insertFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertHyperlinkEx: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_insertNewNumber: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_insertPageBreak: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertPicture: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number, r: number) => [number, number, number, number];
    readonly hwpdocument_insertPictureEx: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_insertTableColumn: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertTableRow: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertText: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertTextAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCellDeferredPagination: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextLogical: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_isEmptyDocument: (a: number) => number;
    readonly hwpdocument_listCharts: (a: number) => [number, number, number, number];
    readonly hwpdocument_logicalToTextOffset: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_measureWidthDiagnostic: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInCell: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInCellByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInFootnote: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInHeaderFooter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeTableCells: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_mergeTableCellsEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_mergeTableWithNext: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_moveControlAt: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_moveLineEndpoint: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_moveLineEndpointEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_moveTableOffset: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_moveVertical: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_moveVerticalByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_moveVerticalEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_navigateHeaderFooterByPage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_navigateNextEditable: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly hwpdocument_new: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_openWithPassword: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_pageCount: (a: number) => number;
    readonly hwpdocument_pageHasFootnoteFootholds: (a: number, b: number) => number;
    readonly hwpdocument_pasteControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtml: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtmlInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtmlInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtmlInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_pasteHwpJson: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_pasteInternal: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_pasteInternalInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_pasteInternalInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_pasteTableCellsTransposed: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_pasteTableCellsTransposedAsTable: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_promoteOleEquation: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_reflowLinesegs: (a: number) => number;
    readonly hwpdocument_registerCanvasMetricReplies: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly hwpdocument_registerExactFontSource: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_registerHftOutlineFont: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly hwpdocument_removeBorderFillTails: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_removeFieldAt: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_removeFieldAtInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly hwpdocument_removeFieldAtInCellEx: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_removeHyperlinkEx: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_renameBookmark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_renameField: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly hwpdocument_renderEquationPreview: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_renderHeaderFooterEditPreviewToCanvas: (a: number, b: number, c: number, d: number, e: number, f: any, g: number) => [number, number];
    readonly hwpdocument_renderPageCanvas: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_renderPageCanvasLegacy: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_renderPageHtml: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_renderPagePatchToCanvasFilteredWithProfile: (a: number, b: number, c: any, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number];
    readonly hwpdocument_renderPageSvg: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_renderPageSvgWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_renderPageToCanvas: (a: number, b: number, c: any, d: number) => [number, number];
    readonly hwpdocument_renderPageToCanvasFiltered: (a: number, b: number, c: any, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_renderPageToCanvasFilteredWithProfile: (a: number, b: number, c: any, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly hwpdocument_renderPageToCanvasLegacy: (a: number, b: number, c: any, d: number) => [number, number];
    readonly hwpdocument_replaceAll: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_replaceBodyTextLocal: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_replaceHyperlinkTextEx: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_replaceOne: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_replaceRangeInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_replaceText: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_replaceTextInCellDeferredPagination: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_resizeControlAt: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_resizeTableCells: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_resizeTableCellsByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_restoreDeleteFragment: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_restoreSectionRaw: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_restoreSnapshot: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_saveSnapshot: (a: number) => number;
    readonly hwpdocument_searchAllText: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_searchText: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_selectCanvasMetrics: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_setActiveField: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_setActiveFieldByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly hwpdocument_setActiveFieldInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly hwpdocument_setActiveFieldInCellEx: (a: number, b: number, c: number) => number;
    readonly hwpdocument_setAnnotateMetricFont: (a: number, b: number) => void;
    readonly hwpdocument_setCaretPosition: (a: number, b: number, c: number, d: number) => void;
    readonly hwpdocument_setCellParaShapeId: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_setCellPicturePropertiesByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setCellProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_setCellShapePropertiesByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setCellZoneProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeId: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeIdInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeIdInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeIdInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeRuns: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_setCharShapeRunsInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setChartData: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setChartDataByIndex: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setClipEnabled: (a: number, b: number) => void;
    readonly hwpdocument_setColumnDef: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setControlFlipAt: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setControlLock: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setControlZOrderAt: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setDpi: (a: number, b: number) => void;
    readonly hwpdocument_setEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setExactFontInstance: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setFallbackFont: (a: number, b: number, c: number) => void;
    readonly hwpdocument_setFieldValue: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setFieldValueByName: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setFileName: (a: number, b: number, c: number) => void;
    readonly hwpdocument_setFontEnvironment: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_setFormValue: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setFormValueInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setFormValueInCellEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setHangul2024Compat: (a: number, b: number) => void;
    readonly hwpdocument_setHeaderFooterPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setNoteEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_setNoteEquationPropertiesEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setNumberingRestart: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setPageBorderFill: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setPageDef: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setPageHide: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setPageHideEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setParaShapeId: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setSectionDef: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setSectionDefAll: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setShapeProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setShowControlCodes: (a: number, b: number) => void;
    readonly hwpdocument_setShowParagraphMarks: (a: number, b: number) => void;
    readonly hwpdocument_setShowTransparentBorders: (a: number, b: number) => void;
    readonly hwpdocument_setTableProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setTextBoxAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_set_debug_overlay: (a: number, b: number) => void;
    readonly hwpdocument_set_respect_vpos_reset: (a: number, b: number) => void;
    readonly hwpdocument_snapshotCapacity: (a: number) => number;
    readonly hwpdocument_splitParaAtCursor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraph: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_splitTable: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCell: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellInto: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellIntoEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellsInRange: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellsInRangeEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_stepDeferredPagination: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_supportsPdfImport: () => number;
    readonly hwpdocument_swapPictureTransform: (a: number, b: number) => [number, number];
    readonly hwpdocument_tableEditAtCursor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_tableMergeAtCursor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_textToLogicalOffset: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_toggleHideHeaderFooter: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_transposeTableCellsInPlace: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_ungroupShape: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_updateClickHereProps: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly hwpdocument_updateConnectorsInSection: (a: number, b: number) => void;
    readonly hwpdocument_updateHyperlinkEx: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_updateStyle: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_updateStyleShapes: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly hwpviewer_new: (a: number) => number;
    readonly hwpviewer_pendingTaskCount: (a: number) => number;
    readonly hwpviewer_renderPageHtml: (a: number, b: number) => [number, number, number, number];
    readonly hwpviewer_renderPageSvg: (a: number, b: number) => [number, number, number, number];
    readonly hwpviewer_renderPageSvgWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpviewer_setZoom: (a: number, b: number) => void;
    readonly hwpviewer_updateViewport: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly hwpviewer_visiblePages: (a: number) => [number, number];
    readonly hwpviewer_pageCount: (a: number) => number;
    readonly hwpviewer_setAnnotateMetricFont: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
