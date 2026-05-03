# Task #557 Stage 2 — npm/editor Wrapper 에 exportHwpx / exportHwpVerify 메서드 추가

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557)
> **브랜치**: `feature/issue-557-expose-hwpx`
> **단계**: Stage 2
> **작성일**: 2026-05-03

---

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| [npm/editor/index.js](../../npm/editor/index.js#L172-L191) | `RhwpEditor` 클래스에 `exportHwpx()` / `exportHwpVerify()` 메서드 + JSDoc 추가 |

## 패치

`exportHwp()` 직후, `get element()` 직전에 두 메서드 삽입. `exportHwp` 패턴 그대로 모방 (Uint8Array 복원 포함).

```js
/**
 * 현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다.
 * @returns {Promise<Uint8Array>} HWPX 파일 bytes
 */
async exportHwpx() {
  const result = await this._request('exportHwpx');
  return result instanceof Uint8Array ? result : new Uint8Array(result || []);
}

/**
 * HWP 직렬화 + 자기 재로드 검증 메타데이터를 반환합니다 (#178).
 *
 * 검증 메타데이터만 반환하며, 실제 HWP bytes 가 필요하면 `exportHwp()` 를 별도 호출하세요.
 *
 * @returns {Promise<{bytesLen: number, pageCountBefore: number, pageCountAfter: number, recovered: boolean}>}
 */
async exportHwpVerify() {
  return this._request('exportHwpVerify');
}
```

## 검증 — 정적

```
$ grep -nE "async (exportHwp|exportHwpx|exportHwpVerify)" npm/editor/index.js
164:  async exportHwp() {
173:  async exportHwpx() {
185:  async exportHwpVerify() {
```

세 메서드 모두 노출. Stage 0 의 정적 grep 결과 (`exportHwp` 만 존재) 와 비교하면 갭이 메워짐.

## 검증 — 자동화 e2e

본 stage 의 변경은 npm/editor Wrapper 만 건드리며 RPC layer 는 손대지 않았으므로 [rhwp-studio/e2e/export-hwpx.test.mjs](../../rhwp-studio/e2e/export-hwpx.test.mjs) 의 응답은 Stage 1 결과와 동일하다 (변동 없음). Wrapper 검증은 본 e2e 의 자동화 범위 외 — RPC 응답이 정상이면 Wrapper 는 단순 `_request` wrapping 이므로 논리적으로 정상.

부모 페이지에서 `import { createEditor } from '@rhwp/editor'` 후 `editor.exportHwpx()` / `editor.exportHwpVerify()` 호출 시:

- 메서드 존재 → `TypeError: editor.exportHwpx is not a function` 미발생 (Stage 0 수동 정적 증거 갭 해소)
- RPC `exportHwpx` 호출 → Stage 1 코드 경로 진입 → 문서 미로드 시 *"문서가 로드되지 않았습니다"*, 로드 후 `Uint8Array` (HWPX bytes) 반환

Stage 4 의 통과 단언에서 정합성을 일괄 검증.

## 다음 단계

Stage 3 — `npm/editor/index.d.ts` 타입 정의 + `npm/editor/README.md` API 문서 갱신.
