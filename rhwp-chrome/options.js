import { SETTINGS_KEYS, loadSettings, saveSettings } from './sw/settings-store.js';

const STATUS_HIDE_MS = 1500;

export async function initializeOptionsPage({
  chromeApi = globalThis.chrome,
  documentRef = globalThis.document,
  setTimeoutFn = globalThis.setTimeout,
} = {}) {
  applyI18n(chromeApi, documentRef);
  const inputs = Object.fromEntries(
    SETTINGS_KEYS.map((id) => [id, requireElement(documentRef, id)]),
  );
  const status = requireElement(documentRef, 'saved');
  setInputsDisabled(inputs, true);

  let currentSettings;
  try {
    currentSettings = await loadSettings(chromeApi);
  } catch (error) {
    console.error('[rhwp-options] 설정 로드 실패:', error);
    showStatus(chromeApi, status, 'optionsLoadError', true, setTimeoutFn);
    return;
  }

  renderSettings(inputs, currentSettings);

  for (const id of SETTINGS_KEYS) {
    inputs[id].addEventListener('change', async () => {
      const nextSettings = { ...currentSettings, [id]: inputs[id].checked };
      setInputsDisabled(inputs, true);
      try {
        currentSettings = await saveSettings(chromeApi, nextSettings);
        renderSettings(inputs, currentSettings);
        showStatus(chromeApi, status, 'optionsSaved', false, setTimeoutFn);
      } catch (error) {
        console.error('[rhwp-options] 설정 저장 실패:', error);
        try {
          currentSettings = await loadSettings(chromeApi);
          renderSettings(inputs, currentSettings);
        } catch (reloadError) {
          console.error('[rhwp-options] 저장 실패 후 설정 복구 실패:', reloadError);
        }
        showStatus(chromeApi, status, 'optionsSaveError', true, setTimeoutFn);
      } finally {
        setInputsDisabled(inputs, false);
      }
    });
  }

  setInputsDisabled(inputs, false);
}

function applyI18n(chromeApi, documentRef) {
  const bindings = {
    title: 'optionsTitle',
    labelAutoOpen: 'optionsAutoOpen',
    labelShowBadges: 'optionsShowBadges',
    labelHoverPreview: 'optionsHoverPreview',
    labelDisableExternalWebFonts: 'optionsDisableExternalWebFonts',
    descDisableExternalWebFonts: 'optionsDisableExternalWebFontsDesc',
    privacy: 'optionsPrivacy',
  };
  for (const [id, messageKey] of Object.entries(bindings)) {
    requireElement(documentRef, id).textContent = chromeApi.i18n.getMessage(messageKey);
  }
  requireElement(documentRef, 'version').textContent = chromeApi.runtime.getManifest().version;
}

function renderSettings(inputs, settings) {
  for (const id of SETTINGS_KEYS) inputs[id].checked = settings[id];
}

function setInputsDisabled(inputs, disabled) {
  for (const input of Object.values(inputs)) input.disabled = disabled;
}

function showStatus(chromeApi, element, messageKey, isError, setTimeoutFn) {
  element.textContent = chromeApi.i18n.getMessage(messageKey);
  element.classList.remove('show', 'error');
  if (isError) element.classList.add('error');
  element.classList.add('show');
  if (!isError) {
    setTimeoutFn(() => element.classList.remove('show'), STATUS_HIDE_MS);
  }
}

function requireElement(documentRef, id) {
  const element = documentRef?.getElementById(id);
  if (!element) throw new Error(`옵션 UI 요소를 찾을 수 없습니다: ${id}`);
  return element;
}

if (typeof document !== 'undefined' && typeof chrome !== 'undefined') {
  void initializeOptionsPage();
}
