/**
 * 커서 좌표계 변환 — 한글 `{list, para, pos}` ↔ studio `{sectionIndex, parentParaIndex, cellPath}`.
 *
 * 두 좌표계는 다르다. 한글은 리스트 아이디 하나로 평면화하고, studio 는 구역·문단·셀 경로로
 * 위치를 표현한다. 다행히 변환에 필요한 사실은 **`getCursorModel()` 이 이미 전부 준다** —
 * `hostListId` 사슬을 따라 루트까지 올라가면 구역과 셀 경로가 나온다.
 *
 * 실측(2026-08-11): 단층 표는 셀 문단 길이 3→7, 중첩 표(깊이 2)는 11→15 로 **정확히 그 셀**을
 * 지목했다. 중첩에서 `cellParaIndex` 를 0 으로 고정하면 실패한다 — 자식 표가 놓인 **부모 셀 안의
 * 문단 번호**가 반드시 들어가야 한다.
 */

/** `getCursorModel()` 결과에서 listId → 엔트리 색인을 만든다. */
export function indexLists(model) {
  const lists = (model && model.lists) || [];
  return new Map(lists.map((entry) => [entry.listId, entry]));
}

/** 루트(본문)까지의 리스트 사슬. 바깥부터 안쪽 순서다. */
export function listChain(byId, listId, guardLimit = 64) {
  const chain = [];
  let cur = byId.get(listId);
  let guard = 0;
  while (cur && guard < guardLimit) {
    chain.unshift(cur);
    if (cur.hostListId === 0) return chain;
    cur = byId.get(cur.hostListId);
    guard += 1;
  }
  return chain.length && chain[0].hostListId === 0 ? chain : null;
}

/**
 * 한글 리스트 아이디를 studio 좌표로.
 *
 * 본문(`listId === 0`)은 셀 경로가 없다. 셀 경로의 마지막 칸이 목표 셀이고, 그 앞칸들의
 * `cellParaIndex` 는 다음 단계 표가 놓인 문단이다.
 */
export function listToStudio(model, listId, targetCellParaIndex = 0) {
  if (listId === 0) return { sectionIndex: 0, parentParaIndex: 0, cellPath: [] };
  const byId = indexLists(model);
  const chain = listChain(byId, listId);
  if (!chain || !chain.length) return null;

  return {
    sectionIndex: chain[0].sectionIndex,
    parentParaIndex: chain[0].hostPara,
    cellPath: chain.map((entry, i) => ({
      controlIndex: entry.controlIndex,
      cellIndex: entry.cellIndex,
      cellParaIndex: i + 1 < chain.length ? chain[i + 1].hostPara : targetCellParaIndex,
    })),
  };
}

/**
 * studio 좌표를 한글 리스트 아이디로.
 *
 * 경로의 마지막 칸이 목표 셀이므로, 그 칸의 `(controlIndex, cellIndex)` 와 바로 위 단계의
 * 위치가 함께 맞는 리스트를 찾는다. 본문 좌표(빈 경로)는 `0` 이다.
 */
export function studioToList(model, { sectionIndex = 0, parentParaIndex = 0, cellPath = [] } = {}) {
  if (!cellPath.length) return 0;
  const byId = indexLists(model);

  for (const entry of byId.values()) {
    const chain = listChain(byId, entry.listId);
    if (!chain || chain.length !== cellPath.length) continue;
    if (chain[0].sectionIndex !== sectionIndex || chain[0].hostPara !== parentParaIndex) continue;

    const same = chain.every((link, i) =>
      link.controlIndex === cellPath[i].controlIndex && link.cellIndex === cellPath[i].cellIndex);
    if (same) return entry.listId;
  }
  return null;
}

/** 사슬 깊이. 본문은 0, 단층 셀은 1, 중첩 셀은 2 이상이다. */
export function listDepth(model, listId) {
  if (listId === 0) return 0;
  const chain = listChain(indexLists(model), listId);
  return chain ? chain.length : -1;
}

/**
 * 글자 번호를 문단 안의 **코드 유닛 자리**(한글 `pos`)로 옮긴다.
 *
 * 코어에는 반대쪽(`getCharIndexAtStreamPos`)만 있다. 그 함수는 `pos` 이상인 첫 글자 번호를 주므로
 * 자리가 커질수록 줄지 않는다. 그래서 캐럿이 설 수 있는 자리(`getCaretStops`) 가운데 글자 번호가
 * `charOffset` 이하인 **가장 큰 자리**가 그 글자의 자리다 — 누름틀 시작 코드 앞 자리와 안내문 시작이
 * 같은 글자로 모일 때는 뒤쪽(필드 안)을 고른다. 이분 탐색이라 호출은 log₂(자리 수) 번이다.
 *
 * @param {number[]} stops 오름차순 캐럿 자리(코드 유닛)
 * @param {(pos: number) => number} charIndexAt 자리 → 글자 번호
 * @param {number} charOffset 글자 번호
 */
export function streamPosAtChar(stops, charIndexAt, charOffset) {
  if (!stops.length) return 0;
  let lo = 0;
  let hi = stops.length - 1;
  if (charIndexAt(stops[lo]) > charOffset) return stops[lo];
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (charIndexAt(stops[mid]) <= charOffset) lo = mid;
    else hi = mid - 1;
  }
  return stops[lo];
}

/** 확장 컨트롤(누름틀 시작·끝 코드) 하나가 차지하는 코드 유닛. */
const FIELD_CODE_UNITS = 8;

/**
 * studio 캐럿을 한글 커서 좌표 `{list, para, pos}` 로.
 *
 * 본문은 구역을 가로질러 하나의 리스트라 문단 번호에 앞 구역의 문단 수를 더한다. 표 칸은
 * `studioToList` 로 리스트를 찾고, 그 리스트 안 문단은 경로 마지막 칸의 `cellParaIndex` 다.
 * 옮길 수 없는 좌표면 `null`.
 *
 * 누름틀 경계에서는 같은 글자 번호가 여러 자리다. `caret.field` 가 화면에서 본 쪽을 알려 주면
 * 그 누름틀의 자리(`getFieldList` 의 `startPos`·`endPos`)로 맞춘다 — 안쪽이면 내용 범위 안(빈
 * 누름틀은 안내문 시작), 시작 코드 앞으로 나간 상태면 시작 코드 앞이다. 끝 코드 뒤는 기본값과 같다.
 *
 * @param {object} doc `getCursorModel`·`getParagraphCount`·`getCaretStops`·`getCharIndexAtStreamPos`·`getFieldList` 를 가진 문서
 * @param {{sectionIndex:number,parentParaIndex:number,cellPath:Array<{controlIndex:number,cellIndex:number,cellParaIndex:number}>,charOffset:number,field?:{id:number,at:'inside'|'before'|'after'}}} caret
 */
export function caretToCursor(doc, caret) {
  if (!caret) return null;
  const cellPath = caret.cellPath || [];
  let list = 0;
  let para = caret.parentParaIndex;
  if (cellPath.length) {
    list = studioToList(JSON.parse(doc.getCursorModel()), caret);
    if (list == null) return null;
    para = cellPath[cellPath.length - 1].cellParaIndex;
  } else {
    for (let s = 0; s < caret.sectionIndex; s += 1) para += doc.getParagraphCount(s);
  }
  const stops = JSON.parse(doc.getCaretStops(list, para) || '[]');
  const charIndexAt = (pos) => JSON.parse(doc.getCharIndexAtStreamPos(list, para, pos)).charIndex;
  let pos = streamPosAtChar(stops, charIndexAt, caret.charOffset);

  if (caret.field && caret.field.at !== 'after') {
    // 문단은 비교하지 않는다 — `getFieldList` 의 본문 `paraInList` 는 구역 안 번호라 둘째 구역부터
    // 커서 모델의 누적 번호와 다르다. 필드 아이디와 리스트로 충분하다.
    const field = JSON.parse(doc.getFieldList()).find((f) =>
      f.fieldId === caret.field.id && !f.cellField && f.listId === list);
    if (field && caret.field.at === 'before') {
      pos = field.startPos - FIELD_CODE_UNITS;
    } else if (field) {
      pos = field.startCharIdx === field.endCharIdx
        ? field.startPos
        : Math.min(Math.max(pos, field.startPos), field.endPos);
    }
  }
  return { list, para, pos };
}
