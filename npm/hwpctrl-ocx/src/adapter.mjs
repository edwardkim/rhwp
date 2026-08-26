/**
 * 실행 모드 어댑터 — 이 층이 문서를 **소유하는가 빌리는가**를 가르는 자리.
 *
 * 두 모드는 같은 API 구현(`index.mjs`)을 공유한다. 갈라 두면 호환 원장이 모드마다 달라져
 * "312/484" 라는 수치가 무의미해진다. 다른 것은 셋뿐이다 — 문서 소유, undo, 문서 교체.
 *
 * | | standalone | plugin |
 * | --- | --- | --- |
 * | 문서 | 이 층이 만든다 | studio 가 소유하고 빌려준다 |
 * | undo | 이 층의 자체 스냅샷 | studio `CommandHistory` 트랜잭션 |
 * | 교체 | `new HwpDocument(...)` | 호스트 `loadDocument` 위임 |
 *
 * 이 파일은 studio 를 import 하지 않는다. 호스트의 **모양**만 안다.
 */

/**
 * 문서를 바꾸지 **않는** API 이름.
 *
 * 분류의 기본값은 "바꾼다" 이다 — 모르는 이름은 트랜잭션으로 감싼다. 반대로 두면 미분류
 * 뮤테이션이 undo 밖으로 새고, 그것이 studio 에서 가장 비싼 종류의 버그다(#2027 계급).
 * 여기 이름을 더할 때는 그 API 가 정말 문서를 안 바꾸는지 확인해야 한다.
 */
export const READ_ONLY_METHODS = new Set([
  // 문서 조회
  'PageCount', 'GetTextFile', 'GetText', 'GetPageText', 'GetFieldText', 'GetFieldList',
  'GetCurFieldName', 'GetFieldViewOption', 'IsModified', 'Version',
  // 커서·선택 조회
  'GetPos', 'GetPosBySet', 'GetSelectedPos', 'GetSelectedPosBySet',
  // 문서 속성 조회
  'GetDocumentInfo', 'GetHeadingString', 'GetMessageBoxMode',
  // 훑기 — InitScan/GetText/ReleaseScan 은 커서 상태만 쓴다
  'InitScan', 'ReleaseScan',
  // 파라미터셋 조회
  'CreateSet', 'GetCurFieldName',
]);

/** 이름으로 트랜잭션 필요 여부를 판정한다. 모르면 **바꾼다**고 본다. */
export function isMutating(method) {
  return !READ_ONLY_METHODS.has(method);
}

/**
 * plugin 모드의 문서 교체 훅.
 *
 * `index.mjs` 의 `adoptDocument` 계약을 채운다 — 바이트를 받으면 호스트에 열기를 위임하고,
 * 빈 문서 요청이면 호스트의 새 문서 경로를 태운 뒤, **새로 빌린 핸들**을 돌려준다.
 */
export function createAdoptDocument(host) {
  return async ({ bytes, blank } = {}) => {
    if (blank) host.createBlankDocument();
    else await host.loadDocument(bytes);

    const lease = host.borrowDocument();
    if (!lease) throw new Error('문서 교체 후 핸들을 빌릴 수 없습니다');
    return lease.handle;
  };
}
