import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

function slice(s: string, from: string, to: string): string {
  const start = s.indexOf(from);
  assert.notEqual(start, -1, `${from} not found`);
  const end = s.indexOf(to, start + from.length);
  return end === -1 ? s.slice(start) : s.slice(start, end);
}

test('마지막 ModalDialog 종료만 편집기 포커스 복원 이벤트를 발행한다', () => {
  const dialog = source('src/ui/dialog.ts');
  const hide = slice(dialog, 'hide(): void {', '\n  /** 서브클래스에서 본문 DOM을 생성 */');

  assert.match(dialog, /export const MODAL_DIALOG_CLOSED_EVENT = 'rhwp-modal-dialog-closed'/,
    '공통 이벤트 이름을 dialog 모듈이 소유');
  assert.match(hide, /document\.querySelector\('\.modal-overlay'\)/,
    '중첩 모달이 남았는지 확인');
  assert.match(hide, /document\.dispatchEvent\(new Event\(MODAL_DIALOG_CLOSED_EVENT\)\)/,
    '마지막 모달 종료 이벤트 발행');

  const remove = hide.indexOf('this.overlay?.remove()');
  const afterClose = hide.indexOf('this.afterClose?.()');
  const lastOverlay = hide.indexOf("document.querySelector('.modal-overlay')");
  const dispatch = hide.indexOf('document.dispatchEvent');
  assert.ok(remove < afterClose, 'overlay를 먼저 제거');
  assert.ok(afterClose < lastOverlay, '기존 afterClose 후처리를 보존');
  assert.ok(lastOverlay < dispatch, '마지막 overlay일 때만 이벤트 발행');
});

test('앱은 활성 InputHandler에만 마지막 모달 종료 포커스를 복원한다', () => {
  const main = source('src/main.ts');
  const setup = slice(main, 'function setupModalFocusRestore(): void {', '\n/**\n * 전역 단축키');

  assert.match(setup, /document\.addEventListener\(MODAL_DIALOG_CLOSED_EVENT/, '공통 이벤트 구독');
  assert.match(setup, /if \(inputHandler\?\.isActive\(\)\) inputHandler\.focus\(\)/,
    '활성 handler일 때만 textarea 포커스 복원');
  const install = main.indexOf('setupModalFocusRestore();');
  const shortcuts = main.indexOf('setupGlobalShortcuts();');
  assert.ok(install !== -1 && install < shortcuts, '단축키 설치 전에 포커스 복원 구독 설치');
});
