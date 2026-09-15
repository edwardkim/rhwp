import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { LOCALE_STORAGE_KEY, LOCALE_QUERY_PARAM } from '../src/i18n/resolve.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const initSource = readFileSync(join(rootDir, 'public', 'locale-init.js'), 'utf8');

// public/locale-init.js 는 번들 밖에서 첫 페인트 전에 도는 별도 스크립트라 resolve.ts 를
// import 할 수 없다. 그래서 저장소 키와 질의 이름을 따로 적는다 — 둘이 어긋나면 영어 사용자가
// 첫 페인트에 한국어를 보고 번들이 뜬 뒤 바뀌는 깜빡임이 되돌아온다. 이 테스트가 그 끈이다.
test('locale-init.js 는 resolve.ts 와 같은 저장소 키를 읽는다', () => {
  assert.ok(initSource.includes(`localStorage.getItem('${LOCALE_STORAGE_KEY}')`),
    `locale-init.js 가 '${LOCALE_STORAGE_KEY}' 를 읽어야 한다`);
});

test('locale-init.js 는 resolve.ts 와 같은 질의 이름을 읽는다', () => {
  assert.ok(initSource.includes(`.get('${LOCALE_QUERY_PARAM}')`),
    `locale-init.js 가 ?${LOCALE_QUERY_PARAM}= 을 읽어야 한다`);
});

test('locale-init.js 는 브라우저 언어를 보지 않는다 — resolve.ts 의 기본과 같아야 한다', () => {
  assert.ok(!/navigator\.language/.test(initSource));
});

test('index.html 은 theme-init 바로 뒤에서 locale-init 을 동기 로드한다', () => {
  const html = readFileSync(join(rootDir, 'index.html'), 'utf8');
  const theme = html.indexOf('<script src="/theme-init.js"></script>');
  const locale = html.indexOf('<script src="/locale-init.js"></script>');
  assert.ok(theme >= 0 && locale > theme, 'locale-init 은 theme-init 뒤, 번들 앞이어야 한다');
  assert.ok(!/<script[^>]*src="\/locale-init\.js"[^>]*(defer|type="module")/.test(html),
    'defer/module 이면 번들 뒤에 돌아 깜빡임을 못 막는다');
});
