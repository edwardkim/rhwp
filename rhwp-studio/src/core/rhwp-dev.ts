import type { WasmBridge } from './wasm-bridge';
import type { SearchHit, DocumentPosition } from './types';

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

    search(text: string, includeCells: boolean = false): SearchHit[] {
      const results = wasm.searchAllText(text, false, includeCells);
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

    goto(hit: SearchHit, options?: { select?: boolean }): boolean {
      const select = options?.select ?? true;
      const ih = (window as any).__inputHandler;
      if (!ih || !ih.cursor) {
        console.warn('[rhwpDev] inputHandler 미초기화 — 문서 로드 후 호출하세요. rhwpDev.help() 영역 영역 사용 예시 점검.');
        return false;
      }

      const startPos: DocumentPosition = hit.cellContext
        ? {
            sectionIndex: hit.sec,
            paragraphIndex: hit.cellContext.parentPara,
            charOffset: hit.charOffset,
            parentParaIndex: hit.cellContext.parentPara,
            controlIndex: hit.cellContext.ctrlIdx,
            cellIndex: hit.cellContext.cellIdx,
            cellParaIndex: hit.cellContext.cellPara,
          }
        : {
            sectionIndex: hit.sec,
            paragraphIndex: hit.para,
            charOffset: hit.charOffset,
          };

      ih.cursor.clearSelection();
      ih.cursor.moveTo(startPos);

      if (select && hit.length > 0) {
        ih.cursor.setAnchor();
        ih.cursor.moveTo({ ...startPos, charOffset: hit.charOffset + hit.length });
      }

      try { ih.updateCaret(); } catch { /* 캐럿 갱신 실패 무시 */ }

      const containerLabel = hit.cellContext
        ? `cell[p${hit.cellContext.parentPara},c${hit.cellContext.ctrlIdx},i${hit.cellContext.cellIdx},cp${hit.cellContext.cellPara}]`
        : 'body';
      console.log(`[rhwpDev] goto: sec=${hit.sec} para=${hit.para} offset=${hit.charOffset} length=${hit.length} (${containerLabel})`);
      return true;
    },

    help(): void {
      const title = '%c[rhwpDev]%c rhwp-studio 개발자 콘솔 도구\n';
      const titleStyle = 'color:#2563eb;font-weight:bold;font-size:1.1em';
      const reset = 'color:inherit';

      console.log(title +
`
%c개요%c
DEV 모드 (vite dev server) 영역 영역 자동 로드되는 디버깅 헬퍼.
브라우저 콘솔 영역 영역 \`rhwpDev.<메서드>()\` 영역 영역 호출.

%c메서드%c

  rhwpDev.search(text, includeCells?)
    문서 영역 영역 모든 매치 일괄 검색. 반환: SearchHit[]
    - text: 검색 문자열
    - includeCells: 표 셀/글상자 포함 여부 (기본 false)

  rhwpDev.goto(hit, options?)
    SearchHit 영역 영역 캐럿 이동 + 화면 스크롤. 반환: boolean
    - hit: rhwpDev.search() 결과 항목
    - options.select: length 만큼 선택 (기본 true), false 시 캐럿만 이동

  rhwpDev.showAllIds(pageNum?)
    페이지 영역 영역 모든 문단 ID 영역 영역 console.table 표시
    - pageNum 생략 시 전체 페이지

  rhwpDev.findNearest(targetId, pageNum?)
    가장 가까운 paraIdx 검색. 반환: {paraIdx, distance, text, container}

  rhwpDev.help()
    이 도움말 표시

%c사용 예시%c

  %c// 1) "사업계획" 모든 매치 검색%c
  const hits = rhwpDev.search("사업계획");
  // → 9 match(es), console.table 자동 표시

  %c// 2) 3번째 매치 영역 영역 이동 + 선택%c
  rhwpDev.goto(hits[2]);
  //   인덱스 0=첫 매치, 1=두번째, 2=세번째...

  %c// 3) 캐럿만 이동 (선택 부재)%c
  rhwpDev.goto(hits[2], { select: false });

  %c// 4) 표 셀 포함 검색%c
  const cellHits = rhwpDev.search("text", true);
  // SearchHit.cellContext 영역 영역 표 셀 매치 식별

  %c// 5) 셀 매치 영역 영역 이동%c
  const cellHit = cellHits.find(h => h.cellContext);
  if (cellHit) rhwpDev.goto(cellHit);

  %c// 6) 한 줄 패턴 (변수 부재)%c
  rhwpDev.goto(rhwpDev.search("text")[0]);

%cSearchHit 구조%c

  {
    sec: number,         // 구역 인덱스
    para: number,        // 본문: 문단 인덱스 / 셀: parentPara
    charOffset: number,  // 문단 안 글자 오프셋
    length: number,      // 매치 길이
    cellContext?: {      // 셀/글상자 매치 시
      parentPara: number,
      ctrlIdx: number,
      cellIdx: number,
      cellPara: number,
    }
  }

%c주의%c
- rhwpDev.goto() 영역 영역 inputHandler 초기화 후 (문서 로드 후) 영역 영역만 동작
- DEV 모드 (vite dev server) 영역 영역만 등록 — 프로덕션 빌드 영역 영역 부재`,
        titleStyle, reset,
        'color:#059669;font-weight:bold', reset,    // 개요
        'color:#059669;font-weight:bold', reset,    // 메서드
        'color:#059669;font-weight:bold', reset,    // 사용 예시
        'color:#9ca3af', reset,  // // 1) 주석
        'color:#9ca3af', reset,  // // 2) 주석
        'color:#9ca3af', reset,  // // 3) 주석
        'color:#9ca3af', reset,  // // 4) 주석
        'color:#9ca3af', reset,  // // 5) 주석
        'color:#9ca3af', reset,  // // 6) 주석
        'color:#059669;font-weight:bold', reset,    // SearchHit 구조
        'color:#dc2626;font-weight:bold', reset,    // 주의
      );
    },
  };

  (window as any).rhwpDev = dev;
  console.log(
    '%c[rhwpDev]%c 개발자 도구 로드 — %crhwpDev.help()%c 영역 영역 사용 방법 점검',
    'color:#2563eb;font-weight:bold',
    'color:inherit',
    'color:#059669;font-weight:bold;background:#f0fdf4;padding:1px 4px',
    'color:inherit',
  );
}
