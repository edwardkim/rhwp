import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const FONT_FILES = readdirSync(path.join(ROOT, 'web/fonts'))
  .filter((file) => file.endsWith('.woff2'))
  .sort();

for (const browser of ['chrome', 'firefox']) {
  test(`${browser} extension dist contract`, () => {
    const sourceDir = path.join(ROOT, `rhwp-${browser}`);
    const distDir = path.join(sourceDir, 'dist');
    const sourceManifest = readJson(path.join(sourceDir, 'manifest.json'));
    const distManifest = readJson(path.join(distDir, 'manifest.json'));

    assert.deepEqual(distManifest, sourceManifest);
    assert.match(sourceManifest.content_security_policy.extension_pages, /'wasm-unsafe-eval'/);
    assert.deepEqual(
      sourceManifest.web_accessible_resources[0].resources,
      ['wasm/*', 'fonts/*', 'icons/*', 'dev-tools-inject.js'],
    );

    const viteConfig = readFileSync(path.join(sourceDir, 'vite.config.ts'), 'utf8');
    assert.match(viteConfig, /publicDir:\s*false/);

    const viewerHtml = readFileSync(path.join(distDir, 'viewer.html'), 'utf8');
    const inlineScripts = findInlineScriptTags(viewerHtml);
    assert.equal(inlineScripts.length, 0, 'viewer.html must not contain inline scripts');
    assertInlineScriptDetectorRejectsMalformedEndTags();

    assert.deepEqual(
      readdirSync(path.join(distDir, 'fonts')).filter((file) => file.endsWith('.woff2')).sort(),
      FONT_FILES,
    );
    for (const file of ['rhwp.js', 'rhwp.d.ts', 'rhwp_bg.wasm', 'rhwp_bg.wasm.d.ts']) {
      assert.ok(statSync(path.join(distDir, 'wasm', file)).size > 0, `${file} must be copied`);
    }
  });
}

test('safari manifest keeps the stricter WAR surface', () => {
  const manifest = readJson(path.join(ROOT, 'rhwp-safari/src/manifest.json'));
  assert.deepEqual(
    manifest.web_accessible_resources[0].resources,
    ['wasm/*', 'fonts/*', 'icons/*'],
  );
  assert.doesNotMatch(JSON.stringify(manifest.web_accessible_resources), /dev-tools-inject\.js/);
});

function readJson(file) {
  return JSON.parse(readFileSync(file, 'utf8'));
}

function findInlineScriptTags(html) {
  return [...html.matchAll(/<script\b([^>]*)>/gi)]
    .filter((match) => !/(?:^|[\t\n\f\r ])src[\t\n\f\r ]*=/i.test(match[1]));
}

function assertInlineScriptDetectorRejectsMalformedEndTags() {
  for (const html of [
    '<script>alert(1)</script >',
    '<script>alert(1)</script foo="bar">',
    '<script data-src="/ignored.js">alert(1)</script>',
  ]) {
    assert.equal(findInlineScriptTags(html).length, 1, `inline script must be detected: ${html}`);
  }
  assert.equal(findInlineScriptTags('<script src="/app.js"></script>').length, 0);
}
