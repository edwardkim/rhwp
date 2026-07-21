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

const INPUTS = ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts'];
const DEFAULTS = { autoOpen: true, showBadges: true, hoverPreview: true, disableExternalWebFonts: false };

function isAllDefaults(s) {
  return s.autoOpen === DEFAULTS.autoOpen
    && s.showBadges === DEFAULTS.showBadges
    && s.hoverPreview === DEFAULTS.hoverPreview
    && s.disableExternalWebFonts === DEFAULTS.disableExternalWebFonts;
}

function applySettings(settings) {
  for (const id of INPUTS) {
    document.getElementById(id).checked = settings[id];
  }
}

async function loadSettings() {
  try {
    const syncSettings = await browser.storage.sync.get(DEFAULTS);
    if (!isAllDefaults(syncSettings)) {
      applySettings(syncSettings);
      return;
    }
  } catch (err) {
    console.error('[rhwp] Sync 설정 읽기 오류:', err);
  }
  try {
    const localSettings = await browser.storage.local.get(DEFAULTS);
    if (!isAllDefaults(localSettings)) {
      applySettings(localSettings);
      return;
    }
  } catch (err) {
    console.error('[rhwp] Local 설정 읽기 오류:', err);
  }
  applySettings(DEFAULTS);
}

async function saveSettings() {
  const settings = {};
  for (const id of INPUTS) {
    settings[id] = document.getElementById(id).checked;
  }
  try { await browser.storage.sync.set(settings); }
  catch (err) { console.error('[rhwp] Sync 저장 오류:', err); }
  try { await browser.storage.local.set(settings); }
  catch (err) { console.error('[rhwp] Local 백업 저장 오류:', err); }
  const saved = document.getElementById('saved');
  saved.classList.add('show');
  setTimeout(() => saved.classList.remove('show'), 1500);
}

for (const id of INPUTS) {
  document.getElementById(id).addEventListener('change', () => {
    saveSettings().catch((err) => console.error('[rhwp] 옵션 저장 오류:', err));
  });
}
loadSettings().catch((err) => console.error('[rhwp] 옵션 로드 오류:', err));
