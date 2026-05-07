import type { WasmBridge } from './wasm-bridge';

interface TextRunInfo {
  secIdx: number;
  paraIdx: number;
  charStart: number;
  x: number;
  y: number;
  text: string;
  parentParaIdx?: number;
  controlIdx?: number;
  cellIdx?: number;
  cellParaIdx?: number;
}

interface SearchResult {
  found: boolean;
  sec?: number;
  para?: number;
  charOffset?: number;
  length?: number;
}

function containerKey(run: TextRunInfo): string {
  if (run.parentParaIdx != null) {
    return `cell[p${run.parentParaIdx},c${run.controlIdx ?? 0},i${run.cellIdx ?? 0}]`;
  }
  return 'body';
}


export function initRhwpDev(wasm: WasmBridge): void {
  const dev = {
    showAllIds(pageNum?: number): void {
      const totalPages = wasm.pageCount;
      const startPage = pageNum ?? 0;
      const endPage = pageNum != null ? pageNum + 1 : totalPages;
      const entries: Array<{
        page: number; container: string;
        secIdx: number; paraIdx: number; charStart: number;
        x: number; y: number; text: string;
      }> = [];

      for (let p = startPage; p < endPage; p++) {
        let data: any;
        try {
          const layout = (wasm as any).doc.getPageTextLayout(p);
          data = JSON.parse(layout);
        } catch { continue; }
        if (!data || !Array.isArray(data.runs)) continue;

        for (const run of data.runs as TextRunInfo[]) {
          if (run.secIdx == null || run.paraIdx == null) continue;
          entries.push({
            page: p,
            container: containerKey(run),
            secIdx: run.secIdx,
            paraIdx: run.paraIdx,
            charStart: run.charStart ?? 0,
            x: run.x ?? 0,
            y: run.y ?? 0,
            text: (run.text ?? '').slice(0, 20),
          });
        }
      }

      const seen = new Set<string>();
      const unique = entries.filter(e => {
        const key = `${e.page}:${e.container}:${e.secIdx}:${e.paraIdx}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });

      console.table(unique);
      console.log(`[rhwpDev] showAllIds: ${unique.length} unique paragraph IDs across pages ${startPage}~${endPage - 1}`);
    },

    search(text: string): SearchResult[] {
      const results: SearchResult[] = [];
      let sec = 0, para = 0, charOff = 0;

      for (;;) {
        const r = wasm.searchText(text, sec, para, charOff, true, false);
        if (!r || !r.found) break;
        results.push(r);
        sec = r.sec!;
        para = r.para!;
        charOff = r.charOffset! + (r.length ?? text.length);
      }

      if (results.length === 0) {
        console.warn(`[rhwpDev] search("${text}"): not found`);
      } else {
        console.log(`[rhwpDev] search("${text}"): ${results.length} match(es)`);
        console.table(results);
      }
      return results;
    },

    findNearest(targetId: number, pageNum?: number): { paraIdx: number; distance: number; text: string; container: string } | null {
      const totalPages = wasm.pageCount;
      const page = pageNum ?? 0;
      if (page >= totalPages) return null;

      let data: any;
      try {
        const layout = (wasm as any).doc.getPageTextLayout(page);
        data = JSON.parse(layout);
      } catch { return null; }
      if (!data || !Array.isArray(data.runs)) return null;

      let nearest: { paraIdx: number; distance: number; text: string; container: string } | null = null;
      for (const run of data.runs as TextRunInfo[]) {
        const id = run.paraIdx;
        if (id == null) continue;
        const dist = Math.abs(id - targetId);
        if (!nearest || dist < nearest.distance) {
          nearest = { paraIdx: id, distance: dist, text: (run.text ?? '').slice(0, 30), container: containerKey(run) };
        }
      }
      if (nearest) {
        console.log(`[rhwpDev] findNearest(${targetId}, page=${page}): closest paraIdx=${nearest.paraIdx} (${nearest.container}, distance=${nearest.distance}) "${nearest.text}"`);
      }
      return nearest;
    },

    help(): void {
      console.log(`%c[rhwpDev]%c Debugging Toolkit
  .showAllIds(page?)      — list all paragraph IDs with container context (console.table)
  .search("text")         — find all matches: section/paragraph/offset (returns array)
  .findNearest(id, page?) — find nearest valid paraIdx to a given ID
  .help()                 — this message`, 'color:#2563eb;font-weight:bold', 'color:inherit');
    },
  };

  (window as any).rhwpDev = dev;
  console.log('%c[rhwpDev]%c Debugging toolkit loaded — rhwpDev.help() for usage', 'color:#2563eb;font-weight:bold', 'color:inherit');
}
