(function () {
  'use strict';

  if (typeof browser === 'undefined') return;

  try {
    globalThis.chrome = browser;
  } catch {
    try {
      Object.defineProperty(globalThis, 'chrome', {
        value: browser,
        configurable: true,
      });
    } catch {
      // Ignore. Firefox normally exposes chrome for compatibility already.
    }
  }
})();
