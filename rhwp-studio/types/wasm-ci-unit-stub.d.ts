/**
 * Stage 3 unit-lane boundary.
 *
 * Studio unit CI type-checks application code without paying for a fresh Rust/WASM build.
 * The package lane still runs wasm-pack first and type-checks against pkg/rhwp.d.ts, so
 * generated binding compatibility remains a package-level contract.
 * Only src/core/wasm-bridge.ts and src/hwpctl/index.ts import this module directly; the
 * impact classifier routes both directories to the package lane with real bindings.
 */
declare module '@wasm/rhwp.js' {
  export default function init(input?: unknown): Promise<unknown>;

  export class HwpDocument {
    constructor(data: Uint8Array);
    static createEmpty(): HwpDocument;
    static openWithPassword(data: Uint8Array, password: string): HwpDocument;
    // 인덱스 시그니처는 이름 있는 프로퍼티를 대신하지 못한다 — 구조적 할당 검사에서
    // `exportHwp` 를 요구하는 쪽(document-agent 의 ExportableDocument)에 닿지 않는다.
    // 생성된 `pkg/rhwp.d.ts` 와 같은 시그니처로 명시한다.
    exportHwp(): Uint8Array;
    exportHwpx(): Uint8Array;
    // AI 도구 표면(src/ai/tools 의 DocumentReader·DocumentWriter)에 구조적으로 할당되도록
    // 그 인터페이스가 쓰는 메서드도 생성된 `pkg/rhwp.d.ts` 의 시그니처 그대로 명시한다.
    addBookmark(sec: number, para: number, char_offset: number, name: string): string;
    applyCharFormat(sec_idx: number, para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    applyCharFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, start_offset: number, end_offset: number, props_json: string): string;
    applyParaFormat(sec_idx: number, para_idx: number, props_json: string): string;
    applyParaFormatInCell(sec_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, props_json: string): string;
    applyStyle(sec_idx: number, para_idx: number, style_id: number): string;
    createHeaderFooter(section_idx: number, is_header: boolean, apply_to: number): string;
    createShapeControl(json: string): string;
    createTable(section_idx: number, para_idx: number, char_offset: number, row_count: number, col_count: number): string;
    deleteParagraph(section_idx: number, para_idx: number): string;
    deleteTableColumn(section_idx: number, parent_para_idx: number, control_idx: number, col_idx: number): string;
    deleteTableRow(section_idx: number, parent_para_idx: number, control_idx: number, row_idx: number): string;
    deleteText(section_idx: number, para_idx: number, char_offset: number, count: number): string;
    deleteTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    deleteTextInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, count: number): string;
    ensureDefaultBullet(bullet_char_str: string): number;
    ensureDefaultNumbering(): number;
    findOrCreateFontId(name: string): number;
    getBookmarks(): string;
    getCellInfo(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number): string;
    getCellParagraphLength(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number): number;
    getChartDataByIndex(index: number): string;
    getControlTextPositions(section_idx: number, para_idx: number): string;
    getDocumentInfo(): string;
    getFieldList(): string;
    getHeaderFooter(section_idx: number, is_header: boolean, apply_to: number): string;
    getLogicalLength(section_idx: number, para_idx: number): number;
    getOutlineNavigation(): string;
    getPageBorderFill(section_idx: number): string;
    getPageDef(section_idx: number): string;
    getPageTextLayout(page_num: number): string;
    getParaPropertiesAt(sec_idx: number, para_idx: number): string;
    getParagraphCount(section_idx: number): number;
    getParagraphLength(section_idx: number, para_idx: number): number;
    getSectionCount(): number;
    getStyleList(): string;
    getTableDimensions(section_idx: number, parent_para_idx: number, control_idx: number): string;
    getTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, count: number): string;
    getTextRange(section_idx: number, para_idx: number, char_offset: number, count: number): string;
    insertClickHereField(section_idx: number, para_idx: number, char_offset: number, guide: string, memo: string, name: string, editable: boolean): string;
    insertEquation(section_idx: number, para_idx: number, char_offset: number, script: string, font_size: number, color: number): string;
    insertFootnote(section_idx: number, para_idx: number, char_offset: number): string;
    insertHyperlinkEx(options_json: string): number;
    insertNewNumber(section_idx: number, para_idx: number, char_offset: number, start_num: number): string;
    insertPicture(section_idx: number, para_idx: number, char_offset: number, cell_path_json: string, image_data: Uint8Array, width: number, height: number, natural_width_px: number, natural_height_px: number, extension: string, description: string, paper_offset_x_hu?: number | null, paper_offset_y_hu?: number | null): string;
    insertTableColumn(section_idx: number, parent_para_idx: number, control_idx: number, col_idx: number, right: boolean): string;
    insertTableRow(section_idx: number, parent_para_idx: number, control_idx: number, row_idx: number, below: boolean): string;
    insertText(section_idx: number, para_idx: number, char_offset: number, text: string): string;
    insertTextInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, text: string): string;
    insertTextInFootnote(section_idx: number, para_idx: number, control_idx: number, fn_para_idx: number, char_offset: number, text: string): string;
    insertTextInHeaderFooter(section_idx: number, is_header: boolean, apply_to: number, hf_para_idx: number, char_offset: number, text: string): string;
    mergeTableCells(section_idx: number, parent_para_idx: number, control_idx: number, start_row: number, start_col: number, end_row: number, end_col: number): string;
    replaceAll(query: string, new_text: string, case_sensitive: boolean): string;
    replaceText(sec: number, para: number, char_offset: number, length: number, new_text: string): string;
    searchAllText(query: string, case_sensitive: boolean, include_cells: boolean): string;
    setCellProperties(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, json: string): string;
    setChartDataByIndex(index: number, edits_json: string): string;
    setColumnDef(section_idx: number, column_count: number, column_type: number, same_width: number, spacing_hu: number): string;
    setFieldValue(field_id: number, value: string): string;
    setPageBorderFill(section_idx: number, json: string): string;
    setPageDef(section_idx: number, json: string): string;
    setPictureProperties(section_idx: number, parent_para_idx: number, control_idx: number, props_json: string): string;
    splitParagraph(section_idx: number, para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    splitParagraphInCell(section_idx: number, parent_para_idx: number, control_idx: number, cell_idx: number, cell_para_idx: number, char_offset: number, removed_para_meta?: string | null): string;
    [name: string]: any;
  }

  export function version(): string;
  export const __wasm: Record<string, unknown>;
}
