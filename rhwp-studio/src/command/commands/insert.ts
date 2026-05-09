import type { CommandDef } from '../types';
import { PicturePropsDialog } from '@/ui/picture-props-dialog';
import { EquationEditorDialog } from '@/ui/equation-editor-dialog';
import { SymbolsDialog } from '@/ui/symbols-dialog';
import { BookmarkDialog } from '@/ui/bookmark-dialog';
import { showShapePicker } from '@/ui/shape-picker';
import type { ShapeType } from '@/ui/shape-picker';

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
let symbolsDialog: SymbolsDialog | null = null;
let bookmarkDialog: BookmarkDialog | null = null;

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
        const data = new Uint8Array(await file.arrayBuffer());
        const ext = file.name.split('.').pop()?.toLowerCase() || 'png';
        // Image 엘리먼트로 원본 크기 측정
        const img = new Image();
        img.src = URL.createObjectURL(file);
        await new Promise<void>(r => { img.onload = () => r(); });
        URL.revokeObjectURL(img.src);
        // 마우스 영역 지정 모드 진입
        ih.enterImagePlacementMode(data, ext, img.naturalWidth, img.naturalHeight, file.name);
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
    shortcutLabel: 'Ctrl+N,M',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
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
            equationEditorDialog = new EquationEditorDialog(services.wasm, services.eventBus);
          }
          equationEditorDialog.open(pos.sectionIndex, result.paraIdx, result.controlIdx);
        }
      } catch (err) {
        console.warn('[insert:equation] 수식 삽입 실패:', err);
      }
    },
  },
  stub('insert:field', '필드 입력', undefined, 'Ctrl+K+E'),
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
    canExecute: () => true,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      console.log('[footnote] pos:', pos);
      try {
        const result = services.wasm.insertFootnote(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
        console.log('[footnote] result:', result);
        if (result.ok) {
          services.eventBus.emit('document-changed');
        }
      } catch (err) {
        console.warn('[insert:footnote] 각주 삽입 실패:', err);
      }
    },
  },
  stub('insert:endnote', '미주', 'icon-endnote'),
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
      if (!ref || ref.type === 'equation' || ref.type === 'group') return;
      if (!picturePropsDialog) {
        picturePropsDialog = new PicturePropsDialog(services.wasm, services.eventBus);
      }
      picturePropsDialog.open(ref.sec, ref.ppi, ref.ci, ref.type);
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
        equationEditorDialog = new EquationEditorDialog(services.wasm, services.eventBus);
      }
      equationEditorDialog.open(ref.sec, ref.ppi, ref.ci, ref.cellIdx, ref.cellParaIdx);
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
        props = ref.type === 'image'
          ? services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci)
          : services.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci);
      } catch (e) { return; }
      if (!props) return;
      // 캡션 없으면 추가 (기본: 아래, 크기 30mm, 간격 3mm)
      let charOffset = 0;
      if (!props.hasCaption) {
        const setProps = {
          hasCaption: true,
          captionDirection: 'Bottom',
          captionVertAlign: 'Top',
          captionWidth: Math.round(30 * 283.46),
          captionSpacing: Math.round(3 * 283.46),
          captionIncludeMargin: false,
        };
        let result: any;
        if (ref.type === 'image') {
          result = services.wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, setProps);
        } else {
          result = services.wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, setProps);
        }
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

/** 선택 개체의 속성을 조회/변경 헬퍼 (shape/picture 분기) */
function getProps(services: import('../types').CommandServices, ref: { sec: number; ppi: number; ci: number; type: string }): Record<string, unknown> {
  if (ref.type === 'shape') {
    return services.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
  }
  return services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
}

function setProps(services: import('../types').CommandServices, ref: { sec: number; ppi: number; ci: number; type: string }, props: Record<string, unknown>): void {
  if (ref.type === 'shape') {
    services.wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, props);
  } else {
    services.wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, props);
  }
}

/** 현재 회전각에 delta(도)를 더한다 (Shape만 지원). */
function applyRotationDelta(services: import('../types').CommandServices, delta: number): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const ref = ih.getSelectedPictureRef();
  if (!ref || ref.type !== 'shape') return;
  const props = getProps(services, ref);
  const cur = ((props.rotationAngle as number) ?? 0);
  let next = cur + delta;
  // -180 ~ 180 범위로 정규화
  next = ((next % 360) + 360) % 360;
  if (next > 180) next -= 360;
  setProps(services, ref, { rotationAngle: next });
  services.eventBus.emit('document-changed');
}

/** horzFlip/vertFlip을 토글한다 (Shape만 지원). */
function toggleFlip(services: import('../types').CommandServices, key: 'horzFlip' | 'vertFlip'): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const ref = ih.getSelectedPictureRef();
  if (!ref || ref.type !== 'shape') return;
  const props = getProps(services, ref);
  const cur = !!props[key];
  setProps(services, ref, { [key]: !cur });
  services.eventBus.emit('document-changed');
}
