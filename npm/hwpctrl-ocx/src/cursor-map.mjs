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
