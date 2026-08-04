import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const recoveryUi = readFileSync(new URL('../src/recovery/recovery-ui.ts', import.meta.url), 'utf8');
const dialogs = readFileSync(new URL('../src/styles/dialogs.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

test('recovery dialog scopes viewport containment and scrolling to the draft list', () => {
  assert.match(recoveryUi, /this\.dialog\.classList\.add\('recovery-dialog'\)/);
  assert.match(recoveryUi, /body\.classList\.add\('recovery-dialog-body'\)/);
  assert.match(recoveryUi, /list\.classList\.add\('recovery-draft-list'\)/);
  assert.match(recoveryUi, /text\.classList\.add\('recovery-draft-copy'\)/);
  assert.match(recoveryUi, /title\.classList\.add\('recovery-draft-title'\)/);

  assert.match(
    dialogs,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*display:\s*flex;[^}]*flex-direction:\s*column;[^}]*max-height:\s*calc\(100dvh - 32px\);[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    dialogs,
    /\.recovery-dialog\s*>\s*\.recovery-dialog-body\s*\{[^}]*display:\s*flex;[^}]*flex-direction:\s*column;[^}]*min-height:\s*0;[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    dialogs,
    /\.recovery-draft-list\s*\{[^}]*flex:\s*1 1 auto;[^}]*min-height:\s*0;[^}]*overflow-y:\s*auto;/s,
  );
  assert.match(dialogs, /\.recovery-draft-copy\s*\{[^}]*min-width:\s*0;/s);
  assert.match(dialogs, /\.recovery-draft-title\s*\{[^}]*overflow-wrap:\s*anywhere;/s);
  assert.match(
    responsive,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*max-height:\s*90dvh;[^}]*overflow:\s*hidden;/s,
  );
});
