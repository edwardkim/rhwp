/* @ts-self-types="./rhwp.d.ts" */

/**
 * WASM에서 사용할 HWP 문서 래퍼
 *
 * 도메인 로직은 `DocumentCore`에 구현되어 있으며,
 * `Deref`/`DerefMut`를 통해 투명하게 접근한다.
 */
export class HwpDocument {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(HwpDocument.prototype);
        obj.__wbg_ptr = ptr;
        HwpDocumentFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        HwpDocumentFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_hwpdocument_free(ptr, 0);
    }
    /**
     * 책갈피 추가
     * @param {number} sec
     * @param {number} para
     * @param {number} char_offset
     * @param {string} name
     * @returns {string}
     */
    addBookmark(sec, para, char_offset, name) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_addBookmark(this.__wbg_ptr, sec, para, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 스타일을 적용한다 (셀 내 문단).
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} style_id
     * @returns {string}
     */
    applyCellStyle(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, style_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_applyCellStyle(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, style_id);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 글자 서식을 적용한다 (본문 문단).
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {string} props_json
     * @returns {string}
     */
    applyCharFormat(sec_idx, para_idx, start_offset, end_offset, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormat(this.__wbg_ptr, sec_idx, para_idx, start_offset, end_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 글자 서식을 적용한다 (셀 내 문단).
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {string} props_json
     * @returns {string}
     */
    applyCharFormatInCell(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, start_offset, end_offset, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormatInCell(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, start_offset, end_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 마당(템플릿)을 적용한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} template_id
     * @returns {string}
     */
    applyHfTemplate(section_idx, is_header, apply_to, template_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_applyHfTemplate(this.__wbg_ptr, section_idx, is_header, apply_to, template_id);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {string} props_json
     * @returns {string}
     */
    applyParaFormat(sec_idx, para_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyParaFormat(this.__wbg_ptr, sec_idx, para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 문단 서식을 적용한다 (셀 내 문단).
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {string} props_json
     * @returns {string}
     */
    applyParaFormatInCell(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyParaFormatInCell(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 문단에 문단 서식을 적용한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {string} props_json
     * @returns {string}
     */
    applyParaFormatInHf(section_idx, is_header, apply_to, hf_para_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyParaFormatInHf(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 스타일을 적용한다 (본문 문단).
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {number} style_id
     * @returns {string}
     */
    applyStyle(sec_idx, para_idx, style_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_applyStyle(this.__wbg_ptr, sec_idx, para_idx, style_id);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Batch 모드를 시작한다. 이후 Command 호출 시 paginate()를 건너뛴다.
     * @returns {string}
     */
    beginBatch() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_beginBatch(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Shape z-order 변경
     * operation: "front" | "back" | "forward" | "backward"
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} operation
     * @returns {string}
     */
    changeShapeZOrder(section_idx, parent_para_idx, control_idx, operation) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(operation, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_changeShapeZOrder(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 활성 필드를 해제한다 (안내문 다시 표시).
     */
    clearActiveField() {
        wasm.hwpdocument_clearActiveField(this.__wbg_ptr);
    }
    /**
     * 내부 클립보드를 초기화한다.
     */
    clearClipboard() {
        wasm.hwpdocument_clearClipboard(this.__wbg_ptr);
    }
    /**
     * 내부 클립보드에 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
     * @returns {boolean}
     */
    clipboardHasControl() {
        const ret = wasm.hwpdocument_clipboardHasControl(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다.
     *
     * 반환값: JSON `{"ok":true,"converted":true}` 또는 `{"ok":true,"converted":false}`
     * @returns {string}
     */
    convertToEditable() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_convertToEditable(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    copyControl(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_copyControl(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 선택 영역을 내부 클립보드에 복사한다.
     *
     * 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
     * @param {number} section_idx
     * @param {number} start_para_idx
     * @param {number} start_char_offset
     * @param {number} end_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    copySelection(section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_copySelection(this.__wbg_ptr, section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 셀 내부 선택 영역을 내부 클립보드에 복사한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    copySelectionInCell(section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_copySelectionInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 내장 템플릿에서 빈 문서를 생성한다.
     *
     * saved/blank2010.hwp를 WASM 바이너리에 포함하여 유효한 HWP 문서를 즉시 생성.
     * DocInfo raw_stream이 온전하므로 FIX-4 워크어라운드와 호환됨.
     * @returns {string}
     */
    createBlankDocument() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_createBlankDocument(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 빈 문서 생성 (테스트/미리보기용)
     * @returns {HwpDocument}
     */
    static createEmpty() {
        const ret = wasm.hwpdocument_createEmpty();
        return HwpDocument.__wrap(ret);
    }
    /**
     * 머리말/꼬리말 생성 (빈 문단 1개 포함)
     *
     * 반환: JSON `{"ok":true,"kind":"header/footer","applyTo":N,...}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @returns {string}
     */
    createHeaderFooter(section_idx, is_header, apply_to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_createHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * JSON으로 지정된 번호 형식으로 Numbering 정의를 생성한다.
     *
     * json: {"levelFormats":["^1.","^2)",...],"numberFormats":[0,8,...],"startNumber":1}
     * 반환값: Numbering ID (1-based)
     * @param {string} json
     * @returns {number}
     */
    createNumbering(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_createNumbering(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * 커서 위치에 글상자(Rectangle + TextBox)를 삽입한다.
     *
     * json: `{"sectionIdx":N,"paraIdx":N,"charOffset":N,"width":N,"height":N,
     *         "horzOffset":N,"vertOffset":N,"treatAsChar":bool,"textWrap":"Square"}`
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     * @param {string} json
     * @returns {string}
     */
    createShapeControl(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_createShapeControl(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 새 스타일을 생성한다.
     *
     * json: {"name":"...", "englishName":"...", "type":0, "nextStyleId":0}
     * 반환값: 새 스타일 ID (0-based)
     * @param {string} json
     * @returns {number}
     */
    createStyle(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_createStyle(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * 커서 위치에 새 표를 삽입한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} row_count
     * @param {number} col_count
     * @returns {string}
     */
    createTable(section_idx, para_idx, char_offset, row_count, col_count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_createTable(this.__wbg_ptr, section_idx, para_idx, char_offset, row_count, col_count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 커서 위치에 표를 삽입한다 (확장, JSON 옵션).
     *
     * options JSON: { sectionIdx, paraIdx, charOffset, rowCount, colCount,
     *                 treatAsChar?: bool, colWidths?: [u32, ...] }
     * @param {string} options_json
     * @returns {string}
     */
    createTableEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_createTableEx(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 책갈피 삭제
     * @param {number} sec
     * @param {number} para
     * @param {number} ctrl_idx
     * @returns {string}
     */
    deleteBookmark(sec, para, ctrl_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteBookmark(this.__wbg_ptr, sec, para, ctrl_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 본문 각주 컨트롤을 삭제한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    deleteFootnote(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말을 삭제한다 (컨트롤 자체 제거).
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @returns {string}
     */
    deleteHeaderFooter(section_idx, is_header, apply_to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    deleteParagraph(section_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteParagraph(this.__wbg_ptr, section_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 그림 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    deletePictureControl(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deletePictureControl(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 본문 선택 영역을 삭제한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
     * @param {number} section_idx
     * @param {number} start_para_idx
     * @param {number} start_char_offset
     * @param {number} end_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    deleteRange(section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteRange(this.__wbg_ptr, section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내 선택 영역을 삭제한다.
     *
     * 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    deleteRangeInCell(section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteRangeInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Shape(글상자) 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    deleteShapeControl(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteShapeControl(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 스타일을 삭제한다.
     *
     * 바탕글(ID 0)은 삭제할 수 없다.
     * 삭제된 스타일을 사용 중인 문단은 바탕글(ID 0)로 변경된다.
     * @param {number} style_id
     * @returns {boolean}
     */
    deleteStyle(style_id) {
        const ret = wasm.hwpdocument_deleteStyle(this.__wbg_ptr, style_id);
        return ret !== 0;
    }
    /**
     * 표에서 열을 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} col_idx
     * @returns {string}
     */
    deleteTableColumn(section_idx, parent_para_idx, control_idx, col_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTableColumn(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, col_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    deleteTableControl(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTableControl(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표에서 행을 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} row_idx
     * @returns {string}
     */
    deleteTableRow(section_idx, parent_para_idx, control_idx, row_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTableRow(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, row_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단에서 텍스트를 삭제한다.
     *
     * 삭제 후 구역을 재구성하고 재페이지네이션한다.
     * 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteText(section_idx, para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteText(this.__wbg_ptr, section_idx, para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 셀 내부 문단에서 텍스트를 삭제한다.
     *
     * 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteTextInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTextInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteTextInCellByPath(section_idx, parent_para_idx, path_json, char_offset, count) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteTextInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, count);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 각주 내 텍스트를 삭제한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteTextInFootnote(section_idx, para_idx, control_idx, fn_para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTextInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내 텍스트 삭제
     *
     * 반환: JSON `{"ok":true,"charOffset":<offset>}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteTextInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTextInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 지정 ID의 스냅샷을 제거하여 메모리를 해제한다.
     * @param {number} id
     */
    discardSnapshot(id) {
        wasm.hwpdocument_discardSnapshot(this.__wbg_ptr, id);
    }
    /**
     * Batch 모드를 종료하고 누적된 이벤트를 반환한다.
     * @returns {string}
     */
    endBatch() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_endBatch(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 특정 문자의 글머리표 정의가 없으면 생성한다.
     *
     * 반환값: Bullet ID (1-based)
     * @param {string} bullet_char_str
     * @returns {number}
     */
    ensureDefaultBullet(bullet_char_str) {
        const ptr0 = passStringToWasm0(bullet_char_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_ensureDefaultBullet(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * 문서에 기본 문단 번호 정의가 없으면 생성한다.
     *
     * 반환값: Numbering ID (1-based)
     * @returns {number}
     */
    ensureDefaultNumbering() {
        const ret = wasm.hwpdocument_ensureDefaultNumbering(this.__wbg_ptr);
        return ret;
    }
    /**
     * 표 셀에서 계산식을 실행한다.
     *
     * formula: "=SUM(A1:A5)", "=A1+B2*3" 등
     * write_result: true이면 결과를 셀에 기록
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} target_row
     * @param {number} target_col
     * @param {string} formula
     * @param {boolean} write_result
     * @returns {string}
     */
    evaluateTableFormula(section_idx, parent_para_idx, control_idx, target_row, target_col, formula, write_result) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(formula, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_evaluateTableFormula(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, target_row, target_col, ptr0, len0, write_result);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 컨트롤 객체를 HTML 문자열로 변환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    exportControlHtml(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_exportControlHtml(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문서를 HWP 바이너리로 내보낸다.
     *
     * Document IR을 HWP 5.0 CFB 바이너리로 직렬화하여 반환한다.
     * HWPX 출처 문서는 `export_hwp_with_adapter` 를 통해 HWPX→HWP IR 매핑 어댑터를
     * 자동 적용하여 한컴 호환성과 자기 재로드 페이지 보존을 보장한다 (#178).
     * HWP 출처는 어댑터가 no-op 이므로 기존 동작과 동일.
     * @returns {Uint8Array}
     */
    exportHwp() {
        const ret = wasm.hwpdocument_exportHwp(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
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
     * @returns {string}
     */
    exportHwpVerify() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_exportHwpVerify(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Document IR을 HWPX(ZIP+XML)로 직렬화하여 반환한다.
     * @returns {Uint8Array}
     */
    exportHwpx() {
        const ret = wasm.hwpdocument_exportHwpx(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (본문).
     * @param {number} section_idx
     * @param {number} start_para_idx
     * @param {number} start_char_offset
     * @param {number} end_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    exportSelectionHtml(section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_exportSelectionHtml(this.__wbg_ptr, section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    exportSelectionInCellHtml(section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_exportSelectionInCellHtml(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다 (F11 키).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    findNearestControlBackward(section_idx, para_idx, char_offset) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_findNearestControlBackward(this.__wbg_ptr, section_idx, para_idx, char_offset);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 현재 위치 이후의 가장 가까운 선택 가능 컨트롤을 찾는다 (Shift+F11).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    findNearestControlForward(section_idx, para_idx, char_offset) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_findNearestControlForward(this.__wbg_ptr, section_idx, para_idx, char_offset);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다.
     * delta=+1(앞), delta=-1(뒤). ctrl_idx=-1이면 본문 텍스트에서 출발.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} ctrl_idx
     * @param {number} delta
     * @returns {string}
     */
    findNextEditableControl(section_idx, para_idx, ctrl_idx, delta) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_findNextEditableControl(this.__wbg_ptr, section_idx, para_idx, ctrl_idx, delta);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 글꼴 이름으로 font_id를 조회하거나 새로 생성한다.
     *
     * 한글(0번) 카테고리에서 이름 검색 → 없으면 7개 전체 카테고리에 신규 등록.
     * 반환값: font_id (u16), 실패 시 -1
     * @param {string} name
     * @returns {number}
     */
    findOrCreateFontId(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_findOrCreateFontId(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * 특정 언어 카테고리에서 글꼴 이름으로 ID를 찾거나 등록한다.
     * @param {number} lang
     * @param {string} name
     * @returns {number}
     */
    findOrCreateFontIdForLang(lang, name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_findOrCreateFontIdForLang(this.__wbg_ptr, lang, ptr0, len0);
        return ret;
    }
    /**
     * 문서 내 모든 책갈피 목록 반환
     * @returns {string}
     */
    getBookmarks() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getBookmarks(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문서에 정의된 글머리표(Bullet) 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, char }, ...]
     * id는 1-based (ParaShape.numbering_id와 동일)
     * @returns {string}
     */
    getBulletList() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getBulletList(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서에 저장된 캐럿 위치를 반환한다 (문서 로딩 시 캐럿 자동 배치용).
     *
     * 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
     * @returns {string}
     */
    getCaretPosition() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCaretPosition(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내부 문단의 글자 속성을 조회한다.
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCellCharPropertiesAt(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCellCharPropertiesAt(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 셀의 행/열/병합 정보를 반환한다.
     *
     * 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @returns {string}
     */
    getCellInfo(section_idx, parent_para_idx, control_idx, cell_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCellInfo(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 경로 기반 셀 정보 조회 (중첩 표용).
     *
     * 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {string}
     */
    getCellInfoByPath(section_idx, parent_para_idx, path_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCellInfoByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 셀 내부 문단의 문단 속성을 조회한다.
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    getCellParaPropertiesAt(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCellParaPropertiesAt(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 셀 내 문단 수를 반환한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @returns {number}
     */
    getCellParagraphCount(section_idx, parent_para_idx, control_idx, cell_idx) {
        const ret = wasm.hwpdocument_getCellParagraphCount(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 경로 기반: 셀/글상자 내 문단 수를 반환한다 (중첩 표/글상자 지원).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {number}
     */
    getCellParagraphCountByPath(section_idx, parent_para_idx, path_json) {
        const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_getCellParagraphCountByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 표 셀 내 문단의 글자 수를 반환한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {number}
     */
    getCellParagraphLength(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        const ret = wasm.hwpdocument_getCellParagraphLength(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 경로 기반: 셀 내 문단의 글자 수를 반환한다 (중첩 표 지원).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {number}
     */
    getCellParagraphLengthByPath(section_idx, parent_para_idx, path_json) {
        const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_getCellParagraphLengthByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 셀 속성을 조회한다.
     *
     * 반환: JSON `{width, height, paddingLeft, paddingRight, paddingTop, paddingBottom, verticalAlign, textDirection, isHeader}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @returns {string}
     */
    getCellProperties(section_idx, parent_para_idx, control_idx, cell_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCellProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내부 문단의 스타일을 조회한다.
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    getCellStyleAt(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getCellStyleAt(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 표 셀의 텍스트 방향을 반환한다 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @returns {number}
     */
    getCellTextDirection(section_idx, parent_para_idx, control_idx, cell_idx) {
        const ret = wasm.hwpdocument_getCellTextDirection(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 캐럿 위치의 글자 속성을 조회한다.
     *
     * 반환값: JSON 객체 (fontFamily, fontSize, bold, italic, underline, strikethrough, textColor 등)
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCharPropertiesAt(sec_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCharPropertiesAt(this.__wbg_ptr, sec_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 누름틀 필드의 속성을 조회한다.
     *
     * 반환: JSON `{"ok":true,"guide":"안내문","memo":"메모","name":"이름","editable":true}`
     * @param {number} field_id
     * @returns {string}
     */
    getClickHereProps(field_id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getClickHereProps(this.__wbg_ptr, field_id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 내부 클립보드의 플레인 텍스트를 반환한다.
     * @returns {string}
     */
    getClipboardText() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getClipboardText(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 컨트롤의 이미지 바이너리 데이터를 반환한다 (Uint8Array).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {Uint8Array}
     */
    getControlImageData(section_idx, para_idx, control_idx) {
        const ret = wasm.hwpdocument_getControlImageData(this.__wbg_ptr, section_idx, para_idx, control_idx);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * 컨트롤의 이미지 MIME 타입을 반환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getControlImageMime(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getControlImageMime(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단 내 컨트롤의 텍스트 위치 배열을 반환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    getControlTextPositions(section_idx, para_idx) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getControlTextPositions(this.__wbg_ptr, section_idx, para_idx);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 커서 위치의 픽셀 좌표를 반환한다.
     *
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCursorRect(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRect(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 경로 기반 커서 좌표 조회 (중첩 표용).
     *
     * path_json: `[{"controlIndex":N,"cellIndex":N,"cellParaIndex":N}, ...]`
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {string}
     */
    getCursorRectByPath(section_idx, parent_para_idx, path_json, char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCursorRectByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 표 셀 내부 커서 위치의 픽셀 좌표를 반환한다.
     *
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCursorRectInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 각주 내 커서 렉트 계산
     * @param {number} page_num
     * @param {number} footnote_index
     * @param {number} fn_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCursorRectInFootnote(page_num, footnote_index, fn_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectInFootnote(this.__wbg_ptr, page_num, footnote_index, fn_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내 커서 위치의 픽셀 좌표를 반환한다.
     *
     * preferred_page: 선호 페이지 (더블클릭한 페이지). -1이면 첫 번째 발견 페이지 사용.
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {number} preferred_page
     * @returns {string}
     */
    getCursorRectInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset, preferred_page) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, preferred_page);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문서 정보를 JSON 문자열로 반환한다.
     * @returns {string}
     */
    getDocumentInfo() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getDocumentInfo(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 현재 DPI를 반환한다.
     * @returns {number}
     */
    getDpi() {
        const ret = wasm.hwpdocument_getDpi(this.__wbg_ptr);
        return ret;
    }
    /**
     * 수식 컨트롤의 속성을 조회한다.
     *
     * 반환: JSON `{ script, fontSize, color, baseline, fontName }`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    getEquationProperties(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getEquationProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 현재 이벤트 로그를 JSON으로 반환한다.
     * @returns {string}
     */
    getEventLog() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getEventLog(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 현재 대체 폰트 경로를 반환한다.
     * @returns {string}
     */
    getFallbackFont() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getFallbackFont(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 커서 위치의 필드 범위 정보를 조회한다 (본문 문단).
     *
     * 반환: `{inField, fieldId?, startCharIdx?, endCharIdx?, isGuide?, guideName?}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getFieldInfoAt(section_idx, para_idx, char_offset) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getFieldInfoAt(this.__wbg_ptr, section_idx, para_idx, char_offset);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * path 기반: 중첩 표 셀의 필드 범위 정보를 조회한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {string}
     */
    getFieldInfoAtByPath(section_idx, parent_para_idx, path_json, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getFieldInfoAtByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 커서 위치의 필드 범위 정보를 조회한다 (셀/글상자 내 문단).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {boolean} is_textbox
     * @returns {string}
     */
    getFieldInfoAtInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getFieldInfoAtInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서 내 모든 필드 목록을 JSON 배열로 반환한다.
     *
     * 반환: `[{fieldId, fieldType, name, guide, command, value, location}]`
     * @returns {string}
     */
    getFieldList() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getFieldList(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * field_id로 필드 값을 조회한다.
     *
     * 반환: `{ok, value}`
     * @param {number} field_id
     * @returns {string}
     */
    getFieldValue(field_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getFieldValue(this.__wbg_ptr, field_id);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 필드 이름으로 값을 조회한다.
     *
     * 반환: `{ok, fieldId, value}`
     * @param {string} name
     * @returns {string}
     */
    getFieldValueByName(name) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getFieldValueByName(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 본문 커서 위치의 각주 마커를 조회한다.
     *
     * direction: "backward" 또는 "forward"
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} direction
     * @returns {string}
     */
    getFootnoteAtCursor(section_idx, para_idx, char_offset, direction) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(direction, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getFootnoteAtCursor(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 각주 정보를 조회한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getFootnoteInfo(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getFootnoteInfo(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지 좌표에서 양식 개체를 찾는다.
     *
     * 반환: `{found, sec, para, ci, formType, name, value, caption, text, bbox}`
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    getFormObjectAt(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getFormObjectAt(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 양식 개체 상세 정보를 반환한다 (properties 포함).
     *
     * 반환: `{ok, formType, name, value, text, caption, enabled, width, height, foreColor, backColor, properties}`
     * @param {number} sec
     * @param {number} para
     * @param {number} ci
     * @returns {string}
     */
    getFormObjectInfo(sec, para, ci) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getFormObjectInfo(this.__wbg_ptr, sec, para, ci);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 양식 개체 값을 조회한다.
     *
     * 반환: `{ok, formType, name, value, text, caption, enabled}`
     * @param {number} sec
     * @param {number} para
     * @param {number} ci
     * @returns {string}
     */
    getFormValue(sec, para, ci) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getFormValue(this.__wbg_ptr, sec, para, ci);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 조회
     *
     * 반환: JSON `{"ok":true,"exists":true/false,...}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @returns {string}
     */
    getHeaderFooter(section_idx, is_header, apply_to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문서 전체의 머리말/꼬리말 목록을 반환한다.
     * @param {number} current_section_idx
     * @param {boolean} current_is_header
     * @param {number} current_apply_to
     * @returns {string}
     */
    getHeaderFooterList(current_section_idx, current_is_header, current_apply_to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooterList(this.__wbg_ptr, current_section_idx, current_is_header, current_apply_to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 문단 정보 조회
     *
     * 반환: JSON `{"ok":true,"paraCount":N,"charCount":N}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @returns {string}
     */
    getHeaderFooterParaInfo(section_idx, is_header, apply_to, hf_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooterParaInfo(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단 내 줄 정보를 반환한다 (커서 수직 이동/Home/End용).
     *
     * 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getLineInfo(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getLineInfo(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표 셀 내 문단의 줄 정보를 반환한다.
     *
     * 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getLineInfoInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getLineInfoInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단의 논리적 길이를 반환한다 (텍스트 문자 + 인라인 컨트롤 수).
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {number}
     */
    getLogicalLength(section_idx, para_idx) {
        const ret = wasm.hwpdocument_getLogicalLength(this.__wbg_ptr, section_idx, para_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 문서에 정의된 문단 번호(Numbering) 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, levelFormats: [...] }, ...]
     * id는 1-based (ParaShape.numbering_id와 동일)
     * @returns {string}
     */
    getNumberingList() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getNumberingList(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 컨트롤(표, 이미지 등) 레이아웃 정보를 반환한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageControlLayout(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageControlLayout(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getPageDef(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageDef(this.__wbg_ptr, section_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지의 각주 참조 정보
     * @param {number} page_num
     * @param {number} footnote_index
     * @returns {string}
     */
    getPageFootnoteInfo(page_num, footnote_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageFootnoteInfo(this.__wbg_ptr, page_num, footnote_index);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 감추기 조회
     * @param {number} sec
     * @param {number} para
     * @returns {string}
     */
    getPageHide(sec, para) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageHide(this.__wbg_ptr, sec, para);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지 정보를 JSON 문자열로 반환한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageInfo(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageInfo(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지 레이어 트리를 JSON 문자열로 반환한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageLayerTree(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageLayerTree(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 위치에 해당하는 글로벌 쪽 번호 반환
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    getPageOfPosition(section_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageOfPosition(this.__wbg_ptr, section_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지 렌더 트리를 JSON 문자열로 반환한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageRenderTree(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageRenderTree(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 특정 페이지의 텍스트 레이아웃 정보를 JSON 문자열로 반환한다.
     *
     * 각 TextRun의 위치, 텍스트, 글자별 X 좌표 경계값을 포함한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageTextLayout(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageTextLayout(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 캐럿 위치의 문단 속성을 조회한다.
     *
     * 반환값: JSON 객체 (alignment, lineSpacing, marginLeft, marginRight, indent 등)
     * @param {number} sec_idx
     * @param {number} para_idx
     * @returns {string}
     */
    getParaPropertiesAt(sec_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getParaPropertiesAt(this.__wbg_ptr, sec_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 문단의 문단 속성을 조회한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @returns {string}
     */
    getParaPropertiesInHf(section_idx, is_header, apply_to, hf_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getParaPropertiesInHf(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 구역 내 문단 수를 반환한다.
     * @param {number} section_idx
     * @returns {number}
     */
    getParagraphCount(section_idx) {
        const ret = wasm.hwpdocument_getParagraphCount(this.__wbg_ptr, section_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 문단의 글자 수(char 개수)를 반환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {number}
     */
    getParagraphLength(section_idx, para_idx) {
        const ret = wasm.hwpdocument_getParagraphLength(this.__wbg_ptr, section_idx, para_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 그림 컨트롤의 속성을 조회한다.
     *
     * 반환: JSON `{ width, height, treatAsChar, ... }`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getPictureProperties(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPictureProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 글로벌 쪽 번호에 해당하는 첫 문단 위치 반환
     * @param {number} global_page
     * @returns {string}
     */
    getPositionOfPage(global_page) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPositionOfPage(this.__wbg_ptr, global_page);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 구역(Section) 수를 반환한다.
     * @returns {number}
     */
    getSectionCount() {
        const ret = wasm.hwpdocument_getSectionCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 구역 정의(SectionDef)를 JSON으로 반환한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getSectionDef(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getSectionDef(this.__wbg_ptr, section_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 본문 선택 영역의 줄별 사각형을 반환한다.
     *
     * 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
     * @param {number} section_idx
     * @param {number} start_para_idx
     * @param {number} start_char_offset
     * @param {number} end_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    getSelectionRects(section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getSelectionRects(this.__wbg_ptr, section_idx, start_para_idx, start_char_offset, end_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내 선택 영역의 줄별 사각형을 반환한다.
     *
     * 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    getSelectionRectsInCell(section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getSelectionRectsInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Shape(글상자) 속성을 조회한다.
     *
     * 반환: JSON `{ width, height, treatAsChar, tbMarginLeft, ... }`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getShapeProperties(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getShapeProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 조판부호 표시 여부를 반환한다.
     * @returns {boolean}
     */
    getShowControlCodes() {
        const ret = wasm.hwpdocument_getShowControlCodes(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 투명선 표시 여부를 반환한다.
     * @returns {boolean}
     */
    getShowTransparentBorders() {
        const ret = wasm.hwpdocument_getShowTransparentBorders(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 원본 파일 형식을 반환한다 ("hwp" 또는 "hwpx").
     * @returns {string}
     */
    getSourceFormat() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getSourceFormat(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 특정 문단의 스타일을 조회한다.
     *
     * 반환값: JSON { id, name }
     * @param {number} sec_idx
     * @param {number} para_idx
     * @returns {string}
     */
    getStyleAt(sec_idx, para_idx) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getStyleAt(this.__wbg_ptr, sec_idx, para_idx);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 특정 스타일의 CharShape/ParaShape 속성을 상세 조회한다.
     *
     * 반환값: JSON { charProps: {...}, paraProps: {...} }
     * @param {number} style_id
     * @returns {string}
     */
    getStyleDetail(style_id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getStyleDetail(this.__wbg_ptr, style_id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서에 정의된 스타일 목록을 조회한다.
     *
     * 반환값: JSON 배열 [{ id, name, englishName, type, paraShapeId, charShapeId }, ...]
     * @returns {string}
     */
    getStyleList() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getStyleList(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 표 전체의 바운딩박스를 반환한다.
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getTableBBox(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTableBBox(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표의 모든 셀 bbox를 반환한다 (F5 셀 선택 모드용).
     *
     * 반환: JSON `[{cellIdx, row, col, rowSpan, colSpan, pageIndex, x, y, w, h}, ...]`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number | null} [page_hint]
     * @returns {string}
     */
    getTableCellBboxes(section_idx, parent_para_idx, control_idx, page_hint) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTableCellBboxes(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, isLikeNone(page_hint) ? 0x100000001 : (page_hint) >>> 0);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 경로 기반 표 셀 바운딩박스 조회 (중첩 표용).
     *
     * 반환: JSON 배열 `[{"cellIdx":N,"row":N,"col":N,...,"x":F,"y":F,"w":F,"h":F}, ...]`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {string}
     */
    getTableCellBboxesByPath(section_idx, parent_para_idx, path_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getTableCellBboxesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 표의 행/열/셀 수를 반환한다.
     *
     * 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getTableDimensions(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTableDimensions(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 경로 기반 표 차원 조회 (중첩 표용).
     *
     * 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {string}
     */
    getTableDimensionsByPath(section_idx, parent_para_idx, path_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getTableDimensionsByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 표 속성을 조회한다.
     *
     * 반환: JSON `{cellSpacing, paddingLeft, paddingRight, paddingTop, paddingBottom, pageBreak, repeatHeader}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getTableProperties(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTableProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단에 텍스트박스가 있는 Shape 컨트롤이 있으면 해당 control_index를 반환한다.
     * 없으면 -1을 반환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {number}
     */
    getTextBoxControlIndex(section_idx, para_idx) {
        const ret = wasm.hwpdocument_getTextBoxControlIndex(this.__wbg_ptr, section_idx, para_idx);
        return ret;
    }
    /**
     * 표 셀 내 문단에서 텍스트 부분 문자열을 반환한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    getTextInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTextInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    getTextInCellByPath(section_idx, parent_para_idx, path_json, char_offset, count) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getTextInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, count);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 문단에서 텍스트 부분 문자열을 반환한다 (Undo용 텍스트 보존).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    getTextRange(section_idx, para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTextRange(this.__wbg_ptr, section_idx, para_idx, char_offset, count);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
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
     * @returns {string}
     */
    getValidationWarnings() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getValidationWarnings(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 선택된 개체들을 하나의 GroupShape로 묶는다.
     * json: `{"sectionIdx":N, "targets":[{"paraIdx":N,"controlIdx":N},...]}`
     * 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
     * @param {string} json
     * @returns {string}
     */
    groupShapes(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_groupShapes(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 내부 클립보드에 데이터가 있는지 확인한다.
     * @returns {boolean}
     */
    hasInternalClipboard() {
        const ret = wasm.hwpdocument_hasInternalClipboard(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 페이지 좌표에서 문서 위치를 찾는다.
     *
     * 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTest(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTest(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 본문 인라인 각주 마커 히트테스트
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestBodyFootnoteMarker(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestBodyFootnoteMarker(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 각주 영역 히트테스트
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestFootnote(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestFootnote(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 페이지 좌표가 머리말/꼬리말 영역에 해당하는지 판별한다.
     *
     * 반환: JSON `{"hit":true/false,"isHeader":bool,"sectionIndex":N,"applyTo":N}`
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestHeaderFooter(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestHeaderFooter(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 각주 내부 텍스트 히트테스트
     * @param {number} page_num
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestInFootnote(page_num, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestInFootnote(this.__wbg_ptr, page_num, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내부 텍스트 히트테스트.
     *
     * 편집 모드에서 클릭한 좌표의 문단·문자 위치를 반환.
     * 반환: JSON `{"hit":true,"paraIndex":N,"charOffset":N,"cursorRect":{...}}`
     * @param {number} page_num
     * @param {boolean} is_header
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestInHeaderFooter(page_num, is_header, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestInHeaderFooter(this.__wbg_ptr, page_num, is_header, x, y);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 단 나누기 삽입 (Ctrl+Shift+Enter)
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    insertColumnBreak(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertColumnBreak(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 문단에 필드 마커를 삽입한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {number} field_type
     * @returns {string}
     */
    insertFieldInHf(section_idx, is_header, apply_to, hf_para_idx, char_offset, field_type) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertFieldInHf(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, field_type);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 각주를 삽입한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    insertFootnote(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertFootnote(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 강제 쪽 나누기 삽입 (Ctrl+Enter)
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    insertPageBreak(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertPageBreak(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    insertParagraph(section_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertParagraph(this.__wbg_ptr, section_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 커서 위치에 그림을 삽입한다.
     *
     * image_data: 이미지 바이너리 데이터 (PNG/JPG/GIF/BMP 등)
     * width, height: HWPUNIT 단위 크기
     * extension: 파일 확장자 (jpg, png 등)
     *
     * 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {Uint8Array} image_data
     * @param {number} width
     * @param {number} height
     * @param {number} natural_width_px
     * @param {number} natural_height_px
     * @param {string} extension
     * @param {string} description
     * @returns {string}
     */
    insertPicture(section_idx, para_idx, char_offset, image_data, width, height, natural_width_px, natural_height_px, extension, description) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(extension, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(description, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertPicture(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0, width, height, natural_width_px, natural_height_px, ptr1, len1, ptr2, len2);
            var ptr4 = ret[0];
            var len4 = ret[1];
            if (ret[3]) {
                ptr4 = 0; len4 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred5_0 = ptr4;
            deferred5_1 = len4;
            return getStringFromWasm0(ptr4, len4);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * 표에 열을 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} col_idx
     * @param {boolean} right
     * @returns {string}
     */
    insertTableColumn(section_idx, parent_para_idx, control_idx, col_idx, right) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertTableColumn(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, col_idx, right);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표에 행을 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} row_idx
     * @param {boolean} below
     * @returns {string}
     */
    insertTableRow(section_idx, parent_para_idx, control_idx, row_idx, below) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertTableRow(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, row_idx, below);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문단에 텍스트를 삽입한다.
     *
     * 삽입 후 구역을 재구성하고 재페이지네이션한다.
     * 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertText(section_idx, para_idx, char_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertText(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 표 셀 내부 문단에 텍스트를 삽입한다.
     *
     * 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextInCellByPath(section_idx, parent_para_idx, path_json, char_offset, text) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * 각주 내 텍스트를 삽입한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextInFootnote(section_idx, para_idx, control_idx, fn_para_idx, char_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내 텍스트 삽입
     *
     * 반환: JSON `{"ok":true,"charOffset":<new_offset>}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 논리적 오프셋으로 텍스트를 삽입한다.
     *
     * logical_offset: 텍스트 문자 + 인라인 컨트롤을 각각 1로 세는 위치.
     * 예: "abc[표]XYZ" → a(0) b(1) c(2) [표](3) X(4) Y(5) Z(6)
     * logical_offset=4이면 표 뒤의 X 앞에 삽입.
     * 반환값: JSON `{"ok":true,"logicalOffset":<new_logical_offset>}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} logical_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextLogical(section_idx, para_idx, logical_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextLogical(this.__wbg_ptr, section_idx, para_idx, logical_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 논리적 오프셋 → 텍스트 오프셋 변환.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} logical_offset
     * @returns {number}
     */
    logicalToTextOffset(section_idx, para_idx, logical_offset) {
        const ret = wasm.hwpdocument_logicalToTextOffset(this.__wbg_ptr, section_idx, para_idx, logical_offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 문단별 줄 폭 측정 진단 (WASM)
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    measureWidthDiagnostic(section_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_measureWidthDiagnostic(this.__wbg_ptr, section_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 현재 문단을 이전 문단에 병합한다 (Backspace at start).
     *
     * para_idx의 텍스트가 para_idx-1에 결합되고 para_idx는 삭제된다.
     * 반환값: JSON `{"ok":true,"paraIdx":<merged_para_idx>,"charOffset":<merge_point>}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @returns {string}
     */
    mergeParagraph(section_idx, para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeParagraph(this.__wbg_ptr, section_idx, para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내부 문단을 이전 문단에 병합한다 (셀 내 Backspace at start).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<prev_idx>,"charOffset":<merge_point>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    mergeParagraphInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeParagraphInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @returns {string}
     */
    mergeParagraphInCellByPath(section_idx, parent_para_idx, path_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_mergeParagraphInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 각주 내 문단을 병합한다 (Backspace at start).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @returns {string}
     */
    mergeParagraphInFootnote(section_idx, para_idx, control_idx, fn_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeParagraphInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내 문단 병합 (Backspace at start)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<prev_idx>,"charOffset":<merge_point>}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @returns {string}
     */
    mergeParagraphInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeParagraphInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표의 셀을 병합한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} start_row
     * @param {number} start_col
     * @param {number} end_row
     * @param {number} end_col
     * @returns {string}
     */
    mergeTableCells(section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeTableCells(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 직선 끝점 이동 (글로벌 HWPUNIT 좌표)
     * @param {number} sec
     * @param {number} para
     * @param {number} ci
     * @param {number} sx
     * @param {number} sy
     * @param {number} ex
     * @param {number} ey
     * @returns {string}
     */
    moveLineEndpoint(sec, para, ci, sx, sy, ex, ey) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_moveLineEndpoint(this.__wbg_ptr, sec, para, ci, sx, sy, ex, ey);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 표의 위치 오프셋(vertical_offset, horizontal_offset)을 이동한다.
     *
     * delta_h, delta_v: HWPUNIT 단위 이동량 (양수=오른쪽/아래, 음수=왼쪽/위)
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} delta_h
     * @param {number} delta_v
     * @returns {string}
     */
    moveTableOffset(section_idx, parent_para_idx, control_idx, delta_h, delta_v) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_moveTableOffset(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, delta_h, delta_v);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 수직 커서 이동 (ArrowUp/Down) — 단일 호출로 줄/문단/표/구역 경계를 모두 처리한다.
     *
     * delta: -1=위, +1=아래
     * preferred_x: 이전 반환값의 preferredX (최초 이동 시 -1.0 전달)
     * 셀 컨텍스트: 본문이면 모두 0xFFFFFFFF 전달
     *
     * 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} delta
     * @param {number} preferred_x
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    moveVertical(section_idx, para_idx, char_offset, delta, preferred_x, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_moveVertical(this.__wbg_ptr, section_idx, para_idx, char_offset, delta, preferred_x, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 경로 기반 수직 커서 이동 (중첩 표용).
     *
     * 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {number} delta
     * @param {number} preferred_x
     * @returns {string}
     */
    moveVerticalByPath(section_idx, parent_para_idx, path_json, char_offset, delta, preferred_x) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_moveVerticalByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, delta, preferred_x);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 페이지 단위로 이전/다음 머리말·꼬리말로 이동한다.
     *
     * 반환: JSON `{"ok":true,"pageIndex":N,"sectionIdx":N,"isHeader":bool,"applyTo":N}`
     * 또는 더 이상 이동할 페이지가 없으면 `{"ok":false}`
     * @param {number} current_page
     * @param {boolean} is_header
     * @param {number} direction
     * @returns {string}
     */
    navigateHeaderFooterByPage(current_page, is_header, direction) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_navigateHeaderFooterByPage(this.__wbg_ptr, current_page, is_header, direction);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 문서 트리 DFS 기반 다음/이전 편집 가능 위치를 반환한다.
     * context_json: NavContextEntry 배열의 JSON (빈 배열 "[]" = body)
     * @param {number} sec
     * @param {number} para
     * @param {number} char_offset
     * @param {number} delta
     * @param {string} context_json
     * @returns {string}
     */
    navigateNextEditable(sec, para, char_offset, delta, context_json) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(context_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_navigateNextEditable(this.__wbg_ptr, sec, para, char_offset, delta, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * HWP 파일 바이트를 로드하여 문서 객체를 생성한다.
     * @param {Uint8Array} data
     */
    constructor(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        HwpDocumentFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * 총 페이지 수를 반환한다.
     * @returns {number}
     */
    pageCount() {
        const ret = wasm.hwpdocument_pageCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    pasteControl(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_pasteControl(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * HTML 문자열을 파싱하여 캐럿 위치에 삽입한다 (본문).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} html
     * @returns {string}
     */
    pasteHtml(section_idx, para_idx, char_offset, html) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteHtml(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * HTML 문자열을 파싱하여 셀 내부 캐럿 위치에 삽입한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {string} html
     * @returns {string}
     */
    pasteHtmlInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, html) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteHtmlInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 내부 클립보드의 내용을 캐럿 위치에 붙여넣는다 (본문 문단).
     *
     * 반환값: JSON `{"ok":true,"paraIdx":<idx>,"charOffset":<offset>}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    pasteInternal(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_pasteInternal(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 내부 클립보드의 내용을 표 셀 내부에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    pasteInternalInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_pasteInternalInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 사용자 명시 요청에 의한 lineseg 전체 reflow (#177).
     *
     * `reflow_zero_height_paragraphs` 의 자동 경로와 달리, "빈 line_segs + text 존재"
     * 케이스까지 포함해 재계산한다. 반환값은 실제로 reflow 된 문단 개수.
     *
     * 호출 이후 렌더 캐시·페이지네이션이 갱신되므로 즉시 렌더링하면 보정된 결과가 보인다.
     * @returns {number}
     */
    reflowLinesegs() {
        const ret = wasm.hwpdocument_reflowLinesegs(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (본문 문단).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    removeFieldAt(section_idx, para_idx, char_offset) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_removeFieldAt(this.__wbg_ptr, section_idx, para_idx, char_offset);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 커서 위치의 누름틀 필드를 제거한다 (셀/글상자 내 문단).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {boolean} is_textbox
     * @returns {string}
     */
    removeFieldAtInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_removeFieldAtInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 책갈피 이름 변경
     * @param {number} sec
     * @param {number} para
     * @param {number} ctrl_idx
     * @param {string} new_name
     * @returns {string}
     */
    renameBookmark(sec, para, ctrl_idx, new_name) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(new_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_renameBookmark(this.__wbg_ptr, sec, para, ctrl_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
     *
     * 반환: 완전한 `<svg>` 문자열
     * @param {string} script
     * @param {number} font_size_hwpunit
     * @param {number} color
     * @returns {string}
     */
    renderEquationPreview(script, font_size_hwpunit, color) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(script, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_renderEquationPreview(this.__wbg_ptr, ptr0, len0, font_size_hwpunit, color);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 특정 페이지를 Canvas 명령 수로 반환한다.
     * @param {number} page_num
     * @returns {number}
     */
    renderPageCanvas(page_num) {
        const ret = wasm.hwpdocument_renderPageCanvas(this.__wbg_ptr, page_num);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @param {number} page_num
     * @returns {number}
     */
    renderPageCanvasLegacy(page_num) {
        const ret = wasm.hwpdocument_renderPageCanvasLegacy(this.__wbg_ptr, page_num);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 특정 페이지를 HTML 문자열로 렌더링한다.
     * @param {number} page_num
     * @returns {string}
     */
    renderPageHtml(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_renderPageHtml(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 특정 페이지를 SVG 문자열로 렌더링한다.
     * @param {number} page_num
     * @returns {string}
     */
    renderPageSvg(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_renderPageSvg(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 특정 페이지를 Canvas 2D에 직접 렌더링한다.
     *
     * WASM 환경에서만 사용 가능하다. Canvas 크기는 페이지 크기 × scale로 설정된다.
     * scale이 0 이하이면 1.0으로 처리한다 (하위호환).
     * @param {number} page_num
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     */
    renderPageToCanvas(page_num, canvas, scale) {
        const ret = wasm.hwpdocument_renderPageToCanvas(this.__wbg_ptr, page_num, canvas, scale);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
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
     * @param {number} page_num
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     * @param {string} layer_kind
     */
    renderPageToCanvasFiltered(page_num, canvas, scale, layer_kind) {
        const ptr0 = passStringToWasm0(layer_kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_renderPageToCanvasFiltered(this.__wbg_ptr, page_num, canvas, scale, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * 특정 페이지를 기존 PageRenderTree 경로로 Canvas 2D에 직접 렌더링한다.
     * @param {number} page_num
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     */
    renderPageToCanvasLegacy(page_num, canvas, scale) {
        const ret = wasm.hwpdocument_renderPageToCanvasLegacy(this.__wbg_ptr, page_num, canvas, scale);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * 전체 치환
     * @param {string} query
     * @param {string} new_text
     * @param {boolean} case_sensitive
     * @returns {string}
     */
    replaceAll(query, new_text, case_sensitive) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(new_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceAll(this.__wbg_ptr, ptr0, len0, ptr1, len1, case_sensitive);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * 단일 치환 (검색어 기반) — 첫 번째 매치만 교체
     * @param {string} query
     * @param {string} new_text
     * @param {boolean} case_sensitive
     * @returns {string}
     */
    replaceOne(query, new_text, case_sensitive) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(new_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceOne(this.__wbg_ptr, ptr0, len0, ptr1, len1, case_sensitive);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * 텍스트 치환 (단일)
     * @param {number} sec
     * @param {number} para
     * @param {number} char_offset
     * @param {number} length
     * @param {string} new_text
     * @returns {string}
     */
    replaceText(sec, para, char_offset, length, new_text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(new_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceText(this.__wbg_ptr, sec, para, char_offset, length, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 여러 셀의 width/height를 한 번에 조절한다 (배치).
     *
     * json: `[{"cellIdx":0,"widthDelta":150},{"cellIdx":2,"heightDelta":-100}]`
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} json
     * @returns {string}
     */
    resizeTableCells(section_idx, parent_para_idx, control_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_resizeTableCells(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 지정 ID의 스냅샷으로 Document를 복원한다.
     * @param {number} id
     * @returns {string}
     */
    restoreSnapshot(id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_restoreSnapshot(this.__wbg_ptr, id);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Document 스냅샷을 저장하고 ID를 반환한다.
     * @returns {number}
     */
    saveSnapshot() {
        const ret = wasm.hwpdocument_saveSnapshot(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 문서 텍스트 검색
     * @param {string} query
     * @param {number} from_sec
     * @param {number} from_para
     * @param {number} from_char
     * @param {boolean} forward
     * @param {boolean} case_sensitive
     * @returns {string}
     */
    searchText(query, from_sec, from_para, from_char, forward, case_sensitive) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_searchText(this.__wbg_ptr, ptr0, len0, from_sec, from_para, from_char, forward, case_sensitive);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 활성 필드를 설정한다 (본문 문단 — 안내문 숨김용).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {boolean}
     */
    setActiveField(section_idx, para_idx, char_offset) {
        const ret = wasm.hwpdocument_setActiveField(this.__wbg_ptr, section_idx, para_idx, char_offset);
        return ret !== 0;
    }
    /**
     * path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {boolean}
     */
    setActiveFieldByPath(section_idx, parent_para_idx, path_json, char_offset) {
        const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_setActiveFieldByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
        return ret !== 0;
    }
    /**
     * 활성 필드를 설정한다 (셀/글상자 내 문단 — 안내문 숨김용).
     * 변경이 발생하면 true를 반환한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {boolean} is_textbox
     * @returns {boolean}
     */
    setActiveFieldInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox) {
        const ret = wasm.hwpdocument_setActiveFieldInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox);
        return ret !== 0;
    }
    /**
     * 셀 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {string} json
     * @returns {string}
     */
    setCellProperties(section_idx, parent_para_idx, control_idx, cell_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCellProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {boolean} enabled
     */
    setClipEnabled(enabled) {
        wasm.hwpdocument_setClipEnabled(this.__wbg_ptr, enabled);
    }
    /**
     * 다단 설정 변경
     * column_type: 0=일반, 1=배분, 2=평행
     * same_width: 0=다른 너비, 1=같은 너비
     * @param {number} section_idx
     * @param {number} column_count
     * @param {number} column_type
     * @param {number} same_width
     * @param {number} spacing_hu
     * @returns {string}
     */
    setColumnDef(section_idx, column_count, column_type, same_width, spacing_hu) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setColumnDef(this.__wbg_ptr, section_idx, column_count, column_type, same_width, spacing_hu);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * DPI를 설정한다.
     * @param {number} dpi
     */
    setDpi(dpi) {
        wasm.hwpdocument_setDpi(this.__wbg_ptr, dpi);
    }
    /**
     * 수식 컨트롤의 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {string} props_json
     * @returns {string}
     */
    setEquationProperties(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setEquationProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 대체 폰트 경로를 설정한다.
     * @param {string} path
     */
    setFallbackFont(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.hwpdocument_setFallbackFont(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * field_id로 필드 값을 설정한다.
     *
     * 반환: `{ok, fieldId, oldValue, newValue}`
     * @param {number} field_id
     * @param {string} value
     * @returns {string}
     */
    setFieldValue(field_id, value) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setFieldValue(this.__wbg_ptr, field_id, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 필드 이름으로 값을 설정한다.
     *
     * 반환: `{ok, fieldId, oldValue, newValue}`
     * @param {string} name
     * @param {string} value
     * @returns {string}
     */
    setFieldValueByName(name, value) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setFieldValueByName(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * 파일 이름을 설정한다 (머리말/꼬리말 필드 치환용).
     * @param {string} name
     */
    setFileName(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.hwpdocument_setFileName(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * 양식 개체 값을 설정한다.
     *
     * value_json: `{"value":1}` 또는 `{"text":"입력값"}`
     * 반환: `{ok}`
     * @param {number} sec
     * @param {number} para
     * @param {number} ci
     * @param {string} value_json
     * @returns {string}
     */
    setFormValue(sec, para, ci, value_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(value_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setFormValue(this.__wbg_ptr, sec, para, ci, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
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
     * @param {number} sec
     * @param {number} table_para
     * @param {number} table_ci
     * @param {number} cell_idx
     * @param {number} cell_para
     * @param {number} form_ci
     * @param {string} value_json
     * @returns {string}
     */
    setFormValueInCell(sec, table_para, table_ci, cell_idx, cell_para, form_ci, value_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(value_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setFormValueInCell(this.__wbg_ptr, sec, table_para, table_ci, cell_idx, cell_para, form_ci, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 문단 서식을 적용한다 (본문 문단).
     * 문단 번호 시작 방식 설정
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} mode
     * @param {number} start_num
     * @returns {string}
     */
    setNumberingRestart(section_idx, para_idx, mode, start_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setNumberingRestart(this.__wbg_ptr, section_idx, para_idx, mode, start_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션한다.
     * @param {number} section_idx
     * @param {string} json
     * @returns {string}
     */
    setPageDef(section_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setPageDef(this.__wbg_ptr, section_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 감추기 설정
     * @param {number} sec
     * @param {number} para
     * @param {boolean} hide_header
     * @param {boolean} hide_footer
     * @param {boolean} hide_master
     * @param {boolean} hide_border
     * @param {boolean} hide_fill
     * @param {boolean} hide_page_num
     * @returns {string}
     */
    setPageHide(sec, para, hide_header, hide_footer, hide_master, hide_border, hide_fill, hide_page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setPageHide(this.__wbg_ptr, sec, para, hide_header, hide_footer, hide_master, hide_border, hide_fill, hide_page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 그림 컨트롤의 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setPictureProperties(section_idx, parent_para_idx, control_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setPictureProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 구역 정의(SectionDef)를 변경하고 재페이지네이션한다.
     * @param {number} section_idx
     * @param {string} json
     * @returns {string}
     */
    setSectionDef(section_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setSectionDef(this.__wbg_ptr, section_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 모든 구역의 SectionDef를 일괄 변경하고 재페이지네이션한다.
     * @param {string} json
     * @returns {string}
     */
    setSectionDefAll(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setSectionDefAll(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Shape(글상자) 속성을 변경한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setShapeProperties(section_idx, parent_para_idx, control_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setShapeProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 조판부호 표시 여부를 설정한다 (개체 마커 + 문단부호 포함).
     * @param {boolean} enabled
     */
    setShowControlCodes(enabled) {
        wasm.hwpdocument_setShowControlCodes(this.__wbg_ptr, enabled);
    }
    /**
     * 문단부호(¶) 표시 여부를 설정한다.
     * @param {boolean} enabled
     */
    setShowParagraphMarks(enabled) {
        wasm.hwpdocument_setShowParagraphMarks(this.__wbg_ptr, enabled);
    }
    /**
     * 투명선 표시 여부를 설정한다.
     * @param {boolean} enabled
     */
    setShowTransparentBorders(enabled) {
        wasm.hwpdocument_setShowTransparentBorders(this.__wbg_ptr, enabled);
    }
    /**
     * 표 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} json
     * @returns {string}
     */
    setTableProperties(section_idx, parent_para_idx, control_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setTableProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 디버그 오버레이 표시 여부를 설정한다.
     * @param {boolean} enabled
     */
    set_debug_overlay(enabled) {
        wasm.hwpdocument_set_debug_overlay(this.__wbg_ptr, enabled);
    }
    /**
     * LINE_SEG vpos-reset 강제 분리 적용 여부를 설정한다.
     * 변경 시 페이지네이션 결과가 달라지므로 모든 섹션을 재페이지네이션한다.
     * @param {boolean} enabled
     */
    set_respect_vpos_reset(enabled) {
        wasm.hwpdocument_set_respect_vpos_reset(this.__wbg_ptr, enabled);
    }
    /**
     * 캐럿 위치에서 문단을 분할한다 (Enter 키).
     *
     * char_offset 이후의 텍스트가 새 문단으로 이동한다.
     * 반환값: JSON `{"ok":true,"paraIdx":<new_para_idx>,"charOffset":0}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    splitParagraph(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitParagraph(this.__wbg_ptr, section_idx, para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀 내부 문단을 분할한다 (셀 내 Enter 키).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<new_idx>,"charOffset":0}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    splitParagraphInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitParagraphInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {string}
     */
    splitParagraphInCellByPath(section_idx, parent_para_idx, path_json, char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraphInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * 각주 내 문단을 분할한다 (Enter).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    splitParagraphInFootnote(section_idx, para_idx, control_idx, fn_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitParagraphInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 머리말/꼬리말 내 문단 분할 (Enter 키)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<new_idx>,"charOffset":0}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    splitParagraphInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitParagraphInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 병합된 셀을 나눈다 (split).
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} row
     * @param {number} col
     * @returns {string}
     */
    splitTableCell(section_idx, parent_para_idx, control_idx, row, col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitTableCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, row, col);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 셀을 N줄 × M칸으로 분할한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} row
     * @param {number} col
     * @param {number} n_rows
     * @param {number} m_cols
     * @param {boolean} equal_row_height
     * @param {boolean} merge_first
     * @returns {string}
     */
    splitTableCellInto(section_idx, parent_para_idx, control_idx, row, col, n_rows, m_cols, equal_row_height, merge_first) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitTableCellInto(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, row, col, n_rows, m_cols, equal_row_height, merge_first);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 범위 내 셀들을 각각 N줄 × M칸으로 분할한다.
     *
     * 반환값: JSON `{"ok":true,"cellCount":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} start_row
     * @param {number} start_col
     * @param {number} end_row
     * @param {number} end_col
     * @param {number} n_rows
     * @param {number} m_cols
     * @param {boolean} equal_row_height
     * @returns {string}
     */
    splitTableCellsInRange(section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col, n_rows, m_cols, equal_row_height) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitTableCellsInRange(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col, n_rows, m_cols, equal_row_height);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 텍스트 오프셋 → 논리적 오프셋 변환.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} text_offset
     * @returns {number}
     */
    textToLogicalOffset(section_idx, para_idx, text_offset) {
        const ret = wasm.hwpdocument_textToLogicalOffset(this.__wbg_ptr, section_idx, para_idx, text_offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 머리말/꼬리말 감추기를 토글한다 (현재 쪽만).
     *
     * 반환: JSON `{"hidden":true/false}` — 토글 후 상태
     * @param {number} page_index
     * @param {boolean} is_header
     * @returns {string}
     */
    toggleHideHeaderFooter(page_index, is_header) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_toggleHideHeaderFooter(this.__wbg_ptr, page_index, is_header);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * GroupShape를 풀어 자식 개체들을 개별로 복원한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    ungroupShape(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_ungroupShape(this.__wbg_ptr, section_idx, para_idx, control_idx);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 누름틀 필드의 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true}` 또는 `{"ok":false}`
     * @param {number} field_id
     * @param {string} guide
     * @param {string} memo
     * @param {string} name
     * @param {boolean} editable
     * @returns {string}
     */
    updateClickHereProps(field_id, guide, memo, name, editable) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(guide, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(memo, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_updateClickHereProps(this.__wbg_ptr, field_id, ptr0, len0, ptr1, len1, ptr2, len2, editable);
            deferred4_0 = ret[0];
            deferred4_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * 구역 내 모든 연결선의 좌표를 연결된 도형 위치에 맞게 갱신한다.
     * @param {number} section_idx
     */
    updateConnectorsInSection(section_idx) {
        wasm.hwpdocument_updateConnectorsInSection(this.__wbg_ptr, section_idx);
    }
    /**
     * 스타일의 메타 정보(이름/영문이름/nextStyleId)를 수정한다.
     *
     * json: {"name":"...", "englishName":"...", "nextStyleId":0}
     * @param {number} style_id
     * @param {string} json
     * @returns {boolean}
     */
    updateStyle(style_id, json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_updateStyle(this.__wbg_ptr, style_id, ptr0, len0);
        return ret !== 0;
    }
    /**
     * 스타일의 CharShape/ParaShape를 수정한다.
     *
     * charMods/paraMods는 기존 parse_char_shape_mods/parse_para_shape_mods와 동일한 JSON 형식
     * @param {number} style_id
     * @param {string} char_mods_json
     * @param {string} para_mods_json
     * @returns {boolean}
     */
    updateStyleShapes(style_id, char_mods_json, para_mods_json) {
        const ptr0 = passStringToWasm0(char_mods_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(para_mods_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_updateStyleShapes(this.__wbg_ptr, style_id, ptr0, len0, ptr1, len1);
        return ret !== 0;
    }
}
if (Symbol.dispose) HwpDocument.prototype[Symbol.dispose] = HwpDocument.prototype.free;

/**
 * WASM 뷰어 컨트롤러 (뷰포트 관리 + 스케줄링)
 */
export class HwpViewer {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        HwpViewerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_hwpviewer_free(ptr, 0);
    }
    /**
     * 뷰어 생성
     * @param {HwpDocument} document
     */
    constructor(document) {
        _assertClass(document, HwpDocument);
        var ptr0 = document.__destroy_into_raw();
        const ret = wasm.hwpviewer_new(ptr0);
        this.__wbg_ptr = ret >>> 0;
        HwpViewerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * 총 페이지 수
     * @returns {number}
     */
    pageCount() {
        const ret = wasm.hwpdocument_pageCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 대기 중인 렌더링 작업 수
     * @returns {number}
     */
    pendingTaskCount() {
        const ret = wasm.hwpviewer_pendingTaskCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 특정 페이지 HTML 렌더링
     * @param {number} page_num
     * @returns {string}
     */
    renderPageHtml(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpviewer_renderPageHtml(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 특정 페이지 SVG 렌더링
     * @param {number} page_num
     * @returns {string}
     */
    renderPageSvg(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpviewer_renderPageSvg(this.__wbg_ptr, page_num);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * 줌 변경
     * @param {number} zoom
     */
    setZoom(zoom) {
        wasm.hwpviewer_setZoom(this.__wbg_ptr, zoom);
    }
    /**
     * 뷰포트 업데이트 (스크롤/리사이즈 시 호출)
     * @param {number} scroll_x
     * @param {number} scroll_y
     * @param {number} width
     * @param {number} height
     */
    updateViewport(scroll_x, scroll_y, width, height) {
        wasm.hwpviewer_updateViewport(this.__wbg_ptr, scroll_x, scroll_y, width, height);
    }
    /**
     * 현재 보이는 페이지 목록 반환
     * @returns {Uint32Array}
     */
    visiblePages() {
        const ret = wasm.hwpviewer_visiblePages(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) HwpViewer.prototype[Symbol.dispose] = HwpViewer.prototype.free;

/**
 * HWP 파일에서 썸네일 이미지만 경량 추출 (전체 파싱 없이)
 *
 * 반환: JSON `{ "format": "png"|"gif", "base64": "...", "width": N, "height": N }`
 * PrvImage가 없으면 `null` 반환
 * @param {Uint8Array} data
 * @returns {any}
 */
export function extractThumbnail(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.extractThumbnail(ptr0, len0);
    return ret;
}

/**
 * WASM panic hook 초기화 (한 번만 실행)
 */
export function init_panic_hook() {
    wasm.init_panic_hook();
}

/**
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_addColorStop_2f80f11dfad35dec: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addColorStop(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_arcTo_ddf6b8adf3bf5084: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arcTo(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_arc_60bf829e1bd2add5: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_beginPath_9873f939d695759c: function(arg0) {
            arg0.beginPath();
        },
        __wbg_bezierCurveTo_38509204f815cfd5: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.bezierCurveTo(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_clip_f2700ac6171df6c4: function(arg0) {
            arg0.clip();
        },
        __wbg_closePath_de4e48859360b1b1: function(arg0) {
            arg0.closePath();
        },
        __wbg_complete_40b841a95e35ff5a: function(arg0) {
            const ret = arg0.complete;
            return ret;
        },
        __wbg_createElement_49f60fdcaae809c8: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createLinearGradient_b3d3d1a53abe5362: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0.createLinearGradient(arg1, arg2, arg3, arg4);
            return ret;
        },
        __wbg_createPattern_c1739fd477f91847: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.createPattern(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_createRadialGradient_b43c38d4bed3b571: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            const ret = arg0.createRadialGradient(arg1, arg2, arg3, arg4, arg5, arg6);
            return ret;
        }, arguments); },
        __wbg_document_ee35a3d3ae34ef6c: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_drawImage_c3679fd2d5c86a5f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
        }, arguments); },
        __wbg_drawImage_ca2a49df50c3765b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_ellipse_3343f79b255f83a4: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.ellipse(arg1, arg2, arg3, arg4, arg5, arg6, arg7);
        }, arguments); },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_fillRect_d44afec47e3a3fab: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_4a931850b976cc62: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fill_1eb35c386c8676aa: function(arg0) {
            arg0.fill();
        },
        __wbg_getContext_2a5764d48600bc43: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_height_38750dc6de41ee75: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_instanceof_CanvasRenderingContext2d_4bb052fd1c3d134d: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_3f2f6e1edb1c9792: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_ed49b2db8df90359: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_lineTo_c584cff6c760c4a5: function(arg0, arg1, arg2) {
            arg0.lineTo(arg1, arg2);
        },
        __wbg_measureTextWidth_384ee9c8dcf2ded0: function(arg0, arg1, arg2, arg3) {
            const ret = globalThis.measureTextWidth(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        },
        __wbg_moveTo_e9190fc700d55b40: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_naturalWidth_d899aa855a564eb0: function(arg0) {
            const ret = arg0.naturalWidth;
            return ret;
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_ed07138f2680efbf: function() { return handleError(function () {
            const ret = new Image();
            return ret;
        }, arguments); },
        __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_push_8ffdcb2063340ba5: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_quadraticCurveTo_b39b7adc73767cc0: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.quadraticCurveTo(arg1, arg2, arg3, arg4);
        },
        __wbg_rect_967665357db991e9: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.rect(arg1, arg2, arg3, arg4);
        },
        __wbg_restore_0d233789d098ba64: function(arg0) {
            arg0.restore();
        },
        __wbg_rotate_31f482965274db16: function() { return handleError(function (arg0, arg1) {
            arg0.rotate(arg1);
        }, arguments); },
        __wbg_save_e0cc2e58b36d33c9: function(arg0) {
            arg0.save();
        },
        __wbg_scale_543277ecf8cf836b: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.scale(arg1, arg2);
        }, arguments); },
        __wbg_setLineDash_ecf27050368658c9: function() { return handleError(function (arg0, arg1) {
            arg0.setLineDash(arg1);
        }, arguments); },
        __wbg_set_fillStyle_783d3f7489475421: function(arg0, arg1, arg2) {
            arg0.fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_fillStyle_9bd3ccbe7ecf6c2a: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_fillStyle_f3cda410d17e5cd2: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_filter_d53eab270f7245d0: function(arg0, arg1, arg2) {
            arg0.filter = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_575685c8f7e56957: function(arg0, arg1, arg2) {
            arg0.font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_globalAlpha_c32898c5532572f4: function(arg0, arg1) {
            arg0.globalAlpha = arg1;
        },
        __wbg_set_height_f21f985387070100: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_lineCap_59a017de1ad2b0be: function(arg0, arg1, arg2) {
            arg0.lineCap = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_89fa506592f5b994: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_shadowBlur_8aa041f690cac8d0: function(arg0, arg1) {
            arg0.shadowBlur = arg1;
        },
        __wbg_set_shadowColor_cd8db5e06be21e95: function(arg0, arg1, arg2) {
            arg0.shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_shadowOffsetX_b7f3141c508230dc: function(arg0, arg1) {
            arg0.shadowOffsetX = arg1;
        },
        __wbg_set_shadowOffsetY_beb753d9e6d4a3ea: function(arg0, arg1) {
            arg0.shadowOffsetY = arg1;
        },
        __wbg_set_src_55abd261cc86df86: function(arg0, arg1, arg2) {
            arg0.src = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_strokeStyle_087121ed5350b038: function(arg0, arg1, arg2) {
            arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_cdfa5b9f1c14f5c6: function(arg0, arg1, arg2) {
            arg0.textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_c7ec6538cc52b073: function(arg0, arg1, arg2) {
            arg0.textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_width_d60bc4f2f20c56a4: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_strokeRect_4da24de25ed7fbaf: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.strokeRect(arg1, arg2, arg3, arg4);
        },
        __wbg_strokeText_477071174dda8ce7: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.strokeText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_stroke_240ea7f2407d73c0: function(arg0) {
            arg0.stroke();
        },
        __wbg_translate_3aa10730376a8c06: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.translate(arg1, arg2);
        }, arguments); },
        __wbg_width_5f66bde2e810fbde: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./rhwp_bg.js": import0,
    };
}

const HwpDocumentFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_hwpdocument_free(ptr >>> 0, 1));
const HwpViewerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_hwpviewer_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('rhwp_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
