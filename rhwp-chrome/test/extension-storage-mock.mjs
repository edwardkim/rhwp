export function createStorageArea(initial = {}) {
  const values = new Map(Object.entries(initial));
  const calls = { get: [], set: [], remove: [] };
  let nextGetError = null;
  let nextSetError = null;

  return {
    calls,
    values,
    failNextGet(error) {
      nextGetError = error;
    },
    failNextSet(error) {
      nextSetError = error;
    },
    async get(query) {
      calls.get.push(query);
      if (nextGetError) {
        const error = nextGetError;
        nextGetError = null;
        throw error;
      }
      return selectValues(values, query);
    },
    async set(items) {
      calls.set.push(structuredClone(items));
      if (nextSetError) {
        const error = nextSetError;
        nextSetError = null;
        throw error;
      }
      for (const [key, value] of Object.entries(items)) {
        values.set(key, structuredClone(value));
      }
    },
    async remove(query) {
      calls.remove.push(query);
      const keys = Array.isArray(query) ? query : [query];
      for (const key of keys) values.delete(key);
    },
    dump() {
      return Object.fromEntries(values);
    },
  };
}

export function createChromeStorageMock({ sync = {}, local = {}, version = '0.2.8' } = {}) {
  const syncArea = createStorageArea(sync);
  const localArea = createStorageArea(local);
  return {
    chrome: {
      runtime: {
        getManifest() {
          return { version };
        },
      },
      storage: {
        sync: syncArea,
        local: localArea,
      },
    },
    syncArea,
    localArea,
  };
}

function selectValues(values, query) {
  if (query == null) return Object.fromEntries(values);
  if (typeof query === 'string') {
    return values.has(query) ? { [query]: structuredClone(values.get(query)) } : {};
  }
  if (Array.isArray(query)) {
    return Object.fromEntries(
      query
        .filter((key) => values.has(key))
        .map((key) => [key, structuredClone(values.get(key))]),
    );
  }
  if (typeof query === 'object') {
    return Object.fromEntries(
      Object.entries(query).map(([key, fallback]) => [
        key,
        values.has(key) ? structuredClone(values.get(key)) : fallback,
      ]),
    );
  }
  return {};
}
