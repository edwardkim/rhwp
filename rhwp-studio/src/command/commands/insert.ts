import type { CommandDef } from '../types';
import { PicturePropsDialog } from '@/ui/picture-props-dialog';
import { ChartDataDialog } from '@/ui/chart-data-dialog';
import { chartTargetFromSelection, matchChartRef } from '@/core/chart-data-target';
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
import { SetObjectPropsCommand, SetZOrderCommand, type RefreshPolicy } from '@/engine/command';
import { getObjectProps, setObjectProps, type ObjectPropsRef } from '@/engine/object-props';

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
let chartDataDialog: ChartDataDialog | null = null;
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
    opensDialog: true,
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
    opensDialog: true,
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
    opensDialog: true,
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
    opensDialog: true,
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
    opensDialog: true,
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
    opensDialog: true,
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
    id: 'insert:chart-data-edit',
    opensDialog: true,
    label: '차트 데이터 편집',
    canExecute: (ctx) => ctx.inPictureObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedPictureRef();
      if (!ref) return;
      // [#4694] 선택 → 열거 대조 → 정본 주소(문서 순번). 대조 실패는 조용히 무시 —
      // 메뉴 노출 판정(input-handler)과 같은 경로라 여기 도달하면 보통 성공한다.
      const target = chartTargetFromSelection(ref);
      if (!target) return;
      let matched = null;
      try {
        matched = matchChartRef(services.wasm.listCharts(), target);
      } catch {
        return;
      }
      if (!matched) return;
      if (!chartDataDialog) {
        chartDataDialog = new ChartDataDialog(services.wasm, services.eventBus, services);
      }
      // 닫힘 후 편집기 포커스 복구 — 없으면 이어지는 Ctrl+Z 가 문서에 닿지 않는다
      // (field:edit 의 onClose 복원과 동형).
      chartDataDialog.afterClose = () => {
        requestAnimationFrame(() => ih.focus());
      };
      chartDataDialog.open(matched);
    },
  },
  {
    id: 'insert:equation-edit',
    opensDialog: true,
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
        // [Task #3230] `setProps` 래퍼가 사라져 공유 라우팅을 직접 부른다. 이 경로는 종전부터
        // 라우터를 거치지 않고 직접 적용하고 `document-changed` 를 스스로 emit 한다 —
        // 회전/대칭과 달리 이번 변경 대상이 아니라 종전 동작 그대로 둔다.
        result = setObjectProps(services.wasm, ref, captionProps);
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
      changeZOrder(ih, ref, 'front');
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
      changeZOrder(ih, ref, 'forward');
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
      changeZOrder(ih, ref, 'backward');
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
      changeZOrder(ih, ref, 'back');
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

/**
 * 선택 개체 ref 타입 — cursor.selectedPictureRef 와 정합 (headerFooter optional, [Task #831]).
 *
 * [Task #3230] 분기 본체는 `engine/object-props.ts` 로 옮겼다 — 역연산 커맨드가 undo 시점에
 * 같은 분기를 써야 하는데, 커맨드는 `services` 가 아니라 `WasmBridge` 만 받기 때문이다.
 */
type PictureRef = ObjectPropsRef;

/** 선택 개체의 속성 조회 (shape/picture·셀·머리말꼬리말 분기). */
function getProps(services: import('../types').CommandServices, ref: PictureRef): Record<string, unknown> {
  return getObjectProps(services.wasm, ref);
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
  // [Task #3351] 개체 선택은 `cursor.position` 을 옮기지 않는다. 그래서 여기서 그냥
  // `getCursorPosition()` 을 잡으면 **개체를 선택하기 직전 캐럿**이 기록되고, undo/redo 가
  // 조작과 무관한 자리(문서 상단일 수도 있다)로 착지한다.
  //
  // 한컴 2024 실측: 개체 선택은 캐럿을 앵커로 끌고 가고, 캐럿을 다른 문단으로 옮기면 선택이
  // 풀린다 — "캐럿은 딴 곳, 개체는 선택" 이라는 상태 자체가 없다. 삭제·undo·redo 내내 캐럿은
  // 개체 인접 문단에 머문다. Delete 키 경로(`performDelete`)가 이미 그렇게 하고 있으므로
  // 메뉴 경로도 같은 자리를 기록한다.
  const pos = ih.getPositionOutsideSelectedPicture() ?? ih.getCursorPosition();
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
 * [Task #2370 클러스터 A → #5769 후속] z순서 변경 — 스냅샷 대신 역연산 커맨드로 기록한다.
 *
 * 무변경 판정은 Rust 응답의 moves 로 한다 — change_shape_z_order_native 는 "이미 맨 앞/뒤"
 * 경계에서 문서를 건드리지 않고 빈 moves 를 돌려주고, 실제 변경 시 대상(+교환 이웃)의
 * before/after 쌍을 담는다. SetZOrderCommand 가 그 쌍을 undo/redo 의 절대 복원값으로
 * 소비한다 — 되돌릴 것이 스칼라 1~2개인데 문서 전체 클론을 스택에 얹지 않는다(#5769).
 * 양식 모드 게이트는 kind:'command' 라우팅이 execute 보다 먼저 통과시킨다(#3230 계약).
 */
function changeZOrder(
  ih: InputHandler,
  ref: PictureRef,
  operation: 'front' | 'forward' | 'backward' | 'back',
): void {
  const pos = ih.getPositionOutsideSelectedPicture() ?? ih.getCursorPosition();
  ih.executeOperation({
    kind: 'command',
    command: new SetZOrderCommand(ref.sec, ref.ppi, ref.ci, operation, pos),
    meta: DEFER_REFRESH_TO_EXIT,
  });
  // 참고: 무변경(이미 맨 앞/뒤)일 때도 command 분기는 반환 위치로 커서를 앵커에
  // 맞춘다 — 종전 snapshot 경로의 조기 break 와 다른 유일한 동작 델타로, 개체
  // 선택 UX(캐럿이 개체 인접에 머무른다, #3351)와 같은 자리라 의도적으로 둔다.
  ih.exitPictureObjectSelectionAndAfterEdit();
}

/**
 * [Task #3230] 절대 속성 하나를 **역연산 커맨드로** 적용하고 기록한다.
 *
 * 스냅샷(`recordObjectMutation`) 대신 `kind:'command'`를 쓴다 — 되돌릴 것이 스칼라 하나인데
 * `Document` 통째 클론 2개(문서에 따라 최대 21 MB)를 스택에 얹을 이유가 없다. 호출부가 이미
 * 적용 전 값을 읽어 두었으므로 before 가 정확하다. setter를 라우터 전에 직접 호출하지 않아야
 * 양식 모드의 편집 허용 검사가 실제 변경보다 먼저 실행된다.
 *
 * refresh 를 'full' 로 명시한다 — command 라우팅의 자동 판정에 맡기면 개체 회전/대칭의 화면 반영이
 * 보장되지 않는다. 종전 스냅샷 라우팅의 'full' 이 `afterEdit()` → `document-changed` 를 대신
 * emit 해 주고 있었다(그래서 호출부의 수동 emit 이 [undo P3 정리] 에서 제거됐다).
 */
function executeAbsolutePropChange(
  ih: InputHandler,
  ref: PictureRef,
  before: Record<string, unknown>,
  after: Record<string, unknown>,
): void {
  ih.executeOperation({
    kind: 'command',
    command: new SetObjectPropsCommand(ref, before, after),
    meta: { refresh: 'full' },
  });
}

/**
 * [Task #3230] 속성 왕복이 **참인 역연산인 대상**인지.
 *
 * 도형은 참이다 — `rotationAngle` 은 평범한 필드 대입이고 flip 은 비트를 대칭으로 set/clear
 * 한다(`document_core/commands/object_ops/shape.rs`). 같은 값을 다시 넣으면 원래 상태다.
 *
 * **그림·OLE 는 아니다.** `rotationAngle` 이 섞이면 `refresh_picture_rotation_layout_for_save`
 * 가 각도와 무관하게 — 0 으로 되돌리는 경우까지 — `rotate_image = true` 와
 * `flip |= 0x0008_0000` 을 세운다(`object_ops/picture.rs:242-243`). 이 둘을 다시 내리는 경로는
 * 저장소에 없어서, 90° 돌렸다 되돌린 그림은 화면은 같아도 저장 바이트가 원본과 달라진다
 * (HWPX `hp:rotationInfo rotateimage`·HWP5 `HWPTAG_SHAPE_COMPONENT` 의 flip 비트).
 * 대칭도 `TRANSFORM_KEYS`(picture.rs:176-184)에 걸려 `raw_rendering`·`render_*` 캐시를
 * 기본값으로 리셋한다.
 *
 * 속성 bag 만으로는 이것들을 복원할 수 없다. 문서를 통째로 되돌리는 스냅샷이라야 정확하므로
 * 그림·OLE 는 종전 경로를 유지한다 — 되돌리기의 정확성이 스냅샷 비용보다 앞선다.
 */
function absolutePropChangeIsInvertible(ref: PictureRef): boolean {
  return ref.type === 'shape';
}

/** 역연산이 참이면 커맨드로, 아니면 종전 스냅샷으로 기록한다. */
function applyAbsolutePropChange(
  services: import('../types').CommandServices,
  ih: InputHandler,
  ref: PictureRef,
  operationType: string,
  before: Record<string, unknown>,
  after: Record<string, unknown>,
): void {
  if (absolutePropChangeIsInvertible(ref)) {
    executeAbsolutePropChange(ih, ref, before, after);
    return;
  }
  recordObjectMutation(ih, operationType, (wasm) => {
    setObjectProps(wasm, ref, after);
  });
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
  // [Task #3230] 도형은 역연산, 그림·OLE 는 스냅샷 유지(`absolutePropChangeIsInvertible`).
  // `cur` 는 이미 위에서 읽은 적용 전 각도이고 setter 가 절대값이라 되돌리기가 자명하다.
  applyAbsolutePropChange(
    services, ih, ref, 'rotateObject', { rotationAngle: cur }, { rotationAngle: next },
  );
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
  // 위 rotate 와 동일 — 토글이지만 setter 에 넘기는 값은 절대값(`!cur`)이라 역연산이 자명하다.
  applyAbsolutePropChange(services, ih, ref, 'flipObject', { [key]: cur }, { [key]: !cur });
}
