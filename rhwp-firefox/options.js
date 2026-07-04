// rhwp Firefox 옵션 페이지 스크립트
// options.html에서 분리 — Firefox MV3 CSP 호환

document.getElementById('title').textContent = browser.i18n.getMessage('optionsTitle');
document.getElementById('labelAutoOpen').textContent = browser.i18n.getMessage('optionsAutoOpen');
document.getElementById('labelShowBadges').textContent = browser.i18n.getMessage('optionsShowBadges');
document.getElementById('labelHoverPreview').textContent = browser.i18n.getMessage('optionsHoverPreview');
document.getElementById('labelDisableExternalWebFonts').textContent = browser.i18n.getMessage('optionsDisableExternalWebFonts');
document.getElementById('descDisableExternalWebFonts').textContent = browser.i18n.getMessage('optionsDisableExternalWebFontsDesc');
document.getElementById('saved').textContent = browser.i18n.getMessage('optionsSaved');
document.getElementById('privacy').textContent = browser.i18n.getMessage('optionsPrivacy');
document.getElementById('version').textContent = browser.runtime.getManifest().version;

const inputs = ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts'];

async function loadSettings() {
  const settings = await browser.storage.sync.get({
    autoOpen: true,
    showBadges: true,
    hoverPreview: true,
    disableExternalWebFonts: false
  });
  for (const id of inputs) {
    document.getElementById(id).checked = settings[id];
  }
}

async function saveSettings() {
  const settings = {};
  for (const id of inputs) {
    settings[id] = document.getElementById(id).checked;
  }
  await browser.storage.sync.set(settings);
  const saved = document.getElementById('saved');
  saved.classList.add('show');
  setTimeout(() => saved.classList.remove('show'), 1500);
}

for (const id of inputs) {
  document.getElementById(id).addEventListener('change', () => {
    saveSettings().catch((err) => console.error('[rhwp] 옵션 저장 오류:', err));
  });
}

loadSettings().catch((err) => console.error('[rhwp] 옵션 로드 오류:', err));
