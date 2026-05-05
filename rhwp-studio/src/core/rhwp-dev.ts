import type { WasmBridge } from './wasm-bridge';

interface TextRunInfo {
  secIdx: number;
  paraIdx: number;
  charStart: number;
  x: number;
  y: number;
  text: string;
}

interface SearchResult {
  found: boolean;
  sec?: number;
  para?: number;
  charOffset?: number;
  length?: number;
}

export function initRhwpDev(wasm: WasmBridge): void {
  const dev = {
    showAllIds(pageNum?: number): void {
      const totalPages = wasm.pageCount;
      const startPage = pageNum ?? 0;
      const endPage = pageNum != null ? pageNum + 1 : totalPages;
      const entries: Array<{ page: number; secIdx: number; paraIdx: number; charStart: number; x: number; y: number; text: string }> = [];

      for (let p = startPage; p < endPage; p++) {
        let layout: string;
        try {
          layout = (wasm as any).doc.getPageTextLayout(p);
        } catch { continue; }
        const data = JSON.parse(layout);
        if (!data || !Array.isArray(data.runs)) continue;

        for (const run of data.runs) {
          if (run.secIdx == null || run.paraIdx == null) continue;
          entries.push({
            page: p,
            secIdx: run.secIdx,
            paraIdx: run.paraIdx,
            charStart: run.charStart ?? 0,
            x: run.x ?? 0,
            y: run.y ?? 0,
            text: (run.text ?? '').slice(0, 20),
          });
        }
      }

      // 중복 제거 (같은 page+secIdx+paraIdx 는 첫 런만)
      const seen = new Set<string>();
      const unique = entries.filter(e => {
        const key = `${e.page}:${e.secIdx}:${e.paraIdx}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });

      console.table(unique);
      console.log(`[rhwpDev] showAllIds: ${unique.length} unique paragraph IDs across pages ${startPage}~${endPage - 1}`);
    },

    search(text: string): SearchResult | null {
      const result = wasm.searchText(text, 0, 0, 0, true, false);
      if (!result || !result.found) {
        console.warn(`[rhwpDev] search("${text}"): not found`);
        return null;
      }
      console.log(`[rhwpDev] search("${text}"): sec=${result.sec} para=${result.para} offset=${result.charOffset} len=${result.length}`);
      return result;
    },

    findNearest(targetId: number, pageNum?: number): { paraIdx: number; distance: number; text: string } | null {
      const totalPages = wasm.pageCount;
      const page = pageNum ?? 0;
      if (page >= totalPages) return null;

      let layout: string;
      try {
        layout = (wasm as any).doc.getPageTextLayout(page);
      } catch { return null; }
      const data = JSON.parse(layout);
      if (!data || !Array.isArray(data.runs)) return null;

      let nearest: { paraIdx: number; distance: number; text: string } | null = null;
      for (const run of data.runs) {
        const id = run.paraIdx as number;
        if (id == null) continue;
        const dist = Math.abs(id - targetId);
        if (!nearest || dist < nearest.distance) {
          nearest = { paraIdx: id, distance: dist, text: (run.text ?? '').slice(0, 30) };
        }
      }
      if (nearest) {
        console.log(`[rhwpDev] findNearest(${targetId}, page=${page}): closest paraIdx=${nearest.paraIdx} (distance=${nearest.distance}) "${nearest.text}"`);
      }
      return nearest;
    },

    help(): void {
      console.log(`%c[rhwpDev]%c Debugging Toolkit
  .showAllIds(page?)      — list all paragraph IDs (console.table)
  .search("text")         — find section/paragraph/offset for text
  .findNearest(id, page?) — find nearest valid paraIdx to a given ID
  .help()                 — this message`, 'color:#2563eb;font-weight:bold', 'color:inherit');
    },
  };

  (window as any).rhwpDev = dev;
  console.log('%c[rhwpDev]%c Debugging toolkit loaded — rhwpDev.help() for usage', 'color:#2563eb;font-weight:bold', 'color:inherit');
}
