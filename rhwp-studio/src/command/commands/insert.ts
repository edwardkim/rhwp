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
      try {
        const defaultFontSize = 1000; // 10pt → HWPUNIT
        const defaultColor = 0x00000000; // 검정
        const result = services.wasm.insertEquation(
          pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
          '', defaultFontSize, defaultColor
        );
        if (result.ok) {
          services.eventBus.emit('document-changed');
          if (!equationEditorDialog) {
            equationEditorDialog = new EquationEditorDialog(services.wasm, services.eventBus, services);
          }
          equationEditorDialog.open(pos.sectionIndex, result.paraIdx, result.controlIdx);
        }
      } catch (err) {
        console.warn('[insert:equation] 수식 삽입 실패:', err);
      }
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
          const result = services.wasm.insertClickHereField(
            pos,
            props.guide,
            props.memo,
            props.name,
            props.editable,
          );
          if (result.ok) {
            const insertedPos = { ...pos, charOffset: result.charOffset ?? pos.charOffset };
            ih.moveCursorTo(insertedPos);
            ih.markCurrentFieldEndOutside();
            services.wasm.clearActiveField();
            services.eventBus.emit('document-mutated', 'insert-field');
            services.eventBus.emit('document-changed');
          }
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
      if (!services.getContext().hasDocument) return;
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        const result = services.wasm.insertFootnote(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
        if (result.ok) {
          services.eventBus.emit('document-changed');
          enterNoteEditing(services, ih, pos.sectionIndex, result.paraIdx, result.controlIdx);
        }
      } catch (err) {
        console.warn('[insert:footnote] 각주 삽입 실패:', err);
      }
    },
  },
  {
    id: 'insert:endnote',
    label: '미주',
    icon: 'icon-endnote',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (!services.getContext().hasDocument) return;
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        const result = services.wasm.insertEndnote(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
        if (result.ok) {
          services.eventBus.emit('document-changed');
          enterNoteEditing(services, ih, pos.sectionIndex, result.paraIdx, result.controlIdx);
        }
      } catch (err) {
        console.warn('[insert:endnote] 미주 삽입 실패:', err);
      }
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
      endnoteShapeDialog = new EndnoteShapeDialog(services.wasm, services.eventBus, sectionIdx);
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
      services.wasm.changeShapeZOrder(ref.sec, ref.ppi, ref.ci, 'front');
      ih.exitPictureObjectSelectionAndAfterEdit();
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
      services.wasm.changeShapeZOrder(ref.sec, ref.ppi, ref.ci, 'forward');
      ih.exitPictureObjectSelectionAndAfterEdit();
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
      services.wasm.changeShapeZOrder(ref.sec, ref.ppi, ref.ci, 'backward');
      ih.exitPictureObjectSelectionAndAfterEdit();
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
      services.wasm.changeShapeZOrder(ref.sec, ref.ppi, ref.ci, 'back');
      ih.exitPictureObjectSelectionAndAfterEdit();
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
      if (ref.type === 'shape' || ref.type === 'line' || ref.type === 'group') {
        services.wasm.deleteShapeControl(ref.sec, ref.ppi, ref.ci);
      } else if (ref.type === 'equation') {
        services.wasm.deleteEquationControl(ref.sec, ref.ppi, ref.ci);
      } else if (ref.cellPath && ref.cellPath.length > 0) {
        services.wasm.deleteCellPictureControlByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci);
      } else {
        services.wasm.deletePictureControl(ref.sec, ref.ppi, ref.ci);
      }
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
        const result = services.wasm.groupShapes(sec, targets);
        ih.exitPictureObjectSelectionAndAfterEdit();
        // 생성된 GroupShape를 선택
        ih.selectPictureObject(sec, result.paraIdx, result.controlIdx, 'group');
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
        services.wasm.ungroupShape(ref.sec, ref.ppi, ref.ci);
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
  setProps(services, ref, { rotationAngle: next });
  services.eventBus.emit('document-changed');
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
  setProps(services, ref, { [key]: !cur });
  services.eventBus.emit('document-changed');
}
