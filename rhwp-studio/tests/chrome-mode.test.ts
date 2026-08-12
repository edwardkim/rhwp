import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  EMBED_HIDDEN_EDIT_COMMAND_IDS,
  EMBED_HIDDEN_FILE_COMMAND_IDS,
  isEmbedSwallowedFileShortcut,
  resolveChromeMode,
  resolveChromeModeRequest,
} from '../src/ui/chrome-mode.ts';
import { CommandRegistry } from '../src/command/registry.ts';
import { defaultShortcuts, matchShortcut } from '../src/command/shortcut-map.ts';

test('chrome 프로파일 리졸버는 full을 기본값으로 두고 embed만 opt-in으로 받는다', () => {
  assert.equal(resolveChromeMode(''), 'full');
  assert.equal(resolveChromeMode('?chrome=embed'), 'embed');
  assert.equal(resolveChromeMode('?chrome=EMBED'), 'embed');
  assert.equal(resolveChromeMode('?chrome=full'), 'full');
  assert.deepEqual(resolveChromeModeRequest(''), { mode: 'full', source: 'default' });
  assert.deepEqual(resolveChromeModeRequest('?chrome=embed'), {
    mode: 'embed',
    source: 'url',
    requested: 'embed',
  });
  assert.deepEqual(resolveChromeModeRequest('?chrome=full'), {
    mode: 'full',
    source: 'url',
    requested: 'full',
  });
});

test('미지원 chrome 값은 full로 폴백하고 이유를 보고한다', () => {
  assert.equal(resolveChromeMode('?chrome=kiosk'), 'full');
  assert.deepEqual(resolveChromeModeRequest('?chrome=kiosk'), {
    mode: 'full',
    source: 'url',
    requested: 'kiosk',
    unsupportedReason: 'unsupportedChromeMode',
  });
  // 빈 값은 명시 요청이 아니라 기본으로 취급한다 (renderer 리졸버와 동일).
  assert.deepEqual(resolveChromeModeRequest('?chrome='), { mode: 'full', source: 'default' });
});

test('chrome 프로파일 리졸버는 저장소로 지속하는 opt-in 경로를 두지 않는다', () => {
  const source = readFileSync(new URL('../src/ui/chrome-mode.ts', import.meta.url), 'utf8');
  assert.equal(source.includes('localStorage'), false);
  assert.equal(source.includes('persistChromeMode'), false);
});

test('embed 프로파일이 걸러내는 커맨드는 전부 fileCommands에 실존한다', () => {
  const commandSource = readFileSync(
    new URL('../src/command/commands/file.ts', import.meta.url),
    'utf8',
  );
  assert.ok(EMBED_HIDDEN_FILE_COMMAND_IDS.length > 0);
  for (const id of EMBED_HIDDEN_FILE_COMMAND_IDS) {
    assert.match(commandSource, new RegExp(`id: '${id}',`), id);
  }
});

test('embed 프로파일도 편집 용지와 제품 정보 표면은 유지한다', () => {
  assert.equal(EMBED_HIDDEN_FILE_COMMAND_IDS.includes('file:page-setup'), false);
  assert.equal(EMBED_HIDDEN_FILE_COMMAND_IDS.includes('file:about'), false);
});

test('main은 embed에서 수명주기 커맨드 등록만 거르고 메뉴는 런타임에 정리한다', () => {
  const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  assert.match(mainSource, /resolveChromeModeRequest\(window\.location\.search\)/);
  assert.match(mainSource, /fileCommands\.filter\(/);
  assert.match(mainSource, /EMBED_HIDDEN_FILE_COMMAND_IDS/);
  assert.match(mainSource, /pruneEmbedChrome\(\)/);
  // 자동저장 복구 다이얼로그의 드래프트 복원도 호스트가 감지할 수 없는 문서 교체
  // 경로이므로 embed에서는 띄우지 않는다.
  assert.match(
    mainSource,
    /if \(chromeMode !== 'embed'\) void offerAutosaveRecoveryIfIdle\(\);/,
  );
  // index.html은 수정하지 않는다 — 기본 full 프로파일의 정적 마크업 검사가 그대로 유효하다.
  const indexHtml = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
  assert.match(indexHtml, /data-cmd="file:save"/);
  assert.match(indexHtml, /data-recent/);
});

test('shortcut-map은 embed 프로파일과 무관하게 파일 단축키 매핑을 유지한다', () => {
  // 매핑이 남아야 Ctrl+S/Ctrl+P가 preventDefault로 계속 삼켜져 브라우저 저장/인쇄
  // 대화상자로 빠지지 않고, Ctrl+Shift+S가 후순위 table:block-sum 매핑으로
  // 폴스루하지 않는다. 미등록 커맨드 dispatch는 무해하게 false를 반환한다.
  const shortcutSource = readFileSync(
    new URL('../src/command/shortcut-map.ts', import.meta.url),
    'utf8',
  );
  assert.match(shortcutSource, /'file:save'\]/);
  assert.match(shortcutSource, /'file:save-as'\]/);
  assert.match(shortcutSource, /'file:print'\]/);
  assert.doesNotMatch(shortcutSource, /chrome-mode|ChromeMode|EMBED_HIDDEN/);
});

test('embed 프로파일이 걸러내는 편집 커맨드는 문서 비교뿐이고 editCommands에 실존한다', () => {
  // 문서 비교는 비교 실행 시 오른쪽 문서를 현재 에디터에 로드하는 문서 교체 진입점이다.
  assert.deepEqual([...EMBED_HIDDEN_EDIT_COMMAND_IDS], ['edit:compare-documents']);
  const editSource = readFileSync(
    new URL('../src/command/commands/edit.ts', import.meta.url),
    'utf8',
  );
  assert.match(editSource, /id: 'edit:compare-documents',/);
});

test('embed에서는 메뉴·도구막대·단축키·팔레트 어느 경로로도 문서 비교가 실행되지 않는다', () => {
  const registry = new CommandRegistry();
  const stubDefs = ['edit:compare-documents', 'edit:find'].map((id) => ({
    id,
    label: id,
    execute: () => {},
  }));
  // main.ts의 embed 등록 필터와 동일한 형태.
  registry.registerAll(stubDefs.filter((cmd) => !EMBED_HIDDEN_EDIT_COMMAND_IDS.includes(cmd.id)));

  // 메뉴 클릭·도구막대 버튼·split 메뉴 항목·단축키는 전부 dispatcher.dispatch를
  // 지나고, dispatch는 registry 조회로 시작한다 — 미등록이면 어떤 경로도 실행에
  // 이르지 못한다.
  assert.equal(registry.has('edit:compare-documents'), false);
  assert.equal(registry.get('edit:compare-documents'), undefined);
  // 커맨드 팔레트는 registry 열거로 목록을 만든다 — 미등록이면 목록에 없다.
  assert.equal(registry.getAllIds().includes('edit:compare-documents'), false);
  // 걸러지지 않은 편집 커맨드는 남는다 — 필터가 과도하게 넓지 않다.
  assert.equal(registry.has('edit:find'), true);

  // 단축키 매핑은 남지만(다른 커맨드로의 폴스루 방지) 미등록 dispatch는 무해하게
  // false를 반환한다 — dispatcher는 Node 타입 스트리핑이 지원하지 않는 문법을 써서
  // 계약을 소스로 잠근다.
  const shortcutEvent = {
    key: 'v',
    code: 'KeyV',
    altKey: true,
    shiftKey: true,
    ctrlKey: false,
    metaKey: false,
  } as KeyboardEvent;
  assert.equal(matchShortcut(shortcutEvent, defaultShortcuts, 'other'), 'edit:compare-documents');
  const dispatcherSource = readFileSync(
    new URL('../src/command/dispatcher.ts', import.meta.url),
    'utf8',
  );
  assert.match(dispatcherSource, /if \(!def\) \{\n\s*console\.warn\([^)]*\);\n\s*return false;\n\s*\}/);
});

test('main은 embed에서 문서 비교 표면을 등록·노출하지 않는다', () => {
  const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  assert.match(mainSource, /editCommands\.filter\(/);
  assert.match(mainSource, /EMBED_HIDDEN_EDIT_COMMAND_IDS/);
  // 런타임 정리는 숨김 커맨드를 참조하는 모든 표면을 data-cmd로 일괄 제거한다.
  assert.match(mainSource, /document\.querySelectorAll\(`\[data-cmd="\$\{id\}"\]`\)/);
  // index.html은 수정하지 않는다 — full 프로파일의 세 표면(편집 메뉴·도구막대
  // 버튼·split 메뉴 항목)이 그대로 남아 있어야 한다.
  const indexHtml = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
  const surfaces = indexHtml.match(/data-cmd="edit:compare-documents"/g) ?? [];
  assert.equal(surfaces.length, 3);
});

test('embed 저장·인쇄 단축키 판정은 문서 로드 여부와 무관한 순수 함수다', () => {
  const ev = (input: Partial<Parameters<typeof isEmbedSwallowedFileShortcut>[0]>) => ({
    key: '',
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...input,
  });
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 's', ctrlKey: true })), true);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'S', ctrlKey: true, shiftKey: true })), true);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'p', ctrlKey: true })), true);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 's', metaKey: true })), true);
  // 한글 IME 키 — 전역 핸들러의 ㅜ/ㅐ 처리와 같은 이유.
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'ㄴ', ctrlKey: true })), true);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'ㅔ', ctrlKey: true })), true);
  // Ctrl+O/Alt+N은 전역 단축키 핸들러가 문서 유무와 무관하게 이미 삼킨다.
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'o', ctrlKey: true })), false);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 's' })), false);
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 's', ctrlKey: true, altKey: true })), false);
  // Ctrl+Shift+P는 크롬의 시스템 인쇄 대화상자 경로 — 함께 삼킨다.
  assert.equal(isEmbedSwallowedFileShortcut(ev({ key: 'p', ctrlKey: true, shiftKey: true })), true);

  // main 배선: capture 단계 리스너라 다이얼로그 등의 stopPropagation보다 먼저 돈다.
  const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  assert.match(
    mainSource,
    /document\.addEventListener\('keydown', \(e\) => \{\n\s*if \(isEmbedSwallowedFileShortcut\(e\)\) e\.preventDefault\(\);\n\s*\}, true\);/,
  );
});

test('embed의 unsaved guard는 로컬 저장 선택지를 막고 자동 discard는 하지 않는다', () => {
  // 이 경로의 저장은 registry를 우회한 직접 호출(saveCurrentDocument)이라 커맨드
  // 미등록으로는 닫히지 않는다 — 다이얼로그 canSave로 막는다.
  const fileSource = readFileSync(
    new URL('../src/command/commands/file.ts', import.meta.url),
    'utf8',
  );
  assert.match(fileSource, /allowLocalSave\?: boolean/);
  assert.match(fileSource, /canSave: allowLocalSave/);
  assert.match(fileSource, /if \(!allowLocalSave\) return false;/);

  const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  assert.match(mainSource, /allowLocalSave: chromeMode !== 'embed'/);

  // 다이얼로그는 저장 비활성 사유를 받아 표시하고, '저장 안 함'/'취소' 선택은
  // 사용자에게 남는다(자동 discard 없음).
  const dialogSource = readFileSync(
    new URL('../src/ui/unsaved-changes-dialog.ts', import.meta.url),
    'utf8',
  );
  assert.match(dialogSource, /saveUnavailableReason\?: string/);
  assert.match(dialogSource, /'discard'/);
  assert.match(dialogSource, /'cancel'/);
});
