import type { CommandDef } from '../types';
import { PageSetupDialog } from '@/ui/page-setup-dialog';
import { SectionSettingsDialog } from '@/ui/section-settings-dialog';
import { ColumnSettingsDialog } from '@/ui/column-settings-dialog';

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

/** 머리말/꼬리말 생성 + 편집 모드 진입 공통 함수 */
function enterHeaderFooterEditing(
  services: Parameters<CommandDef['execute']>[0],
  isHeader: boolean,
  applyTo: number,
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const cursor = (ih as any).cursor;
  if (!cursor) return;

  // 현재 보고 있는 페이지 기준으로 머리말/꼬리말이 있는 페이지 탐색
  const currentPage = cursor.rect?.pageIndex ?? 0;

  // 1) 현재 페이지에 머리말/꼬리말이 있는지 확인
  let targetResult = services.wasm.navigateHeaderFooterByPage(currentPage - 1, isHeader, 1);
  // navigateHeaderFooterByPage는 currentPage 다음부터 찾으므로, currentPage-1을 전달하면 currentPage부터 탐색
  // 결과의 pageIndex가 currentPage이면 현재 페이지에 존재
  if (!targetResult.ok || targetResult.pageIndex !== currentPage) {
    // 2) 현재 페이지에 없으면 이전 방향으로 탐색
    targetResult = services.wasm.navigateHeaderFooterByPage(currentPage, isHeader, -1);
  }
  if (!targetResult.ok) {
    // 3) 이전에도 없으면 다음 방향으로 탐색
    targetResult = services.wasm.navigateHeaderFooterByPage(currentPage, isHeader, 1);
  }

  if (targetResult.ok) {
    // 기존 머리말/꼬리말로 편집 모드 진입
    cursor.enterHeaderFooterMode(
      targetResult.isHeader!,
      targetResult.sectionIdx!,
      targetResult.applyTo!,
      targetResult.pageIndex!,
    );
  } else {
    // 문서에 머리말/꼬리말이 전혀 없으면 현재 구역에 새로 생성
    const sectionIdx = cursor.getPosition().sectionIndex;
    services.wasm.createHeaderFooter(sectionIdx, isHeader, applyTo);
    cursor.enterHeaderFooterMode(isHeader, sectionIdx, applyTo, currentPage);
  }

  services.eventBus.emit('headerFooterModeChanged', isHeader ? 'header' : 'footer');
  (ih as any).updateCaret?.();
  (ih as any).textarea?.focus();
}

/** 머리말/꼬리말 편집 중 필드 마커를 삽입하는 공통 함수 */
function insertHfField(
  services: Parameters<CommandDef['execute']>[0],
  fieldType: number,
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const cursor = (ih as any).cursor;
  if (!cursor || !cursor.isInHeaderFooter()) return;
  const isHeader = cursor.headerFooterMode === 'header';
  try {
    const result = services.wasm.insertFieldInHf(
      cursor.hfSectionIdx, isHeader, cursor.hfApplyTo,
      cursor.hfParaIdx, cursor.hfCharOffset, fieldType,
    );
    if (result.ok && result.charOffset !== undefined) {
      cursor.setHfCursorPosition(cursor.hfParaIdx, result.charOffset);
    }
    (ih as any).afterEdit?.();
    (ih as any).updateCaret?.();
  } catch (e) {
    console.warn('[page] 필드 삽입 실패:', e);
  }
}

/** 페이지 단위로 이전/다음 머리말·꼬리말로 이동하는 공통 함수 */
function navigateHeaderFooter(
  services: Parameters<CommandDef['execute']>[0],
  direction: -1 | 1,
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const cursor = (ih as any).cursor;
  if (!cursor || !cursor.isInHeaderFooter()) return;

  const currentPage = cursor.rect?.pageIndex ?? 0;
  const isHeader = cursor.headerFooterMode === 'header';

  const result = services.wasm.navigateHeaderFooterByPage(currentPage, isHeader, direction);
  if (!result.ok) return; // 더 이상 이동할 페이지 없음 → 현재 위치 유지

  // 대상 페이지의 머리말/꼬리말로 전환
  cursor.switchHeaderFooterTarget(
    result.isHeader!,
    result.sectionIdx!,
    result.applyTo!,
    result.pageIndex!,
  );
  services.eventBus.emit('headerFooterModeChanged', result.isHeader ? 'header' : 'footer');
  (ih as any).updateCaret?.();
  (ih as any).textarea?.focus();
}

/** 머리말/꼬리말 마당 템플릿 적용 공통 함수 */
function applyHfTemplate(
  services: Parameters<CommandDef['execute']>[0],
  isHeader: boolean,
  applyTo: number,
  templateId: number,
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const cursor = (ih as any).cursor;
  // 현재 편집 중이면 종료
  let sectionIdx: number;
  if (cursor?.isInHeaderFooter()) {
    sectionIdx = cursor.hfSectionIdx;
    cursor.exitHeaderFooterMode();
  } else {
    sectionIdx = cursor?.getPosition()?.sectionIndex ?? 0;
  }

  try {
    services.wasm.applyHfTemplate(sectionIdx, isHeader, applyTo, templateId);
    // 템플릿 적용 후 편집 모드로 진입
    const currentPage = cursor?.rect?.pageIndex ?? 0;
    let targetResult = services.wasm.navigateHeaderFooterByPage(currentPage - 1, isHeader, 1);
    if (!targetResult.ok || targetResult.pageIndex !== currentPage) {
      targetResult = services.wasm.navigateHeaderFooterByPage(currentPage, isHeader, -1);
    }
    if (!targetResult.ok) {
      targetResult = services.wasm.navigateHeaderFooterByPage(currentPage, isHeader, 1);
    }
    if (targetResult.ok) {
      cursor.enterHeaderFooterMode(
        targetResult.isHeader!, targetResult.sectionIdx!,
        targetResult.applyTo!, targetResult.pageIndex!,
      );
    } else {
      cursor.enterHeaderFooterMode(isHeader, sectionIdx, applyTo, currentPage);
    }
    services.eventBus.emit('headerFooterModeChanged', isHeader ? 'header' : 'footer');
  } catch (e) {
    console.warn('[page] 마당 적용 실패:', e);
  }
  (ih as any).afterEdit?.();
  (ih as any).updateCaret?.();
  // 메뉴 닫기 후 포커스가 유실될 수 있으므로 지연 포커스
  const textarea = (ih as any).textarea;
  textarea?.focus();
  setTimeout(() => textarea?.focus(), 50);
}

export const pageCommands: CommandDef[] = [
  {
    id: 'page:setup',
    label: '편집 용지',
    icon: 'icon-page-setup',
    shortcutLabel: 'F7',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      const cursor = ih ? (ih as any).cursor : null;
      const sectionIdx = cursor?.getPosition()?.sectionIndex ?? 0;
      const dialog = new PageSetupDialog(services.wasm, services.eventBus, sectionIdx);
      dialog.show();
    },
  },
  // ─── 머리말 ──────────────────────────────────
  {
    id: 'page:header-create',
    label: '머리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      enterHeaderFooterEditing(services, true, 0); // Both
    },
  },
  // ─── 꼬리말 ──────────────────────────────────
  {
    id: 'page:footer-create',
    label: '꼬리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      enterHeaderFooterEditing(services, false, 0); // Both
    },
  },
  // ─── 머리말/꼬리말 닫기 ────────────────────────
  {
    id: 'page:headerfooter-close',
    label: '머리말/꼬리말 닫기',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor || !cursor.isInHeaderFooter()) return;
      // 현재 보고 있는 페이지 기억
      const hfPage = cursor.rect?.pageIndex ?? 0;
      cursor.exitHeaderFooterMode();
      services.eventBus.emit('headerFooterModeChanged', 'none');
      // 해당 페이지의 본문 첫 문단 시작점으로 커서 이동
      try {
        const pageInfo = services.wasm.getPageInfo(hfPage);
        const bodyX = pageInfo.marginLeft + 1;
        const bodyY = pageInfo.marginTop + pageInfo.marginHeader + 1;
        const hit = services.wasm.hitTest(hfPage, bodyX, bodyY);
        if (hit.paragraphIndex < 0xFFFFFF00) {
          cursor.moveTo(hit);
        }
      } catch { /* hitTest 실패 시 기존 위치 유지 */ }
      (ih as any).afterEdit?.();
      (ih as any).textarea?.focus();
    },
  },
  // ─── 머리말/꼬리말 지우기 ──────────────────────
  {
    id: 'page:headerfooter-delete',
    label: '머리말/꼬리말 지우기',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor || !cursor.isInHeaderFooter()) return;
      const sectionIdx = cursor.hfSectionIdx;
      const isHeader = cursor.headerFooterMode === 'header';
      const applyTo = cursor.hfApplyTo;
      // 편집 모드 탈출
      cursor.exitHeaderFooterMode();
      services.eventBus.emit('headerFooterModeChanged', 'none');
      // 컨트롤 삭제
      try {
        services.wasm.deleteHeaderFooter(sectionIdx, isHeader, applyTo);
      } catch (e) {
        console.warn('[page] 머리말/꼬리말 삭제 실패:', e);
      }
      (ih as any).afterEdit?.();
      (ih as any).textarea?.focus();
    },
  },
  // ─── 머리말/꼬리말 이전/다음 이동 ─────────────────
  {
    id: 'page:headerfooter-prev',
    label: '이전 머리말/꼬리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      navigateHeaderFooter(services, -1);
    },
  },
  {
    id: 'page:headerfooter-next',
    label: '다음 머리말/꼬리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      navigateHeaderFooter(services, 1);
    },
  },
  stub('page:new-page-num', '새 번호로 시작'),
  // ─── 머리말/꼬리말 현재 쪽 감추기 ──────────────
  {
    id: 'page:hide-headerfooter',
    label: '머리말/꼬리말 현재 쪽 감추기',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor || !cursor.isInHeaderFooter()) return;
      const isHeader = cursor.headerFooterMode === 'header';
      const pageIndex = cursor.rect?.pageIndex ?? 0;
      try {
        const result = services.wasm.toggleHideHeaderFooter(pageIndex, isHeader);
        console.log(`[page] ${isHeader ? '머리말' : '꼬리말'} 쪽 ${pageIndex} 감추기: ${result.hidden}`);
      } catch (e) {
        console.warn('[page] 감추기 토글 실패:', e);
      }
      (ih as any).afterEdit?.();
      (ih as any).updateCaret?.();
    },
  },
  // ─── 머리말/꼬리말 필드 삽입 ────────────────────
  {
    id: 'page:insert-field-pagenum',
    label: '쪽 번호 삽입',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) { insertHfField(services, 1); },
  },
  {
    id: 'page:insert-field-totalpage',
    label: '총 쪽수 삽입',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) { insertHfField(services, 2); },
  },
  {
    id: 'page:insert-field-filename',
    label: '파일 이름 삽입',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) { insertHfField(services, 3); },
  },
  // ─── 머리말/꼬리말 마당 (템플릿) ─────────────────────
  {
    id: 'page:apply-hf-template',
    label: '머리말/꼬리말 마당',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services, params) {
      const isHeader = params?.isHeader === 'true';
      const applyTo = parseInt(params?.applyTo as string ?? '0', 10);
      const templateId = parseInt(params?.templateId as string ?? '0', 10);
      applyHfTemplate(services, isHeader, applyTo, templateId);
    },
  },
  {
    id: 'page:break',
    label: '쪽 나누기',
    shortcutLabel: 'Ctrl+Enter',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        const result = JSON.parse(services.wasm.insertPageBreak(pos.sectionIndex, pos.paragraphIndex, pos.charOffset));
        if (result.ok) {
          services.eventBus.emit('document-changed');
        }
      } catch (err) {
        console.warn('[page:break] 쪽 나누기 실패:', err);
      }
    },
  },
  {
    id: 'page:hide',
    label: '감추기',
    shortcutLabel: 'Ctrl+N,S',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        // 현재 문단의 감추기 상태 조회
        const result = JSON.parse((services.wasm as any).doc.getPageHide(pos.sectionIndex, pos.paragraphIndex));
        if (result.exists) {
          // 이미 감추기 있음 → 토글 (제거)
          (services.wasm as any).doc.setPageHide(
            pos.sectionIndex, pos.paragraphIndex,
            false, false, false, false, false, false,
          );
        } else {
          // 감추기 없음 → 쪽 번호 감추기 기본 적용
          (services.wasm as any).doc.setPageHide(
            pos.sectionIndex, pos.paragraphIndex,
            false, false, false, false, false, true,
          );
        }
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[page:hide] 감추기 실패:', err);
      }
    },
  },
  {
    id: 'page:column-break',
    label: '단 나누기',
    shortcutLabel: 'Ctrl+Shift+Enter',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        const result = JSON.parse(services.wasm.insertColumnBreak(pos.sectionIndex, pos.paragraphIndex, pos.charOffset));
        if (result.ok) {
          services.eventBus.emit('document-changed');
        }
      } catch (err) {
        console.warn('[page:column-break] 단 나누기 실패:', err);
      }
    },
  },
  // 다단 프리셋 — ColumnDef 컨트롤 수정 (SectionDef와 독립)
  ...[
    { id: 'page:col-1', label: '단 - 하나', cols: 1 },
    { id: 'page:col-2', label: '단 - 둘', cols: 2 },
    { id: 'page:col-3', label: '단 - 셋', cols: 3 },
  ].map((def): CommandDef => ({
    id: def.id,
    label: def.label,
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        // 일반 다단(0), 같은 너비(1), 간격 8mm=2268HU
        services.wasm.setColumnDef(pos.sectionIndex, def.cols, 0, 1, 2268);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn(`[${def.id}] 다단 설정 실패:`, err);
      }
    },
  })),
  {
    id: 'page:col-left',
    label: '단 - 왼쪽',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      try {
        services.wasm.setColumnDef(ih.getPosition().sectionIndex, 2, 0, 0, 2268);
        services.eventBus.emit('document-changed');
      } catch (err) { console.warn('[page:col-left]', err); }
    },
  },
  {
    id: 'page:col-right',
    label: '단 - 오른쪽',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      try {
        services.wasm.setColumnDef(ih.getPosition().sectionIndex, 2, 0, 0, 2268);
        services.eventBus.emit('document-changed');
      } catch (err) { console.warn('[page:col-right]', err); }
    },
  },
  {
    id: 'page:col-settings',
    label: '다단 설정',
    shortcutLabel: 'Ctrl+Alt+Enter',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      const dlg = new ColumnSettingsDialog(services.wasm, services.eventBus, pos.sectionIndex);
      dlg.show();
    },
  },
  // ─── 구역 설정 ──────────────────────────────────
  {
    id: 'page:section-settings',
    label: '구역 설정',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      const sectionIdx = cursor?.getPosition()?.sectionIndex ?? 0;
      const dialog = new SectionSettingsDialog(services.wasm, services.eventBus, sectionIdx);
      dialog.show();
    },
  },
];
