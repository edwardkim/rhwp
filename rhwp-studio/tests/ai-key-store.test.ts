import test from 'node:test';
import assert from 'node:assert/strict';

import {
  KeyStore,
  KeyStoreError,
  createMemoryKeyBackend,
  maskKey,
  normalizeKey,
  type KeyBackend,
} from '../src/ai/settings/key-store.ts';

const KEY = 'sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
const ITER = 1000;

const code = (expected: string) => (error: unknown) =>
  error instanceof KeyStoreError && error.code === expected;

test('세션 모드(기본)는 메모리에만 두고 저장소에 남기지 않는다', async () => {
  const backend = createMemoryKeyBackend();
  const store = new KeyStore(backend, undefined, ITER);
  assert.equal(await store.status('claude'), 'none');
  assert.deepEqual(await store.setKey('claude', `  ${KEY}\n`, { remember: false }), { persisted: false });
  assert.equal(store.getKey('claude'), KEY, '앞뒤 공백은 지운다');
  assert.equal(await store.status('claude'), 'ready');
  assert.equal(backend.records.size, 0);

  const restarted = new KeyStore(backend, undefined, ITER);
  assert.equal(await restarted.status('claude'), 'none', '새 실행에는 남지 않는다');
});

test('기억 모드는 다음 실행에서 바로 쓸 수 있다', async () => {
  const backend = createMemoryKeyBackend();
  assert.deepEqual(await new KeyStore(backend, undefined, ITER).setKey('claude', KEY, { remember: true }),
    { persisted: true });
  const restarted = new KeyStore(backend, undefined, ITER);
  assert.equal(restarted.getKey('claude'), null, 'status 전에는 올라오지 않는다');
  assert.equal(await restarted.status('claude'), 'ready');
  assert.equal(restarted.getKey('claude'), KEY);
});

test('암호 모드는 암호문만 남기고 맞는 암호로만 열린다', async () => {
  const backend = createMemoryKeyBackend();
  await new KeyStore(backend, undefined, ITER).setKey('claude', KEY, { remember: true, passphrase: '비밀 문구' });
  const record = backend.records.get('claude')!;
  assert.equal(record.mode, 'encrypted');
  assert.ok(!JSON.stringify(record).includes(KEY), '저장값에 키 원문이 없다');
  assert.ok(!JSON.stringify(record).includes('비밀 문구'), '저장값에 암호가 없다');

  const restarted = new KeyStore(backend, undefined, ITER);
  assert.equal(await restarted.status('claude'), 'locked');
  await assert.rejects(restarted.unlock('claude', '틀린 암호'), code('WRONG_PASSPHRASE'));
  await assert.rejects(restarted.unlock('claude', ''), code('EMPTY_PASSPHRASE'));
  assert.equal(restarted.getKey('claude'), null);
  await restarted.unlock('claude', '비밀 문구');
  assert.equal(restarted.getKey('claude'), KEY);
  assert.equal(await restarted.status('claude'), 'ready');
});

test('같은 키·암호라도 저장할 때마다 salt 와 iv 가 다르다', async () => {
  const a = createMemoryKeyBackend();
  const b = createMemoryKeyBackend();
  await new KeyStore(a, undefined, ITER).setKey('claude', KEY, { remember: true, passphrase: 'p' });
  await new KeyStore(b, undefined, ITER).setKey('claude', KEY, { remember: true, passphrase: 'p' });
  const ra = a.records.get('claude') as { salt: string; iv: string; ciphertext: string };
  const rb = b.records.get('claude') as { salt: string; iv: string; ciphertext: string };
  assert.notEqual(ra.salt, rb.salt);
  assert.notEqual(ra.iv, rb.iv);
  assert.notEqual(ra.ciphertext, rb.ciphertext);
});

test('기억을 끄면 저장소의 키를 지우고, 삭제는 메모리와 저장소를 모두 비운다', async () => {
  const backend = createMemoryKeyBackend();
  const store = new KeyStore(backend, undefined, ITER);
  await store.setKey('claude', KEY, { remember: true });
  await store.setRemember('claude', { remember: false });
  assert.equal(backend.records.size, 0);
  assert.equal(store.getKey('claude'), KEY, '세션 키는 남는다');

  await store.setRemember('claude', { remember: true, passphrase: 'p' });
  assert.equal(backend.records.get('claude')?.mode, 'encrypted', '키를 다시 넣지 않고 보관 방식만 바꾼다');

  await store.clear('claude');
  assert.equal(store.getKey('claude'), null);
  assert.equal(backend.records.size, 0);
  assert.equal(await store.status('claude'), 'none');
});

test('형식이 틀린 키와 빈 암호는 거절하고 기존 상태를 바꾸지 않는다', async () => {
  const backend = createMemoryKeyBackend();
  const store = new KeyStore(backend, undefined, ITER);
  await assert.rejects(store.setKey('claude', '   ', { remember: false }), code('INVALID_KEY'));
  await assert.rejects(store.setKey('claude', 'sk-ant abc', { remember: false }), code('INVALID_KEY'));
  await assert.rejects(store.setKey('claude', KEY, { remember: true, passphrase: '' }), code('EMPTY_PASSPHRASE'));
  assert.equal(store.getKey('claude'), null);
  assert.equal(backend.records.size, 0);
  assert.throws(() => normalizeKey('a'.repeat(513)), code('INVALID_KEY'));
});

test('저장소가 없거나 쓰기가 실패하면 세션 모드로 내려가고 그 사실을 알린다', async () => {
  const none = new KeyStore(null, undefined, ITER);
  assert.equal(none.canPersist, false);
  assert.deepEqual(await none.setKey('claude', KEY, { remember: true }), { persisted: false });
  assert.equal(none.getKey('claude'), KEY);

  const broken: KeyBackend = {
    get: async () => { throw new Error('blocked'); },
    put: async () => { throw new Error('blocked'); },
    delete: async () => { throw new Error('blocked'); },
  };
  const store = new KeyStore(broken, undefined, ITER);
  assert.deepEqual(await store.setKey('claude', KEY, { remember: true }), { persisted: false });
  assert.equal(await store.status('claude'), 'ready');
  await store.clear('claude');
  assert.equal(await store.status('claude'), 'none');
});

test('가린 표시는 앞 7자와 끝 4자만 보인다', () => {
  assert.equal(maskKey(KEY), 'sk-ant-…6789');
  assert.equal(maskKey('short-key'), '••••');
  assert.ok(!maskKey(KEY).includes('ABCDEFG'));
});

test('암호화 API 가 없는 비보안 출처(http://IP)에서는 암호 저장·해제를 이유를 밝혀 거절한다', async () => {
  // 브라우저는 https·localhost 가 아닌 http 출처에서 crypto.subtle 을 주지 않는다.
  const backend = createMemoryKeyBackend();
  const insecure = new KeyStore(backend, null, ITER);
  assert.equal(insecure.canEncrypt, false);
  await assert.rejects(insecure.setKey('claude', KEY, { remember: true, passphrase: 'p' }), code('CRYPTO_UNAVAILABLE'));
  assert.equal(insecure.getKey('claude'), null, '거절하면 세션 키도 바꾸지 않는다');
  assert.equal(backend.records.size, 0);

  // 암호 없는 기억·세션 모드는 그대로 된다.
  assert.deepEqual(await insecure.setKey('claude', KEY, { remember: true }), { persisted: true });
  assert.equal(insecure.getKey('claude'), KEY);

  // 보안 출처에서 암호로 저장한 키를 비보안 출처에서 열려 하면 「암호 틀림」이 아니라 이 이유로 거절한다.
  const secureBackend = createMemoryKeyBackend();
  await new KeyStore(secureBackend, undefined, ITER).setKey('claude', KEY, { remember: true, passphrase: 'p' });
  const reopened = new KeyStore(secureBackend, null, ITER);
  assert.equal(await reopened.status('claude'), 'locked');
  await assert.rejects(reopened.unlock('claude', 'p'), code('CRYPTO_UNAVAILABLE'));
});
