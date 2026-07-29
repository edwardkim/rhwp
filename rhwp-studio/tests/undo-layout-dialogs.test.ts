import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #layout-dialogs] 쪽/구역 레이아웃 다이얼로그 히스토리 라우팅 소스 가드.
//
// page-setup/section/column/page-border 다이얼로그는 (wasm,eventBus) 로만 생성돼 InputHandler
// (편집 라우터)에 도달 못 했다 → 편집 용지/구역/다단/쪽테두리 변경이 미기록(undo 불가).
// services 를 생성자에 주입하고 onConfirm 을 executeOperation snapshot 으로 라우팅했는지,
// services 미주입 fallback(직접 적용)을 유지했는지 정적으로 핀한다. 행위 증명은 브라우저 왕복.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, `src/ui/${rel}`), 'utf8');

const DIALOGS: Array<{ file: string; op: string }> = [
  { file: 'page-setup-dialog.ts', op: 'pageSetup' },
  { file: 'section-settings-dialog.ts', op: 'sectionSettings' },
  { file: 'column-settings-dialog.ts', op: 'columnSettings' },
  { file: 'page-border-dialog.ts', op: 'pageBorder' },
];

test('양식 모드에서 file:page-setup 은 차단된다(#2361 리뷰 — snapshot 드롭 무언 폐기 방지)', () => {
  // page:setup 은 'page:' prefix 로 차단되지만 파일 메뉴/F7 변형(file:page-setup)은 목록에
  // 없어 양식 모드에서 다이얼로그가 열렸고, 라우팅된 snapshot 이 입력-핸들러 게이트에서
  // 드롭돼 확인이 무언 폐기됐다. 두 진입점의 차단 정합을 핀한다.
  const dispatcher = readFileSync(join(rootDir, 'src/command/dispatcher.ts'), 'utf8');
  const blockedIds = dispatcher.slice(
    dispatcher.indexOf('FORM_MODE_BLOCKED_IDS'),
    dispatcher.indexOf('FORM_MODE_BLOCKED_PREFIXES'),
  );
  assert.match(blockedIds, /'file:page-setup'/, 'file:page-setup 이 FORM_MODE_BLOCKED_IDS 에 있어야 함');
});

for (const { file, op } of DIALOGS) {
  test(`${file} 는 services 주입 + snapshot 라우팅 + fallback 을 갖춘다`, () => {
    const s = src(file);
    // services 를 생성자에 주입(라우터 도달 경로 확보).
    assert.match(s, /services\?:\s*CommandServices/, `${file}: 생성자에 services 주입`);
    assert.match(s, /import type \{ CommandServices \}/, `${file}: CommandServices import`);
    // onConfirm 이 라우터 경유로 snapshot 기록.
    assert.match(s, /this\.services\?\.getInputHandler\(\)/, `${file}: getInputHandler 로 라우터 도달`);
    assert.match(s, new RegExp(`operationType:\\s*'${op}'`), `${file}: ${op} snapshot 라우팅`);
    // services 미주입 환경 호환 fallback(직접 적용 + emit) 유지.
    assert.match(s, /this\.eventBus\.emit\('document-changed'\)/, `${file}: fallback emit 유지`);
  });
}
