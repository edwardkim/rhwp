// 실행: node --test rhwp-chrome/dev-tools-version.test.mjs

import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';

const root = new URL('./', import.meta.url);
const contentScript = fs.readFileSync(new URL('content-script.js', root), 'utf8');
const devToolsScript = fs.readFileSync(new URL('dev-tools-inject.js', root), 'utf8');

test('Chrome DevTools helper uses the manifest version attribute set before injection', () => {
  const attribute = 'data-hwp-extension-version';
  const setAttribute = `setAttribute('${attribute}', EXT_VERSION)`;
  const injectScript = "devScript.src = chrome.runtime.getURL('dev-tools-inject.js')";

  assert.match(contentScript, /const EXT_VERSION = chrome\.runtime\.getManifest\(\)\.version;/);
  assert.ok(contentScript.indexOf(setAttribute) >= 0);
  assert.ok(contentScript.indexOf(setAttribute) < contentScript.indexOf(injectScript));
  assert.match(devToolsScript, new RegExp(`getAttribute\\('${attribute}'\\) \\|\\| 'unknown'`));
  assert.doesNotMatch(contentScript, /version: '0\.2\.8'/);
  assert.doesNotMatch(devToolsScript, /const VERSION = '0\.2\.8'/);
});
