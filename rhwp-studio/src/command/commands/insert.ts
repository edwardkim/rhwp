import type { CommandDef } from '../types';
import { PicturePropsDialog } from '@/ui/picture-props-dialog';
import { EquationEditorDialog } from '@/ui/equation-editor-dialog';
import { EquationPropertiesDialog } from '@/ui/equation-props-dialog';
import { SymbolsDialog } from '@/ui/symbols-dialog';
import { BookmarkDialog } from '@/ui/bookmark-dialog';
import { EndnoteShapeDialog } from '@/ui/endnote-shape-dialog';
import { FieldInsertDialog } from '@/ui/field-insert-dialog';
import { showShapePicker } from '@/ui/shape-picker';
import { showToast } from '@/ui/toast';
import type { ShapeType } from '@/ui/shape-picker';
import type { CellPathLike } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { InputHandler } from '@/engine/input-handler';
import type { RefreshPolicy } from '@/engine/command';

/** 스텁 커맨드 생성 헬퍼 */
function stub(id: string, label: string, icon?: string, shortcut?: string): CommandDef {
  return {
    id,
    label,
    icon,
    shortcutLabel: shortcut,
    canExecute: () => false,
    execute() { /* TODO */ },
  };
}

let picturePropsDialog: PicturePropsDialog | null = null;
let equationEditorDialog: EquationEditorDialog | null = null;
let equationPropsDialog: EquationPropertiesDialog | null = null;
let symbolsDialog: SymbolsDialog | null = null;
let bookmarkDialog: BookmarkDialog | null = null;
let endnoteShapeDialog: EndnoteShapeDialog | null = null;
let fieldInsertDialog: FieldInsertDialog | null = null;

function enterNoteEditing(
  services: any,
  ih: any,
  sectionIdx: number,
  paraIdx: number,
  controlIdx: number,
): void {
  const info = services.wasm.getNoteEditInfo(sectionIdx, paraIdx, controlIdx);
  if (!info?.ok) return;
  const cursor = (ih as any).cursor;
  if (!cursor?.enterFootnoteMode) return;
  cursor.enterFootnoteMode(
    sectionIdx,
    paraIdx,
    controlIdx,
    info.footnoteIndex ?? 0,
    info.pageNum ?? 0,
  );
  cursor.setFnCursorPosition(info.fnParaIndex ?? 0, info.charOffset ?? 2);
  services.eventBus.emit('footnoteModeChanged', true);
  (ih as any).active = true;
  (ih as any).updateCaret?.();
  (ih as any).textarea?.focus();
}

/**
 * [Task #3207] 각주/미주 삽입을 snapshot 으로 기록한 뒤 노트 편집 모드로 진입한다.
 *
 * 삽입은 본문에 노트 참조를 넣어 문자 수를 바꾸므로 미기록 시 undo 불가 + 후속 undo
 * 오프셋 오염으로 이어진다. undo 시 노트 모드 이탈은 별도 배선이 필요 없다 —
 * SnapshotCommand 는 editContext() 를 노출하지 않아 restoreEditContextAfterHistory 의
 * 본문 분기를 타고, 그 분기가 노트 모드를 빠져나와 삽입 위치로 커서를 되돌린다.
 */
function insertNote(
  services: Parameters<CommandDef['execute']>[0],
  kind: 'footnote' | 'endnote',
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const pos = ih.getPosition();
  let result: { ok: boolean; paraIdx: number; controlIdx: number } | undefined;
  ih.executeOperation({
    kind: 'snapshot',
    operationType: kind === 'footnote' ? 'insertFootnote' : 'insertEndnote',
    operation: (wasm) => {
      result = kind === 'footnote'
        ? wasm.insertFootnote(pos.sectionIndex, pos.paragraphIndex, pos.charOffset)
        : wasm.insertEndnote(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
      if (!result.ok) throw new Error(`[insert:${kind}] 삽입 실패`);
      return pos;
    },
  });
  // 편집 모드 게이트로 라우터가 작업을 드롭했으면 result 가 없다.
  if (result) enterNoteEditing(services, ih, pos.sectionIndex, result.paraIdx, result.controlIdx);
}

export const insertCommands: CommandDef[] = [
  {
    id: 'insert:shape',
    label: '도형',
    icon: 'icon-shape',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const anchor = document.getElementById('tb-shape');
      if (!anchor) return;
      showShapePicker(anchor, {
        onSelect(type: ShapeType) {
          const ih = services.getInputHandler();
          if (ih) ih.enterShapePlacementMode(type);
        },
      });
    },
  },
  {
    id: 'insert:image',
    label: '그림',
    icon: 'icon-image',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = 'image/png,image/jpeg,image/gif,image/bmp,image/webp';
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        let objectUrl = '';
        try {
          const data = new Uint8Array(await file.arrayBuffer());
          const ext = file.name.split('.').pop()?.toLowerCase() || 'png';
          const img = new Image();
          objectUrl = URL.createObjectURL(file);
          await new Promise<void>((resolve, reject) => {
            img.onload = () => {
              if (img.naturalWidth <= 0 || img.naturalHeight <= 0) {
                reject(new Error('이미지 크기를 확인할 수 없습니다.'));
                return;
              }
              resolve();
            };
            img.onerror = () => reject(new Error('브라우저가 이 이미지 파일을 읽지 못했습니다.'));
            img.src = objectUrl;
          });
          ih.enterImagePlacementMode(data, ext, img.naturalWidth, img.naturalHeight, file.name);
          showToast({
            message: '그림을 넣을 위치를 문서 본문 또는 표 셀 안에서 클릭하거나 드래그하세요.',
            durationMs: 3500,
          });
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          console.warn('[insert:image] 이미지 준비 실패:', err);
          showToast({
            message: `그림을 삽입할 수 없습니다.\n${msg}`,
            durationMs: 6000,
          });
        } finally {
          if (objectUrl) URL.revokeObjectURL(objectUrl);
        }
      };
      input.click();
    },
  },
  {
    id: 'insert:textbox',
    label: '글상자',
    icon: 'icon-textbox',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      ih.enterTextboxPlacementMode();
    },
  },
  {
    id: 'insert:equation',
    label: '수식',
    shortcutLabel: 'Ctrl+M,M',
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      // 본문 전용 — 표 셀 내부에서는 실행하지 않음
      if ((pos as any).cellIndex !== undefined && (pos as any).cellIndex >= 0) return;
      const defaultFontSize = 1000; // 10pt → HWPUNIT
      const defaultColor = 0x00000000; // 검정
      // [Task #3207] 수식 삽입도 본문 문자 수를 바꾸므로 snapshot 으로 기록한다(각주/미주와 동형).
      let result: { ok: boolean; paraIdx: number; controlIdx: number } | undefined;
      ih.executeOperation({
        kind: 'snapshot',
        operationType: 'insertEquation',
        operation: (wasm) => {
          result = wasm.insertEquation(
            pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
            '', defaultFontSize, defaultColor,
          );
          if (!result.ok) throw new Error('[insert:equation] 삽입 실패');
          return pos;
        },
      });
      if (!result) return;
      equationEditorDialog ??= new EquationEditorDialog(services.wasm, services.eventBus, services);
      equationEditorDialog.open(pos.sectionIndex, result.paraIdx, result.controlIdx);
    },
  },
  {
    id: 'insert:field',
    label: '필드 입력',
    shortcutLabel: 'Ctrl+K+E',
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      fieldInsertDialog = new FieldInsertDialog();
      fieldInsertDialog.onApply = (props) => {
        try {
          // [Task #2377] 누름틀 삽입은 안내문 텍스트를 문서에 넣는다(문자 수 변경) —
          // 미기록 시 undo 불가 + 후속 undo 오프셋 오염. snapshot 으로 라우팅한다(이 커맨드는
          // 일반 모드 전용이라 게이트 드롭 없음). 실패 시 throw 로 엔트리 생성을 막는다.
          ih.executeOperation({
            kind: 'snapshot',
            operationType: 'insertField',
            operation: (wasm) => {
              const result = wasm.insertClickHereField(pos, props.guide, props.memo, props.name, props.editable);
              if (!result.ok) throw new Error('insertClickHereField not ok');
              return { ...pos, charOffset: result.charOffset ?? pos.charOffset };
            },
          });
          // 커서는 라우터가 삽입 위치로 이동시킨다 — 필드 끝 밖 마킹·활성 필드 해제는 기존대로.
          ih.markCurrentFieldEndOutside();
          services.wasm.clearActiveField();
          // [Task #2370] 수동 emit 제거 — 스냅샷 라우팅의 'full' refresh 가 afterEdit() 를
          // 부르고 거기서 이미 'document-mutated'/'document-changed' 를 emit 한다.
          // 구독자(markDirty·autosave)는 reason 을 라벨로만 쓰므로 중복 emit 은 순손해다.
          // 모달 확인 버튼으로 옮겨간 포커스를 편집기로 복원 — 종전엔 moveCursorTo 끝의
          // focusTextarea 가 담당했으나 라우터 경로엔 없다(field:edit 의 onClose 복원과 동형).
          ih.focus();
        } catch (err) {
          console.warn('[insert:field] 누름틀 삽입 실패:', err);
        }
      };
      fieldInsertDialog.show();
    },
  },
  stub('insert:caption-top', '캡션 - 위'),
  stub('insert:caption-lt', '캡션 - 왼쪽 위'),
  stub('insert:caption-lm', '캡션 - 왼쪽 가운데'),
  stub('insert:caption-lb', '캡션 - 왼쪽 아래'),
  stub('insert:caption-rt', '캡션 - 오른쪽 위'),
  stub('insert:caption-rm', '캡션 - 오른쪽 가운데'),
  stub('insert:caption-rb', '캡션 - 오른쪽 아래'),
  stub('insert:caption-bottom', '캡션 - 아래'),
  stub('insert:caption-none', '캡션 없음'),
  stub('insert:para-band', '문단 띠'),
  stub('insert:comment', '주석', 'icon-comment'),
  {
    id: 'insert:footnote',
    label: '각주',
    icon: 'icon-footnote',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      insertNote(services, 'footnote');
    },
  },
  {
    id: 'insert:endnote',
    label: '미주',
    icon: 'icon-endnote',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      insertNote(services, 'endnote');
    },
  },
  {
    id: 'insert:note-close',
    label: '닫기',
    icon: 'icon-delete',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor?.isInFootnote?.()) return;
      cursor.exitFootnoteMode();
      services.eventBus.emit('footnoteModeChanged', false);
      (ih as any).updateCaret?.();
      (ih as any).textarea?.focus();
    },
  },
  {
    id: 'insert:endnote-shape',
    label: '미주 모양',
    icon: 'icon-endnote',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const pos = services.getInputHandler()?.getPosition();
      const sectionIdx = pos?.sectionIndex ?? 0;
      endnoteShapeDialog = new EndnoteShapeDialog(services.wasm, services.eventBus, sectionIdx, services);
      endnoteShapeDialog.show();
    },
  },
  {
    id: 'insert:symbols',
    label: '문자표',
    icon: 'icon-symbols',
    shortcutLabel: 'Alt+F10',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (!symbolsDialog) {
        symbolsDialog = new SymbolsDialog(services);
      }
      symbolsDialog.show();
    },
  },
  stub('insert:hyperlink', '하이퍼링크', 'icon-hyperlink', 'Ctrl+K+H'),
  {
    id: 'insert:bookmark',
    label: '책갈피',
    shortcutLabel: 'Ctrl+K,B',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (!bookmarkDialog) {
        bookmarkDialog = new BookmarkDialog(services);
      }
      bookmarkDialog.show();
    },
  },
  {
    id: 'insert:picture-props',
    label: '개체 속성',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref) return;
      if (ref.type === 'equation') {
        if (!equationPropsDialog) {
          equationPropsDialog = new EquationPropertiesDialog(services.wasm, services.eventBus, services);
        }
        equationPropsDialog.open(ref.sec, ref.ppi, ref.ci, ref.cellIdx, ref.cellParaIdx, ref.noteRef);
        return;
      }
      if (!picturePropsDialog) {
        picturePropsDialog = new PicturePropsDialog(services.wasm, services.eventBus, services);
      }
      // [Task #825] 머리말/꼬리말 그림은 ref.headerFooter 동반 — dialog 에 전달.
      // [Task #1138] 표 셀 내 도형(shape/line) 은 cellPath 구성하여 dialog 에 전달
      // → by_path API 사용.
      // [Task #1151 v4] picture (image) 도 셀 안 inline picture (tac-img-02.hwp 같은
      // 케이스) 의 경우 cellPath 구성 필요 — getCellPicturePropertiesByPath /
      // setCellPicturePropertiesByPath wasm API 호출. cell context (cellIdx/cellParaIdx/
      // outerTableControlIdx) 가 모두 있으면 셀 안 picture.
      const cellPath: CellPathLike | undefined = ref.cellPath ?? (
        (
          ref.cellIdx !== undefined &&
          ref.cellParaIdx !== undefined &&
          (ref as any).outerTableControlIdx !== undefined &&
          (ref.type === 'shape' || ref.type === 'line' || ref.type === 'image' || ref.type === 'ole')
        )
          ? [{
              controlIdx: (ref as any).outerTableControlIdx as number,
              cellIdx: ref.cellIdx,
              cellParaIdx: ref.cellParaIdx,
            }]
          : undefined
      );
      picturePropsDialog.open(
        ref.sec, ref.ppi, ref.ci, ref.type, ref.headerFooter,
        cellPath, cellPath ? ref.ci : undefined,
      );
    },
  },
  {
    id: 'insert:equation-edit',
    label: '수식 편집',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'equation') return;
      if (!equationEditorDialog) {
        equationEditorDialog = new EquationEditorDialog(services.wasm, services.eventBus, services);
      }
      equationEditorDialog.open(ref.sec, ref.ppi, ref.ci, ref.cellIdx, ref.cellParaIdx, ref.noteRef);
    },
  },
  {
    id: 'insert:caption-toggle',
    label: '캡션 넣기',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type === 'equation' || ref.type === 'group') return;
      // 현재 캡션 상태 조회
      let props: any;
      try {
        props = getProps(services, ref);
      } catch (e) { return; }
      if (!props) return;
      // 캡션 없으면 추가 (기본: 아래, 크기 30mm, 간격 3mm)
      let charOffset = 0;
      if (!props.hasCaption) {
        const captionProps = {
          hasCaption: true,
          captionDirection: 'Bottom',
          captionVertAlign: 'Top',
          captionWidth: Math.round(30 * 283.46),
          captionSpacing: Math.round(3 * 283.46),
          captionIncludeMargin: false,
        };
        let result: any;
        result = setProps(services, ref, captionProps);
        // "그림 N " 끝 위치를 Rust가 반환
        charOffset = result?.captionCharOffset ?? 4;
        services.eventBus.emit('document-changed');
      } else {
        // 이미 캡션이 있으면 캡션 텍스트 끝에 캐럿
        try {
          const len = services.wasm.getCellParagraphLength(ref.sec, ref.ppi, ref.ci, 0, 0);
          charOffset = len;
        } catch { charOffset = 0; }
      }
      // 캡션 텍스트 편집 모드 진입
      ih.exitPictureObjectSelectionAndAfterEdit();
      ih.enterInlineEditing(ref.sec, ref.ppi, ref.ci, charOffset);
    },
  },
  {
    id: 'insert:arrange-front',
    label: '맨 앞으로',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'shape') return;
      changeZOrder(services, ih, ref, 'front');
    },
  },
  {
    id: 'insert:arrange-forward',
    label: '앞으로',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'shape') return;
      changeZOrder(services, ih, ref, 'forward');
    },
  },
  {
    id: 'insert:arrange-backward',
    label: '뒤로',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'shape') return;
      changeZOrder(services, ih, ref, 'backward');
    },
  },
  {
    id: 'insert:arrange-back',
    label: '맨 뒤로',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'shape') return;
      changeZOrder(services, ih, ref, 'back');
    },
  },
  {
    id: 'insert:picture-delete',
    label: '개체 지우기',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref) return;
      recordObjectMutation(ih, 'deleteObject', (wasm) => {
        if (ref.type === 'shape' || ref.type === 'line' || ref.type === 'group') {
          wasm.deleteShapeControl(ref.sec, ref.ppi, ref.ci);
        } else if (ref.type === 'equation') {
          wasm.deleteEquationControl(ref.sec, ref.ppi, ref.ci);
        } else if (ref.cellPath && ref.cellPath.length > 0) {
          wasm.deleteCellPictureControlByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci);
        } else {
          wasm.deletePictureControl(ref.sec, ref.ppi, ref.ci);
        }
      }, DEFER_REFRESH_TO_EXIT);
      ih.exitPictureObjectSelectionAndAfterEdit();
    },
  },
  // ─── 개체 묶기/풀기 ──────────────────────────────
  {
    id: 'insert:group-shapes',
    label: '개체 묶기',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const refs = ih.getSelectedPictureRefs();
      if (refs.length < 2) return;
      const sec = refs[0].sec;
      const targets = refs.map(r => ({ paraIdx: r.ppi, controlIdx: r.ci }));
      try {
        let result: ReturnType<typeof services.wasm.groupShapes> | undefined;
        recordObjectMutation(ih, 'groupShapes', (wasm) => { result = wasm.groupShapes(sec, targets); }, DEFER_REFRESH_TO_EXIT);
        ih.exitPictureObjectSelectionAndAfterEdit();
        // 생성된 GroupShape를 선택
        if (result) ih.selectPictureObject(sec, result.paraIdx, result.controlIdx, 'group');
      } catch (err) {
        console.warn('[group-shapes] 개체 묶기 실패:', err);
      }
    },
  },
  {
    id: 'insert:ungroup-shapes',
    label: '개체 풀기',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref || ref.type !== 'group') return;
      try {
        recordObjectMutation(ih, 'ungroupShape', (wasm) => { wasm.ungroupShape(ref.sec, ref.ppi, ref.ci); }, DEFER_REFRESH_TO_EXIT);
        ih.exitPictureObjectSelectionAndAfterEdit();
      } catch (err) {
        console.warn('[ungroup-shapes] 개체 풀기 실패:', err);
      }
    },
  },
  // ─── 회전/대칭 ──────────────────────────────────
  {
    id: 'insert:rotate-cw',
    label: '오른쪽 90° 회전',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      applyRotationDelta(services, 90);
    },
  },
  {
    id: 'insert:rotate-ccw',
    label: '왼쪽 90° 회전',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      applyRotationDelta(services, -90);
    },
  },
  {
    id: 'insert:flip-horz',
    label: '좌우 대칭',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      toggleFlip(services, 'horzFlip');
    },
  },
  {
    id: 'insert:flip-vert',
    label: '상하 대칭',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      toggleFlip(services, 'vertFlip');
    },
  },
];

/** 선택 개체 ref 타입 — cursor.selectedPictureRef 와 정합 (headerFooter optional, [Task #831]) */
type PictureRef = {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellPath?: CellPathLike;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
};

/** 선택 개체의 속성을 조회/변경 헬퍼 (shape/picture 분기) */
function getProps(services: import('../types').CommandServices, ref: PictureRef): Record<string, unknown> {
  if (ref.type === 'shape') {
    if (ref.cellPath && ref.cellPath.length > 0) {
      return services.wasm.getCellShapePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci) as unknown as Record<string, unknown>;
    }
    return services.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
  }
  // [Task #831] 머리말/꼬리말 picture 의 경우 별도 API 호출 (PR #832 의 wasm-bridge).
  // 미적용 시 본문 lookup 실패 → props 빈/stale → 회전/대칭 무동작.
  if (ref.headerFooter) {
    return services.wasm.getHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
    ) as unknown as Record<string, unknown>;
  }
  if (ref.cellPath && ref.cellPath.length > 0) {
    return services.wasm.getCellPicturePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci) as unknown as Record<string, unknown>;
  }
  return services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
}

/**
 * [계급 1 이관] 개체 조작 뮤테이션을 snapshot 으로 기록해 undo/redo 를 보장한다.
 * 메뉴/도구상자 커맨드가 `services.wasm.*` 를 직접 호출하면 히스토리를 우회한다(같은
 * 삭제라도 Delete 키 경로는 이미 `executeOperation({kind:'snapshot'})` 로 기록됨,
 * input-handler-keyboard.ts). 그 경로와 동형으로 위임한다 — 뮤테이션만 기록하고 선택
 * 해제·afterEdit·재선택 등 UI 후처리는 호출부가 기존대로 수행한다.
 */
function recordObjectMutation(
  ih: InputHandler,
  operationType: string,
  mutate: (wasm: WasmBridge) => boolean | void,
  opts?: { refresh?: RefreshPolicy },
): void {
  const pos = ih.getCursorPosition();
  ih.executeOperation({
    kind: 'snapshot',
    operationType,
    // [Task #2370] mutate 가 명시적으로 false 를 반환하면 문서 무변경 → 기록 취소.
    // void 반환(대부분의 호출부)은 종전대로 항상 기록한다.
    operation: (wasm) => (mutate(wasm) === false ? null : pos),
    meta: opts?.refresh ? { refresh: opts.refresh } : undefined,
  });
}

/**
 * [Task #2370 클러스터 B] 뒤에 `exitPictureObjectSelectionAndAfterEdit()` 가 따라오는
 * 호출부용 옵션. 스냅샷 라우팅의 기본 'full' refresh 가 `afterEdit()` 를 부르고 곧바로
 * 선택 해제가 또 `afterEdit()` 를 불러 중복 repaint 가 된다. 스냅샷 쪽 리프레시를 끄면
 * 최종 상태(선택 해제까지 끝난) 기준 한 번만 그린다.
 */
const DEFER_REFRESH_TO_EXIT = { refresh: 'none' } as const;

/**
 * [Task #2370 클러스터 A] z순서 변경 — 이미 맨 앞/뒤라 바뀔 것이 없으면 기록하지 않는다.
 *
 * `change_shape_z_order_native`(shape.rs)는 경계 케이스에서 문서를 건드리지 않고
 * `{ok:true, zOrder:<현재값>}` 을 그대로 돌려준다("이미 맨 앞/뒤" → `changes = None`).
 * 반대로 실제로 바뀌는 경우 새 z 는 항상 이전과 다르다(front=max+1 · back=min-1 ·
 * forward/backward=이웃 z 또는 ±1). 따라서 **반환 zOrder 와 호출 전 zOrder 의 일치가
 * 곧 무변경 신호**다 — 이를 no-op 으로 보고해 phantom undo 엔트리와 스냅샷 2슬롯 점유를
 * 막는다.
 */
function changeZOrder(
  services: import('../types').CommandServices,
  ih: InputHandler,
  ref: PictureRef,
  operation: 'front' | 'forward' | 'backward' | 'back',
): void {
  const zBefore = (getProps(services, ref) as { zOrder?: number }).zOrder;
  recordObjectMutation(ih, 'changeZOrder', (wasm) => {
    const r = wasm.changeShapeZOrder(ref.sec, ref.ppi, ref.ci, operation);
    return r.ok && r.zOrder !== zBefore;
  }, DEFER_REFRESH_TO_EXIT);
  ih.exitPictureObjectSelectionAndAfterEdit();
}

function setProps(services: import('../types').CommandServices, ref: PictureRef, props: Record<string, unknown>): any {
  if (ref.type === 'shape') {
    if (ref.cellPath && ref.cellPath.length > 0) {
      return services.wasm.setCellShapePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci, props);
    }
    return services.wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, props);
  } else if (ref.headerFooter) {
    // [Task #831] 머리말/꼬리말 picture setter — 5-tuple lookup 으로 IR 갱신.
    return services.wasm.setHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
      props,
    );
  } else {
    if (ref.cellPath && ref.cellPath.length > 0) {
      return services.wasm.setCellPicturePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci, props);
    }
    return services.wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, props);
  }
}

/** 현재 회전각에 delta(도)를 더한다 (shape + image 지원). */
function applyRotationDelta(services: import('../types').CommandServices, delta: number): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const ref = ih.getSelectedPictureRef();
  if (!ref || ref.type === 'equation' || ref.type === 'group' || ref.type === 'line') return;
  const props = getProps(services, ref);
  if (props.sizeProtect) return;
  const cur = ((props.rotationAngle as number) ?? 0);
  let next = cur + delta;
  // -180 ~ 180 범위로 정규화
  next = ((next % 360) + 360) % 360;
  if (next > 180) next -= 360;
  // recordObjectMutation → executeOperation snapshot 의 'full' refresh 가 afterEdit()→
  // 'document-changed' 를 이미 emit 한다. 수동 emit 은 중복(이중 렌더)이라 제거. [undo P3 정리]
  recordObjectMutation(ih, 'rotateObject', () => setProps(services, ref, { rotationAngle: next }));
}

/** horzFlip/vertFlip을 토글한다 (shape + image 지원). */
function toggleFlip(services: import('../types').CommandServices, key: 'horzFlip' | 'vertFlip'): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const ref = ih.getSelectedPictureRef();
  if (!ref || ref.type === 'equation' || ref.type === 'group' || ref.type === 'line') return;
  const props = getProps(services, ref);
  if (props.sizeProtect) return;
  const cur = !!props[key];
  // 위 rotate 와 동일 — snapshot 라우팅이 이미 refresh 하므로 수동 emit 제거. [undo P3 정리]
  recordObjectMutation(ih, 'flipObject', () => setProps(services, ref, { [key]: !cur }));
}
