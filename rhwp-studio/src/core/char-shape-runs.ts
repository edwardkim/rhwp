import type { CharShapeRun } from './types';

/** fresh binding의 필수 표면. 구버전 WASM에 단일 ID fallback을 하지 않는다. */
export interface CharShapeRunsDocument {
  getCharShapeRuns(sec: number, para: number, start: number, end: number): string;
  setCharShapeRuns(sec: number, para: number, start: number, end: number, json: string): string;
  getCharShapeRunsInCellByPath(sec: number, para: number, path: string, start: number, end: number): string;
  setCharShapeRunsInCellByPath(sec: number, para: number, path: string, start: number, end: number, json: string): string;
}

export function requireCharShapeRunsDocument(doc: unknown): CharShapeRunsDocument {
  const methods = ['getCharShapeRuns', 'setCharShapeRuns', 'getCharShapeRunsInCellByPath', 'setCharShapeRunsInCellByPath'] as const;
  if (!doc || methods.some((key) => typeof (doc as CharShapeRunsDocument)[key] !== 'function')) {
    throw new Error('구간별 글자 서식 복원을 지원하는 최신 WASM이 필요합니다. 앱을 새로고침해 주세요.');
  }
  return doc as CharShapeRunsDocument;
}

export function parseCharShapeRuns(json: string, start: number, end: number): CharShapeRun[] {
  const runs: unknown = JSON.parse(json);
  if (!Array.isArray(runs) || !Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || start > end) {
    throw new Error('잘못된 글자 모양 구간 응답');
  }
  let next = start;
  for (const run of runs) {
    if (!run || Object.keys(run).length !== 3 || !Number.isSafeInteger(run.startOffset) || run.startOffset !== next
      || !Number.isSafeInteger(run.endOffset) || run.endOffset <= next || run.endOffset > end
      || !Number.isSafeInteger(run.charShapeId) || run.charShapeId < 0 || run.charShapeId > 0xffffffff) {
      throw new Error('잘못된 글자 모양 구간 응답');
    }
    next = run.endOffset;
  }
  if (next !== end) throw new Error('글자 모양 구간 응답에 빈 범위가 있습니다');
  return runs;
}
