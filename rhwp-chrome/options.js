(function () {
  'use strict';

  // i18n 적용
  document.getElementById('title').textContent = chrome.i18n.getMessage('optionsTitle');
  document.getElementById('labelAutoOpen').textContent = chrome.i18n.getMessage('optionsAutoOpen');
  document.getElementById('labelShowBadges').textContent = chrome.i18n.getMessage('optionsShowBadges');
  document.getElementById('labelHoverPreview').textContent = chrome.i18n.getMessage('optionsHoverPreview');
  document.getElementById('labelDisableExternalWebFonts').textContent = chrome.i18n.getMessage('optionsDisableExternalWebFonts');
  document.getElementById('descDisableExternalWebFonts').textContent = chrome.i18n.getMessage('optionsDisableExternalWebFontsDesc');
  document.getElementById('saved').textContent = chrome.i18n.getMessage('optionsSaved');
  document.getElementById('privacy').textContent = chrome.i18n.getMessage('optionsPrivacy');
  document.getElementById('version').textContent = chrome.runtime.getManifest().version;

  const inputs = ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts'];

  // 설정 로드
  chrome.storage.sync.get(
    { autoOpen: true, showBadges: true, hoverPreview: true, disableExternalWebFonts: false },
    (settings) => {
      for (const id of inputs) {
        document.getElementById(id).checked = settings[id];
      }
    }
  );

  // 설정 저장
  for (const id of inputs) {
    document.getElementById(id).addEventListener('change', () => {
      const settings = {};
      for (const id2 of inputs) {
        settings[id2] = document.getElementById(id2).checked;
      }
      chrome.storage.sync.set(settings, () => {
        const saved = document.getElementById('saved');
        saved.classList.add('show');
        setTimeout(() => saved.classList.remove('show'), 1500);
      });
    });
  }
})();
