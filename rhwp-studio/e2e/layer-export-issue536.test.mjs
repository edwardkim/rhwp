import assert from 'node:assert/strict';
import path from 'node:path';
import os from 'node:os';
import { existsSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import puppeteer from 'puppeteer-core';

// This checks serialized API contracts only: no renderer, pixels or new fixtures.
// Build the current pkg/ WASM and provide the existing samples/ inputs before running.
// CHROMIUM_PATH may select a local Chromium distribution explicitly.
const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(studioRoot, '..');
const playwrightCache = path.join(os.homedir(), '.cache/ms-playwright');
const chromiumCandidates = existsSync(playwrightCache) ? readdirSync(playwrightCache)
  .filter(name => name.startsWith('chromium-')).sort().reverse()
  .flatMap(name => ['chrome-linux64/chrome', 'chrome-linux/chrome'].map(binary => path.join(playwrightCache, name, binary))) : [];
const executablePath = [process.env.CHROMIUM_PATH, '/usr/bin/chromium', '/usr/bin/chromium-browser', ...chromiumCandidates]
  .find(candidate => candidate && existsSync(candidate));
assert.ok(executablePath, 'Chromium is required; set CHROMIUM_PATH to a Chromium executable');
for (const input of ['pkg/rhwp.js', 'samples/test-image.hwpx', 'samples/render-p35-font-native-bitmap.hwpx', 'samples/para-001.hwp']) {
  assert.ok(existsSync(path.join(repoRoot, input)), `Required WASM contract input is missing: ${input}`);
}
const server = await createServer({
  root: studioRoot,
  server: { host: '127.0.0.1', port: 0 },
  plugins: [{
    name: 'layer-export-test-document',
    configureServer(vite) {
      vite.middlewares.use('/__layer_export_test', (_request, response) => {
        response.setHeader('Content-Type', 'text/html');
        response.end('<!doctype html><title>Layer export contract</title>');
      });
    },
  }],
});
let browser;
try {
  await server.listen();
  const address = server.httpServer.address();
  browser = await puppeteer.launch({ executablePath, headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'] });
  const page = await browser.newPage();
  page.setDefaultTimeout(120_000);
  await page.goto(`http://127.0.0.1:${address.port}/__layer_export_test`);
  const result = await page.evaluate(async ({ moduleUrl, samples }) => {
    const wasm = await import(moduleUrl);
    await wasm.default();
    let checks = 0;
    const require = (condition, message) => {
      if (!condition) throw new Error(message);
      checks += 1;
    };
    const bytesBase64 = (bytes) => {
      let text = '';
      for (let offset = 0; offset < bytes.length; offset += 8192) {
        text += String.fromCharCode(...bytes.subarray(offset, offset + 8192));
      }
      return btoa(text);
    };
    const normalized = (value) => JSON.stringify(value, (_key, item) =>
      item instanceof Uint8Array ? bytesBase64(item) : item);
    let imageSlots = 0;
    let fontSlots = 0;
    const measurements = [];
    const pendingStrictSuccess = [];
    for (const sample of samples) {
      const response = await fetch(`/samples/${sample}`);
      require(response.ok, `sample loaded: ${sample}`);
      const doc = new wasm.HwpDocument(new Uint8Array(await response.arrayBuffer()));
      try {
        for (const profile of ['screen', 'print', 'highQuality']) {
          const jsonStart = performance.now();
          const json = JSON.parse(doc.getPageLayerTreeWithProfile(0, profile, false, false));
          const jsonMs = performance.now() - jsonStart;
          const objectStart = performance.now();
          const value = doc.getPageLayerTreeValueWithProfile(0, profile);
          const objectMs = performance.now() - objectStart;
          require(normalized(value) === normalized(json), `${sample}/${profile}: JSON/object parity`);
          require(value.schemaVersion === 1, `${sample}/${profile}: default remains v1`);
          const byKey = doc.getPageLayerTreeValueWithProfileAndResourceKeys(
            0, profile, [], [], true, true,
          );
          require(normalized(byKey) === normalized(JSON.parse(
            doc.getPageLayerTreeWithProfile(0, profile, true, true),
          )), 'opt-in object transport preserves current by-key JSON policy');
          require(byKey.resources.fontBlobs.length === 0, 'font bytes not resent in by-key object transport');
          for (const key of byKey.resources.fontBlobKeys) {
            require(doc.getSourceFontBytes(key).length > 0, 'omitted exact font can be fetched by key');
          }
          for (const [omitFontBytes, omitImageBytes] of [
            [false, false], [true, false], [false, true], [true, true],
          ]) {
            const selective = doc.getPageLayerTreeValueWithProfileAndResourceKeys(
              0, profile, [], [], omitFontBytes, omitImageBytes,
            );
            // The existing JSON API takes image then font flags; the opt-in
            // object overload takes font then image. Check each independently.
            const expected = JSON.parse(doc.getPageLayerTreeWithProfile(
              0, profile, omitImageBytes, omitFontBytes,
            ));
            require(normalized(selective) === normalized(expected),
              `${sample}/${profile}: independent object byte policy ${omitFontBytes}/${omitImageBytes}`);
          }
          const resources = value.resources;
          const hinted = doc.getPageLayerTreeValueWithProfileAndResourceKeys(
            0, profile, resources.imageKeys, resources.svgKeys,
          );
          require(hinted.resources.images.length === resources.images.length,
            `${sample}/${profile}: image indexes preserved`);
          for (let index = 0; index < resources.images.length; index += 1) {
            require(resources.images[index] instanceof Uint8Array, 'image is typed bytes');
            require(hinted.resources.images[index] === undefined, 'known image omitted by content key');
            hinted.resources.images[index] = resources.images[index];
            imageSlots += 1;
          }
          for (let index = 0; index < resources.svgFragments.length; index += 1) {
            require(hinted.resources.svgFragments[index] === undefined, 'known SVG omitted by content key');
            hinted.resources.svgFragments[index] = resources.svgFragments[index];
          }
          require(normalized(hinted) === normalized(value), 'content-key rehydration parity');
          for (const bytes of [...resources.images, ...resources.fontBlobs]) {
            require(bytes instanceof Uint8Array, 'typed resource payload');
            if (bytes.length > 0) bytes[0] ^= 0xff;
          }
          fontSlots += resources.fontBlobs.length;
          require(normalized(doc.getPageLayerTreeValueWithProfile(0, profile)) === normalized(json),
            'caller mutation does not modify document or later exports');
          const v2Json = JSON.parse(doc.getPageLayerTreeV2CompatWithProfile(0, profile));
          const v2Value = doc.getPageLayerTreeValueV2CompatWithProfile(0, profile);
          require(v2Json.schemaVersion === 2 && normalized(v2Value) === normalized(v2Json), 'opt-in v2 parity');
          const v2Hinted = doc.getPageLayerTreeValueV2CompatWithProfileAndResourceKeys(
            0, profile, v2Value.resources.imageKeys, v2Value.resources.svgKeys,
          );
          for (const key of ['images', 'svgFragments']) {
            require(v2Hinted.resources[key].length === v2Value.resources[key].length, 'v2 indexes preserved');
            for (let index = 0; index < v2Value.resources[key].length; index += 1) {
              require(v2Hinted.resources[key][index] === undefined, 'v2 known resource omitted');
              v2Hinted.resources[key][index] = v2Value.resources[key][index];
            }
          }
          require(normalized(v2Hinted) === normalized(v2Value), 'v2 content-key rehydration parity');
          if (sample === 'render-p35-font-native-bitmap.hwpx') {
            for (const kind of ['GlyphOutline', 'GlyphRun']) {
              let strictJsonText;
              try {
                strictJsonText = doc[`getPageLayerTreeV2Strict${kind}WithProfile`](0, profile);
              } catch (issues) {
                // This HWPX is not equivalent to the direct Rust TextRun fixture:
                // its PUA source has a display projection, so the nominal font
                // producer intentionally declines the GlyphRun alternative.
                require(kind === 'GlyphRun' && Array.isArray(issues) && issues.length > 0
                  && issues.every(issue => issue.code === 'strictVisualVariantMissing'),
                `${sample}/${profile}/${kind}: unexpected strict error ${JSON.stringify(issues)}`);
                const pending = [v2Value.root];
                let projectedText = false;
                while (pending.length > 0) {
                  const node = pending.pop();
                  if (node.kind === 'group') pending.push(...node.children);
                  else if (node.kind === 'clipRect') pending.push(node.child);
                  else for (const op of node.ops) {
                    if (op.type !== 'text') continue;
                    for (const variant of op.variants) for (const part of variant.parts) {
                      const payload = part.payload;
                      projectedText ||= payload.type === 'textRun' && payload.displayText !== undefined
                        && payload.displayText !== payload.text;
                    }
                  }
                }
                require(projectedText, 'strict rejection is backed by a projected source text slot');
                const rejectingMethods = [
                  ['getPageLayerTreeValueV2StrictGlyphRunWithProfile', [0, profile]],
                  ['getPageLayerTreeValueV2StrictGlyphRunWithProfileAndResourceKeys', [0, profile, [], []]],
                  ...(profile === 'screen' ? [
                    ['getPageLayerTreeV2StrictGlyphRun', [0]],
                    ['getPageLayerTreeValueV2StrictGlyphRun', [0]],
                  ] : []),
                ];
                for (const [method, args] of rejectingMethods) {
                  let actual;
                  try { doc[method](...args); } catch (error) { actual = error; }
                  require(normalized(actual) === normalized(issues),
                    `${method}: projected text rejection retains the structured JSON issues`);
                }
                // Keep the eligible success branch below; report its missing
                // fixture coverage explicitly rather than counting this rejection as success.
                pendingStrictSuccess.push({ sample, profile, kind, issues });
                continue;
              }
              const strictJson = JSON.parse(strictJsonText);
              const strictValue = doc[`getPageLayerTreeValueV2Strict${kind}WithProfile`](0, profile);
              require(normalized(strictValue) === normalized(strictJson), `${kind}: proven strict JSON/object parity`);
              require(strictJson.textV2.strictVisualFallbackFree === true, `${kind}: real fixture is fallback-free`);
              const nodes = [strictJson.root];
              let slots = 0;
              while (nodes.length > 0) {
                const node = nodes.pop();
                if (node.kind === 'group') nodes.push(...node.children);
                else if (node.kind === 'clipRect') nodes.push(node.child);
                else for (const op of node.ops) {
                  if (op.type !== 'text') continue;
                  require(op.fallbackPolicy === 'none' && op.variants.length === 1,
                    `${kind}: no compatibility text silently retained`);
                  slots += 1;
                }
              }
              require(slots > 0, `${kind}: strict success is not an empty document`);
              const hintedStrict = doc[`getPageLayerTreeValueV2Strict${kind}WithProfileAndResourceKeys`](
                0, profile, strictValue.resources.imageKeys, strictValue.resources.svgKeys,
              );
              for (const key of ['images', 'svgFragments']) {
                for (let index = 0; index < strictValue.resources[key].length; index += 1) {
                  require(hintedStrict.resources[key][index] === undefined, `${kind}: known strict resource omitted`);
                  hintedStrict.resources[key][index] = strictValue.resources[key][index];
                }
              }
              require(normalized(hintedStrict) === normalized(strictValue), `${kind}: strict resource rehydration parity`);
              if (profile === 'screen') {
                require(normalized(JSON.parse(doc[`getPageLayerTreeV2Strict${kind}`](0))) === normalized(strictJson),
                  `${kind}: strict default JSON alias`);
                require(normalized(doc[`getPageLayerTreeValueV2Strict${kind}`](0)) === normalized(strictValue),
                  `${kind}: strict default object alias`);
              }
            }
          }
          measurements.push({ sample, profile, jsonMs, objectMs });
        }
        require(normalized(doc.getPageLayerTreeValue(0)) === normalized(JSON.parse(doc.getPageLayerTree(0))),
          'default object alias');
        require(normalized(doc.getPageLayerTreeValueV2Compat(0)) === normalized(JSON.parse(doc.getPageLayerTreeV2Compat(0))),
          'default v2 object alias');
        for (const keys of [{}, [1], Array(4097).fill('x'), ['x'.repeat(1025)]]) {
          let threw = false;
          try { doc.getPageLayerTreeValueWithProfileAndResourceKeys(0, 'screen', keys, []); } catch { threw = true; }
          require(threw, 'invalid resource hints rejected');
        }
        let invalidProfile = false;
        try { doc.getPageLayerTreeValueWithProfile(0, 'not-a-profile'); } catch { invalidProfile = true; }
        require(invalidProfile, 'invalid profile rejected');
        if (sample === 'render-p35-font-native-bitmap.hwpx') {
          // The same embedded smoke font has a bitmap-only and a SVG-only glyph.
          // Use the real editing API to exercise a new document generation.
          doc.replaceOne('\uE100', '\uE101', true);
          const svgValue = doc.getPageLayerTreeValueV2Compat(0);
          require(svgValue.resources.svgFragments.length > 0, 'edited real fixture exercises font SVG resources');
          const svgHinted = doc.getPageLayerTreeValueV2CompatWithProfileAndResourceKeys(
            0, 'screen', [], svgValue.resources.svgKeys,
          );
          for (let index = 0; index < svgValue.resources.svgFragments.length; index += 1) {
            require(svgHinted.resources.svgFragments[index] === undefined, 'known real SVG omitted');
            svgHinted.resources.svgFragments[index] = svgValue.resources.svgFragments[index];
          }
          require(normalized(svgHinted) === normalized(svgValue), 'real SVG rehydration parity after edit');
        }
      } finally { doc.free(); }
    }
    const response = await fetch('/samples/para-001.hwp');
    const doc = new wasm.HwpDocument(new Uint8Array(await response.arrayBuffer()));
    try {
      for (const kind of ['GlyphOutline', 'GlyphRun']) {
        for (const [method, args] of [
          [`getPageLayerTreeV2Strict${kind}`, [0]],
          [`getPageLayerTreeV2Strict${kind}WithProfile`, [0, 'print']],
          [`getPageLayerTreeValueV2Strict${kind}`, [0]],
          [`getPageLayerTreeValueV2Strict${kind}WithProfile`, [0, 'print']],
          [`getPageLayerTreeValueV2Strict${kind}WithProfileAndResourceKeys`, [0, 'print', [], []]],
        ]) {
          let error;
          try { doc[method](...args); } catch (caught) { error = caught; }
          require(Array.isArray(error) && error.length > 0 && typeof error[0].code === 'string',
            `${method}: unproven fallback-free text returns structured validation issues`);
        }
      }
    } finally { doc.free(); }
    require(imageSlots > 0, 'real fixture exercises image resources');
    const pendingCoverage = [];
    if (pendingStrictSuccess.length > 0) {
      pendingCoverage.push({ contract: 'strictGlyphRunSuccess',
        reason: 'existing bitmap HWPX projects PUA source text; a proven success fixture is still needed',
        cases: pendingStrictSuccess });
    }
    if (fontSlots === 0) {
      pendingCoverage.push({ contract: 'portableFontResourceSuccess',
        reason: 'no fixture in this harness emitted portable font bytes' });
    }
    return { coverage: pendingCoverage.length > 0 ? 'partial' : 'complete',
      checks, imageSlots, fontSlots, pendingCoverage, measurements };
  }, {
    moduleUrl: `/@fs${path.join(repoRoot, 'pkg/rhwp.js')}`,
    samples: ['test-image.hwpx', 'render-p35-font-native-bitmap.hwpx'],
  });
  assert.ok(result.checks > 30);
  console.log(JSON.stringify(result));
} finally {
  if (browser) await browser.close();
  await server.close();
}
