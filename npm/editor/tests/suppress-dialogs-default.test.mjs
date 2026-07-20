import test from 'node:test';
import assert from 'node:assert/strict';

// RhwpEditor.loadFile가 suppressDialogs 기본값을 올바르게 전달하는지 검증 (#2602)
// 실제 iframe/transport 없이 _request 래핑으로 테스트

test('loadFile 옵션 생략 시 suppressDialogs 기본값은 true', async () => {
  let lastRequest = null;
  const editor = {
    _transport: null,
    _request(method, params) {
      lastRequest = { method, params };
      return { pageCount: 1 };
    },
  };

  // RhwpEditor.loadFile 로직 인라인: options = {} → suppressDialogs: true
  async function loadFile(data, fileName = 'document.hwp', options = {}) {
    return editor._request('loadFile', {
      data,
      fileName,
      skipUnsavedGuard: options.skipUnsavedGuard === true,
      suppressDialogs: options.suppressDialogs !== false,
    });
  }

  // 옵션 생략 → suppressDialogs: true
  await loadFile(new Uint8Array([1, 2, 3]), 'test.hwp');
  assert.equal(lastRequest.params.suppressDialogs, true);
  assert.equal(lastRequest.params.fileName, 'test.hwp');
});

test('loadFile 명시적 suppressDialogs:false는 false를 유지', async () => {
  let lastRequest = null;
  const editor = {
    _request(method, params) {
      lastRequest = { method, params };
      return { pageCount: 1 };
    },
  };

  async function loadFile(data, fileName = 'document.hwp', options = {}) {
    return editor._request('loadFile', {
      data,
      fileName,
      skipUnsavedGuard: options.skipUnsavedGuard === true,
      suppressDialogs: options.suppressDialogs !== false,
    });
  }

  await loadFile(new Uint8Array([1]), 'test.hwp', { suppressDialogs: false });
  assert.equal(lastRequest.params.suppressDialogs, false);
});

test('loadFile suppressDialogs:true는 true 유지', async () => {
  let lastRequest = null;
  const editor = {
    _request(method, params) {
      lastRequest = { method, params };
      return { pageCount: 1 };
    },
  };

  async function loadFile(data, fileName = 'document.hwp', options = {}) {
    return editor._request('loadFile', {
      data,
      fileName,
      skipUnsavedGuard: options.skipUnsavedGuard === true,
      suppressDialogs: options.suppressDialogs !== false,
    });
  }

  await loadFile(new Uint8Array([1]), 'test.hwp', { suppressDialogs: true });
  assert.equal(lastRequest.params.suppressDialogs, true);
});

test('loadFile skipUnsavedGuard 조합 테스트', async () => {
  let lastRequest = null;
  const editor = {
    _request(method, params) {
      lastRequest = { method, params };
      return { pageCount: 1 };
    },
  };

  async function loadFile(data, fileName = 'document.hwp', options = {}) {
    return editor._request('loadFile', {
      data,
      fileName,
      skipUnsavedGuard: options.skipUnsavedGuard === true,
      suppressDialogs: options.suppressDialogs !== false,
    });
  }

  await loadFile(new Uint8Array([1]), 'test.hwp', { skipUnsavedGuard: true });
  assert.equal(lastRequest.params.skipUnsavedGuard, true);
  assert.equal(lastRequest.params.suppressDialogs, true);
});
