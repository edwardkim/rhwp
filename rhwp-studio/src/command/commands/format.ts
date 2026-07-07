import type { CommandDef } from '../types';
import { CharShapeDialog } from '@/ui/char-shape-dialog';
import { ParaShapeDialog } from '@/ui/para-shape-dialog';
import { NumberingDialog } from '@/ui/numbering-dialog';
import { StyleDialog } from '@/ui/style-dialog';
import { StyleEditDialog } from '@/ui/style-edit-dialog';
import { PicturePropsDialog } from '@/ui/picture-props-dialog';
import { EquationPropertiesDialog } from '@/ui/equation-props-dialog';
import { TableCellPropsDialog } from '@/ui/table-cell-props-dialog';

export const formatCommands: CommandDef[] = [
  {
    id: 'format:bold',
    label: '굵게',
    shortcutLabel: 'Ctrl+B',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('bold');
    },
  },
  {
    id: 'format:italic',
    label: '기울임',
    shortcutLabel: 'Ctrl+I',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('italic');
    },
  },
  {
    id: 'format:underline',
    label: '밑줄',
    shortcutLabel: 'Ctrl+U',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('underline');
    },
  },
  {
    id: 'format:strikethrough',
    label: '취소선',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('strikethrough');
    },
  },
  // 양각/음각/외곽선/위첨자/아래첨자
  {
    id: 'format:emboss',
    label: '양각',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('emboss');
    },
  },
  {
    id: 'format:engrave',
    label: '음각',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('engrave');
    },
  },
  {
    id: 'format:outline',
    label: '외곽선',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('outline');
    },
  },
  {
    id: 'format:superscript',
    label: '위 첨자',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('superscript');
    },
  },
  {
    id: 'format:subscript',
    label: '아래 첨자',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleFormat('subscript');
    },
  },
  // 줄 간격
  {
    id: 'format:line-spacing',
    label: '줄 간격',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services, params) {
      const value = params?.value as number | undefined;
      if (value === undefined) return;
      services.getInputHandler()?.setLineSpacing(value);
    },
  },
  // 줄 간격 줄이기 (Alt+Shift+A)
  {
    id: 'format:line-spacing-decrease',
    label: '줄 간격 줄이기',
    shortcutLabel: 'Alt+Shift+A',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const props = ih.getParaProperties();
      const current = props?.lineSpacing ?? 160;
      const newValue = current - 10;
      ih.setLineSpacing(newValue);
    },
  },
  // 줄 간격 늘리기 (Alt+Shift+Z)
  {
    id: 'format:line-spacing-increase',
    label: '줄 간격 늘리기',
    shortcutLabel: 'Alt+Shift+Z',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const props = ih.getParaProperties();
      const current = props?.lineSpacing ?? 160;
      const newValue = Math.min(500, current + 10);
      ih.setLineSpacing(newValue);
    },
  },
  // 글꼴 크기 크게 (Alt+Shift+E)
  {
    id: 'format:font-size-increase',
    label: '글꼴 크기 크게',
    shortcutLabel: 'Alt+Shift+E',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustFontSize(100); // +1pt
    },
  },
  // 글꼴 크기 작게 (Alt+Shift+R)
  {
    id: 'format:font-size-decrease',
    label: '글꼴 크기 작게',
    shortcutLabel: 'Alt+Shift+R',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustFontSize(-100); // -1pt
    },
  },
  // 장평 줄이기 (Shift+Alt+J)
  {
    id: 'format:char-ratio-decrease',
    label: '장평 줄이기',
    shortcutLabel: 'Shift+Alt+J',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustCharRatio(-1);
    },
  },
  // 장평 늘리기 (Shift+Alt+K)
  {
    id: 'format:char-ratio-increase',
    label: '장평 늘리기',
    shortcutLabel: 'Shift+Alt+K',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustCharRatio(1);
    },
  },
  // 자간 줄이기 (Shift+Alt+N)
  {
    id: 'format:char-spacing-decrease',
    label: '자간 줄이기',
    shortcutLabel: 'Shift+Alt+N',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustCharSpacing(-1);
    },
  },
  // 자간 늘리기 (Shift+Alt+W)
  {
    id: 'format:char-spacing-increase',
    label: '자간 늘리기',
    shortcutLabel: 'Shift+Alt+W',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.adjustCharSpacing(1);
    },
  },
  // 문단 정렬
  {
    id: 'format:align-left',
    label: '왼쪽 정렬',
    shortcutLabel: 'Ctrl+Shift+L',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('left');
    },
  },
  {
    id: 'format:align-center',
    label: '가운데 정렬',
    shortcutLabel: 'Alt+Shift+C',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('center');
    },
  },
  {
    id: 'format:align-right',
    label: '오른쪽 정렬',
    shortcutLabel: 'Alt+Shift+H',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('right');
    },
  },
  {
    id: 'format:align-justify',
    label: '양쪽 정렬',
    shortcutLabel: 'Ctrl+Shift+M',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('justify');
    },
  },
  {
    id: 'format:align-distribute',
    label: '배분 정렬',
    shortcutLabel: 'Alt+Shift+D',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('distribute');
    },
  },
  {
    id: 'format:align-split',
    label: '나눔 정렬',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.applyParaAlign('split');
    },
  },
  // 글자 모양 대화상자
  {
    id: 'format:char-shape',
    label: '글자 모양',
    icon: 'icon-char-shape',
    shortcutLabel: 'Alt+L',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const charProps = ih.getCharProperties();
      // 대화상자 열기 전 선택 범위를 저장 (대화상자 조작 중 선택이 풀릴 수 있음)
      const savedSel = ih.getSelection();
      if (!savedSel) return;
      const dialog = new CharShapeDialog(services.wasm, services.eventBus);
      dialog.onApply = (mods) => {
        // fontName → fontId 변환 (WASM parse_char_shape_mods는 fontId만 인식)
        if (mods.fontName) {
          const fontId = services.wasm.findOrCreateFontId(mods.fontName);
          if (fontId >= 0) mods.fontId = fontId;
          delete mods.fontName;
        }
        ih.applyCharPropsToRange(savedSel.start, savedSel.end, mods);
      };
      dialog.onClose = () => ih.focus();
      dialog.show(charProps);
    },
  },
  {
    id: 'format:para-shape',
    label: '문단 모양',
    icon: 'icon-para-shape',
    shortcutLabel: 'Alt+T',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const paraProps = ih.getParaProperties();
      const sel = ih.getSelection();
      const curPos = ih.getCursorPosition();
      const range = sel ?? { start: curPos, end: curPos };
      const dialog = new ParaShapeDialog(services.wasm, services.eventBus);
      dialog.onApply = (mods) => {
        ih.applyParaPropsToRange(range.start, range.end, mods);
      };
      dialog.onClose = () => ih.focus();
      dialog.show(paraProps);
    },
  },
  {
    id: 'format:apply-style',
    label: '스타일 적용',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services, params) {
      const styleId = params?.styleId as number | undefined;
      if (styleId === undefined) return;
      services.getInputHandler()?.applyStyle(styleId);
    },
  },
  {
    id: 'format:toggle-numbering',
    label: '문단 번호 토글',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.toggleNumbering();
    },
  },
  {
    id: 'format:toggle-bullet',
    label: '글머리표 토글',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services, params) {
      const bulletChar = params?.bulletChar as string | undefined;
      services.getInputHandler()?.toggleBullet(bulletChar);
    },
  },
  {
    id: 'format:apply-bullet',
    label: '글머리표 적용',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services, params) {
      const bulletChar = params?.bulletChar as string | undefined;
      if (!bulletChar) return;
      services.getInputHandler()?.applyBullet(bulletChar);
    },
  },
  {
    id: 'format:para-num-shape',
    label: '문단 번호 모양',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      // 현재 문단의 번호 정보 전달
      const props = ih.getParaProperties();
      const dialog = new NumberingDialog(services.wasm, services.eventBus);
      dialog.currentHeadType = props.headType ?? 'None';
      dialog.currentNumberingId = props.numberingId ?? 0;
      dialog.currentRestartMode = (props as any).numberingRestartMode ?? 0;
      // Bullet일 때 현재 bullet 문자 전달
      if (props.headType === 'Bullet' && props.numberingId && props.numberingId > 0) {
        try {
          const bullets = services.wasm.getBulletList();
          const b = bullets.find((item: any) => item.id === props.numberingId);
          if (b) {
            // rawCode(PUA 원본)로 프리셋 매칭, 없으면 mapped char 사용
            const rawChar = b.rawCode ? String.fromCodePoint(b.rawCode) : b.char;
            dialog.currentBulletChar = rawChar;
          }
        } catch { /* ignore */ }
      }
      dialog.onApply = (nid, restartMode, startNum) => {
        if (nid === 0) {
          // "(없음)": 번호 해제
          ih.applyParaPropsAtCursor({ headType: 'None', numberingId: 0 });
        } else if (restartMode === 0) {
          // "앞 번호 이어": 이전 번호 문단의 numbering_id를 찾아서 적용
          const prevNid = (props as any).numberingId ?? nid;
          ih.applyNumbering(prevNid > 0 ? prevNid : nid);
        } else if (restartMode === 2) {
          // "새 번호 시작": 새 Numbering 정의 적용 (다른 numbering_id)
          ih.applyNumbering(nid);
        } else {
          // "이전 번호 이어": 현재 numbering_id 유지
          ih.applyNumbering(nid);
        }
      };
      dialog.onApplyBullet = (bulletChar) => {
        ih.applyBullet(bulletChar);
      };
      dialog.onClose = () => ih.focus();
      dialog.show();
    },
  },
  {
    id: 'format:bullet-shape',
    label: '글머리표 모양',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      // 글머리표 버튼의 팝업을 프로그래밍적으로 열기
      const btn = document.getElementById('tb-bullet');
      if (btn) btn.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    },
  },
  {
    id: 'format:level-increase',
    label: '한 수준 증가',
    shortcutLabel: 'Ctrl+Num -',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.changeOutlineLevel(-1);
    },
  },
  {
    id: 'format:level-decrease',
    label: '한 수준 감소',
    shortcutLabel: 'Ctrl+Num +',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.changeOutlineLevel(1);
    },
  },
  // 스타일 대화상자
  {
    id: 'format:style-dialog',
    label: '스타일',
    shortcutLabel: 'F6',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const dialog = new StyleDialog(services.wasm, services.eventBus);

      // 편집 요청
      dialog.onEditRequest = (styleId: number) => {
        const styles = services.wasm.getStyleList();
        const style = styles.find((s: { id: number }) => s.id === styleId);
        if (!style) return;
        const editDlg = new StyleEditDialog(services.wasm, services.eventBus, 'edit', {
          id: style.id, name: style.name, englishName: style.englishName,
          type: style.type, nextStyleId: style.nextStyleId,
        });
        editDlg.onSave = () => dialog.refresh();
        editDlg.show();
      };

      // 추가 요청
      dialog.onAddRequest = () => {
        let baseInfo = {};
        try {
          baseInfo = {
            charProps: ih.getCharProperties(),
            paraProps: ih.getParaProperties(),
          };
        } catch {
          baseInfo = {};
        }
        const addDlg = new StyleEditDialog(services.wasm, services.eventBus, 'add', undefined, baseInfo);
        addDlg.onSave = () => dialog.refresh();
        addDlg.show();
      };

      // 설정(적용)
      dialog.onApply = (styleId: number) => {
        ih.applyStyle(styleId);
      };
      dialog.onClose = () => ih.focus();

      const curStyleId = ih.getCurrentStyleId();
      dialog.show();
      dialog.setCurrentStyleId(curStyleId);
    },
  },
  {
    id: 'format:object-properties',
    label: '개체 속성',
    icon: 'icon-obj-props',
    shortcutLabel: 'P',
    canExecute: (ctx) => ctx.inPictureObjectSelection || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;

      // 그림/도형 선택 시
      if (ih.isInPictureObjectSelection()) {
        const ref = ih.getSelectedPictureRef();
        if (!ref) return;
        if (ref.type === 'equation') {
          const dialog = new EquationPropertiesDialog(services.wasm, services.eventBus);
          dialog.open(ref.sec, ref.ppi, ref.ci, ref.cellIdx, ref.cellParaIdx, ref.noteRef);
          return;
        }
        const dialog = new PicturePropsDialog(services.wasm, services.eventBus, services);
        dialog.open(ref.sec, ref.ppi, ref.ci, ref.type);
        return;
      }

      // 표 선택 시
      if (ih.isInTableObjectSelection()) {
        const pos = ih.getCursorPosition();
        if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
        const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
        const dialog = new TableCellPropsDialog(services.wasm, services.eventBus, tableCtx, pos.cellIndex, 'table', services);
        dialog.show();
      }
    },
  },
];
