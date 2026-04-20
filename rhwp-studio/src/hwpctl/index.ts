/**
 * hwpctl 호환 HwpCtrl 클래스
 *
 * 한컴 웹기안기의 HwpCtrl ActiveX/JavaScript 객체와 동일한 인터페이스.
 * 내부적으로 rhwp WASM API를 호출한다.
 */
import { Action } from './action';
import { ParameterSet } from './parameter-set';
import { getActionDef, getRegisteredCount, getImplementedCount, getAllActions } from './action-registry';

// Wave 1~6: Action executor 등록 (import 시 자동 등록)
import './actions/table';
import './actions/text';
import './actions/format';
import './actions/table-edit';
import './actions/navigate';
import './actions/clipboard';
import './actions/page';

export { ParameterSet } from './parameter-set';
export { Action } from './action';

export class HwpCtrl {
  /** rhwp WASM 문서 객체 */
  private wasmDoc: any;
  /** 현재 커서 위치 */
  private cursorSection = 0;
  private cursorPara = 0;
  private cursorPos = 0;
  /** 이벤트 리스너 */
  private listeners: Map<number, Function[]> = new Map();

  constructor(wasmDoc: any) {
    this.wasmDoc = wasmDoc;
  }

  /** 내부: WASM 문서 객체 접근 */
  getWasmDoc(): any {
    return this.wasmDoc;
  }

  /** 내부: 현재 커서 위치 */
  getCursor(): { section: number; para: number; pos: number } {
    return { section: this.cursorSection, para: this.cursorPara, pos: this.cursorPos };
  }

  // ── HwpCtrl API ──

  /** 문서 열기 (Blob/ArrayBuffer) */
  Open(data: ArrayBuffer | Uint8Array, callback?: (success: boolean) => void): boolean {
    try {
      const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : data;
      this.wasmDoc = new (this.wasmDoc.constructor)(bytes);
      callback?.(true);
      return true;
    } catch (e) {
      console.error('[hwpctl] Open 실패:', e);
      callback?.(false);
      return false;
    }
  }

  /** 빈 문서 생성 */
  Clear(): void {
    this.wasmDoc.createBlankDocument();
    this.cursorSection = 0;
    this.cursorPara = 0;
    this.cursorPos = 0;
  }

  /** 원본 파일 형식에 맞게 HWP 또는 HWPX로 내보내기 */
  SaveAs(filename: string, format?: string, arg?: string): boolean {
    try {
      const sourceFormat = this.wasmDoc.getSourceFormat();
      // #196: HWPX 출처는 저장 비활성화 (베타 단계, #197 완전 변환기 완료 시까지)
      if (sourceFormat === 'hwpx' && format !== 'hwp') {
        console.warn('[hwpctl] SaveAs: HWPX 출처 저장은 현재 베타 단계로 비활성화되어 있습니다 (#196)');
        return false;
      }
      const isHwpx = format === 'hwpx' || (!format && sourceFormat === 'hwpx');
      console.log(`[hwpctl] SaveAs: filename=${filename}, sourceFormat=${sourceFormat}, isHwpx=${isHwpx}`);

      let bytes: Uint8Array;
      let mimeType: string;
      let ext: string;

      if (isHwpx) {
        bytes = this.wasmDoc.exportHwpx();
        mimeType = 'application/hwp+zip';
        ext = '.hwpx';
      } else {
        bytes = this.wasmDoc.exportHwp();
        mimeType = 'application/x-hwp';
        ext = '.hwp';
      }

      // 파일명에 확장자가 없으면 원본 형식에 맞게 추가
      if (!filename.endsWith(ext) && !filename.endsWith('.hwp') && !filename.endsWith('.hwpx')) {
        filename += ext;
      }

      console.log(`[hwpctl] SaveAs: ${isHwpx ? 'HWPX' : 'HWP'}, ${bytes.length} bytes, ext=${ext}`);
      const blob = new Blob([bytes as BlobPart], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
      return true;
    } catch (e) {
      console.error('[hwpctl] SaveAs 실패:', e);
      return false;
    }
  }

  /** Action 생성 */
  CreateAction(actionId: string): Action {
    const def = getActionDef(actionId);
    if (!def) {
      console.warn(`[hwpctl] Action "${actionId}" 미등록`);
      return new Action(this, {
        id: actionId, parameterSetId: null,
        description: '미등록', executor: null,
      });
    }
    return new Action(this, def);
  }

  /** ParameterSet 생성 */
  CreateSet(setName: string): ParameterSet {
    return new ParameterSet(setName);
  }

  /** 컨트롤 삽입 (InsertCtrl) */
  InsertCtrl(ctrlName: string, set?: ParameterSet): boolean {
    // ctrlCode → Action 매핑
    const actionMap: Record<string, string> = {
      'tbl': 'TableCreate',
      'secd': 'PageSetup',
      'cold': 'BreakColDef',
    };
    const actionId = actionMap[ctrlName] || ctrlName;
    const action = this.CreateAction(actionId);
    return action.Execute(set || new ParameterSet(actionId));
  }

  /** 텍스트 삽입 */
  InsertText(text: string): boolean {
    try {
      this.wasmDoc.insertText(
        this.cursorSection, this.cursorPara, this.cursorPos, text,
      );
      this.cursorPos += text.length;
      return true;
    } catch (e) {
      console.error('[hwpctl] InsertText 실패:', e);
      return false;
    }
  }

  /** Action 단순 실행 */
  Run(actionId: string): boolean {
    const action = this.CreateAction(actionId);
    return action.Run();
  }

  /** 커서 위치 설정 */
  SetPos(list: number, para: number, pos: number): boolean {
    this.cursorSection = list;
    this.cursorPara = para;
    this.cursorPos = pos;
    return true;
  }

  /** 커서 위치 반환 */
  GetPos(): { list: number; para: number; pos: number } {
    return { list: this.cursorSection, para: this.cursorPara, pos: this.cursorPos };
  }

  /** 페이지 수 */
  PageCount(): number {
    return this.wasmDoc.pageCount();
  }

  // ── 표 셀 텍스트 API ──

  /** 표 셀에 텍스트 설정 (행렬 좌표 기반)
   * @param tableParaIdx 표가 포함된 문단 인덱스
   * @param row 행 (0부터)
   * @param col 열 (0부터)
   * @param text 삽입할 텍스트
   * @param colCount 열 수 (생략 시 cellIdx = row * colCount + col 계산 불가 → cellIdx 직접 사용)
   * @param controlIdx 표 컨트롤 인덱스 (기본 0)
   */
  SetCellText(tableParaIdx: number, row: number, col: number, text: string, colCount: number, controlIdx = 0): boolean {
    try {
      const cellIdx = row * colCount + col;
      const result = this.wasmDoc.insertTextInCell(
        this.cursorSection, tableParaIdx, controlIdx, cellIdx, 0, 0, text,
      );
      const parsed = JSON.parse(result);
      return parsed.ok === true;
    } catch (e) {
      console.error(`[hwpctl] SetCellText(pi=${tableParaIdx}, r=${row}, c=${col}) 실패:`, e);
      return false;
    }
  }

  /** 표 셀 텍스트 조회 (행렬 좌표 기반) */
  GetCellText(tableParaIdx: number, row: number, col: number, colCount: number, controlIdx = 0): string {
    try {
      const cellIdx = row * colCount + col;
      const path = `s${this.cursorSection}:p${tableParaIdx}:c${controlIdx}:cell${cellIdx}:p0`;
      const result = this.wasmDoc.getTextInCellByPath(path);
      return result || '';
    } catch (e) {
      console.error(`[hwpctl] GetCellText(pi=${tableParaIdx}, r=${row}, c=${col}) 실패:`, e);
      return '';
    }
  }

  /** 표 셀에서 계산식 실행 */
  EvaluateFormula(tableParaIdx: number, row: number, col: number, formula: string, writeResult = true, controlIdx = 0): any {
    try {
      const result = this.wasmDoc.evaluateTableFormula(
        this.cursorSection, tableParaIdx, controlIdx, row, col, formula, writeResult,
      );
      return JSON.parse(result);
    } catch (e) {
      console.error(`[hwpctl] EvaluateFormula 실패:`, e);
      return { ok: false, error: String(e) };
    }
  }

  /** 이벤트 리스너 등록 */
  addEventListener(eventType: number, callback: Function): void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, []);
    }
    this.listeners.get(eventType)!.push(callback);
  }

  // ── Field API (누름틀) ──

  /** 필드 목록 조회 */
  GetFieldList(): any[] {
    try {
      const json = this.wasmDoc.getFieldList();
      return JSON.parse(json);
    } catch (e) {
      console.error('[hwpctl] GetFieldList 실패:', e);
      return [];
    }
  }

  /** 필드로 커서 이동 */
  MoveToField(field: string, getText?: boolean, moveStart?: boolean, select?: boolean): boolean {
    try {
      const fields = this.GetFieldList();
      const found = fields.find((f: any) => f.name === field);
      if (!found) {
        console.warn(`[hwpctl] 필드 "${field}" 없음`);
        return false;
      }
      const loc = found.location;
      this.cursorSection = loc.sectionIndex ?? 0;
      this.cursorPara = loc.paraIndex ?? 0;
      this.cursorPos = 0;
      return true;
    } catch (e) {
      console.error('[hwpctl] MoveToField 실패:', e);
      return false;
    }
  }

  /** 필드 텍스트 설정 (한컴 호환: PutFieldText) */
  PutFieldText(field: string, text: string): boolean {
    try {
      const result = this.wasmDoc.setFieldValueByName(field, text);
      const parsed = JSON.parse(result);
      return parsed.ok === true;
    } catch (e) {
      console.error(`[hwpctl] PutFieldText("${field}") 실패:`, e);
      return false;
    }
  }

  /** 필드 텍스트 조회 (한컴 호환: GetFieldText) */
  GetFieldText(field: string): string {
    try {
      const result = this.wasmDoc.getFieldValueByName(field);
      const parsed = JSON.parse(result);
      return parsed.ok ? parsed.value : '';
    } catch (e) {
      console.error(`[hwpctl] GetFieldText("${field}") 실패:`, e);
      return '';
    }
  }

  /** 커서 위치 이동 (한컴 호환: MovePos) */
  MovePos(pos: number): boolean {
    try {
      switch (pos) {
        case 2: // 문서 끝
          const pageCount = this.wasmDoc.pageCount();
          // 마지막 구역, 마지막 문단으로 이동
          this.cursorSection = 0;
          this.cursorPara = 0;
          this.cursorPos = 0;
          break;
        case 3: // 문서 시작
          this.cursorSection = 0;
          this.cursorPara = 0;
          this.cursorPos = 0;
          break;
        default:
          console.warn(`[hwpctl] MovePos(${pos}) 미지원`);
      }
      return true;
    } catch (e) {
      console.error('[hwpctl] MovePos 실패:', e);
      return false;
    }
  }

  /** 현재 필드 이름 설정 */
  SetCurFieldName(name: string): boolean {
    console.info(`[hwpctl] SetCurFieldName("${name}") — stub`);
    return true;
  }

  /** 필드 이름 변경 */
  RenameField(oldName: string, newName: string): boolean {
    console.info(`[hwpctl] RenameField("${oldName}" → "${newName}") — stub`);
    return true;
  }

  // ── 진행률 추적 ──

  /** 등록된 Action 수 */
  static getRegisteredActionCount(): number {
    return getRegisteredCount();
  }

  /** 구현된 Action 수 */
  static getImplementedActionCount(): number {
    return getImplementedCount();
  }

  /** 전체 Action 목록 (디버깅/테스트용) */
  static getAllActions() {
    return getAllActions();
  }
}

/**
 * hwpctl 호환 HwpCtrl 생성 (비동기 초기화)
 *
 * 사용:
 * ```javascript
 * const HwpCtrl = await createHwpCtrl({ wasmUrl: '/pkg/rhwp_bg.wasm' });
 * HwpCtrl.Open(fileBlob);
 * ```
 */
export async function createHwpCtrl(options: {
  wasmUrl?: string;
  wasmModule?: any;
}): Promise<HwpCtrl> {
  let wasmDoc: any;

  if (options.wasmModule) {
    // 이미 로딩된 WASM 모듈 사용
    wasmDoc = options.wasmModule;
  } else {
    // 동적 로딩
    const { default: init, HwpDocument } = await import('@wasm/rhwp.js');
    await init(options.wasmUrl);
    wasmDoc = HwpDocument.createEmpty();
  }

  return new HwpCtrl(wasmDoc);
}
