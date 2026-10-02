/* @ts-self-types="./rhwp.d.ts" */

/**
 * 한 번의 문서 내보내기 결과.
 *
 * 바이트와 content-loss 보고서가 같은 객체에 있어 다른 저장의 상태와 섞이지 않는다.
 * `takeBytes()`는 Rust 결과의 바이트 소유권을 한 번만 소비하며, 보고서는 그 전후 어느
 * 순서로든 읽을 수 있다. 바이트를 두 번 꺼내는 것은 명시적 오류다.
 */
export class DocumentExport {
    static __wrap(ptr) {
        const obj = Object.create(DocumentExport.prototype);
        obj.__wbg_ptr = ptr;
        DocumentExportFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DocumentExportFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_documentexport_free(ptr, 0);
    }
    /**
     * 이번 산출물의 content-loss 보고서(JSON). `takeBytes()` 뒤에도 읽을 수 있다.
     * @returns {string}
     */
    contentLoss() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.documentexport_contentLoss(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 아직 JS로 옮기지 않은 바이트를 소유하는지 반환한다.
     * @returns {boolean}
     */
    hasBytes() {
        const ret = wasm.documentexport_hasBytes(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 산출 바이트 소유권을 한 번 꺼낸다.
     * @returns {Uint8Array}
     */
    takeBytes() {
        const ret = wasm.documentexport_takeBytes(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
}
if (Symbol.dispose) DocumentExport.prototype[Symbol.dispose] = DocumentExport.prototype.free;

/**
 * WASM에서 사용할 HWP 문서 래퍼
 *
 * 도메인 로직은 `DocumentCore`에 구현되어 있으며,
 * `Deref`/`DerefMut`를 통해 투명하게 접근한다.
 */
export class HwpDocument {
    static __wrap(ptr) {
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
     * [#5959] 셀/zone border_fill_id 직접 대입 (undo·redo 전용).
     *
     * 스타일 테이블을 건드리지 않고 execute 의 변경 기록을 되돌린다.
     * json: `{"cells":[{"cellIdx":0,"id":3}],"zones":[{"startRow":..,"startCol":..,
     * "endRow":..,"endCol":..,"id":5}]}` — zone `id` 가 null 이면 그 범위의 zone 을
     * 제거한다. 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} json
     * @returns {string}
     */
    applyCellBorderFillIds(section_idx, parent_para_idx, control_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCellBorderFillIds(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
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
     * 커서 좌표(list/para/pos)로 글자 서식을 건다 — 웹한글컨트롤 `Run("CharShape*")`.
     *
     * `endPos` 가 문단 길이를 넘으면 끝까지로 자른다. `pos` 는 코드 유닛이다.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} start_pos
     * @param {number} end_pos
     * @param {string} props_json
     * @returns {string}
     */
    applyCharFormatAtCursor(list_id, para_in_list, start_pos, end_pos, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormatAtCursor(this.__wbg_ptr, list_id, para_in_list, start_pos, end_pos, ptr0, len0);
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
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {string} props_json
     * @returns {string}
     */
    applyCharFormatInCellByPath(section_idx, parent_para_idx, path_json, start_offset, end_offset, props_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormatInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_offset, end_offset, ptr1, len1);
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
     * `applyCharFormatInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * startOffset, endOffset, props: object }`. `props` 는 글자 서식 JSON 객체(positional
     * 의 props_json 과 동일). positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    applyCharFormatInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormatInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 머리말/꼬리말 선택 범위에 글자 서식을 적용한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} start_hf_para_idx
     * @param {number} start_char_offset
     * @param {number} end_hf_para_idx
     * @param {number} end_char_offset
     * @param {string} props_json
     * @returns {string}
     */
    applyCharFormatInHeaderFooter(section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyCharFormatInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset, ptr0, len0);
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
     * 미주 모양을 적용한다.
     * @param {number} section_idx
     * @param {string} props_json
     * @returns {string}
     */
    applyEndnoteShape(section_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyEndnoteShape(this.__wbg_ptr, section_idx, ptr0, len0);
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
     * 커서 좌표(list/para)로 문단 서식을 건다 — 웹한글컨트롤 `Run("ParagraphShape*")`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {string} props_json
     * @returns {string}
     */
    applyParaFormatAtCursor(list_id, para_in_list, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyParaFormatAtCursor(this.__wbg_ptr, list_id, para_in_list, ptr0, len0);
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
     * 각주/미주 내부 문단 속성 적용
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @param {string} props_json
     * @returns {string}
     */
    applyParaFormatInFootnote(section_idx, para_idx, control_idx, fn_para_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyParaFormatInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx, ptr0, len0);
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
     * Shape z 순서 절대 대입(#5769 후속) — 속성쌍 커맨드의 undo/redo 경로.
     * pairs_json: `[{"ppi":N,"ci":N,"z":N},...]`
     * 반환: JSON `{"ok":true,"applied":N}`
     * @param {number} section_idx
     * @param {string} pairs_json
     * @returns {string}
     */
    applyShapeZOrderPairs(section_idx, pairs_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(pairs_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyShapeZOrderPairs(this.__wbg_ptr, section_idx, ptr0, len0);
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
     * Options JSON: {operation:{action,request},dryRun?:boolean}.
     * Preview prepares detached changes; it never mutates or saves the document.
     * @param {string} options_json
     * @returns {string}
     */
    applyTemplateOperation(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_applyTemplateOperation(this.__wbg_ptr, ptr0, len0);
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
     * [Task #2230] 기존 Picture 컨트롤에 이미지를 지정한다 — 그림 미지정
     * placeholder(missing image 컨트롤)의 편집 뷰 그림 삽입.
     *
     * `cell_path_json` 이 빈 문자열 또는 `"[]"` 면 본문 문단의 컨트롤,
     * 그 외에는 셀/글상자 안 문단의 컨트롤을 대상으로 한다. 개체 틀 크기는
     * 유지되고(한컴 placeholder 는 틀에 그림을 맞춤) BinData 등록 규칙은
     * insertPicture 와 공유한다.
     *
     * 반환: `{"ok":true,"binDataId":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} control_idx
     * @param {Uint8Array} image_data
     * @param {number} natural_width_px
     * @param {number} natural_height_px
     * @param {string} extension
     * @returns {string}
     */
    assignPictureImage(section_idx, parent_para_idx, cell_path_json, control_idx, image_data, natural_width_px, natural_height_px, extension) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(extension, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_assignPictureImage(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, control_idx, ptr1, len1, natural_width_px, natural_height_px, ptr2, len2);
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
     * 개체에 캡션을 붙인다 — 웹한글컨트롤 `Run("ShapeObjAttachCaption")`.
     * @param {number} para_in_list
     * @param {number} control_index
     * @returns {string}
     */
    attachCaptionAt(para_in_list, control_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_attachCaptionAt(this.__wbg_ptr, para_in_list, control_index);
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
     * 대형 표 continuation shadow job을 시작한다. 공개 페이지는 완료 전까지 유지된다.
     * @param {number} fragment_budget
     * @returns {string}
     */
    beginDeferredPagination(fragment_budget) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_beginDeferredPagination(this.__wbg_ptr, fragment_budget);
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
     * 나누기 — 웹한글컨트롤 `Run("BreakPage"·"BreakColumn"·"BreakColDef"·"BreakSection")`.
     *
     * `kind` 는 `page`·`column`·`colDef`·`section`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @param {string} kind
     * @returns {string}
     */
    breakAtCursor(list_id, para_in_list, pos, kind) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_breakAtCursor(this.__wbg_ptr, list_id, para_in_list, pos, ptr0, len0);
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
     * @returns {boolean}
     */
    cancelDeferredPagination() {
        const ret = wasm.hwpdocument_cancelDeferredPagination(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    canvasMetricsActive() {
        const ret = wasm.hwpdocument_canvasMetricsActive(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 삭제 직전 문단 범위 원본을 조각으로 보관한다 (#5769).
     *
     * 반드시 `deleteRangeNative` 호출 **전**에 불린다. 반환 조각 ID 는
     * `restoreDeleteFragment`/`discardDeleteFragment` 에 쓴다.
     * @param {number} section_idx
     * @param {number} start_para
     * @param {number} end_para
     * @returns {number}
     */
    captureDeleteRange(section_idx, start_para, end_para) {
        const ret = wasm.hwpdocument_captureDeleteRange(this.__wbg_ptr, section_idx, start_para, end_para);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 그림 리사이즈 전에 원본 변환만 보관한다.
     * @param {string} target_json
     * @returns {number}
     */
    capturePictureTransform(target_json) {
        const ptr0 = passStringToWasm0(target_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_capturePictureTransform(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 속성 변경 직전 구역 raw 스트림+봉인을 보관한다 (#5769 Stage 4).
     *
     * 반드시 `setSectionDef` 호출 **전**에 불린다. 반환 ID 는
     * `restoreSectionRaw`/`discardSectionRaw` 에 쓴다.
     * @param {number} section_idx
     * @returns {number}
     */
    captureSectionRaw(section_idx) {
        const ret = wasm.hwpdocument_captureSectionRaw(this.__wbg_ptr, section_idx);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Shape z-order 변경
     * operation: "front" | "back" | "forward" | "backward"
     * 반환: `{"ok":true,"zOrder":N,"moves":[{"ppi","ci","before","after"}...]}`
     * [#5769 후속] moves 는 실제로 대입된 (대상+교환 이웃) before/after 쌍이다 —
     * SetZOrderCommand 가 undo/redo 절대 복원 쌍으로 소비한다.
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
     * exact slot 하나의 명시 variable-font instance 요청을 제거한다.
     * 없는 요청의 clear는 멱등이며 다른 slot의 요청을 건드리지 않는다.
     * @param {string} options_json
     * @returns {string}
     */
    clearExactFontInstance(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_clearExactFontInstance(this.__wbg_ptr, ptr0, len0);
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
     * 셀 블록이 덮은 칸들의 글을 비운다 — `Run("TableDeleteCell")`. 규약은 merge 와 같다.
     * @param {number} list_id
     * @param {number} end_row
     * @param {number} end_col
     * @returns {string}
     */
    clearTableCellsAtCursor(list_id, end_row, end_col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_clearTableCellsAtCursor(this.__wbg_ptr, list_id, end_row, end_col);
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
     * 내부 클립보드에 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
     * @returns {boolean}
     */
    clipboardHasControl() {
        const ret = wasm.hwpdocument_clipboardHasControl(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} document
     * @param {number} fonts
     * @returns {string}
     */
    collectCanvasMetricRequests(document, fonts) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_collectCanvasMetricRequests(this.__wbg_ptr, document, fonts);
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
     *
     * [Task #1161] `cell_path_json` 이 빈 문자열/`"[]"` 면 본문, 그 외에는 셀/글상자
     * 경로(`[{"controlIndex","cellIndex","cellParaIndex"}, ...]`)의 컨트롤을 복사한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {string} cell_path_json
     * @param {number} control_idx
     * @returns {string}
     */
    copyControl(section_idx, para_idx, cell_path_json, control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_copyControl(this.__wbg_ptr, section_idx, para_idx, ptr0, len0, control_idx);
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
     * 전체 cellPath가 가리키는 중첩 셀의 선택 영역을 내부 클립보드에 복사한다(#4272).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    copySelectionInCellByPath(section_idx, parent_para_idx, path_json, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_copySelectionInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
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
     * `copySelectionInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    copySelectionInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_copySelectionInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 머리말/꼬리말 선택 범위를 내부 클립보드에 복사한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} start_hf_para_idx
     * @param {number} start_char_offset
     * @param {number} end_hf_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    copySelectionInHeaderFooter(section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_copySelectionInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset);
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
     * 선택된 표 셀 범위를 행/열 바꿈 복사용 내부 버퍼에 저장한다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} start_row
     * @param {number} start_col
     * @param {number} end_row
     * @param {number} end_col
     * @returns {string}
     */
    copyTableCellsTransposed(section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_copyTableCellsTransposed(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col);
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
     *
     * 기본 A4 구역 1개 + 빈 문단 1개를 포함한다. 구역 0개 문서는 모든
     * 편집/조회 API가 "구역 인덱스 0 범위 초과"로 실패해 사용 불가하므로
     * 생성 직후 바로 편집 가능한 최소 구조를 보장한다 (#1386).
     *
     * 여기서 만든 문단은 **구역 정의·단 정의를 안 진다** — 실제 HWP 문서는 예외 없이 그
     * 둘을 첫 문단에 지므로 이 문서는 그 점에서 실물과 다르다. 한글 호환이 필요한 자리
     * (`Clear`)는 번들 템플릿을 쓰는 [`create_blank_document`](Self::create_blank_document)
     * 를 쓴다. 여기에 그 둘을 넣으면 `char_shapes` 자리가 16칸씩 밀려 기존 호출부가 깨진다.
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
     * 커서 좌표(list/para/pos)로 글자를 지운다 — 웹한글컨트롤 `Run("Delete*")`.
     *
     * `pos` 는 코드 유닛이고, 빈 범위면 아무 일도 하지 않는다.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} start_pos
     * @param {number} end_pos
     * @returns {string}
     */
    deleteAtCursor(list_id, para_in_list, start_pos, end_pos) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteAtCursor(this.__wbg_ptr, list_id, para_in_list, start_pos, end_pos);
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
     * [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (by_path).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @returns {string}
     */
    deleteCellPictureControlByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteCellPictureControlByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx);
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
     * [#6771] 표 셀/글상자 내부 **표** 삭제 (by_path).
     *
     * 셀 안 1×1 안내 상자처럼 본문 리스트 밖에 있는 표를 지운다 — `deleteControlAt` 은
     * 본문만, `deleteTableControl` 은 `(구역, 문단, 컨트롤)` 만 다뤄 짚지 못하던 자리다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @returns {string}
     */
    deleteCellTableControlByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteCellTableControlByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx);
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
     * 컨트롤 하나를 지운다 — 웹한글컨트롤 `DeleteCtrl`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} control_index
     * @returns {string}
     */
    deleteControlAt(list_id, para_in_list, control_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteControlAt(this.__wbg_ptr, list_id, para_in_list, control_index);
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
     * 수식 컨트롤을 문단에서 삭제한다.
     *
     * 반환: JSON `{"ok":true}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    deleteEquationControl(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteEquationControl(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
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
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} start_para
     * @param {number} start_offset
     * @param {number} end_para
     * @param {number} end_offset
     * @returns {string}
     */
    deleteRangeInCellByPath(section_idx, parent_para_idx, path_json, start_para, start_offset, end_para, end_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteRangeInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_para, start_offset, end_para, end_offset);
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
     * `deleteRangeInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    deleteRangeInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteRangeInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 표 셀 내부 문단에서 텍스트를 삭제하되 전체 페이지네이션은 호출자가 지연한다.
     *
     * 결과 JSON은 `charOffset`과 상대 cell-flow 변화 신호 `cellFlowChanged`를 포함한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {number} count
     * @returns {string}
     */
    deleteTextInCellDeferredPagination(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_deleteTextInCellDeferredPagination(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, count);
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
     * `deleteTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, count }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    deleteTextInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_deleteTextInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 개체에서 캡션을 뗀다 — 웹한글컨트롤 `Run("ShapeObjDetachCaption")`.
     * @param {number} para_in_list
     * @param {number} control_index
     * @returns {string}
     */
    detachCaptionAt(para_in_list, control_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_detachCaptionAt(this.__wbg_ptr, para_in_list, control_index);
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
     * 삭제 조각을 제거하여 메모리를 해제한다 — 히스토리 축출·클리어 시 스냅샷
     * `discardSnapshot` 과 짝으로 호출한다(#5769).
     * @param {number} id
     */
    discardDeleteFragment(id) {
        wasm.hwpdocument_discardDeleteFragment(this.__wbg_ptr, id);
    }
    /**
     * @param {number} id
     */
    discardPictureTransform(id) {
        wasm.hwpdocument_discardPictureTransform(this.__wbg_ptr, id);
    }
    /**
     * 구역 raw 캡처를 제거하여 메모리를 해제한다 — 히스토리 축출·클리어 계약 (#5769 Stage 4).
     * @param {number} id
     */
    discardSectionRaw(id) {
        wasm.hwpdocument_discardSectionRaw(this.__wbg_ptr, id);
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
     * `evaluateTableFormula` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, targetRow, targetCol,
     * formula: string, writeResult? }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    evaluateTableFormulaEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_evaluateTableFormulaEx(this.__wbg_ptr, ptr0, len0);
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
     * @param {string} cell_path_json
     * @param {number} control_idx
     * @returns {string}
     */
    exportControlHtml(section_idx, para_idx, cell_path_json, control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_exportControlHtml(this.__wbg_ptr, section_idx, para_idx, ptr0, len0, control_idx);
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
     * HML 원본의 공통 IR을 HWPML 2.91 XML로 직렬화하여 반환한다.
     * @returns {Uint8Array}
     */
    exportHml() {
        const ret = wasm.hwpdocument_exportHml(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
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
     * 문서를 HWP5 EncryptVersion 4 비밀번호 문서로 내보낸다.
     *
     * browser UI는 암호를 저장하지 않고 저장 시점에만 전달한다. HWPX 출처 문서는 일반
     * HWP 저장과 동일하게 HWPX-to-HWP adapter를 먼저 적용한다.
     * @param {string} password
     * @returns {Uint8Array}
     */
    exportHwpWithPassword(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_exportHwpWithPassword(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * 비밀번호 HWP 바이트 + 내용 손실 보고 (#4430).
     * @param {string} password
     * @returns {DocumentExport}
     */
    exportHwpWithPasswordAndReport(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_exportHwpWithPasswordAndReport(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return DocumentExport.__wrap(ret[0]);
    }
    /**
     * HWP 바이트와 이번 산출물의 내용 손실을 같은 결과로 반환한다 (#4430).
     *
     * 명시적 Studio 저장은 이 API를 사용한다. 기존 `exportHwp()`는 호환성을 위해
     * byte-only로 유지되며, autosave/embed/history/compare/hwpctl/digest 등 별도
     * 소비자는 아직 보고서를 받지 않는다.
     * @returns {DocumentExport}
     */
    exportHwpWithReport() {
        const ret = wasm.hwpdocument_exportHwpWithReport(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return DocumentExport.__wrap(ret[0]);
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
     * 문서를 ODF AES-256-CBC/PBKDF2 비밀번호 보호 HWPX로 내보낸다.
     * @param {string} password
     * @returns {Uint8Array}
     */
    exportHwpxWithPassword(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_exportHwpxWithPassword(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * 비밀번호 HWPX 바이트 + 내용 손실 보고 (#4430).
     * @param {string} password
     * @returns {DocumentExport}
     */
    exportHwpxWithPasswordAndReport(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_exportHwpxWithPasswordAndReport(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return DocumentExport.__wrap(ret[0]);
    }
    /**
     * HWPX 바이트와 이번 산출물의 내용 손실을 같은 결과로 반환한다 (#4430).
     * @returns {DocumentExport}
     */
    exportHwpxWithReport() {
        const ret = wasm.hwpdocument_exportHwpxWithReport(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return DocumentExport.__wrap(ret[0]);
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
     * 전체 cellPath가 가리키는 중첩 셀 선택을 HTML로 변환한다(#4272).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    exportSelectionInCellHtmlByPath(section_idx, parent_para_idx, path_json, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_exportSelectionInCellHtmlByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
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
     * `exportSelectionInCellHtml` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    exportSelectionInCellHtmlEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_exportSelectionInCellHtmlEx(this.__wbg_ptr, ptr0, len0);
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
     * 지연된 페이지네이션을 동기 barrier로 flush하고 최신 페이지 수를 반환한다.
     * @returns {string}
     */
    flushDeferredPagination() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_flushDeferredPagination(this.__wbg_ptr);
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
     * [#4709] SVG 메트릭 face 주석 부착 여부를 반환한다.
     * @returns {boolean}
     */
    getAnnotateMetricFont() {
        const ret = wasm.hwpdocument_getAnnotateMetricFont(this.__wbg_ptr);
        return ret !== 0;
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
     * 문서 전체의 bounded CanvasKit direct replay capability를 compact JSON으로 반환한다.
     * @param {string} mode
     * @param {string} profile
     * @returns {string}
     */
    getCanvasKitDocumentPreflight(mode, profile) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCanvasKitDocumentPreflight(this.__wbg_ptr, ptr0, len0, ptr1, len1);
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
     * CanvasKit direct replay 정책 진단을 JSON 문자열로 반환한다.
     *
     * `mode` 는 `"default"` 또는 `"compat"` 를 받는다. 빈 문자열은 `"default"` 로 처리한다.
     * 현재 두 mode 모두 hidden Canvas2D overlay 없이 direct replay required 정책을 따른다.
     * `compat` 는 API/URL 호환성과 이후 보수적인 direct replay 튜닝을 위해 남겨 둔 선택지다.
     * @param {number} page_num
     * @param {string} mode
     * @returns {string}
     */
    getCanvasKitReplayPlan(page_num, mode) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCanvasKitReplayPlan(this.__wbg_ptr, page_num, ptr0, len0);
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
     * @param {number} page_num
     * @param {string} mode
     * @param {string} profile
     * @returns {string}
     */
    getCanvasKitReplayPlanWithProfile(page_num, mode, profile) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCanvasKitReplayPlanWithProfile(this.__wbg_ptr, page_num, ptr0, len0, ptr1, len1);
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
     * @param {number} page
     * @param {string} profile
     * @param {boolean} omit_font_bytes
     * @returns {string}
     */
    getCanvasPageLayerTree(page, profile, omit_font_bytes) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCanvasPageLayerTree(this.__wbg_ptr, page, ptr0, len0, omit_font_bytes);
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
     * 캐럿이 설 수 있는 자리들 — 한 글자 이동(`MoveNextChar` 류)이 딛는 눈금.
     * @param {number} list_id
     * @param {number} para_in_list
     * @returns {string}
     */
    getCaretStops(list_id, para_in_list) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getCaretStops(this.__wbg_ptr, list_id, para_in_list);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {string}
     */
    getCellCharPropertiesAtByPath(section_idx, parent_para_idx, path_json, char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCellCharPropertiesAtByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
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
     * 셀 고유 속성을 조회한다.
     *
     * cellzone overlay를 합성하지 않고 셀 자체의 borderFill만 반환한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @returns {string}
     */
    getCellOwnProperties(section_idx, parent_para_idx, control_idx, cell_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCellOwnProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx);
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
     * [Task #1151 v4] 표 셀 내 Picture 속성 조회 (by_path). Shape 패턴 정합.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @returns {string}
     */
    getCellPicturePropertiesByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCellPicturePropertiesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx);
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
     * 셀 속성을 조회한다.
     *
     * 반환: JSON `{width, height, paddingLeft, paddingRight, paddingTop, paddingBottom, applyInnerMargin, verticalAlign, textDirection, isHeader, cellProtect, fieldName, editableInForm, ...borderFill}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} cell_idx
     * @returns {string}
     */
    getCellPropertiesByPath(section_idx, parent_para_idx, cell_path_json, cell_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCellPropertiesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, cell_idx);
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
     * [Task #1138] 표 셀 내 Shape(글상자/사각형/도형) 속성 조회 (by_path).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @returns {string}
     */
    getCellShapePropertiesByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCellShapePropertiesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx);
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
     * 커서가 든 셀의 모양 — 웹한글컨트롤 `CellShape` 파라미터셋.
     * @param {number} list_id
     * @returns {string}
     */
    getCellShapeSet(list_id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getCellShapeSet(this.__wbg_ptr, list_id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 스트림 자리를 글자 번호로 옮긴다 — 글자 번호를 받는 코어 API 에 넘길 때 쓴다.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @returns {string}
     */
    getCharIndexAtStreamPos(list_id, para_in_list, pos) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCharIndexAtStreamPos(this.__wbg_ptr, list_id, para_in_list, pos);
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
     * 머리말/꼬리말 캐럿 위치의 글자 속성을 조회한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCharPropertiesInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCharPropertiesInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset);
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
     * 문자 offset 범위의 모양 구간 목록을 조회한다.
     * @param {number} sec
     * @param {number} para
     * @param {number} start
     * @param {number} end
     * @returns {string}
     */
    getCharShapeRuns(sec, para, start, end) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCharShapeRuns(this.__wbg_ptr, sec, para, start, end);
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
     * @param {number} sec
     * @param {number} para
     * @param {string} path_json
     * @param {number} start
     * @param {number} end
     * @returns {string}
     */
    getCharShapeRunsInCellByPath(sec, para, path_json, start, end) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCharShapeRunsInCellByPath(this.__wbg_ptr, sec, para, ptr0, len0, start, end);
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
     * 커서 자리의 글자 모양 — 웹한글컨트롤 `CharShape` 파라미터셋 값(§8.2.2).
     *
     * 항목 이름과 단위는 한글 것이다(`Height` 는 HWPUNIT, `AlignType` 은 코드값).
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @returns {string}
     */
    getCharShapeSet(list_id, para_in_list, pos) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getCharShapeSet(this.__wbg_ptr, list_id, para_in_list, pos);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * [#4694] 본문 직속 차트의 숫자 데이터를 조회한다 (3인자 주소).
     *
     * 컨테이너(글상자·표 셀·머리말) 안 차트는 이 주소로 표현할 수 없다 —
     * `getChartDataByIndex` 를 쓴다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getChartData(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getChartData(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
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
     * [#4694] 문서 순번(0-based)으로 차트 데이터를 조회한다 — 정본 주소.
     * @param {number} index
     * @returns {string}
     */
    getChartDataByIndex(index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getChartDataByIndex(this.__wbg_ptr, index);
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
     * 현재 구역의 다단 설정을 JSON으로 반환한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getColumnDef(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getColumnDef(this.__wbg_ptr, section_idx);
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
     * 컨트롤의 이미지 바이너리 데이터를 반환한다 (Uint8Array).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {string} cell_path_json
     * @param {number} control_idx
     * @returns {Uint8Array}
     */
    getControlImageData(section_idx, para_idx, cell_path_json, control_idx) {
        const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_getControlImageData(this.__wbg_ptr, section_idx, para_idx, ptr0, len0, control_idx);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * 컨트롤의 이미지 MIME 타입을 반환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {string} cell_path_json
     * @param {number} control_idx
     * @returns {string}
     */
    getControlImageMime(section_idx, para_idx, cell_path_json, control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getControlImageMime(this.__wbg_ptr, section_idx, para_idx, ptr0, len0, control_idx);
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
     * 문서가 담은 컨트롤 사슬 — `HeadCtrl`·`LastCtrl` 과 `Next`·`Prev` 가 딛는다.
     * @returns {string}
     */
    getControls() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getControls(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 커서가 든 필드의 상태 — 웹한글컨트롤 `CurFieldState`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @returns {number}
     */
    getCurFieldState(list_id, para_in_list, pos) {
        const ret = wasm.hwpdocument_getCurFieldState(this.__wbg_ptr, list_id, para_in_list, pos);
        return ret >>> 0;
    }
    /**
     * 한글 커서 좌표계(`list`/`para`/`pos`)를 쓰는 데 필요한 문서 사실.
     *
     * 리스트 표와 루트 리스트의 시작·끝 위치를 함께 준다. 자세한 계약은
     * `DocumentCore::get_cursor_model_json`.
     * @returns {string}
     */
    getCursorModel() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getCursorModel(this.__wbg_ptr);
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
     * [#2021] 경로 기반 커서 좌표 조회 + 페이지 힌트 — 직전 캐럿 페이지를 전달하면
     * 해당 페이지(±1)를 먼저 탐색해, 거대 표 문서에서 캐시 무효화 직후의 선형 페이지
     * 재빌드 비용을 피한다. 힌트가 틀려도 종전 전체 탐색으로 fallback (좌표 불변).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {number} hint_page
     * @returns {string}
     */
    getCursorRectByPathNear(section_idx, parent_para_idx, path_json, char_offset, hint_page) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getCursorRectByPathNear(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, hint_page);
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
     * `preview_page_hint`: 편집 정의를 투영할 대표 페이지 힌트. Studio는 구역의 첫 페이지를
     * 전달한다. 음수이면 호환 경로로 실제 적용 페이지를 앞에서부터 찾는다.
     * 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {number} preview_page_hint
     * @returns {string}
     */
    getCursorRectInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset, preview_page_hint) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, preview_page_hint);
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
     * 각주/미주 내부 커서 렉트 계산
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} note_para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    getCursorRectInNote(section_idx, para_idx, control_idx, note_para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectInNote(this.__wbg_ptr, section_idx, para_idx, control_idx, note_para_idx, char_offset);
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
     * 줄 경계 offset을 특정 시각 줄 기준으로 해석한 커서 좌표를 반환한다.
     *
     * `at_end=false`이면 lineIndex 줄의 시작, `at_end=true`이면 lineIndex 줄의 끝을 반환한다.
     * soft-wrap 경계에서는 같은 charOffset이 이전 줄 끝과 다음 줄 시작을 동시에 뜻할 수 있어
     * Home/End가 이 API로 시각 줄 affinity를 명시한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} line_index
     * @param {boolean} at_end
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @returns {string}
     */
    getCursorRectOnLine(section_idx, para_idx, line_index, at_end, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getCursorRectOnLine(this.__wbg_ptr, section_idx, para_idx, line_index, at_end, parent_para_idx, control_idx, cell_idx, cell_para_idx);
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
     * 미주 모양을 조회한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getEndnoteShape(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getEndnoteShape(this.__wbg_ptr, section_idx);
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
     * [Task #741 후속] 외부 file path 그림 영역 영역 영역 영역 basename 목록 영역 반환.
     *
     * HWP3 파일 영역 image 영역 영역 절대 경로 영역 저장 영역. WASM 환경 영역 영역 file
     * system access 부재 영역, JS 영역 영역 영역 영역 fetch 영역 영역 영역 file 영역 load
     * 영역 후 `injectExternalImage` 영역 영역 영역 inject 영역.
     *
     * 반환: JSON 배열 `["oracle.gif", "rdb02.gif", ...]` (중복 제거)
     * @returns {string}
     */
    getExternalImageBasenames() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getExternalImageBasenames(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * [Task #1142] 외부 file path 그림 reference 목록을 구조화된 JSON 배열로 반환한다.
     *
     * 반환: JSON 배열 `[{ key, binDataId, originalPath, basename, extension, loaded }, ...]`
     * @returns {string}
     */
    getExternalImageReferences() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getExternalImageReferences(this.__wbg_ptr);
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
     * 반환: `{inField, fieldId?, startCharIdx?, endCharIdx?, isGuide?, guideName?, editableInForm?}`
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
     * `getFieldInfoAtInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
     * @param {string} options_json
     * @returns {string}
     */
    getFieldInfoAtInCellEx(options_json) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getFieldInfoAtInCellEx(this.__wbg_ptr, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
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
     * 페이지의 bounded layout font decision trace를 JSON으로 반환한다 (#4961).
     * @param {number} page_num
     * @param {string} options_json
     * @returns {string}
     */
    getFontDecisionTrace(page_num, options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getFontDecisionTrace(this.__wbg_ptr, page_num, ptr0, len0);
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
     * 이 쪽에서 머리말/꼬리말을 편집할 때 대상이 되는 (구역, applyTo) 를 반환한다.
     *
     * 좌표 없이 쪽만으로 묻는 경로(툴바 `머리말`/`꼬리말`)용 — 히트테스트와 같은 답을 쓴다.
     * 반환: JSON `{"ok":true,"sectionIndex":N,"applyTo":N}`
     * @param {number} page_num
     * @param {boolean} is_header
     * @returns {string}
     */
    getHeaderFooterEditTarget(page_num, is_header) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooterEditTarget(this.__wbg_ptr, page_num, is_header);
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
     * 반환: JSON `{"ok":true,"paraCount":N,"charCount":N,"text":"..."}`
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
     * [Task #825] 머리말/꼬리말 안 그림의 속성 조회.
     * path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
     *       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
     * @param {number} section_idx
     * @param {number} outer_para_idx
     * @param {number} outer_control_idx
     * @param {number} inner_para_idx
     * @param {number} inner_control_idx
     * @returns {string}
     */
    getHeaderFooterPictureProperties(section_idx, outer_para_idx, outer_control_idx, inner_para_idx, inner_control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooterPictureProperties(this.__wbg_ptr, section_idx, outer_para_idx, outer_control_idx, inner_para_idx, inner_control_idx);
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
     * 머리말/꼬리말 정의가 속한 구역의 대표 편집 페이지(구역 첫 페이지)를 반환한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getHeaderFooterPreviewPage(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getHeaderFooterPreviewPage(this.__wbg_ptr, section_idx);
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
     * HML 열기 메타데이터와 손실 진단을 JSON으로 반환한다.
     * 다른 입력 포맷에서는 `null`을 반환한다.
     * @returns {string}
     */
    getHmlOpenMetadata() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getHmlOpenMetadata(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * HML 저장 가능 여부와 모든 차단 진단을 canonical JSON DTO로 반환한다.
     * @returns {string}
     */
    getHmlSaveState() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getHmlSaveState(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 표시 문자는 Unicode scalar 축이다. 대상 누락 시 본문으로 폴백하지 않는다.
     * @param {string} target_json
     * @returns {string}
     */
    getHyperlinkContext(target_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(target_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getHyperlinkContext(this.__wbg_ptr, ptr0, len0);
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
     * 줄이 시작하는 자리들 — `MoveLineBegin`·`MoveLineEnd` 가 딛는 값(코드 유닛).
     * @param {number} list_id
     * @param {number} para_in_list
     * @returns {string}
     */
    getLineStarts(list_id, para_in_list) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getLineStarts(this.__wbg_ptr, list_id, para_in_list);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 각주/미주 편집 모드 진입 대상 조회
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getNoteEditInfo(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getNoteEditInfo(this.__wbg_ptr, section_idx, para_idx, control_idx);
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
     * 각주/미주 내부 수식 컨트롤의 속성을 조회한다.
     * @param {string} kind
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} note_control_idx
     * @param {number} note_para_idx
     * @param {number} inner_control_idx
     * @returns {string}
     */
    getNoteEquationProperties(kind, section_idx, parent_para_idx, note_control_idx, note_para_idx, inner_control_idx) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getNoteEquationProperties(this.__wbg_ptr, ptr0, len0, section_idx, parent_para_idx, note_control_idx, note_para_idx, inner_control_idx);
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
     * 개체 사이를 도는 차례(쪽·z) — 웹한글컨트롤 `Run("ShapeObjNext/PrevObject")` 용.
     * @returns {string}
     */
    getObjectCycle() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getObjectCycle(this.__wbg_ptr);
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
     * 본문에 놓인 개체 목록 — `Run("ShapeObjNextObject")` 따위가 딛는다.
     * @returns {string}
     */
    getObjects() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getObjects(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문단 모양의 개요 번호만 탐색 정보로 반환한다.
     *
     * 일반 문단의 `1.` 같은 텍스트는 분석하지 않는다.
     * @returns {string}
     */
    getOutlineNavigation() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getOutlineNavigation(this.__wbg_ptr);
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
     * 구역의 쪽 테두리/배경 설정을 JSON으로 반환한다.
     * @param {number} section_idx
     * @returns {string}
     */
    getPageBorderFill(section_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageBorderFill(this.__wbg_ptr, section_idx);
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
     * 쪽마다 캐럿이 설 수 있는 첫 자리 — 웹한글컨트롤 `Run("MovePage*")` 용.
     * @returns {string}
     */
    getPageCaretStarts() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageCaretStarts(this.__wbg_ptr);
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
     * 본문(flow) 그림의 배치 정보만 작은 JSON 으로 반환한다 (Task #3315).
     *
     * 전체 레이어 트리를 받아 flow 그림을 걸러내던 studio 경로를 대체한다. 바이트는 빠져
     * 있고 `sourceImageKey` 로 `getSourceImageBytes` 를 부르면 된다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageFlowImageOps(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageFlowImageOps(this.__wbg_ptr, page_num);
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
     *
     * screen profile 기본값이므로 `getPageLayerTreeWithProfile` 로 위임한다 — 같은 핫패치
     * 경계를 지나야 한다. PageRenderer 가 좁은 질의를 못 쓸 때 되돌아오는 경로다.
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
     * @param {number} page_num
     * @param {string} profile
     * @param {boolean | null} [omit_image_bytes]
     * @param {boolean | null} [omit_font_bytes]
     * @returns {string}
     */
    getPageLayerTreeWithProfile(page_num, profile, omit_image_bytes, omit_font_bytes) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getPageLayerTreeWithProfile(this.__wbg_ptr, page_num, ptr0, len0, isLikeNone(omit_image_bytes) ? 0xFFFFFF : omit_image_bytes ? 1 : 0, isLikeNone(omit_font_bytes) ? 0xFFFFFF : omit_font_bytes ? 1 : 0);
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
     * 페이지 overlay 이미지 정보만 JSON 문자열로 반환한다.
     * @param {number} page_num
     * @returns {string}
     */
    getPageOverlayImages(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageOverlayImages(this.__wbg_ptr, page_num);
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
     * 페이지가 그리는 그림들의 신원 키만 작은 JSON 으로 반환한다 (Task #3315).
     * @param {number} page_num
     * @returns {string}
     */
    getPageSourceImageKeys(page_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageSourceImageKeys(this.__wbg_ptr, page_num);
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
     * 쪽 하나의 글 — 웹한글컨트롤 `GetPageText`.
     * @param {number} page_index
     * @returns {string}
     */
    getPageText(page_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getPageText(this.__wbg_ptr, page_index);
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
     * 문단 하나의 캐럿 경계 — `MoveParaBegin`·`MoveParaEnd`·`MoveListBegin/End` 가 딛는 값.
     * @param {number} list_id
     * @param {number} para_in_list
     * @returns {string}
     */
    getParaBounds(list_id, para_in_list) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getParaBounds(this.__wbg_ptr, list_id, para_in_list);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 각주/미주 내부 문단 속성 조회
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @returns {string}
     */
    getParaPropertiesInFootnote(section_idx, para_idx, control_idx, fn_para_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getParaPropertiesInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx);
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
     * 커서 자리의 문단 모양 — 웹한글컨트롤 `ParaShape` 파라미터셋 값(§8.2.11).
     * @param {number} list_id
     * @param {number} para_in_list
     * @returns {string}
     */
    getParaShapeSet(list_id, para_in_list) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getParaShapeSet(this.__wbg_ptr, list_id, para_in_list);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 문서 글을 한글 스캔 차례로 — `InitScan`·`GetText`·`ReleaseScan` 이 쓴다.
     * @returns {string}
     */
    getScanItems() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getScanItems(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 구역마다 첫 본문 문단 번호 — `MoveSectionUp`·`MoveSectionDown` 이 딛는다.
     * @returns {string}
     */
    getSectionStarts() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getSectionStarts(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
     * 전체 cellPath로 중첩 셀 선택 영역의 줄별 사각형을 반환한다(#4272).
     *
     * `path_json`의 마지막 엔트리는 선택 대상 셀을 지정하며, 시작·끝 문단 인덱스는
     * 별도 인자로 받아 여러 문단 선택도 같은 컨테이너 경로에서 처리한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} start_cell_para_idx
     * @param {number} start_char_offset
     * @param {number} end_cell_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    getSelectionRectsInCellByPath(section_idx, parent_para_idx, path_json, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getSelectionRectsInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_cell_para_idx, start_char_offset, end_cell_para_idx, end_char_offset);
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
     * `getSelectionRectsInCellByPath`의 page hint options 변형(#4272).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, path, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset, startPageHint?, endPageHint? }`.
     * `path`는 cellPath JSON 문자열이다.
     * @param {string} options_json
     * @returns {string}
     */
    getSelectionRectsInCellByPathEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getSelectionRectsInCellByPathEx(this.__wbg_ptr, ptr0, len0);
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
     * `getSelectionRectsInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
     * startCharOffset, endCellParaIdx, endCharOffset, startPageHint?, endPageHint? }`.
     * page hint가 누락되거나 유효하지 않으면 positional 과 동일한 전체 탐색을 사용한다.
     * @param {string} options_json
     * @returns {string}
     */
    getSelectionRectsInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getSelectionRectsInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 각주/미주 내부 선택 영역의 줄별 사각형을 반환한다.
     * @param {number} page_num
     * @param {number} footnote_index
     * @param {number} start_fn_para_idx
     * @param {number} start_char_offset
     * @param {number} end_fn_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    getSelectionRectsInFootnote(page_num, footnote_index, start_fn_para_idx, start_char_offset, end_fn_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getSelectionRectsInFootnote(this.__wbg_ptr, page_num, footnote_index, start_fn_para_idx, start_char_offset, end_fn_para_idx, end_char_offset);
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
     * 요청한 한 페이지의 머리말/꼬리말 선택 사각형을 반환한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} page_num
     * @param {number} start_hf_para_idx
     * @param {number} start_char_offset
     * @param {number} end_hf_para_idx
     * @param {number} end_char_offset
     * @returns {string}
     */
    getSelectionRectsInHeaderFooter(section_idx, is_header, apply_to, page_num, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getSelectionRectsInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, page_num, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset);
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
     * [Task #919] 글상자/도형 컨트롤의 페이지 좌표 바운딩박스를 반환한다.
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     * studio 의 `isShapeBorderClick` 헬퍼에서 외곽 경계선 클릭 판별에 사용.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    getShapeBBox(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getShapeBBox(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
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
     * 문단부호(¶) 표시 여부를 반환한다.
     * @returns {boolean}
     */
    getShowParagraphMarks() {
        const ret = wasm.hwpdocument_getShowParagraphMarks(this.__wbg_ptr);
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
     * Portable font resource key로 exact source bytes를 Uint8Array로 반환한다 (Task #4969).
     *
     * `getPageLayerTreeWithProfile(page, profile, imageMode, true)`의 opt-in 경로다.
     * key가 현재 document generation의 registry source와 정확히 일치하지 않으면 던진다.
     * @param {string} key
     * @returns {Uint8Array}
     */
    getSourceFontBytes(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_getSourceFontBytes(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * 원본 파일 형식을 반환한다 ("hwp", "hwpx", 또는 "hml").
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
     * 그림 신원 키로 바이트를 Uint8Array 로 반환한다 (Task #3315).
     *
     * `getPageLayerTreeWithProfile(page, profile, true)` 로 base64 를 생략했을 때 바이트를
     * 받는 경로다. mime 은 레이어 트리의 그림 op 이 계속 싣고 있으므로 여기서 되풀이하지
     * 않는다.
     *
     * 키를 풀 수 없으면 던진다 — 세대가 바뀐 낡은 키이거나 없는 그림이다. 호출부는 잡아서
     * 레이어 트리를 다시 받는 쪽으로 되돌아가면 된다.
     * @param {string} key
     * @returns {Uint8Array}
     */
    getSourceImageBytes(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_getSourceImageBytes(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * 문서에 저장된 캐럿 위치를 **원본 값 그대로** 돌려준다.
     *
     * 한글은 문서를 열면 이 자리에 캐럿을 놓는다(`GetPos` 첫 답과 일치). studio 의
     * `getCaretPosition` 은 이 값을 구역/문단으로 해석하지만, 여기서는 해석하지 않는다 —
     * `list` 는 구역 번호가 아니라 리스트 아이디다.
     * @returns {string}
     */
    getStoredCaret() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getStoredCaret(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서 구조(개요/조문) 트리를 JSON으로 반환 (사이드바 목차 네비게이션용)
     *
     * `mode`: `"auto"` | `"outline"` | `"clause"` (인식 불가 시 `auto`).
     * @param {string} mode
     * @returns {string}
     */
    getStructure(mode) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getStructure(this.__wbg_ptr, ptr0, len0);
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
     * 지정 page 에 배치된 표 fragment 의 바운딩박스를 반환한다 (#2400).
     *
     * 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} page_idx
     * @returns {string}
     */
    getTableBBoxAtPage(section_idx, parent_para_idx, control_idx, page_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_getTableBBoxAtPage(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, page_idx);
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
            const ret = wasm.hwpdocument_getTableCellBboxes(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, isLikeNone(page_hint) ? Number.MAX_SAFE_INTEGER : (page_hint) >>> 0);
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
     * 문서 글 전체 — `GetTextFile("TEXT")`. CP949 수치 참조를 적용한 JSON 문자열이다.
     * @returns {string}
     */
    getTextFileText() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getTextFileText(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 문서 글 전체 — `GetTextFile("UNICODE")`. 원문 Unicode JSON 문자열이다.
     * @returns {string}
     */
    getTextFileUnicode() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getTextFileUnicode(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
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
     * `getTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, count }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    getTextInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_getTextInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 지금 단어의 끝 — `MoveWordEnd` 가 가는 자리(다음 공백 글자의 자리).
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @returns {string}
     */
    getWordEnd(list_id, para_in_list, pos) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getWordEnd(this.__wbg_ptr, list_id, para_in_list, pos);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 단어가 시작하는 자리들 — `MoveNextWord` 류가 딛는 눈금(코드 유닛).
     * @param {number} list_id
     * @param {number} para_in_list
     * @returns {string}
     */
    getWordStarts(list_id, para_in_list) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hwpdocument_getWordStarts(this.__wbg_ptr, list_id, para_in_list);
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
     * 행/열 바꿈 복사 버퍼 보유 여부를 반환한다.
     * @returns {boolean}
     */
    hasTableTransposeClipboard() {
        const ret = wasm.hwpdocument_hasTableTransposeClipboard(this.__wbg_ptr);
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
     * 대표 편집 페이지에서 명시한 HF target으로 내부 텍스트를 히트테스트한다.
     * @param {number} page_num
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} x
     * @param {number} y
     * @returns {string}
     */
    hitTestInHeaderFooterTarget(page_num, section_idx, is_header, apply_to, x, y) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_hitTestInHeaderFooterTarget(this.__wbg_ptr, page_num, section_idx, is_header, apply_to, x, y);
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
     * Import from a distinct, read-only document handle; no source bytes in JSON.
     * Options JSON: {request:{sourceSection,sourceStart,sourceEnd,targetSection,insertBefore,count,limits?},dryRun?:boolean}.
     * @param {HwpDocument} source
     * @param {string} options_json
     * @returns {string}
     */
    importParagraphBlock(source, options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            _assertClass(source, HwpDocument);
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_importParagraphBlock(this.__wbg_ptr, source.__wbg_ptr, ptr0, len0);
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
     * @param {string} basename
     * @param {Uint8Array} data
     * @param {string} display_path
     * @returns {number}
     */
    injectExternalImage(basename, data, display_path) {
        const ptr0 = passStringToWasm0(basename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(display_path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_injectExternalImage(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        return ret >>> 0;
    }
    /**
     * [Task #1143] `getExternalImageReferences()` 의 key로 외부 이미지 bytes를 주입한다.
     *
     * 지원 key: `binData:<bin_data_id>`.
     * 잘못된 key, 존재하지 않는 key, 이미 loaded 상태인 reference는 0을 반환한다.
     * @param {string} key
     * @param {Uint8Array} data
     * @param {string} display_path
     * @returns {number}
     */
    injectExternalImageByKey(key, data, display_path) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(display_path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_injectExternalImageByKey(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        return ret >>> 0;
    }
    /**
     * 자동 번호 끼우기 — `InsertPageNum`·`InsertCpNo`·`InsertTpNo`. `kind` 는
     * `page`·`current`·`total`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @param {string} kind
     * @returns {string}
     */
    insertAutoNumberAtCursor(list_id, para_in_list, pos, kind) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertAutoNumberAtCursor(this.__wbg_ptr, list_id, para_in_list, pos, ptr0, len0);
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
     * 현재 본문 위치에 ClickHere 누름틀 필드를 삽입한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} guide
     * @param {string} memo
     * @param {string} name
     * @param {boolean} editable
     * @returns {string}
     */
    insertClickHereField(section_idx, para_idx, char_offset, guide, memo, name, editable) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(guide, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(memo, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereField(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0, ptr1, len1, ptr2, len2, editable);
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
     * 한글 커서 좌표(list/para/pos)에 누름틀을 넣는다 — 웹한글컨트롤 `CreateField`.
     *
     * `pos` 는 코드 유닛이다(확장 컨트롤 하나가 8칸). 글자 번호를 받는
     * `insertClickHereField` 와 좌표계가 다르다.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @param {string} guide
     * @param {string} memo
     * @param {string} name
     * @param {boolean} editable
     * @returns {string}
     */
    insertClickHereFieldAtCursor(list_id, para_in_list, pos, guide, memo, name, editable) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(guide, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(memo, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldAtCursor(this.__wbg_ptr, list_id, para_in_list, pos, ptr0, len0, ptr1, len1, ptr2, len2, editable);
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
     * 현재 중첩 표 cellPath 위치에 ClickHere 누름틀 필드를 삽입한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {string} guide
     * @param {string} memo
     * @param {string} name
     * @param {boolean} editable
     * @returns {string}
     */
    insertClickHereFieldByPath(section_idx, parent_para_idx, path_json, char_offset, guide, memo, name, editable) {
        let deferred6_0;
        let deferred6_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(guide, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(memo, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ptr3 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len3 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, ptr1, len1, ptr2, len2, ptr3, len3, editable);
            var ptr5 = ret[0];
            var len5 = ret[1];
            if (ret[3]) {
                ptr5 = 0; len5 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred6_0 = ptr5;
            deferred6_1 = len5;
            return getStringFromWasm0(ptr5, len5);
        } finally {
            wasm.__wbindgen_free(deferred6_0, deferred6_1, 1);
        }
    }
    /**
     * `insertClickHereFieldByPath` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, path: string, charOffset?, guide?,
     * memo?, name?, editable? }`. `path` 는 cell_path JSON 문자열. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    insertClickHereFieldByPathEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldByPathEx(this.__wbg_ptr, ptr0, len0);
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
     * `insertClickHereField` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, paraIdx, charOffset?, guide?, memo?, name?, editable? }`.
     * positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    insertClickHereFieldEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldEx(this.__wbg_ptr, ptr0, len0);
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
     * 현재 셀/글상자 위치에 ClickHere 누름틀 필드를 삽입한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {boolean} is_textbox
     * @param {string} guide
     * @param {string} memo
     * @param {string} name
     * @param {boolean} editable
     * @returns {string}
     */
    insertClickHereFieldInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox, guide, memo, name, editable) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(guide, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(memo, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox, ptr0, len0, ptr1, len1, ptr2, len2, editable);
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
     * `insertClickHereFieldInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox?, guide?, memo?, name?, editable? }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    insertClickHereFieldInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertClickHereFieldInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 미주를 삽입한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    insertEndnote(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertEndnote(this.__wbg_ptr, section_idx, para_idx, char_offset);
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
     * 수식을 삽입한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} script
     * @param {number} font_size
     * @param {number} color
     * @returns {string}
     */
    insertEquation(section_idx, para_idx, char_offset, script, font_size, color) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(script, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertEquation(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0, font_size, color);
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
     * @param {string} options_json
     * @returns {number}
     */
    insertHyperlinkEx(options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_insertHyperlinkEx(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * 새 번호 지정 컨트롤 삽입 (쪽 > 새 번호로 시작)
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} start_num
     * @returns {string}
     */
    insertNewNumber(section_idx, para_idx, char_offset, start_num) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_insertNewNumber(this.__wbg_ptr, section_idx, para_idx, char_offset, start_num);
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
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} cell_path_json
     * @param {Uint8Array} image_data
     * @param {number} width
     * @param {number} height
     * @param {number} natural_width_px
     * @param {number} natural_height_px
     * @param {string} extension
     * @param {string} description
     * @param {number | null} [paper_offset_x_hu]
     * @param {number | null} [paper_offset_y_hu]
     * @returns {string}
     */
    insertPicture(section_idx, para_idx, char_offset, cell_path_json, image_data, width, height, natural_width_px, natural_height_px, extension, description, paper_offset_x_hu, paper_offset_y_hu) {
        let deferred6_0;
        let deferred6_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(extension, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ptr3 = passStringToWasm0(description, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len3 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertPicture(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0, ptr1, len1, width, height, natural_width_px, natural_height_px, ptr2, len2, ptr3, len3, isLikeNone(paper_offset_x_hu) ? Number.MAX_SAFE_INTEGER : (paper_offset_x_hu) >> 0, isLikeNone(paper_offset_y_hu) ? Number.MAX_SAFE_INTEGER : (paper_offset_y_hu) >> 0);
            var ptr5 = ret[0];
            var len5 = ret[1];
            if (ret[3]) {
                ptr5 = 0; len5 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred6_0 = ptr5;
            deferred6_1 = len5;
            return getStringFromWasm0(ptr5, len5);
        } finally {
            wasm.__wbindgen_free(deferred6_0, deferred6_1, 1);
        }
    }
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
     * @param {string} options_json
     * @param {Uint8Array} image_data
     * @returns {string}
     */
    insertPictureEx(options_json, image_data) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertPictureEx(this.__wbg_ptr, ptr0, len0, ptr1, len1);
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
     * 커서 좌표(list/para/pos)에 글자를 끼운다 — 웹한글컨트롤 `Run("Insert*Space")`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @param {string} text
     * @returns {string}
     */
    insertTextAtCursor(list_id, para_in_list, pos, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextAtCursor(this.__wbg_ptr, list_id, para_in_list, pos, ptr0, len0);
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
     * 표 셀 내부 문단에 텍스트를 삽입하되 전체 페이지네이션은 호출자가 지연한다.
     *
     * Studio의 page-local 단일 입력처럼 현재 페이지를 먼저 갱신하고 idle 시점에
     * 전체 페이지네이션을 한 번만 수행하는 경로에서 사용한다.
     * 결과 JSON은 `charOffset`과 상대 cell-flow 변화 신호 `cellFlowChanged`를 포함한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {string} text
     * @returns {string}
     */
    insertTextInCellDeferredPagination(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInCellDeferredPagination(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, ptr0, len0);
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
     * `insertTextInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, text: string }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    insertTextInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_insertTextInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 아무 내용도 없는 빈 문서인가 — 웹한글컨트롤 `IsEmpty`(§8.2.7).
     * @returns {boolean}
     */
    isEmptyDocument() {
        const ret = wasm.hwpdocument_isEmptyDocument(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * [#4694] 문서의 모든 차트를 문서 순서로 열거한다.
     *
     * 반환: JSON `[{ index, section, paragraph, control, container?, zipPart?, nestedCopy? }]`
     * studio 는 이 목록을 선택 컨트롤과 대조해 정본 주소(문서 순번)를 얻는다.
     * @returns {string}
     */
    listCharts() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_listCharts(this.__wbg_ptr);
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
     * `mergeTableCells` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
     * endRow, endCol }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    mergeTableCellsEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_mergeTableCellsEx(this.__wbg_ptr, ptr0, len0);
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
     * 현재 표에 다음 표를 이어 붙인다 (한컴 [표-표 붙이기]).
     *
     * 반환값: JSON `{"ok":true,"rowCount":<N>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    mergeTableWithNext(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_mergeTableWithNext(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
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
     * 개체를 한 걸음 옮긴다 — 웹한글컨트롤 `ShapeObjMove*`(걸음 56 HWPUNIT).
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {number} dx
     * @param {number} dy
     * @returns {string}
     */
    moveControlAt(para_in_list, control_index, dx, dy) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_moveControlAt(this.__wbg_ptr, para_in_list, control_index, dx, dy);
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
     * `moveLineEndpoint` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, para, ci, sx, sy, ex, ey }` (좌표는 i32). positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    moveLineEndpointEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_moveLineEndpointEx(this.__wbg_ptr, ptr0, len0);
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
     * `moveVertical` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, paraIdx, charOffset?, delta, preferredX,
     * parentParaIdx?, controlIdx?, cellIdx?, cellParaIdx? }`. cell 컨텍스트 키가 모두
     * 생략되면 본문 이동(parentParaIdx=MAX 동작과 동일). positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    moveVerticalEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_moveVerticalEx(this.__wbg_ptr, ptr0, len0);
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
        this.__wbg_ptr = ret[0];
        HwpDocumentFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * 비밀번호로 보호된 HWP/HWPX 파일을 비밀번호와 함께 로드한다.
     *
     * HWP5 EncryptVersion 4, 압축 HWP3와 ODF AES-256-CBC HWPX를 지원한다.
     * 구버전/비압축 HWP3 암호화와 DRM은 지원하지 않는다.
     * 비밀번호가 틀린 경우 JS 측에서 잡을 수 있도록 에러 메시지에
     * "비밀번호가 일치하지 않"이 포함된 `JsValue` 를 반환한다.
     * 암호화되지 않은 일반 문서에 비밀번호를 전달해도 정상 로드된다.
     * @param {Uint8Array} data
     * @param {string} password
     * @returns {HwpDocument}
     */
    static openWithPassword(data, password) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_openWithPassword(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return HwpDocument.__wrap(ret[0]);
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
     * 페이지에 각주 영역이 있는지 빠르게 확인 (hitTestFootnote fast-reject).
     * 페이지네이션 메타데이터만 조회하므로 render tree build가 필요 없다 (#2428).
     * @param {number} page_num
     * @returns {boolean}
     */
    pageHasFootnoteFootholds(page_num) {
        const ret = wasm.hwpdocument_pageHasFootnoteFootholds(this.__wbg_ptr, page_num);
        return ret !== 0;
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
     * HTML 문자열을 파싱하여 cellPath가 가리키는 중첩 표 셀에 삽입한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @param {string} html
     * @returns {string}
     */
    pasteHtmlInCellByPath(section_idx, parent_para_idx, path_json, char_offset, html) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteHtmlInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, ptr1, len1);
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
     * `pasteHtmlInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, html: string }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    pasteHtmlInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteHtmlInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 한글 클립보드 문서모델(hwpjson)을 캐럿 위치에 삽입한다 (본문).
     *
     * 한글은 Ctrl+C 시 클립보드 HTML 끝 주석에 문서 모델 전체를 싣는다. HTML 에는 없는
     * 글꼴 등록·문단모양·쪽 설정·셀 속성·그림 원본이 여기 있어, 이 경로라야 원본과 같은
     * 조판이 나온다. 실패하면 호출한 쪽이 종전 `pasteHtml` 로 되돌아가면 된다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string} json
     * @returns {string}
     */
    pasteHwpJson(section_idx, para_idx, char_offset, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteHwpJson(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0);
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
     * 내부 클립보드의 내용을 cellPath가 가리키는 중첩 표 셀에 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} path_json
     * @param {number} char_offset
     * @returns {string}
     */
    pasteInternalInCellByPath(section_idx, parent_para_idx, path_json, char_offset) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_pasteInternalInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset);
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
     * 행/열 바꿈 복사 버퍼를 대상 시작 셀부터 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} start_row
     * @param {number} start_col
     * @returns {string}
     */
    pasteTableCellsTransposed(section_idx, parent_para_idx, control_idx, start_row, start_col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_pasteTableCellsTransposed(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, start_row, start_col);
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
     * 행/열 바꿈 복사 버퍼를 커서 위치에 새 표로 생성해 붙여넣는다.
     *
     * 반환값: JSON `{"ok":true,"paraIdx":N,"controlIdx":N,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @returns {string}
     */
    pasteTableCellsTransposedAsTable(section_idx, para_idx, char_offset) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_pasteTableCellsTransposedAsTable(this.__wbg_ptr, section_idx, para_idx, char_offset);
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
     * 한/글 5.x/97 OLE 수식을 편집 가능한 native equation으로 변환한다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    promoteOleEquation(section_idx, para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_promoteOleEquation(this.__wbg_ptr, section_idx, para_idx, control_idx);
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
     * @param {number} document
     * @param {number} fonts
     * @param {number} revision
     * @param {string} json
     * @returns {boolean}
     */
    registerCanvasMetricReplies(document, fonts, revision, json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_registerCanvasMetricReplies(this.__wbg_ptr, document, fonts, revision, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Browser/host font selection이 확정한 face bytes를 exact layout slot에 등록한다.
     * family 이름 재탐색 없이 `(charShapeId, languageIndex)`에 직접 결합한다.
     * @param {number} char_shape_id
     * @param {number} language_index
     * @param {Uint8Array} font_bytes
     * @param {number} face_index
     * @returns {string}
     */
    registerExactFontSource(char_shape_id, language_index, font_bytes, face_index) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passArray8ToWasm0(font_bytes, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_registerExactFontSource(this.__wbg_ptr, char_shape_id, language_index, ptr0, len0, face_index);
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
     * [#5959] 이번 apply 가 push 한 BorderFill 꼬리 항목을 절단한다.
     *
     * json: `{"fromLen":12,"dirtyWas":false}` — `from_len` 위 항목을 모두 잘라내고,
     * 원래 길이로 돌아왔으면 dirty 플래그를 `dirtyWas` 로 원복한다.
     * 반환: JSON `{"ok":true,"discarded":N,"fullyDiscarded":bool}`
     * @param {string} json
     * @returns {string}
     */
    removeBorderFillTails(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_removeBorderFillTails(this.__wbg_ptr, ptr0, len0);
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
     * `removeFieldAtInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
     * @param {string} options_json
     * @returns {string}
     */
    removeFieldAtInCellEx(options_json) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_removeFieldAtInCellEx(this.__wbg_ptr, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} options_json
     */
    removeHyperlinkEx(options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_removeHyperlinkEx(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
     * 필드 이름을 바꾼다 — 누름틀과 셀 필드를 모두 다룬다.
     *
     * `updateClickHereProps` 는 누름틀 전용이라 셀 필드에서 `{"ok":false}` 를 돌려준다.
     * 웹한글컨트롤 `RenameField`(§8.3.36)의 계약은 두 갈래를 가리지 않는다.
     *
     * 반환: JSON `{"ok":true,"renamed":N}` / `{"ok":false,"renamed":0}`
     * @param {string} oldname
     * @param {string} newname
     * @returns {string}
     */
    renameField(oldname, newname) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(oldname, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(newname, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_renameField(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            deferred3_0 = ret[0];
            deferred3_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
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
     * 구역 첫 페이지에 요청한 머리말/꼬리말 정의를 가상 투영해 Canvas 2D로 렌더링한다.
     *
     * 일반 page tree cache와 pagination active target은 바꾸지 않는다. Studio는 결과 canvas를
     * 머리말/꼬리말 밴드에만 clip해 편집 중 비인쇄 overlay로 사용한다.
     * @param {number} page_num
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     */
    renderHeaderFooterEditPreviewToCanvas(page_num, section_idx, is_header, apply_to, canvas, scale) {
        const ret = wasm.hwpdocument_renderHeaderFooterEditPreviewToCanvas(this.__wbg_ptr, page_num, section_idx, is_header, apply_to, canvas, scale);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
     * [#3137 Stage 4] 기존 Canvas의 page-space 일부만 다시 재생한다.
     *
     * Canvas 크기와 나머지 픽셀은 유지한다. 호출 조건이나 크기가 맞지 않으면 오류를
     * 반환하며 Studio는 기존 full-page repaint로 폴백한다.
     * @param {number} page_num
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     * @param {string} layer_kind
     * @param {string} profile
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     */
    renderPagePatchToCanvasFilteredWithProfile(page_num, canvas, scale, layer_kind, profile, x, y, width, height) {
        const ptr0 = passStringToWasm0(layer_kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_renderPagePatchToCanvasFilteredWithProfile(this.__wbg_ptr, page_num, canvas, scale, ptr0, len0, ptr1, len1, x, y, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
     * 명시적인 출력 profile로 특정 페이지를 SVG 문자열로 렌더링한다.
     * @param {number} page_num
     * @param {string} profile
     * @returns {string}
     */
    renderPageSvgWithProfile(page_num, profile) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_renderPageSvgWithProfile(this.__wbg_ptr, page_num, ptr0, len0);
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
     * - `"all"` → 모든 PaintOp 렌더 (기본 `renderPageToCanvas` 와 동일)
     * - `"background"` → page background layer
     * - `"flow"` → 본문 layer (BehindText / InFrontOfText plane 제외)
     * - `"flow-dynamic"` → 본문 layer 중 Image/RawSvg 제외
     * - `"flow-static"` → page background + 본문 Image/RawSvg layer
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
     * @param {number} page_num
     * @param {HTMLCanvasElement} canvas
     * @param {number} scale
     * @param {string} layer_kind
     * @param {string} profile
     */
    renderPageToCanvasFilteredWithProfile(page_num, canvas, scale, layer_kind, profile) {
        const ptr0 = passStringToWasm0(layer_kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_renderPageToCanvasFilteredWithProfile(this.__wbg_ptr, page_num, canvas, scale, ptr0, len0, ptr1, len1);
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
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {number} delete_count
     * @param {string} text
     * @returns {string}
     */
    replaceBodyTextLocal(section_idx, para_idx, char_offset, delete_count, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceBodyTextLocal(this.__wbg_ptr, section_idx, para_idx, char_offset, delete_count, ptr0, len0);
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
     * @param {string} options_json
     * @returns {boolean}
     */
    replaceHyperlinkTextEx(options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_replaceHyperlinkTextEx(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
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
     * 머리말/꼬리말 선택 범위를 평문으로 원자 치환한다.
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} start_hf_para_idx
     * @param {number} start_char_offset
     * @param {number} end_hf_para_idx
     * @param {number} end_char_offset
     * @param {string} replacement_text
     * @returns {string}
     */
    replaceRangeInHeaderFooter(section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset, replacement_text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(replacement_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceRangeInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, start_hf_para_idx, start_char_offset, end_hf_para_idx, end_char_offset, ptr0, len0);
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
     * 표 셀 내부의 짧은 IME 조합 문자열을 원자적으로 교체하고 전체 페이지네이션은 지연한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {number} delete_count
     * @param {string} text
     * @returns {string}
     */
    replaceTextInCellDeferredPagination(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, delete_count, text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_replaceTextInCellDeferredPagination(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, delete_count, ptr0, len0);
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
     * 개체 크기를 한 걸음 바꾼다 — 웹한글컨트롤 `ShapeObjResize*`(걸음 283 HWPUNIT).
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {number} d_width
     * @param {number} d_height
     * @returns {string}
     */
    resizeControlAt(para_in_list, control_index, d_width, d_height) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_resizeControlAt(this.__wbg_ptr, para_in_list, control_index, d_width, d_height);
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
     * [#7189] 중첩 표의 셀 크기를 셀 경로로 조절한다 (배치).
     *
     * `cell_path_json`: `[{"controlIndex":0,"cellIndex":0,"cellParaIndex":9},...]`
     * 마지막 항목이 조절할 표를 가리킨다. 깊이 1 이면 평면 API 와 같은 경로로 처리한다.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {string} json
     * @returns {string}
     */
    resizeTableCellsByPath(section_idx, parent_para_idx, cell_path_json, json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_resizeTableCellsByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, ptr1, len1);
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
     * 삭제 조각을 원래 자리에 되돌려 끼운다 — 삭제의 참 역연산 (#5769).
     *
     * 스냅샷 복원과 달리 문서 전체가 아니라 삭제 범위+꼬리 줄 좌표만 되돌린다.
     * 성공 시 조각은 소비(제거)된다.
     * @param {number} id
     * @returns {string}
     */
    restoreDeleteFragment(id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_restoreDeleteFragment(this.__wbg_ptr, id);
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
     * 캡처한 구역 raw 를 되돌린다 — old 속성 재적용(재무효화) **뒤** 에 불린다 (#5769 Stage 4).
     * @param {number} id
     * @returns {string}
     */
    restoreSectionRaw(id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_restoreSectionRaw(this.__wbg_ptr, id);
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
     * 문서 전체 검색 (모든 매치 반환)
     * @param {string} query
     * @param {boolean} case_sensitive
     * @param {boolean} include_cells
     * @returns {string}
     */
    searchAllText(query, case_sensitive, include_cells) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_searchAllText(this.__wbg_ptr, ptr0, len0, case_sensitive, include_cells);
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
     * 문서 텍스트 검색
     *
     * [#3865] `include_cells` 를 참으로 주면 표 셀 안의 일반 텍스트 매치도 돌려준다. 그 경우
     * 결과에 `cellContext`(parentPara·ctrlIdx·cellIdx·cellPara)가 실리므로, 호출자는
     * 그 좌표로 커서를 옮길 수 있어야 한다. 생략하면 종전대로 본문만 본다.
     * @param {string} query
     * @param {number} from_sec
     * @param {number} from_para
     * @param {number} from_char
     * @param {boolean} forward
     * @param {boolean} case_sensitive
     * @param {boolean | null} [include_cells]
     * @returns {string}
     */
    searchText(query, from_sec, from_para, from_char, forward, case_sensitive, include_cells) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_searchText(this.__wbg_ptr, ptr0, len0, from_sec, from_para, from_char, forward, case_sensitive, isLikeNone(include_cells) ? 0xFFFFFF : include_cells ? 1 : 0);
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
     * @returns {boolean}
     */
    selectCanvasMetrics(enabled) {
        const ret = wasm.hwpdocument_selectCanvasMetrics(this.__wbg_ptr, enabled);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
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
     * `setActiveFieldInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * charOffset?, isTextbox? }`. positional 과 동일 동작(bool 반환).
     * @param {string} options_json
     * @returns {boolean}
     */
    setActiveFieldInCellEx(options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_setActiveFieldInCellEx(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * [#4709] SVG 출력에 배치 메트릭 face 주석을 붙일지 설정한다 (기본 꺼짐).
     *
     * 켜면 `renderPageSvg` 계열 출력의 각 `<text>`에 `data-metric-font`,
     * 루트 `<svg>`에 `data-rhwp-metric-fonts`(쉼표 구분 목록)가 붙는다.
     * 임베드 호스트가 해당 폰트 설치 여부 확인·대체 폰트 자간 보정에 쓴다.
     * 배치(레이아웃)에는 영향이 없는 뷰 전용 주석이다.
     * @param {boolean} enabled
     */
    setAnnotateMetricFont(enabled) {
        wasm.hwpdocument_setAnnotateMetricFont(this.__wbg_ptr, enabled);
    }
    /**
     * [#4180] 저장 직전 UI 캐럿을 문서 캐럿 메타데이터에 반영한다
     * (한컴 의미론: 저장 시점 캐럿). 범위 밖 위치는 무시 — 저장을 막지 않는다.
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     */
    setCaretPosition(section_idx, para_idx, char_offset) {
        wasm.hwpdocument_setCaretPosition(this.__wbg_ptr, section_idx, para_idx, char_offset);
    }
    /**
     * 셀 내 문단의 paraShapeId를 직접 설정한다.
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} para_shape_id
     * @returns {string}
     */
    setCellParaShapeId(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, para_shape_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setCellParaShapeId(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, para_shape_id);
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
     * [Task #1151 v4] 표 셀 내 Picture 속성 변경 (by_path). Shape 패턴 정합.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setCellPicturePropertiesByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx, props_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCellPicturePropertiesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx, ptr1, len1);
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
     * 셀 속성을 수정한다.
     *
     * 반환: JSON `{"ok":true,"changes":[{cellIdx,beforeId,afterId}...],
     * "borderFillLenBefore":N,"docInfoDirtyBefore":bool}` — changes 는 [#5959]
     * borderFillId 전환 기록(target+이웃)이다.
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
     * [Task #1138] 표 셀 내 Shape 속성 변경 (by_path).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {string} cell_path_json
     * @param {number} inner_control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setCellShapePropertiesByPath(section_idx, parent_para_idx, cell_path_json, inner_control_idx, props_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(cell_path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCellShapePropertiesByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, inner_control_idx, ptr1, len1);
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
     * 선택 영역을 하나의 셀처럼 취급하는 cellzone 테두리/배경 속성을 적용한다.
     *
     * 반환: JSON `{"ok":true,"startRow":...,"borderFillId":...,"zoneBeforeId":...,
     * "borderFillLenBefore":...,"docInfoDirtyBefore":...}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} start_row
     * @param {number} start_col
     * @param {number} end_row
     * @param {number} end_col
     * @param {string} json
     * @returns {string}
     */
    setCellZoneProperties(section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCellZoneProperties(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, start_row, start_col, end_row, end_col, ptr0, len0);
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
     * 글자 서식 ID를 직접 복원한다 (본문 문단).
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {number} char_shape_id
     * @returns {string}
     */
    setCharShapeId(sec_idx, para_idx, start_offset, end_offset, char_shape_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setCharShapeId(this.__wbg_ptr, sec_idx, para_idx, start_offset, end_offset, char_shape_id);
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
     * 글자 서식 ID를 직접 복원한다 (셀 내 문단).
     * @param {number} sec_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {number} char_shape_id
     * @returns {string}
     */
    setCharShapeIdInCell(sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, start_offset, end_offset, char_shape_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setCharShapeIdInCell(this.__wbg_ptr, sec_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, start_offset, end_offset, char_shape_id);
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
     * @param {number} start_offset
     * @param {number} end_offset
     * @param {number} char_shape_id
     * @returns {string}
     */
    setCharShapeIdInCellByPath(section_idx, parent_para_idx, path_json, start_offset, end_offset, char_shape_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCharShapeIdInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, start_offset, end_offset, char_shape_id);
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
     * `setCharShapeIdInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
     * startOffset, endOffset, charShapeId }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    setCharShapeIdInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCharShapeIdInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * 구간 목록 전체를 검사한 뒤 본문 모양을 복원한다.
     * @param {number} sec
     * @param {number} para
     * @param {number} start
     * @param {number} end
     * @param {string} runs_json
     * @returns {string}
     */
    setCharShapeRuns(sec, para, start, end, runs_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(runs_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCharShapeRuns(this.__wbg_ptr, sec, para, start, end, ptr0, len0);
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
     * @param {number} sec
     * @param {number} para
     * @param {string} path_json
     * @param {number} start
     * @param {number} end
     * @param {string} runs_json
     * @returns {string}
     */
    setCharShapeRunsInCellByPath(sec, para, path_json, start, end, runs_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(runs_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setCharShapeRunsInCellByPath(this.__wbg_ptr, sec, para, ptr0, len0, start, end, ptr1, len1);
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
     * [#4694] 본문 직속 차트의 숫자 데이터를 바꾼다 (3인자 주소).
     *
     * 반환: `{ok, chart, changedCount, changed[], wrote[]}` 또는 `{ok:false, invalid[]}`.
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {string} edits_json
     * @returns {string}
     */
    setChartData(section_idx, parent_para_idx, control_idx, edits_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(edits_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setChartData(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, ptr0, len0);
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
     * [#4694] 문서 순번(0-based)으로 차트 데이터를 바꾼다 — 정본 주소.
     * @param {number} index
     * @param {string} edits_json
     * @returns {string}
     */
    setChartDataByIndex(index, edits_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(edits_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setChartDataByIndex(this.__wbg_ptr, index, ptr0, len0);
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
     * 개체를 뒤집는다 — 웹한글컨트롤 `Run("ShapeObjHorzFlip")` 계열.
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {boolean} vertical
     * @param {boolean} org_state
     * @returns {string}
     */
    setControlFlipAt(para_in_list, control_index, vertical, org_state) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setControlFlipAt(this.__wbg_ptr, para_in_list, control_index, vertical, org_state);
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
     * 개체의 잠금을 켜고 끈다 — 웹한글컨트롤 `ShapeObjLock`·`ShapeObjUnlockAll`.
     *
     * 문단·컨트롤 번호에 `u32::MAX` 를 주면 "모두"라는 뜻이다(모두 풀기가 쓴다).
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {boolean} locked
     * @returns {string}
     */
    setControlLock(para_in_list, control_index, locked) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setControlLock(this.__wbg_ptr, para_in_list, control_index, locked);
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
     * 개체의 앞뒤 순서를 바꾼다 — 웹한글컨트롤 `Run("ShapeObjBringToFront")` 계열.
     *
     * `mode` 는 `front`·`back`·`forward`·`backward`·`inFrontOfText`·`behindText`.
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {string} mode
     * @returns {string}
     */
    setControlZOrderAt(para_in_list, control_index, mode) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setControlZOrderAt(this.__wbg_ptr, para_in_list, control_index, ptr0, len0);
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
     * 이미 등록된 exact font source slot에 명시 variable-font instance 요청을 설정한다.
     * JSON DTO의 검증·canonicalization·mutation 권위는 native command 한 곳에만 둔다.
     * @param {string} options_json
     * @returns {string}
     */
    setExactFontInstance(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setExactFontInstance(this.__wbg_ptr, ptr0, len0);
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
     * Set an explicit layout/paint font environment. None restores the default.
     * @param {string | null} [json]
     * @returns {boolean}
     */
    setFontEnvironment(json) {
        var ptr0 = isLikeNone(json) ? 0 : passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_setFontEnvironment(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
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
     * `setFormValueInCell` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, tablePara, tableCi, cellIdx, cellPara, formCi, value: object }`.
     * positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    setFormValueInCellEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setFormValueInCellEx(this.__wbg_ptr, ptr0, len0);
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
     * [Task #825] 머리말/꼬리말 안 그림 속성 변경.
     * @param {number} section_idx
     * @param {number} outer_para_idx
     * @param {number} outer_control_idx
     * @param {number} inner_para_idx
     * @param {number} inner_control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setHeaderFooterPictureProperties(section_idx, outer_para_idx, outer_control_idx, inner_para_idx, inner_control_idx, props_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setHeaderFooterPictureProperties(this.__wbg_ptr, section_idx, outer_para_idx, outer_control_idx, inner_para_idx, inner_control_idx, ptr0, len0);
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
     * 각주/미주 내부 수식 컨트롤의 속성을 변경한다.
     * @param {string} kind
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} note_control_idx
     * @param {number} note_para_idx
     * @param {number} inner_control_idx
     * @param {string} props_json
     * @returns {string}
     */
    setNoteEquationProperties(kind, section_idx, parent_para_idx, note_control_idx, note_para_idx, inner_control_idx, props_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(props_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setNoteEquationProperties(this.__wbg_ptr, ptr0, len0, section_idx, parent_para_idx, note_control_idx, note_para_idx, inner_control_idx, ptr1, len1);
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
     * `setNoteEquationProperties` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ kind, sectionIdx, parentParaIdx, noteControlIdx, noteParaIdx,
     * innerControlIdx, props: object }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    setNoteEquationPropertiesEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setNoteEquationPropertiesEx(this.__wbg_ptr, ptr0, len0);
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
     * 구역의 쪽 테두리/배경 설정을 변경하고 재페이지네이션한다.
     * @param {number} section_idx
     * @param {string} json
     * @returns {string}
     */
    setPageBorderFill(section_idx, json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setPageBorderFill(this.__wbg_ptr, section_idx, ptr0, len0);
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
     * `setPageHide` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sec, para, hideHeader?, hideFooter?, hideMaster?, hideBorder?,
     * hideFill?, hidePageNum? }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    setPageHideEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_setPageHideEx(this.__wbg_ptr, ptr0, len0);
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
     * 문단의 paraShapeId를 직접 설정한다.
     * @param {number} sec_idx
     * @param {number} para_idx
     * @param {number} para_shape_id
     * @returns {string}
     */
    setParaShapeId(sec_idx, para_idx, para_shape_id) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setParaShapeId(this.__wbg_ptr, sec_idx, para_idx, para_shape_id);
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
     * 개체에 글상자를 붙이거나 뗀다 — 웹한글컨트롤 `Run("ShapeObjAttach/DetachTextBox")`.
     * @param {number} para_in_list
     * @param {number} control_index
     * @param {boolean} attach
     * @returns {string}
     */
    setTextBoxAt(para_in_list, control_index, attach) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_setTextBoxAt(this.__wbg_ptr, para_in_list, control_index, attach);
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
     * undo 스냅샷 저장소의 축출 상한. studio 예산의 유일한 출처다 (#7002 후속).
     *
     * studio 는 이 값에서 예산(`상한 - 2`)을 계산한다. 상수를 양쪽에 두면 순 Rust
     * 변경에서 frontend 레인이 skip 되어 드리프트가 CI 를 통과했다 — 값을 내보내
     * 사본을 없앤다.
     * @returns {number}
     */
    snapshotCapacity() {
        const ret = wasm.hwpdocument_snapshotCapacity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 커서 자리에서 문단을 가른다 — 웹한글컨트롤 `Run("BreakPara")`.
     * @param {number} list_id
     * @param {number} para_in_list
     * @param {number} pos
     * @returns {string}
     */
    splitParaAtCursor(list_id, para_in_list, pos) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitParaAtCursor(this.__wbg_ptr, list_id, para_in_list, pos);
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
     * 캐럿 위치에서 문단을 분할한다 (Enter 키).
     *
     * char_offset 이후의 텍스트가 새 문단으로 이동한다.
     * 반환값: JSON `{"ok":true,"paraIdx":<new_para_idx>,"charOffset":0}`
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} char_offset
     * @param {string | null} [removed_para_meta]
     * @returns {string}
     */
    splitParagraph(section_idx, para_idx, char_offset, removed_para_meta) {
        let deferred3_0;
        let deferred3_1;
        try {
            var ptr0 = isLikeNone(removed_para_meta) ? 0 : passStringToWasm0(removed_para_meta, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraph(this.__wbg_ptr, section_idx, para_idx, char_offset, ptr0, len0);
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
     * 셀 내부 문단을 분할한다 (셀 내 Enter 키).
     *
     * 반환값: JSON `{"ok":true,"cellParaIndex":<new_idx>,"charOffset":0}`
     *
     * `removed_para_meta` 는 병합 undo 가 되돌려주는 값이다 — 본문 `splitParagraph`
     * 와 같은 규약이다 (Task #2342).
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} cell_idx
     * @param {number} cell_para_idx
     * @param {number} char_offset
     * @param {string | null} [removed_para_meta]
     * @returns {string}
     */
    splitParagraphInCell(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, removed_para_meta) {
        let deferred3_0;
        let deferred3_1;
        try {
            var ptr0 = isLikeNone(removed_para_meta) ? 0 : passStringToWasm0(removed_para_meta, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraphInCell(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, ptr0, len0);
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
     * @param {string | null} [removed_para_meta]
     * @returns {string}
     */
    splitParagraphInCellByPath(section_idx, parent_para_idx, path_json, char_offset, removed_para_meta) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(path_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(removed_para_meta) ? 0 : passStringToWasm0(removed_para_meta, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraphInCellByPath(this.__wbg_ptr, section_idx, parent_para_idx, ptr0, len0, char_offset, ptr1, len1);
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
     * 각주 내 문단을 분할한다 (Enter).
     * @param {number} section_idx
     * @param {number} para_idx
     * @param {number} control_idx
     * @param {number} fn_para_idx
     * @param {number} char_offset
     * @param {string | null} [removed_para_meta]
     * @returns {string}
     */
    splitParagraphInFootnote(section_idx, para_idx, control_idx, fn_para_idx, char_offset, removed_para_meta) {
        let deferred3_0;
        let deferred3_1;
        try {
            var ptr0 = isLikeNone(removed_para_meta) ? 0 : passStringToWasm0(removed_para_meta, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraphInFootnote(this.__wbg_ptr, section_idx, para_idx, control_idx, fn_para_idx, char_offset, ptr0, len0);
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
     * 머리말/꼬리말 내 문단 분할 (Enter 키)
     *
     * 반환: JSON `{"ok":true,"hfParaIndex":<new_idx>,"charOffset":0}`
     * @param {number} section_idx
     * @param {boolean} is_header
     * @param {number} apply_to
     * @param {number} hf_para_idx
     * @param {number} char_offset
     * @param {string | null} [removed_para_meta]
     * @returns {string}
     */
    splitParagraphInHeaderFooter(section_idx, is_header, apply_to, hf_para_idx, char_offset, removed_para_meta) {
        let deferred3_0;
        let deferred3_1;
        try {
            var ptr0 = isLikeNone(removed_para_meta) ? 0 : passStringToWasm0(removed_para_meta, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitParagraphInHeaderFooter(this.__wbg_ptr, section_idx, is_header, apply_to, hf_para_idx, char_offset, ptr0, len0);
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
     * 표를 지정 행에서 두 개로 나눈다 (한컴 [표-표 나누기]).
     *
     * 반환값: JSON `{"ok":true,"frontRows":<N>,"backParaIdx":<P>}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @param {number} at_row
     * @returns {string}
     */
    splitTable(section_idx, parent_para_idx, control_idx, at_row) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_splitTable(this.__wbg_ptr, section_idx, parent_para_idx, control_idx, at_row);
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
     * `splitTableCellInto` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, row, col, nRows, mCols,
     * equalRowHeight?, mergeFirst? }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    splitTableCellIntoEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitTableCellIntoEx(this.__wbg_ptr, ptr0, len0);
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
     * `splitTableCellsInRange` 의 options object 변형 (#1413).
     *
     * options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
     * endRow, endCol, nRows, mCols, equalRowHeight? }`. positional 과 동일 동작.
     * @param {string} options_json
     * @returns {string}
     */
    splitTableCellsInRangeEx(options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_splitTableCellsInRangeEx(this.__wbg_ptr, ptr0, len0);
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
     * 대형 표 continuation을 fragment budget만큼 전진한다.
     * @param {number} fragment_budget
     * @returns {string}
     */
    stepDeferredPagination(fragment_budget) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_stepDeferredPagination(this.__wbg_ptr, fragment_budget);
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
     * 저장 상태와 현재 상태를 교환한다. 같은 ID로 Undo/Redo를 수행한다.
     * @param {number} id
     */
    swapPictureTransform(id) {
        const ret = wasm.hwpdocument_swapPictureTransform(this.__wbg_ptr, id);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * 커서가 든 셀을 기준으로 표를 고친다 — 웹한글컨트롤 `Run("TableInsert*"·"TableDelete*")`.
     *
     * `op` 는 `insertRowAbove`·`insertRowBelow`·`insertColLeft`·`insertColRight`·
     * `deleteRow`·`deleteCol`.
     * @param {number} list_id
     * @param {string} op
     * @returns {string}
     */
    tableEditAtCursor(list_id, op) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(op, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpdocument_tableEditAtCursor(this.__wbg_ptr, list_id, ptr0, len0);
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
     * 커서가 든 셀에서 `(endRow, endCol)` 까지를 하나로 합친다 — `Run("TableMergeCell")`.
     * @param {number} list_id
     * @param {number} end_row
     * @param {number} end_col
     * @returns {string}
     */
    tableMergeAtCursor(list_id, end_row, end_col) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_tableMergeAtCursor(this.__wbg_ptr, list_id, end_row, end_col);
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
     * 선택된 전체 표를 제자리에서 전치한다.
     *
     * 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
     * @param {number} section_idx
     * @param {number} parent_para_idx
     * @param {number} control_idx
     * @returns {string}
     */
    transposeTableCellsInPlace(section_idx, parent_para_idx, control_idx) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.hwpdocument_transposeTableCellsInPlace(this.__wbg_ptr, section_idx, parent_para_idx, control_idx);
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
     * @param {string} options_json
     * @returns {boolean}
     */
    updateHyperlinkEx(options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hwpdocument_updateHyperlinkEx(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
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
        this.__wbg_ptr = ret;
        HwpViewerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * 총 페이지 수
     * @returns {number}
     */
    pageCount() {
        const ret = wasm.hwpviewer_pageCount(this.__wbg_ptr);
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
     * 명시적인 출력 profile로 특정 페이지 SVG 렌더링
     * @param {number} page_num
     * @param {string} profile
     * @returns {string}
     */
    renderPageSvgWithProfile(page_num, profile) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.hwpviewer_renderPageSvgWithProfile(this.__wbg_ptr, page_num, ptr0, len0);
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
     * [#4709] SVG 출력에 배치 메트릭 face 주석을 붙일지 설정한다 (기본 꺼짐).
     * @param {boolean} enabled
     */
    setAnnotateMetricFont(enabled) {
        wasm.hwpviewer_setAnnotateMetricFont(this.__wbg_ptr, enabled);
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
        __wbg___wbindgen_is_undefined_6cff064c44e0d823: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_bb96b2010945f0bc: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_addColorStop_35d831fa917ffcd4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addColorStop(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_arcTo_e447860b0ef384fb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arcTo(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_arc_782f59ce766a8abb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_beginPath_4b87fe7ed5408cac: function(arg0) {
            arg0.beginPath();
        },
        __wbg_bezierCurveTo_2ef9bdbf3d7cab27: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.bezierCurveTo(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_clearRect_81c3c80fbe793b63: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearRect(arg1, arg2, arg3, arg4);
        },
        __wbg_clip_11b699e637bb5b4c: function(arg0) {
            arg0.clip();
        },
        __wbg_closePath_c56cc36eea49716f: function(arg0) {
            arg0.closePath();
        },
        __wbg_complete_75431fc0118b19d3: function(arg0) {
            const ret = arg0.complete;
            return ret;
        },
        __wbg_createElement_7f42344eee7bb810: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createLinearGradient_d16f7c26c44e0b0a: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0.createLinearGradient(arg1, arg2, arg3, arg4);
            return ret;
        },
        __wbg_createPattern_f95263b497f37f3c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.createPattern(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_createRadialGradient_c816f53e6e0afb32: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            const ret = arg0.createRadialGradient(arg1, arg2, arg3, arg4, arg5, arg6);
            return ret;
        }, arguments); },
        __wbg_document_ac38448dbfd31a57: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_drawImage_87a05b54f458ec06: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
        }, arguments); },
        __wbg_drawImage_bd52f4fa19cef193: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
        }, arguments); },
        __wbg_drawImage_c9877592da6891fb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_drawImage_df46beb36e04ae8f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawImage(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_ellipse_5101aa9d3056735c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.ellipse(arg1, arg2, arg3, arg4, arg5, arg6, arg7);
        }, arguments); },
        __wbg_error_757e9472f8410341: function(arg0, arg1) {
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
        __wbg_fillRect_3077c0e38eb34cd1: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_2ebd722b6f37129e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fill_33944400e9c94f79: function(arg0) {
            arg0.fill();
        },
        __wbg_font_3157caf33a1ae562: function(arg0, arg1) {
            const ret = arg1.font;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getContext_71c33f14b63da593: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getRandomValues_436a51d0629d84e1: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_height_e56f6fb197710e09: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_instanceof_CanvasRenderingContext2d_d23139c3ef7651a3: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_327e7f7530c72bbd: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_5625ff9937037a38: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_lineTo_9495a068a4f48283: function(arg0, arg1, arg2) {
            arg0.lineTo(arg1, arg2);
        },
        __wbg_measureText_13aedbb9b35a9a0a: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.measureText(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_moveTo_a5882cdf1a7d39d9: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_naturalWidth_8c1fdea17f824ef1: function(arg0) {
            const ret = arg0.naturalWidth;
            return ret;
        },
        __wbg_new_116be93542d39019: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_39d92b34a3fbdc80: function() { return handleError(function () {
            const ret = new Image();
            return ret;
        }, arguments); },
        __wbg_new_with_u8_clamped_array_and_sh_d9a3bf9abac17f51: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = new ImageData(getClampedArrayU8FromWasm0(arg0, arg1), arg2 >>> 0, arg3 >>> 0);
            return ret;
        }, arguments); },
        __wbg_of_598c0ff0cd48a890: function(arg0, arg1) {
            const ret = Array.of(arg0, arg1);
            return ret;
        },
        __wbg_push_adb0107829f02d75: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_putImageData_17fd10517d5503a3: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.putImageData(arg1, arg2, arg3);
        }, arguments); },
        __wbg_quadraticCurveTo_5bb4e18a192b53b9: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.quadraticCurveTo(arg1, arg2, arg3, arg4);
        },
        __wbg_rect_b3341a8d819476e1: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.rect(arg1, arg2, arg3, arg4);
        },
        __wbg_restore_43a0248041b088b5: function(arg0) {
            arg0.restore();
        },
        __wbg_rotate_fc0adc5d1a8fd182: function() { return handleError(function (arg0, arg1) {
            arg0.rotate(arg1);
        }, arguments); },
        __wbg_save_0c65dc2190a45c2a: function(arg0) {
            arg0.save();
        },
        __wbg_scale_1999c309b681811d: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.scale(arg1, arg2);
        }, arguments); },
        __wbg_setLineDash_d915b0269ee28de8: function() { return handleError(function (arg0, arg1) {
            arg0.setLineDash(arg1);
        }, arguments); },
        __wbg_setTransform_af9c1fdc090e1259: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setTransform(arg1, arg2, arg3, arg4, arg5, arg6);
        }, arguments); },
        __wbg_set_fillStyle_392607276a67e12a: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_fillStyle_52e75a25be60a3ff: function(arg0, arg1, arg2) {
            arg0.fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_fillStyle_9215db6210dfdee2: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_filter_c05b047d621641d5: function(arg0, arg1, arg2) {
            arg0.filter = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_63f9cc44d4c6f102: function(arg0, arg1, arg2) {
            arg0.font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_globalAlpha_7990fab00eb6c8f2: function(arg0, arg1) {
            arg0.globalAlpha = arg1;
        },
        __wbg_set_height_d72f2b76484a44de: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_lineCap_ec484c1489fa48bc: function(arg0, arg1, arg2) {
            arg0.lineCap = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_5f9aefcc32e60287: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_shadowBlur_8f0ba721d1bde0ba: function(arg0, arg1) {
            arg0.shadowBlur = arg1;
        },
        __wbg_set_shadowColor_b6af7ba363af9d9a: function(arg0, arg1, arg2) {
            arg0.shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_shadowOffsetX_a4581089b789b241: function(arg0, arg1) {
            arg0.shadowOffsetX = arg1;
        },
        __wbg_set_shadowOffsetY_a18f39815fb95d67: function(arg0, arg1) {
            arg0.shadowOffsetY = arg1;
        },
        __wbg_set_src_8403f74bf6b0fa5f: function(arg0, arg1, arg2) {
            arg0.src = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_strokeStyle_cce50c69cecc2df7: function(arg0, arg1, arg2) {
            arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_9ee229a431a30197: function(arg0, arg1, arg2) {
            arg0.textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_72fc99fa97c6b6f3: function(arg0, arg1, arg2) {
            arg0.textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_width_36ef6630b22fc519: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_THIS_466428f93b4eaa76: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_c7aea38d4de089bc: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_42d4fae05e59267a: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_e0db14a0eba6a812: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_strokeRect_1e1fb12083885dd8: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.strokeRect(arg1, arg2, arg3, arg4);
        },
        __wbg_strokeText_a325642d6ce7368c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.strokeText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_stroke_5f311844f0db0d9a: function(arg0) {
            arg0.stroke();
        },
        __wbg_translate_b7073fdf68217bcc: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.translate(arg1, arg2);
        }, arguments); },
        __wbg_width_1952934caca67137: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_64eb09b40bf1526e: function(arg0) {
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

const DocumentExportFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_documentexport_free(ptr, 1));
const HwpDocumentFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_hwpdocument_free(ptr, 1));
const HwpViewerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_hwpviewer_free(ptr, 1));

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

function getClampedArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ClampedArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
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

let cachedUint8ClampedArrayMemory0 = null;
function getUint8ClampedArrayMemory0() {
    if (cachedUint8ClampedArrayMemory0 === null || cachedUint8ClampedArrayMemory0.byteLength === 0) {
        cachedUint8ClampedArrayMemory0 = new Uint8ClampedArray(wasm.memory.buffer);
    }
    return cachedUint8ClampedArrayMemory0;
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

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    cachedUint8ClampedArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

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
