import test from 'node:test';
import assert from 'node:assert/strict';
import { RecoveryCipher, sameRecoveryDraft } from '../src/recovery/recovery-crypto.ts';
import { AutosaveManager } from '../src/recovery/autosave-manager.ts';
import { clearAutosaveDrafts, saveAutosaveDraft, getAutosaveDraft, type AutosaveDraft } from '../src/recovery/autosave-store.ts';

const password = 'recovery-contract-only-비밀';
const draft = (): AutosaveDraft => ({ id: 'draft-1', fileName: 'private.hwp', sourceFormat: 'hwp',
  savedAt: 1234, byteLength: 26, dirtyReason: 'edit', data: new TextEncoder().encode('private document marker') });

test('protected recovery is authenticated, changes nonce on each save and exposes no key or password', async () => {
  const cipher = await RecoveryCipher.create(password);
  const first = await cipher.protect(draft()), second = await cipher.protect(draft());
  assert.notDeepEqual(first.data, draft().data);
  assert.notDeepEqual(first.protection!.iv, second.protection!.iv);
  assert.equal(second.protection!.revision, first.protection!.revision + 1);
  assert.deepEqual(Reflect.ownKeys(cipher), []);
  assert.equal(JSON.stringify(first).includes(password), false);
  const restored = await RecoveryCipher.unlock(first, password);
  assert.deepEqual(restored.data, draft().data);
  await assert.rejects(RecoveryCipher.unlock(first, 'wrong-password'), /RECOVERY_AUTH_FAILED/);
  restored.cipher.dispose(); cipher.dispose();
});

test('ciphertext and document identity/revision/format are all authenticated', async () => {
  const cipher = await RecoveryCipher.create(password), saved = await cipher.protect(draft());
  const mutations = [
    (d: AutosaveDraft) => { d.data[0] ^= 1; },
    (d: AutosaveDraft) => { d.id += '-other'; },
    (d: AutosaveDraft) => { d.fileName = 'different.hwp'; },
    (d: AutosaveDraft) => { d.sourceFormat = 'hwpx'; },
    (d: AutosaveDraft) => { d.savedAt++; },
    (d: AutosaveDraft) => { d.protection!.revision++; },
    (d: AutosaveDraft) => { d.protection!.iv[0] ^= 1; },
    (d: AutosaveDraft) => { d.protection!.salt[0] ^= 1; },
    (d: AutosaveDraft) => { d.protection!.iterations = 1; },
    (d: AutosaveDraft) => { d.protection!.version = 2 as 1; },
  ];
  for (const mutate of mutations) {
    const damaged = structuredClone(saved); mutate(damaged);
    await assert.rejects(RecoveryCipher.unlock(damaged, password), /RECOVERY_AUTH_FAILED/);
  }
  cipher.dispose();
});

test('discarded session keys cannot finish or begin an encryption', async () => {
  const cipher = await RecoveryCipher.create(password);
  const pending = cipher.protect(draft()); cipher.dispose();
  await assert.rejects(pending, /RECOVERY_AUTH_FAILED/);
  await assert.rejects(cipher.protect(draft()), /RECOVERY_AUTH_FAILED/);
});

test('protected draft store clones metadata and detects stale/deleted snapshots', async () => {
  await clearAutosaveDrafts(); const cipher = await RecoveryCipher.create(password);
  const saved = await cipher.protect(draft()); await saveAutosaveDraft(saved);
  const copy = (await getAutosaveDraft(saved.id))!; assert.equal(sameRecoveryDraft(saved, copy), true);
  copy.protection!.salt.fill(0);
  assert.equal(sameRecoveryDraft(saved, await getAutosaveDraft(saved.id)), true);
  assert.equal(sameRecoveryDraft(saved, copy), false); assert.equal(sameRecoveryDraft(saved, null), false);
  cipher.dispose(); await clearAutosaveDrafts();
});

test('discard/document replacement during encryption never persists a plaintext or resurrected record', async () => {
  for (const replace of [false, true]) {
    let release!: () => void; const wait = new Promise<void>(resolve => { release = resolve; });
    let entered!: () => void; const started = new Promise<void>(resolve => { entered = resolve; });
    const writes: AutosaveDraft[] = [];
    const manager = new AutosaveManager({ exportBytes: () => draft().data,
      protectDraft: async value => { entered(); await wait; return { ...value, data: new Uint8Array([99]) }; },
      store: { saveDraft: async value => { writes.push(value); }, deleteDraft: async () => {} },
      logger: { debug() {}, warn() {} },
    });
    await manager.beginDocument({ fileName: 'protected.hwp', sourceFormat: 'hwp' });
    const saving = manager.flushNow(); await started;
    if (replace) await manager.beginDocument({ fileName: 'other.hwp', sourceFormat: 'hwp' });
    else await manager.discardCurrentDraft();
    release(); await saving; assert.equal(writes.length, 0); manager.dispose();
  }
});
