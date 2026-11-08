/**
 * 문서에서 온 값의 신뢰 경계.
 *
 * 도구 결과는 사용자가 연 문서의 내용이다. 받은 문서에 「앞의 지시를 무시하고 …」 같은 문장이
 * 숨어 있을 수 있으므로, 모델에는 지시가 아니라 데이터로 건넨다(간접 프롬프트 주입 방어,
 * `mydocs/tech/agent_security/indirect_prompt_injection.md`).
 *
 * 방법: 결과 JSON 을 `<document_content id="…">` 경계로 감싼다. `id` 는 회차마다 새로 만든
 * 난수라 문서가 미리 알 수 없다. 또 JSON 안의 `<`·`>` 를 `<`·`>` 로 적어, 문서
 * 텍스트가 어떤 태그를 품어도 경계를 닫거나 새로 열 수 없게 한다.
 */

export function createBoundaryId(random: () => Uint8Array = () => globalThis.crypto.getRandomValues(new Uint8Array(6))): string {
  return Array.from(random(), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

/** 경계를 깰 수 있는 글자를 JSON 이스케이프로 바꾼다. JSON.parse 결과는 원문과 같다. */
export function encodeUntrustedJson(value: unknown): string {
  return JSON.stringify(value)
    .replace(/</g, '\\u003c')
    .replace(/>/g, '\\u003e')
    .replace(/\u2028/g, '\\u2028')
    .replace(/\u2029/g, '\\u2029');
}

export function wrapDocumentContent(value: unknown, boundaryId: string): string {
  return `<document_content id="${boundaryId}">\n${encodeUntrustedJson(value)}\n</document_content id="${boundaryId}">`;
}

/**
 * 문서 텍스트를 모델에 보이기 좋은 평문으로. CRLF 는 LF 로, 개체 자리표시 같은 제어 문자는
 * 지운다(줄바꿈·탭은 남긴다).
 */
export function cleanDocumentText(text: string): string {
  return text
    .replace(/\r\n?/g, '\n')
    // eslint-disable-next-line no-control-regex
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '');
}
