export function openViewer(options = {}) {
  const viewerBase = browser.runtime.getURL('viewer.html');
  const params = new URLSearchParams();

  if (options.url) params.set('url', options.url);
  if (options.filename) params.set('filename', options.filename);

  const query = params.toString();
  const fullUrl = query ? `${viewerBase}?${query}` : viewerBase;

  return browser.tabs.create({ url: fullUrl });
}

export async function openViewerOrReuse(options = {}) {
  const viewerBase = browser.runtime.getURL('viewer.html');
  const tabs = await browser.tabs.query({ url: `${viewerBase}*` });
  const emptyTab = tabs.find((tab) => tab.url === viewerBase);

  if (!emptyTab) {
    return openViewer(options);
  }

  const params = new URLSearchParams();
  if (options.url) params.set('url', options.url);
  if (options.filename) params.set('filename', options.filename);

  return browser.tabs.update(emptyTab.id, {
    url: `${viewerBase}?${params.toString()}`,
    active: true,
  });
}
