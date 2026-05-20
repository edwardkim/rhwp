/* tslint:disable */
/* eslint-disable */

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
     * 스타일을 적용한다 (셀 내 문단).
     */
    applyCellStyle(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, style_id: number): string;
    /**
     * 글자 서식을 적용한다 (본문 문단).
     */
    applyCharFormat(sec_idx: number, para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    /**
     * 글자 서식을 적용한다 (셀 내 문단).
     */
    applyCharFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    /**
     * 머리말/꼬리말 마당(템플릿)을 적용한다.
     */
    applyHfTemplate(section_idx: number, is_header: boolean, apply_to: number, template_id: number): string;
    applyParaFormat(sec_idx: number, para_idx: number, props_json: string): string;
    /**
     * 문단 서식을 적용한다 (셀 내 문단).
     */
    applyParaFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, props_json: string): string;
    /**
     * 머리말/꼬리말 문단에 문단 서식을 적용한다.
     */
    applyParaFormatInHf(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, props_json: string): string;
    /**
     * 스타일을 적용한다 (본문 문단).
     */
    applyStyle(sec_idx: number, para_idx: number, style_id: number): string;
    /**
     * Batch 모드를 시작한다. 이후 Command 호출 시 paginate()를 건너뛴다.
     */
    beginBatch(): string;
    /**
     * Shape z-order 변경
     * operation: "front" | "back" | "forward" | "backward"
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
     * 내부 클립보드에 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
     */
    clipboardHasControl(): boolean;
    /**
     * 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다.
     *
     * 반환값: JSON `{"ok":true,"converted":true}` 또는 `{"ok":true,"converted":false}`
     */
    convertToEditable(): string;
    /**
     * 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
     */
    copyControl(section_idx: number, para_idx: number, control_idx: number): string;
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
     * 내장 템플릿에서 빈 문서를 생성한다.
     *
     * saved/blank2010.hwp를 WASM 바이너리에 포함하여 유효한 HWP 문서를 즉시 생성.
     * DocInfo raw_stream이 온전하므로 FIX-4 워크어라운드와 호환됨.
     */
    createBlankDocument(): string;
    /**
     * 빈 문서 생성 (테스트/미리보기용)
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
     * 책갈피 삭제
     */
    deleteBookmark(sec: number, para: number, ctrl_idx: number): string;
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
     * 컨트롤 객체를 HTML 문자열로 변환한다.
     */
    exportControlHtml(section_idx: number, para_idx: number, control_idx: number): string;
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
     * Document IR을 HWPX(ZIP+XML)로 직렬화하여 반환한다.
     */
    exportHwpx(): Uint8Array;
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (본문).
     */
    exportSelectionHtml(section_idx: number, start_para_idx: number, start_char_offset: number, end_para_idx: number, end_char_offset: number): string;
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
     */
    exportSelectionInCellHtml(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, start_cell_para_idx: number, start_char_offset: number, end_cell_para_idx: number, end_char_offset: number): string;
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
     * CanvasKit direct replay/compat overlay 정책 진단을 JSON 문자열로 반환한다.
     *
     * `mode` 는 `"default"` 또는 `"compat"` 를 받는다. 빈 문자열은 `"default"` 로 처리한다.
     */
    getCanvasKitReplayPlan(page_num: number, mode: string): string;
    /**
     * 문서에 저장된 캐럿 위치를 반환한다 (문서 로딩 시 캐럿 자동 배치용).
     *
     * 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
     */
    getCaretPosition(): string;
    /**
     * 셀 내부 문단의 글자 속성을 조회한다.
     */
    getCellCharPropertiesAt(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
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
     * 셀 속성을 조회한다.
     *
     * 반환: JSON `{width, height, paddingLeft, paddingRight, paddingTop, paddingBottom, verticalAlign, textDirection, isHeader}`
     */
    getCellProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): string;
    /**
     * 셀 내부 문단의 스타일을 조회한다.
     */
    getCellStyleAt(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): string;
    /**
     * 표 셀의 텍스트 방향을 반환한다 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움).
     */
    getCellTextDirection(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): number;
    /**
     * 캐럿 위치의 글자 속성을 조회한다.
     *
     * 반환값: JSON 객체 (fontFamily, fontSize, bold, italic, underline, strikethrough, textColor 등)
     */
    getCharPropertiesAt(sec_idx: number, para_idx: number, char_offset: number): string;
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
    getControlImageData(section_idx: number, para_idx: number, control_idx: number): Uint8Array;
    /**
     * 컨트롤의 이미지 MIME 타입을 반환한다.
     */
    getControlImageMime(section_idx: number, para_idx: number, control_idx: number): string;
    /**
     * 문단 내 컨트롤의 텍스트 위치 배열을 반환한다.
     */
    getControlTextPositions(section_idx: number, para_idx: number): string;
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
     * preferred_page: 선호 페이지 (더블클릭한 페이지). -1이면 첫 번째 발견 페이지 사용.
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     */
    getCursorRectInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, preferred_page: number): string;
    /**
     * 문서 정보를 JSON 문자열로 반환한다.
     */
    getDocumentInfo(): string;
    /**
     * 현재 DPI를 반환한다.
     */
    getDpi(): number;
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
     * 현재 대체 폰트 경로를 반환한다.
     */
    getFallbackFont(): string;
    /**
     * 커서 위치의 필드 범위 정보를 조회한다 (본문 문단).
     *
     * 반환: `{inField, fieldId?, startCharIdx?, endCharIdx?, isGuide?, guideName?}`
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
     * 문서 전체의 머리말/꼬리말 목록을 반환한다.
     */
    getHeaderFooterList(current_section_idx: number, current_is_header: boolean, current_apply_to: number): string;
    /**
     * 머리말/꼬리말 문단 정보 조회
     *
     * 반환: JSON `{"ok":true,"paraCount":N,"charCount":N}`
     */
    getHeaderFooterParaInfo(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number): string;
    /**
     * [Task #825] 머리말/꼬리말 안 그림의 속성 조회.
     * path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
     *       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
     */
    getHeaderFooterPictureProperties(section_idx: number, outer_para_idx: number, outer_control_idx: number, inner_para_idx: number, inner_control_idx: number): string;
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
     * 문단의 논리적 길이를 반환한다 (텍스트 문자 + 인라인 컨트롤 수).
     */
    getLogicalLength(section_idx: number, para_idx: number): number;
    /**
     * 문서에 정의된 문단 번호(Numbering) 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, levelFormats: [...] }, ...]
     * id는 1-based (ParaShape.numbering_id와 동일)
     */
    getNumberingList(): string;
    /**
     * 컨트롤(표, 이미지 등) 레이아웃 정보를 반환한다.
     */
    getPageControlLayout(page_num: number): string;
    /**
     * 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환한다.
     */
    getPageDef(section_idx: number): string;
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
     */
    getPageLayerTree(page_num: number): string;
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
     * 특정 페이지의 텍스트 레이아웃 정보를 JSON 문자열로 반환한다.
     *
     * 각 TextRun의 위치, 텍스트, 글자별 X 좌표 경계값을 포함한다.
     */
    getPageTextLayout(page_num: number): string;
    /**
     * 캐럿 위치의 문단 속성을 조회한다.
     *
     * 반환값: JSON 객체 (alignment, lineSpacing, marginLeft, marginRight, indent 등)
     */
    getParaPropertiesAt(sec_idx: number, para_idx: number): string;
    /**
     * 머리말/꼬리말 문단의 문단 속성을 조회한다.
     */
    getParaPropertiesInHf(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number): string;
    /**
     * 구역 내 문단 수를 반환한다.
     */
    getParagraphCount(section_idx: number): number;
    /**
     * 문단의 글자 수(char 개수)를 반환한다.
     */
    getParagraphLength(section_idx: number, para_idx: number): number;
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
     * 구역(Section) 수를 반환한다.
     */
    getSectionCount(): number;
    /**
     * 구역 정의(SectionDef)를 JSON으로 반환한다.
     */
    getSectionDef(section_idx: number): string;
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
     * 투명선 표시 여부를 반환한다.
     */
    getShowTransparentBorders(): boolean;
    /**
     * 원본 파일 형식을 반환한다 ("hwp" 또는 "hwpx").
     */
    getSourceFormat(): string;
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
     * 표의 모든 셀 bbox를 반환한다 (F5 셀 선택 모드용).
     *
     * 반환: JSON `[{cellIdx, row, col, rowSpan, colSpan, pageIndex, x, y, w, h}, ...]`
     */
    getTableCellBboxes(section_idx: number, parent_para_idx: number, control_idx: number, page_hint?: number | null): string;
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
     * 표 셀 내 문단에서 텍스트 부분 문자열을 반환한다.
     */
    getTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    getTextInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, count: number): string;
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
     * 단 나누기 삽입 (Ctrl+Shift+Enter)
     */
    insertColumnBreak(section_idx: number, para_idx: number, char_offset: number): string;
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
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     */
    insertPicture(section_idx: number, para_idx: number, char_offset: number, image_data: Uint8Array, width: number, height: number, natural_width_px: number, natural_height_px: number, extension: string, description: string): string;
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
     * 표 셀 내부 문단에 텍스트를 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
     */
    insertTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, text: string): string;
    insertTextInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number, text: string): string;
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
     * 직선 끝점 이동 (글로벌 HWPUNIT 좌표)
     */
    moveLineEndpoint(sec: number, para: number, ci: number, sx: number, sy: number, ex: number, ey: number): string;
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
     * 총 페이지 수를 반환한다.
     */
    pageCount(): number;
    /**
     * 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
     */
    pasteControl(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * HTML 문자열을 파싱하여 캐럿 위치에 삽입한다 (본문).
     */
    pasteHtml(section_idx: number, para_idx: number, char_offset: number, html: string): string;
    /**
     * HTML 문자열을 파싱하여 셀 내부 캐럿 위치에 삽입한다.
     */
    pasteHtmlInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, html: string): string;
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
     * 사용자 명시 요청에 의한 lineseg 전체 reflow (#177).
     *
     * `reflow_zero_height_paragraphs` 의 자동 경로와 달리, "빈 line_segs + text 존재"
     * 케이스까지 포함해 재계산한다. 반환값은 실제로 reflow 된 문단 개수.
     *
     * 호출 이후 렌더 캐시·페이지네이션이 갱신되므로 즉시 렌더링하면 보정된 결과가 보인다.
     */
    reflowLinesegs(): number;
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (본문 문단).
     */
    removeFieldAt(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (셀/글상자 내 문단).
     */
    removeFieldAtInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, is_textbox: boolean): string;
    /**
     * 책갈피 이름 변경
     */
    renameBookmark(sec: number, para: number, ctrl_idx: number, new_name: string): string;
    /**
     * 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
     *
     * 반환: 완전한 `<svg>` 문자열
     */
    renderEquationPreview(script: string, font_size_hwpunit: number, color: number): string;
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
     * 특정 페이지를 SVG 문자열로 렌더링한다.
     */
    renderPageSvg(page_num: number): string;
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
     * - `"all"` → 모든 그림 렌더 (기본 `renderPageToCanvas` 와 동일)
     * - `"flow"` → 본문 layer (BehindText / InFrontOfText 그림 제외)
     * - `"behind"` → BehindText overlay layer
     * - `"front"` → InFrontOfText overlay layer
     *
     * 본문 Canvas 와 overlay 컨테이너를 분리하는 다층 layer 아키텍처에서 사용.
     */
    renderPageToCanvasFiltered(page_num: number, canvas: HTMLCanvasElement, scale: number, layer_kind: string): void;
    /**
     * 특정 페이지를 기존 PageRenderTree 경로로 Canvas 2D에 직접 렌더링한다.
     */
    renderPageToCanvasLegacy(page_num: number, canvas: HTMLCanvasElement, scale: number): void;
    /**
     * 전체 치환
     */
    replaceAll(query: string, new_text: string, case_sensitive: boolean): string;
    /**
     * 단일 치환 (검색어 기반) — 첫 번째 매치만 교체
     */
    replaceOne(query: string, new_text: string, case_sensitive: boolean): string;
    /**
     * 텍스트 치환 (단일)
     */
    replaceText(sec: number, para: number, char_offset: number, length: number, new_text: string): string;
    /**
     * 여러 셀의 width/height를 한 번에 조절한다 (배치).
     *
     * json: `[{"cellIdx":0,"widthDelta":150},{"cellIdx":2,"heightDelta":-100}]`
     * 반환: JSON `{"ok":true}`
     */
    resizeTableCells(section_idx: number, parent_para_idx: number, control_idx: number, json: string): string;
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
     */
    searchText(query: string, from_sec: number, from_para: number, from_char: number, forward: boolean, case_sensitive: boolean): string;
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
     * 셀 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}`
     */
    setCellProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, json: string): string;
    setClipEnabled(enabled: boolean): void;
    /**
     * 다단 설정 변경
     * column_type: 0=일반, 1=배분, 2=평행
     * same_width: 0=다른 너비, 1=같은 너비
     */
    setColumnDef(section_idx: number, column_count: number, column_type: number, same_width: number, spacing_hu: number): string;
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
     * [Task #825] 머리말/꼬리말 안 그림 속성 변경.
     */
    setHeaderFooterPictureProperties(section_idx: number, outer_para_idx: number, outer_control_idx: number, inner_para_idx: number, inner_control_idx: number, props_json: string): string;
    /**
     * 문단 서식을 적용한다 (본문 문단).
     * 문단 번호 시작 방식 설정
     */
    setNumberingRestart(section_idx: number, para_idx: number, mode: number, start_num: number): string;
    /**
     * 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션한다.
     */
    setPageDef(section_idx: number, json: string): string;
    /**
     * 감추기 설정
     */
    setPageHide(sec: number, para: number, hide_header: boolean, hide_footer: boolean, hide_master: boolean, hide_border: boolean, hide_fill: boolean, hide_page_num: boolean): string;
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
     * 디버그 오버레이 표시 여부를 설정한다.
     */
    set_debug_overlay(enabled: boolean): void;
    /**
     * LINE_SEG vpos-reset 강제 분리 적용 여부를 설정한다.
     * 변경 시 페이지네이션 결과가 달라지므로 모든 섹션을 재페이지네이션한다.
     */
    set_respect_vpos_reset(enabled: boolean): void;
    /**
     * 캐럿 위치에서 문단을 분할한다 (Enter 키).
     *
     * char_offset 이후의 텍스트가 새 문단으로 이동한다.
     * 반환값: JSON `{"ok":true,"paraIdx":<new_para_idx>,"charOffset":0}`
     */
    splitParagraph(section_idx: number, para_idx: number, char_offset: number): string;
    /**
     * 셀 내부 문단을 분할한다 (셀 내 Enter 키).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<new_idx>,"charOffset":0}`
     */
    splitParagraphInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number): string;
    splitParagraphInCellByPath(section_idx: number, parent_para_idx: number, path_json: string, char_offset: number): string;
    /**
     * 각주 내 문단을 분할한다 (Enter).
     */
    splitParagraphInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, char_offset: number): string;
    /**
     * 머리말/꼬리말 내 문단 분할 (Enter 키)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<new_idx>,"charOffset":0}`
     */
    splitParagraphInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number): string;
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
     * 범위 내 셀들을 각각 N줄 × M칸으로 분할한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     */
    splitTableCellsInRange(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number, n_rows: number, m_cols: number, equal_row_height: boolean): string;
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
    readonly __wbg_hwpdocument_free: (a: number, b: number) => void;
    readonly __wbg_hwpviewer_free: (a: number, b: number) => void;
    readonly extractThumbnail: (a: number, b: number) => any;
    readonly hwpdocument_addBookmark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_applyCellStyle: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormat: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyCharFormatInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_applyHfTemplate: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormat: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_applyParaFormatInHf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_applyStyle: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_beginBatch: (a: number) => [number, number, number, number];
    readonly hwpdocument_changeShapeZOrder: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_clearActiveField: (a: number) => void;
    readonly hwpdocument_clearClipboard: (a: number) => void;
    readonly hwpdocument_clipboardHasControl: (a: number) => number;
    readonly hwpdocument_convertToEditable: (a: number) => [number, number, number, number];
    readonly hwpdocument_copyControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_copySelection: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_copySelectionInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_createBlankDocument: (a: number) => [number, number, number, number];
    readonly hwpdocument_createEmpty: () => number;
    readonly hwpdocument_createHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_createNumbering: (a: number, b: number, c: number) => number;
    readonly hwpdocument_createShapeControl: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_createStyle: (a: number, b: number, c: number) => number;
    readonly hwpdocument_createTable: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_createTableEx: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deleteBookmark: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteEquationControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_deletePictureControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteRange: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_deleteRangeInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_deleteShapeControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteStyle: (a: number, b: number) => number;
    readonly hwpdocument_deleteTableColumn: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteTableControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_deleteTableRow: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteText: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_deleteTextInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_discardSnapshot: (a: number, b: number) => void;
    readonly hwpdocument_endBatch: (a: number) => [number, number, number, number];
    readonly hwpdocument_ensureDefaultBullet: (a: number, b: number, c: number) => number;
    readonly hwpdocument_ensureDefaultNumbering: (a: number) => number;
    readonly hwpdocument_evaluateTableFormula: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_exportControlHtml: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_exportHwp: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpVerify: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportHwpx: (a: number) => [number, number, number, number];
    readonly hwpdocument_exportSelectionHtml: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_exportSelectionInCellHtml: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_findNearestControlBackward: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_findNearestControlForward: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_findNextEditableControl: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly hwpdocument_findOrCreateFontId: (a: number, b: number, c: number) => number;
    readonly hwpdocument_findOrCreateFontIdForLang: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_getBookmarks: (a: number) => [number, number, number, number];
    readonly hwpdocument_getBulletList: (a: number) => [number, number];
    readonly hwpdocument_getCanvasKitReplayPlan: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCaretPosition: (a: number) => [number, number, number, number];
    readonly hwpdocument_getCellCharPropertiesAt: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCellInfo: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellInfoByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellParaPropertiesAt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCellParagraphCount: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphCountByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphLength: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly hwpdocument_getCellParagraphLengthByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCellProperties: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCellStyleAt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_getCellTextDirection: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly hwpdocument_getCharPropertiesAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getClickHereProps: (a: number, b: number) => [number, number];
    readonly hwpdocument_getClipboardText: (a: number) => [number, number];
    readonly hwpdocument_getColumnDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getControlImageData: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getControlImageMime: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getControlTextPositions: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getCursorRect: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInFootnote: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getCursorRectInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getDocumentInfo: (a: number) => [number, number];
    readonly hwpdocument_getDpi: (a: number) => number;
    readonly hwpdocument_getEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getEventLog: (a: number) => [number, number];
    readonly hwpdocument_getExternalImageBasenames: (a: number) => [number, number];
    readonly hwpdocument_getFallbackFont: (a: number) => [number, number];
    readonly hwpdocument_getFieldInfoAt: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_getFieldInfoAtByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_getFieldInfoAtInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly hwpdocument_getFieldList: (a: number) => [number, number];
    readonly hwpdocument_getFieldValue: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getFieldValueByName: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getFootnoteAtCursor: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getFootnoteInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormObjectAt: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormObjectInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getFormValue: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterList: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterParaInfo: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getHeaderFooterPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getLineInfo: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getLineInfoInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getLogicalLength: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_getNumberingList: (a: number) => [number, number];
    readonly hwpdocument_getPageControlLayout: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageFootnoteInfo: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageHide: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageInfo: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageLayerTree: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageOfPosition: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getPageOverlayImages: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageRenderTree: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getPageTextLayout: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getParaPropertiesAt: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_getParaPropertiesInHf: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getParagraphCount: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_getParagraphLength: (a: number, b: number, c: number) => [number, number, number];
    readonly hwpdocument_getPictureProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getPositionOfPage: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getSectionCount: (a: number) => number;
    readonly hwpdocument_getSectionDef: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRects: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_getSelectionRectsInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_getShapeProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getShowControlCodes: (a: number) => number;
    readonly hwpdocument_getShowTransparentBorders: (a: number) => number;
    readonly hwpdocument_getSourceFormat: (a: number) => [number, number];
    readonly hwpdocument_getStyleAt: (a: number, b: number, c: number) => [number, number];
    readonly hwpdocument_getStyleDetail: (a: number, b: number) => [number, number];
    readonly hwpdocument_getStyleList: (a: number) => [number, number];
    readonly hwpdocument_getTableBBox: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTableCellBboxes: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableCellBboxesByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableDimensions: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTableDimensionsByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getTableProperties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_getTextBoxControlIndex: (a: number, b: number, c: number) => number;
    readonly hwpdocument_getTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_getTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_getTextRange: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_getValidationWarnings: (a: number) => [number, number];
    readonly hwpdocument_groupShapes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_hasInternalClipboard: (a: number) => number;
    readonly hwpdocument_hitTest: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestBodyFootnoteMarker: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestHeaderFooter: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestInFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_hitTestInHeaderFooter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_injectExternalImage: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly hwpdocument_insertColumnBreak: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertEquation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertFieldInHf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_insertFootnote: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertNewNumber: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_insertPageBreak: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_insertParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_insertPicture: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => [number, number, number, number];
    readonly hwpdocument_insertTableColumn: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertTableRow: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertText: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_insertTextLogical: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_logicalToTextOffset: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_measureWidthDiagnostic: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraph: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInCell: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInCellByPath: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInFootnote: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeParagraphInHeaderFooter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_mergeTableCells: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_moveLineEndpoint: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_moveTableOffset: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_moveVertical: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_moveVerticalByPath: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_navigateHeaderFooterByPage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_navigateNextEditable: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly hwpdocument_new: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_pageCount: (a: number) => number;
    readonly hwpdocument_pasteControl: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtml: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_pasteHtmlInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_pasteInternal: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_pasteInternalInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_reflowLinesegs: (a: number) => number;
    readonly hwpdocument_removeFieldAt: (a: number, b: number, c: number, d: number) => [number, number];
    readonly hwpdocument_removeFieldAtInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly hwpdocument_renameBookmark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_renderEquationPreview: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_renderPageCanvas: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_renderPageCanvasLegacy: (a: number, b: number) => [number, number, number];
    readonly hwpdocument_renderPageHtml: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_renderPageSvg: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_renderPageToCanvas: (a: number, b: number, c: any, d: number) => [number, number];
    readonly hwpdocument_renderPageToCanvasFiltered: (a: number, b: number, c: any, d: number, e: number, f: number) => [number, number];
    readonly hwpdocument_renderPageToCanvasLegacy: (a: number, b: number, c: any, d: number) => [number, number];
    readonly hwpdocument_replaceAll: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_replaceOne: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_replaceText: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_resizeTableCells: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_restoreSnapshot: (a: number, b: number) => [number, number, number, number];
    readonly hwpdocument_saveSnapshot: (a: number) => number;
    readonly hwpdocument_searchAllText: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_searchText: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setActiveField: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_setActiveFieldByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly hwpdocument_setActiveFieldInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly hwpdocument_setCellProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_setClipEnabled: (a: number, b: number) => void;
    readonly hwpdocument_setColumnDef: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setDpi: (a: number, b: number) => void;
    readonly hwpdocument_setEquationProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setFallbackFont: (a: number, b: number, c: number) => void;
    readonly hwpdocument_setFieldValue: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setFieldValueByName: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setFileName: (a: number, b: number, c: number) => void;
    readonly hwpdocument_setFormValue: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setFormValueInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setHeaderFooterPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly hwpdocument_setNumberingRestart: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly hwpdocument_setPageDef: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setPageHide: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly hwpdocument_setPictureProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setSectionDef: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_setSectionDefAll: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_setShapeProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_setShowControlCodes: (a: number, b: number) => void;
    readonly hwpdocument_setShowParagraphMarks: (a: number, b: number) => void;
    readonly hwpdocument_setShowTransparentBorders: (a: number, b: number) => void;
    readonly hwpdocument_setTableProperties: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_set_debug_overlay: (a: number, b: number) => void;
    readonly hwpdocument_set_respect_vpos_reset: (a: number, b: number) => void;
    readonly hwpdocument_splitParagraph: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInCell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInCellByPath: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInFootnote: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitParagraphInHeaderFooter: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCell: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellInto: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly hwpdocument_splitTableCellsInRange: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number, number];
    readonly hwpdocument_textToLogicalOffset: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly hwpdocument_toggleHideHeaderFooter: (a: number, b: number, c: number) => [number, number, number, number];
    readonly hwpdocument_ungroupShape: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly hwpdocument_updateClickHereProps: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly hwpdocument_updateConnectorsInSection: (a: number, b: number) => void;
    readonly hwpdocument_updateStyle: (a: number, b: number, c: number, d: number) => number;
    readonly hwpdocument_updateStyleShapes: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly hwpviewer_new: (a: number) => number;
    readonly hwpviewer_pendingTaskCount: (a: number) => number;
    readonly hwpviewer_renderPageHtml: (a: number, b: number) => [number, number, number, number];
    readonly hwpviewer_renderPageSvg: (a: number, b: number) => [number, number, number, number];
    readonly hwpviewer_setZoom: (a: number, b: number) => void;
    readonly hwpviewer_updateViewport: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly hwpviewer_visiblePages: (a: number) => [number, number];
    readonly version: () => [number, number];
    readonly init_panic_hook: () => void;
    readonly hwpviewer_pageCount: (a: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
