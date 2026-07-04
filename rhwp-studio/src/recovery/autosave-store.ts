/**
 * 미저장 문서 복구용 자동 백업 저장소.
 *
 * 문서 비교 이력(`rhwpStudioDocHistory`)과 섞지 않기 위해 별도 IndexedDB를 사용한다.
 * IndexedDB를 사용할 수 없는 테스트/제한 환경에서는 메모리 저장소로 폴백한다.
 */

const DB_NAME = 'rhwpStudioAutosave';
const DB_VER = 1;
const DRAFTS = 'drafts';
const MAX_DRAFTS = 12;

export interface AutosaveDraft {
  id: string;
  fileName: string;
  sourceFormat: string;
  savedAt: number;
  byteLength: number;
  data: Uint8Array;
  dirtyReason?: string;
}

type DraftRow = Omit<AutosaveDraft, 'data'> & { data?: ArrayBuffer };

const memory = new Map<string, AutosaveDraft>();

function idbAvailable(): boolean {
  return typeof indexedDB !== 'undefined';
}

function cloneBytes(bytes: Uint8Array): Uint8Array {
  return new Uint8Array(bytes);
}

function bytesToArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  return copy.buffer as ArrayBuffer;
}

function cloneDraft(draft: AutosaveDraft): AutosaveDraft {
  return { ...draft, data: cloneBytes(draft.data) };
}

function rowToDraft(row: DraftRow): AutosaveDraft {
  return {
    ...row,
    byteLength: row.byteLength,
    data: new Uint8Array(row.data ?? new ArrayBuffer(0)),
  };
}

function draftToRow(draft: AutosaveDraft): DraftRow {
  return {
    ...draft,
    byteLength: draft.data.byteLength,
    data: bytesToArrayBuffer(draft.data),
  };
}

function openDb(): Promise<IDBDatabase | null> {
  if (!idbAvailable()) return Promise.resolve(null);
  return new Promise((resolve) => {
    const req = indexedDB.open(DB_NAME, DB_VER);
    req.onerror = () => resolve(null);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(DRAFTS)) {
        db.createObjectStore(DRAFTS, { keyPath: 'id' });
      }
    };
  });
}

async function withDb<T>(fn: (db: IDBDatabase) => Promise<T>, fallback: () => Promise<T>): Promise<T> {
  const db = await openDb();
  if (!db) return fallback();
  try {
    return await fn(db);
  } finally {
    db.close();
  }
}

async function trimMemoryDrafts(): Promise<void> {
  if (memory.size <= MAX_DRAFTS) return;
  const remove = [...memory.values()]
    .sort((a, b) => a.savedAt - b.savedAt)
    .slice(0, memory.size - MAX_DRAFTS);
  for (const draft of remove) {
    memory.delete(draft.id);
  }
}

async function trimDbDrafts(db: IDBDatabase): Promise<void> {
  const rows: DraftRow[] = await new Promise((resolve, reject) => {
    const tx = db.transaction(DRAFTS, 'readonly');
    const req = tx.objectStore(DRAFTS).getAll();
    req.onsuccess = () => resolve((req.result as DraftRow[]) ?? []);
    req.onerror = () => reject(req.error);
  });
  if (rows.length <= MAX_DRAFTS) return;

  const remove = rows
    .sort((a, b) => a.savedAt - b.savedAt)
    .slice(0, rows.length - MAX_DRAFTS);
  for (const draft of remove) {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(DRAFTS, 'readwrite');
      tx.objectStore(DRAFTS).delete(draft.id);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  }
}

export function createAutosaveDraftId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `draft_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
}

export async function saveAutosaveDraft(draft: AutosaveDraft): Promise<void> {
  const normalized = cloneDraft({
    ...draft,
    byteLength: draft.data.byteLength,
  });

  await withDb(
    async (db) => {
      await trimDbDrafts(db);
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction(DRAFTS, 'readwrite');
        tx.objectStore(DRAFTS).put(draftToRow(normalized));
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
      });
    },
    async () => {
      memory.set(normalized.id, normalized);
      await trimMemoryDrafts();
    },
  );
}

export async function getAutosaveDraft(id: string): Promise<AutosaveDraft | null> {
  const mem = memory.get(id);
  if (mem) return cloneDraft(mem);

  return withDb(
    async (db) =>
      new Promise<AutosaveDraft | null>((resolve, reject) => {
        const tx = db.transaction(DRAFTS, 'readonly');
        const req = tx.objectStore(DRAFTS).get(id);
        req.onsuccess = () => {
          const row = req.result as DraftRow | undefined;
          resolve(row ? rowToDraft(row) : null);
        };
        req.onerror = () => reject(req.error);
      }),
    async () => null,
  );
}

export async function listAutosaveDrafts(): Promise<AutosaveDraft[]> {
  return withDb(
    async (db) =>
      new Promise<AutosaveDraft[]>((resolve, reject) => {
        const tx = db.transaction(DRAFTS, 'readonly');
        const req = tx.objectStore(DRAFTS).getAll();
        req.onsuccess = () => {
          const rows = ((req.result as DraftRow[]) ?? []).map(rowToDraft);
          resolve(rows.sort((a, b) => b.savedAt - a.savedAt));
        };
        req.onerror = () => reject(req.error);
      }),
    async () => [...memory.values()].map(cloneDraft).sort((a, b) => b.savedAt - a.savedAt),
  );
}

export async function deleteAutosaveDraft(id: string): Promise<void> {
  memory.delete(id);
  await withDb(
    async (db) =>
      new Promise<void>((resolve, reject) => {
        const tx = db.transaction(DRAFTS, 'readwrite');
        tx.objectStore(DRAFTS).delete(id);
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
      }),
    async () => {},
  );
}

export async function clearAutosaveDrafts(): Promise<void> {
  memory.clear();
  await withDb(
    async (db) =>
      new Promise<void>((resolve, reject) => {
        const tx = db.transaction(DRAFTS, 'readwrite');
        tx.objectStore(DRAFTS).clear();
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
      }),
    async () => {},
  );
}
