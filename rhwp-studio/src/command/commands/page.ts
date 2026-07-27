import type { CommandDef } from '../types';
import type { DocumentPosition } from '@/core/types';
import { PageSetupDialog } from '@/ui/page-setup-dialog';
import { PageBorderDialog } from '@/ui/page-border-dialog';
import { SectionSettingsDialog } from '@/ui/section-settings-dialog';
import { ColumnSettingsDialog } from '@/ui/column-settings-dialog';
import { NewNumberDialog } from '@/ui/new-number-dialog';
import { InsertFieldInHeaderFooterCommand } from '@/engine/command';

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

/**
 * 대상 좌표에 머리말/꼬리말이 없으면 만든다 (Task #3206).
 *
 * 존재 확인은 **생성 네이티브가 중복을 거부하는 조건과 같은 범위**(구역·종류·applyTo)로 해야
 * 한다 — `getHeaderFooter` 와 `createHeaderFooter` 는 같은 `find_header_footer_control` 을
 * 쓰므로 두 판단이 어긋날 수 없다. 대상 좌표를 정하는 일은 호출측(`getHeaderFooterEditTarget`)
 * 몫이고, 여기서는 그 좌표를 그대로 쓴다.
 *
 * `navigateHeaderFooterByPage` 로 존재를 확인하면 안 된다. 이 함수는 `current_page + direction`
 * 부터 훑는 쪽 이동용이라 현재 쪽을 건너뛴다 — 1쪽 문서에서는 어느 방향으로도 대상이 없어,
 * 이미 있는 머리말을 없다고 판단하고 생성으로 넘어가 중복 생성 오류가 났다.
 *
 * [Task #3207] 생성은 문서 구조를 바꾸므로 snapshot 으로 기록한다(이미 있으면 커서 이동뿐이라
 * 기록 대상이 아니다).
 */
function ensureHeaderFooter(
  services: Parameters<CommandDef['execute']>[0],
  ih: NonNullable<ReturnType<Parameters<CommandDef['execute']>[0]['getInputHandler']>>,
  bodyPos: DocumentPosition,
  target: { sectionIndex: number; applyTo: number },
  isHeader: boolean,
): void {
  const { sectionIndex, applyTo } = target;
  const hf = JSON.parse(services.wasm.getHeaderFooter(sectionIndex, isHeader, applyTo));
  if (hf.exists) return;

  ih.executeOperation({
    kind: 'snapshot',
    operationType: 'createHeaderFooter',
    operation: (wasm) => {
      wasm.createHeaderFooter(sectionIndex, isHeader, applyTo);
      return bodyPos;
    },
  });
}

/** 머리말/꼬리말 생성 + 편집 모드 진입 공통 함수 */
function enterHeaderFooterEditing(
  services: Parameters<CommandDef['execute']>[0],
  isHeader: boolean,
): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const cursor = (ih as any).cursor;
  if (!cursor) return;

  // 편집 대상은 이 쪽에 **실제로 렌더되는** 컨트롤이다 — 어느 것이 렌더되는지는 쪽 홀짝이
  // 정하고(홀수/짝수가 양 쪽을 이긴다) 구역에 자기 머리말이 없으면 앞 구역에서 상속된다.
  // `양 쪽` 으로 고정하면 캐럿이 찍힌 머리말과 다른 컨트롤을 편집하게 된다 (Task #3206).
  // 더블클릭 진입(input-handler-mouse)이 히트테스트로 얻는 것과 같은 답이다.
  const currentPage = cursor.rect?.pageIndex ?? 0;
  const target = services.wasm.getHeaderFooterEditTarget(currentPage, isHeader);

  const bodyPos = cursor.getPosition();
  ensureHeaderFooter(services, ih, bodyPos, target, isHeader);
  cursor.enterHeaderFooterMode(isHeader, target.sectionIndex, target.applyTo, currentPage);

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
  const target = { sectionIdx: cursor.hfSectionIdx, isHeader, applyTo: cursor.hfApplyTo };
  const paraIdx = cursor.hfParaIdx;
  const charOffset = cursor.hfCharOffset;
  try {
    const result = services.wasm.insertFieldInHf(
      target.sectionIdx, isHeader, target.applyTo, paraIdx, charOffset, fieldType,
    );
    if (
      result.ok
      && Number.isSafeInteger(result.charOffset)
      && Number.isSafeInteger(result.insertedAt)
      && Number.isSafeInteger(result.insertedLength)
      && result.insertedLength > 0
    ) {
      cursor.setHfCursorPosition(paraIdx, result.charOffset);
      // [Task #3212] 이미 적용된 삽입을 역연산 명령으로 기록한다(#2337 HF 커맨드와 동형).
      // cursor 좌표와 실제 텍스트 삽입 위치는 inline control 뒤에서 다를 수 있다.
      // 따라서 역연산은 native가 준 실제 marker 범위를 사용한다.
      ih.executeOperation({
        kind: 'record',
        command: new InsertFieldInHeaderFooterCommand(
          target, paraIdx, charOffset, result.insertedAt, fieldType,
          result.insertedLength, result.charOffset,
        ),
      });
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
    // [Task #3207] 마당 적용은 HF 내용을 통째로 교체하므로 snapshot 으로 기록한다.
    // 모드 탈출/재진입은 커서 상태라 기록 밖에 둔다(undo 는 본문 분기로 모드를 빠져나온다).
    // 위에서 이미 HF 모드를 종료했으므로 여기서 읽는 위치는 본문 위치다.
    const bodyPos = ih.getPosition();
    let applied = false;
    ih.executeOperation({
      kind: 'snapshot',
      operationType: 'applyHfTemplate',
      operation: (wasm) => {
        // [Task #2370] 의미상 실패(ok:false)는 문서 무변경이므로 null 로 기록을 취소한다.
        // 종전에는 같은 목적으로 throw 했으나(정상 흐름을 예외로 처리) 공용 no-op 신호로 통일.
        applied = wasm.applyHfTemplate(sectionIdx, isHeader, applyTo, templateId).ok;
        return applied ? bodyPos : null;
      },
    });
    if (!applied) {
      // 적용되지 않았으면 HF 가 생겼다는 보장이 없다 → 편집 모드로 들어가지 않는다.
      console.warn('[page] 마당 적용 실패: applyHfTemplate 가 ok:false 를 반환');
      (ih as any).afterEdit?.();
    } else {
      // 템플릿 적용 후 편집 모드로 진입. 마당 적용은 기존 HF 를 지우고 다시 만들므로
      // (`apply_hf_template_native` 1~2단계) 적용 대상 좌표에 HF 가 있음이 보장된다 —
      // 존재 확인이 필요 없다 (Task #3206).
      cursor.enterHeaderFooterMode(isHeader, sectionIdx, applyTo, cursor?.rect?.pageIndex ?? 0);
      services.eventBus.emit('headerFooterModeChanged', isHeader ? 'header' : 'footer');
    }
  } catch (e) {
    console.warn('[page] 마당 적용 실패:', e);
    // 실패 경로 리프레시 — 성공 경로는 executeOperation 이 이미 afterEdit 로 갱신했으므로
    // 여기서 다시 부르지 않는다(성공 경로 이중 afterEdit 제거).
    (ih as any).afterEdit?.();
  }
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
      const dialog = new PageSetupDialog(services.wasm, services.eventBus, sectionIdx, services);
      dialog.show();
    },
  },
  {
    id: 'page:page-border',
    label: '쪽 테두리/배경',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      const cursor = ih ? (ih as any).cursor : null;
      const sectionIdx = cursor?.getPosition()?.sectionIndex ?? 0;
      const dialog = new PageBorderDialog(services.wasm, services.eventBus, sectionIdx, services);
      dialog.show();
    },
  },
  // ─── 머리말 ──────────────────────────────────
  {
    id: 'page:header-create',
    label: '머리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      enterHeaderFooterEditing(services, true);
    },
  },
  // ─── 꼬리말 ──────────────────────────────────
  {
    id: 'page:footer-create',
    label: '꼬리말',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      enterHeaderFooterEditing(services, false);
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
      // 컨트롤 삭제 — [Task #3207] snapshot 으로 기록해 undo 로 되살릴 수 있게 한다.
      // 모드 탈출은 위에서 이미 끝났으므로 여기 커서는 본문이다.
      const bodyPos = ih.getPosition();
      ih.executeOperation({
        kind: 'snapshot',
        operationType: 'deleteHeaderFooter',
        operation: (wasm) => {
          wasm.deleteHeaderFooter(sectionIdx, isHeader, applyTo);
          return bodyPos;
        },
      });
      // executeOperation 이 이미 afterEdit 로 리프레시하므로 별도 afterEdit 를 부르지 않는다.
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
  {
    id: 'page:new-page-num',
    label: '새 번호로 시작',
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor) return;
      const pos = cursor.getPosition();
      const wasm = services.wasm;
      const eventBus = services.eventBus;
      if (!wasm || !eventBus) return;
      const dlg = new NewNumberDialog(wasm, eventBus, {
        sec: pos.sectionIndex, para: pos.paragraphIndex, offset: pos.charOffset,
      }, services);
      dlg.show();
    },
  },
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
      // [Task #3207] 감추기는 히스토리에 기록하지 않는다 — toggle_hide_header_footer_native 는
      // 세션 집합(hidden_header_footer)과 렌더 캐시만 바꾸고 document 는 건드리지 않는데
      // (레포 가드가 Exempt::SessionState/직렬화 비대상으로 분류), 스냅샷은 document 만 담고
      // 복원도 세션 집합을 되돌리지 않는다. 라우팅하면 되돌아가는 것 없이 redo 스택이 버려지고
      // 스냅샷 예산만 소모돼 사용자의 진짜 undo 이력이 축출된다.
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
  {
    id: 'page:hide-current',
    label: '현재 쪽만 감추기',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const cursor = (ih as any).cursor;
      if (!cursor) return;
      const pageIndex = cursor.rect?.pageIndex ?? 0;
      // [Task #3207] 감추기는 세션 상태라 히스토리에 기록하지 않는다(위 page:hide-headerfooter
      // 주석 참조). 스냅샷이 담는 내용이 없어 원자화도 실효가 없으므로 종전 호출을 유지하고,
      // dirty 마킹 경로인 emit 도 그대로 둔다.
      try {
        const headerResult = services.wasm.toggleHideHeaderFooter(pageIndex, true);
        const footerResult = services.wasm.toggleHideHeaderFooter(pageIndex, false);
        if (headerResult.hidden !== footerResult.hidden) {
          services.wasm.toggleHideHeaderFooter(pageIndex, false);
        }
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[page:hide-current] 현재 쪽 감추기 실패:', err);
      }
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
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'pageBreak',
          operation: (wasm) => {
            const result = JSON.parse(wasm.insertPageBreak(pos.sectionIndex, pos.paragraphIndex, pos.charOffset));
            if (result.ok) {
              return {
                sectionIndex: pos.sectionIndex,
                paragraphIndex: result.paraIdx ?? pos.paragraphIndex,
                charOffset: result.charOffset ?? 0,
              };
            }
            return pos;
          },
          meta: { actionId: 'page:break', domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
      } catch (err) {
        console.warn('[page:break] 쪽 나누기 실패:', err);
      }
    },
  },
  {
    id: 'page:hide',
    label: '감추기',
    shortcutLabel: 'Ctrl+M,S',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getPosition();
      try {
        // 현재 문단의 감추기 상태 조회(읽기) 후 토글값 결정.
        const cur = JSON.parse((services.wasm as any).doc.getPageHide(pos.sectionIndex, pos.paragraphIndex));
        // exists → 제거(false), 없음 → 쪽 번호 감추기 적용(true). 나머지 5플래그는 양쪽 모두 false.
        const applyPageNumHide = !cur.exists;
        // [page-hide 이관] 문단 감추기 변경을 snapshot 으로 라우팅(기존 emit-only → undo 불가였음).
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'pageHide',
          operation: (wasm) => {
            (wasm as any).doc.setPageHide(
              pos.sectionIndex, pos.paragraphIndex,
              false, false, false, false, false, applyPageNumHide,
            );
            return pos;
          },
          meta: { actionId: 'page:hide', domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
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
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'columnBreak',
          operation: (wasm) => {
            const result = JSON.parse(wasm.insertColumnBreak(pos.sectionIndex, pos.paragraphIndex, pos.charOffset));
            if (result.ok) {
              return {
                sectionIndex: pos.sectionIndex,
                paragraphIndex: result.paraIdx ?? pos.paragraphIndex,
                charOffset: result.charOffset ?? 0,
              };
            }
            return pos;
          },
          meta: { actionId: 'page:column-break', domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
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
        // 일반 다단(0), 같은 너비(1), 간격 8mm=2268HU. snapshot 으로 라우팅해 undo 가능.
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'setColumnDef',
          operation: (wasm) => {
            wasm.setColumnDef(pos.sectionIndex, def.cols, 0, 1, 2268);
            return pos;
          },
          meta: { actionId: def.id, domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
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
      const pos = ih.getPosition();
      try {
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'setColumnDef',
          operation: (wasm) => {
            wasm.setColumnDef(pos.sectionIndex, 2, 0, 0, 2268);
            return pos;
          },
          meta: { actionId: 'page:col-left', domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
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
      const pos = ih.getPosition();
      try {
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'setColumnDef',
          operation: (wasm) => {
            wasm.setColumnDef(pos.sectionIndex, 2, 0, 0, 2268);
            return pos;
          },
          meta: { actionId: 'page:col-right', domain: 'page', refresh: 'full', dirtyScope: 'document' },
        });
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
      const dlg = new ColumnSettingsDialog(services.wasm, services.eventBus, pos.sectionIndex, services);
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
      const dialog = new SectionSettingsDialog(services.wasm, services.eventBus, sectionIdx, services);
      dialog.show();
    },
  },
];
