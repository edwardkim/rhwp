(function () {
  'use strict';

  const inputs = ['autoOpen', 'showBadges', 'hoverPreview'];

  document.getElementById('title').textContent = browser.i18n.getMessage('optionsTitle');
  document.getElementById('labelAutoOpen').textContent = browser.i18n.getMessage('optionsAutoOpen');
  document.getElementById('labelShowBadges').textContent = browser.i18n.getMessage('optionsShowBadges');
  document.getElementById('labelHoverPreview').textContent = browser.i18n.getMessage('optionsHoverPreview');
  document.getElementById('saved').textContent = browser.i18n.getMessage('optionsSaved');
  document.getElementById('privacy').textContent = browser.i18n.getMessage('optionsPrivacy');
  document.getElementById('version').textContent = browser.runtime.getManifest().version;

  browser.storage.sync.get(
    { autoOpen: true, showBadges: true, hoverPreview: true },
  ).then((settings) => {
    for (const id of inputs) {
      document.getElementById(id).checked = settings[id];
    }
  });

  for (const id of inputs) {
    document.getElementById(id).addEventListener('change', () => {
      const settings = {};
      for (const id2 of inputs) {
        settings[id2] = document.getElementById(id2).checked;
      }

      browser.storage.sync.set(settings).then(() => {
        const saved = document.getElementById('saved');
        saved.classList.add('show');
        setTimeout(() => saved.classList.remove('show'), 1500);
      });
    });
  }
})();
