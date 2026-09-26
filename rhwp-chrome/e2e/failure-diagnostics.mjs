import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

export function recordConsoleMessage(diagnostic, message, pageUrl) {
  const sourceUrl = message.location().url;
  const entry = {
    stage: diagnostic.stage, type: message.type(), text: message.text(),
    url: sourceUrl || pageUrl, pageUrl,
  };
  diagnostic.console.push(entry);
  if (entry.type !== 'error') return;
  // chrome://downloads is browser-owned UI. Preserve its resource diagnostics;
  // the history's visible filenames and zero-viewer assertions remain mandatory.
  // Attribute by source URL so a late extension/fixture error still fails.
  if (sourceUrl?.startsWith('chrome://')) {
    (diagnostic.browserUiErrors ??= []).push(entry);
  } else {
    diagnostic.errors.push(entry.text);
  }
}

export async function observePageDiagnostics(browser, diagnostic) {
  const seen = new WeakSet();
  const append = (key, value) => {
    diagnostic[key] ??= [];
    diagnostic[key].push(value);
    if (diagnostic[key].length > 200) diagnostic[key].shift();
  };
  const attach = page => {
    if (!page || seen.has(page)) return;
    seen.add(page);
    page.on('console', message => append('console', {
      stage: diagnostic.stage, type: message.type(), text: message.text(), url: page.url(),
    }));
    page.on('pageerror', error => append('pageErrors', {
      stage: diagnostic.stage, message: error.message, url: page.url(),
    }));
  };
  browser.on('targetcreated', target => {
    if (target.type() === 'page') target.page().then(attach).catch(error => append('diagnosticErrors', String(error)));
  });
  for (const page of await browser.pages()) attach(page);
}

// Called before browser/profile cleanup. Only allowlisted JSON and PNG outputs
// leave the harness; the source documents and Chrome profile are never copied.
export async function saveFailureDiagnostics(browser, name, diagnostic) {
  const output = process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR;
  if (!output) return;
  await mkdir(output, { recursive: true });
  const targets = browser?.targets() || [];
  const summary = {
    ...diagnostic,
    openExtensionUrls: targets.filter(target => target.url().startsWith('chrome-extension://')).map(target => target.url()),
    workers: targets.filter(target => target.type() === 'service_worker').map(target => target.url()),
  };
  await writeFile(path.join(output, `${name}.json`), JSON.stringify(summary, null, 2));
  const pages = browser ? await browser.pages() : [];
  for (const [index, page] of pages.filter(page => page.url().startsWith('chrome-extension://')).slice(0, 3).entries()) {
    let timer;
    try {
      await Promise.race([
        page.screenshot({ path: path.join(output, `${name}-${index}.png`) }),
        new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('screenshot timeout')), 2_000); }),
      ]);
    } finally { clearTimeout(timer); }
  }
}
