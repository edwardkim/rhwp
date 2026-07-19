/**
 * 최근 열람 문서 저장소.
 *
 * 파일 메뉴 "최근 문서" 목록의 영속 저장을 담당한다. 문서 바이트는 보관하지
 * 않는다. `FileSystemFileHandle`이 있는 열기는 핸들과 메타(파일명/형식/시각)를
 * 저장해 라이브 파일을 재연다. 드롭/`input[type=file]`/URL 로드처럼 핸들이 없는
 * 열기도 메타-only로 기록하며, 이 항목은 선택 시 파일을 다시 고르게 한다.
 *
 * 동일 문서 판정은 `isSameEntry`가 권위다. 브라우저/환경 제약으로 판정이
 * 불가능할 때만 파일명 비교로 폴백하며, 이 폴백은 서로 다른 경로의 같은 이름
 * 문서를 병합할 수 있는 근사임을 전제한다.
 *
 * 자동 백업(`rhwpStudioAutosave`)·비교 이력(`rhwpStudioDocHistory`)과 섞지 않기
 * 위해 별도 IndexedDB(`rhwpStudioRecent`)를 사용한다. IndexedDB를 쓸 수 없는
 * 테스트/제한 환경에서는 메모리 저장소로 폴백한다.
 */

import type { FileSystemFileHandleLike } from '@/command/file-system-access';

const DB_NAME = 'rhwpStudioRecent';
const DB_VER = 1;
const STORE = 'recent';
const MAX_RECENT = 8;

export interface RecentDoc {
  /** 고유 ID (crypto.randomUUID) */
  id: string;
  /** 파일명 (경로 아님 — 브라우저 제약) */
  fileName: string;
  /** 원본 형식 ('hwp' | 'hwpx' | 'hml' 등) */
  sourceFormat: string;
  /** 마지막으로 연 시각 (epoch ms) */
  openedAt: number;
  /**
   * 재열기용 파일 핸들. File System Access 로 연 경우에만 존재한다.
   * 드롭/`input[type=file]`/URL 로드 등 핸들이 없는 경로는 메타-only 로 기록되며
   * (`handle` 미존재), 이 경우 자동 재열기는 불가하고 목록/이력 표시에만 쓰인다.
   */
  handle?: FileSystemFileHandleLike;
}

/** addRecentDoc 입력 (id/openedAt는 내부 생성) */
export interface RecentDocInput {
  fileName: string;
  sourceFormat: string;
  /**
   * 재열기용 핸들. 없으면(null/undefined) 메타-only 로 기록한다 — 파일명/형식/시각만
   * 남기며 자동 재열기는 불가하다. 핸들이 있으면 라이브 파일 재열기가 가능하다.
   */
  handle?: FileSystemFileHandleLike | null;
}

const memory = new Map<string, RecentDoc>();

function idbAvailable(): boolean {
  return typeof indexedDB !== 'undefined';
}

function createRecentId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `recent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
}

function openDb(): Promise<IDBDatabase | null> {
  if (!idbAvailable()) return Promise.resolve(null);
  return new Promise((resolve) => {
    const req = indexedDB.open(DB_NAME, DB_VER);
    req.onerror = () => resolve(null);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE, { keyPath: 'id' });
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

function getAllRows(db: IDBDatabase): Promise<RecentDoc[]> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, 'readonly');
    const req = tx.objectStore(STORE).getAll();
    req.onsuccess = () => resolve((req.result as RecentDoc[]) ?? []);
    req.onerror = () => reject(req.error);
  });
}

function putRow(db: IDBDatabase, row: RecentDoc): Promise<void> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, 'readwrite');
    tx.objectStore(STORE).put(row);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

function deleteRow(db: IDBDatabase, id: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, 'readwrite');
    tx.objectStore(STORE).delete(id);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

/**
 * 동일 파일 판정 — `isSameEntry`가 권위. 환경 제약으로 판정이 불가능하면
 * 파일명 비교 폴백(근사 — 모듈 주석 참조).
 */
async function isSameFile(a: RecentDocInput, existing: RecentDoc): Promise<boolean> {
  const ha = a.handle;
  const hb = existing.handle;
  if (ha && hb && typeof ha.isSameEntry === 'function') {
    try {
      return await ha.isSameEntry(hb);
    } catch {
      // fall through to name compare
    }
  }
  return a.fileName === existing.fileName;
}

/** 최신순(openedAt 내림차순)으로 정렬해 상한까지 자른다. */
function sortAndTrim(rows: RecentDoc[]): RecentDoc[] {
  return rows.sort((a, b) => b.openedAt - a.openedAt).slice(0, MAX_RECENT);
}

/**
 * 최근 문서를 추가한다. 핸들이 있으면 라이브 재열기용으로 함께 저장하고, 없으면
 * 메타-only(파일명/형식/시각)로 기록한다 — 드롭/`input`/URL 등 핸들 없는 열기도
 * 목록에 남긴다. 동일 파일이 이미 있으면 제거 후 맨 앞에 다시 넣고, 최대
 * {@link MAX_RECENT}개를 유지한다.
 * 핸들이 structured clone 불가(DataCloneError 등)한 환경이면 핸들을 떼고 메타-only
 * 로 재시도한다 — 기록 자체는 남긴다.
 */
export async function addRecentDoc(input: RecentDocInput): Promise<void> {
  const entry: RecentDoc = {
    id: createRecentId(),
    fileName: input.fileName,
    sourceFormat: input.sourceFormat,
    openedAt: Date.now(),
    ...(input.handle ? { handle: input.handle } : {}),
  };

  await withDb(
    async (db) => {
      const rows = await getAllRows(db);
      for (const row of rows) {
        if (await isSameFile(input, row)) await deleteRow(db, row.id);
      }
      try {
        await putRow(db, entry);
      } catch {
        // 핸들 직렬화 불가 환경 — 핸들을 떼고 메타-only 로 재시도(기록은 유지).
        try {
          const { handle: _drop, ...metaOnly } = entry;
          await putRow(db, metaOnly);
        } catch {
          return;
        }
      }
      const after = sortAndTrim(await getAllRows(db));
      const keep = new Set(after.map((r) => r.id));
      for (const row of await getAllRows(db)) {
        if (!keep.has(row.id)) await deleteRow(db, row.id);
      }
    },
    async () => {
      for (const [id, row] of memory) {
        if (await isSameFile(input, row)) memory.delete(id);
      }
      memory.set(entry.id, entry);
      const keep = new Set(sortAndTrim([...memory.values()]).map((r) => r.id));
      for (const id of [...memory.keys()]) {
        if (!keep.has(id)) memory.delete(id);
      }
    },
  );
}

/** 최근 문서 목록(최신순). */
export async function listRecentDocs(): Promise<RecentDoc[]> {
  return withDb(
    async (db) => sortAndTrim(await getAllRows(db)),
    async () => sortAndTrim([...memory.values()]),
  );
}

/** 특정 최근 문서를 제거한다. */
export async function removeRecentDoc(id: string): Promise<void> {
  memory.delete(id);
  await withDb(
    async (db) => deleteRow(db, id),
    async () => {},
  );
}

/** 최근 문서 목록 전체 삭제. */
export async function clearRecentDocs(): Promise<void> {
  memory.clear();
  await withDb(
    async (db) =>
      new Promise<void>((resolve, reject) => {
        const tx = db.transaction(STORE, 'readwrite');
        tx.objectStore(STORE).clear();
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
      }),
    async () => {},
  );
}
