import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const recoveryUi = readFileSync(new URL('../src/recovery/recovery-ui.ts', import.meta.url), 'utf8');
const dialogs = readFileSync(new URL('../src/styles/dialogs.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

test('recovery dialog retains its viewport and draft-list hooks', () => {
  assert.match(recoveryUi, /this\.dialog\.classList\.add\('recovery-dialog'\)/);
  assert.match(recoveryUi, /body\.classList\.add\('recovery-dialog-body'\)/);
  assert.match(recoveryUi, /list\.classList\.add\('recovery-draft-list'\)/);
  assert.match(recoveryUi, /text\.classList\.add\('recovery-draft-copy'\)/);
  assert.match(recoveryUi, /title\.classList\.add\('recovery-draft-title'\)/);

  assert.match(
    dialogs,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*display:\s*flex;[^}]*flex-direction:\s*column;[^}]*max-height:\s*calc\(100dvh - 32px\);[^}]*overflow:\s*hidden;/s,
  );
  // #9133: 본문 overflow:hidden·목록 min-height:0 핀은 낮은 화면에서 목록을 0px로 만든다.
  // 실제 목록/본문 스크롤과 복구·나중에 동작은 dialog-viewport-issue9133.test.mjs에서 검증한다.
  assert.match(dialogs, /\.recovery-draft-copy\s*\{[^}]*min-width:\s*0;/s);
  assert.match(dialogs, /\.recovery-draft-title\s*\{[^}]*overflow-wrap:\s*anywhere;/s);
  assert.match(
    responsive,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*max-height:\s*90dvh;[^}]*overflow:\s*hidden;/s,
  );
});
