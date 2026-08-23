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
//
// [Task #2370 클러스터 C] 라우팅 경로를 공용 헬퍼 `applyThroughRouter`(ui/dialog-apply.ts)로
// 통일했다 — 라우터 도달·fallback·실패 처리가 다이얼로그마다 제각각이던 것을 한 자리로 모은다.
// 따라서 가드도 "직접 getInputHandler 를 부르는가" 대신 "헬퍼를 경유하는가"를 본다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, `src/ui/${rel}`), 'utf8');

const DIALOGS: Array<{ file: string; op: string; commandOnly?: boolean }> = [
  { file: 'page-setup-dialog.ts', op: 'pageSetup' },
  // [#5769 후속2] 현재 구역·all 모두 역연산화 — snapshot 라우팅이 없다.
  { file: 'section-settings-dialog.ts', op: 'sectionSettings', commandOnly: true },
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

for (const { file, op, commandOnly } of DIALOGS) {
  test(`${file} 는 services 주입 + 공용 헬퍼 라우팅 + fallback 을 갖춘다`, () => {
    const s = src(file);
    // services 를 생성자에 주입(라우터 도달 경로 확보).
    assert.match(s, /services\?:\s*CommandServices/, `${file}: 생성자에 services 주입`);
    assert.match(s, /import type \{ CommandServices \}/, `${file}: CommandServices import`);
    // onConfirm 이 공용 헬퍼 경유로 기록(헬퍼가 getInputHandler 도달을 담당).
    // [#5769 Stage 4→후속2] 헬퍼 계열 확장 수용 — snapshot(applyThroughRouter)과
    // command(applyCommandThroughRouter) 어느 쪽이든 공용 헬퍼 경유 자체를 핀한다.
    assert.match(s, /import \{[^}]*apply(Command)?ThroughRouter[^}]*\} from '\.\/dialog-apply'/, `${file}: 공용 헬퍼 import`);
    assert.match(s, /return apply(Command)?ThroughRouter\(\{/, `${file}: onConfirm 이 헬퍼 결과를 반환`);
    if (!commandOnly) {
      assert.match(s, new RegExp(`operationType:\\s*'${op}'`), `${file}: ${op} snapshot 라우팅`);
    }
    // services 미주입 환경 호환 fallback(직접 적용 + emit) 유지.
    assert.match(s, /fallback: \(\) => \{[^}]*emit\('document-changed'\)/, `${file}: fallback emit 유지`);
    // 실패 처리를 다이얼로그가 따로 두면 표준화가 다시 갈라진다.
    assert.doesNotMatch(s, /catch[^\n]*\n[^\n]*적용 실패/, `${file}: 실패 처리는 헬퍼가 담당`);
  });
}

test('[#5769 Stage 4→후속2] section-settings 는 현재 구역·문서 전체 모두 속성쌍 커맨드로 역연산화한다', () => {
  const s = src('section-settings-dialog.ts');
  assert.match(s, /new SetSectionPropsCommand\(/,
    '현재 구역 적용은 raw 저널 포함 속성쌍 커맨드로 기록해야 한다');
  // [#5769 후속2] all 도 역연산화 — 다구역 raw 저널 커맨드로 스냅샷을 대체했다.
  assert.match(s, /new SetSectionPropsAllCommand\(/,
    '문서 전체(all) 적용은 다구역 저널 속성쌍 커맨드로 기록해야 한다');
  assert.doesNotMatch(s, /kind:\s*'snapshot'/,
    'section-settings 에 snapshot 라우팅 잔류는 슬롯을 다시 먹는다');
  assert.match(s, /kind: 'command',/, '커맨드 경로는 kind:command 다');

  // 커맨드 본체의 저널 배선 핀 — 캡처→적용 순서와 undo 의 old 재적용→복원 순서.
  const cmdSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');
  const pin = (className: string, all: boolean) => {
    const start = cmdSrc.indexOf(`export class ${className}`);
    assert(start !== -1, `${className} 이 있어야 한다`);
    const rest = cmdSrc.slice(start);
    const body = rest.slice(0, rest.indexOf('\nexport class ', 1) === -1 ? undefined : rest.indexOf('\nexport class ', 1));
    assert.match(body, all ? /setSectionDefAll/ : /wasm\.setSectionDef\(this\.sectionIdx, this\.after\)/,
      'execute 는 after 를 적용해야 한다');
    assert.match(body, new RegExp(all
      ? 'setSectionDef\\(s\\.idx, s\\.before\\)'
      : 'wasm\\.setSectionDef\\(this\\.sectionIdx, this\\.before\\)'),
      'undo 는 old 재적용으로 raw 를 재무효화해야 한다');
    assert.match(body, /restoreSectionRaw/,
      'undo 는 old 재적용 뒤 raw 를 복원해야 한다');
    assert.match(body, /snapshotResourceCount\(\): number \{ return 0; \}/,
      '속성쌍 경로는 스냅샷 예산을 쓰지 않는다');
  };
  pin('SetSectionPropsCommand', false);
  pin('SetSectionPropsAllCommand', true);
});
