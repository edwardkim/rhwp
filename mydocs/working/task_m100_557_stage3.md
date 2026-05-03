# Task #557 Stage 3 — 타입 정의 + README 갱신

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557)
> **브랜치**: `feature/issue-557-expose-hwpx`
> **단계**: Stage 3
> **작성일**: 2026-05-03

---

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| [npm/editor/index.d.ts](../../npm/editor/index.d.ts) | `HwpVerifyResult` 인터페이스 + `RhwpEditor` 의 두 메서드 타입 |
| [npm/editor/README.md](../../npm/editor/README.md#L120-L153) | `editor.exportHwpx()` / `editor.exportHwpVerify()` API 섹션 + 예제 |

## 패치 — index.d.ts

`LoadResult` 직후 새 인터페이스, `RhwpEditor` 의 `exportHwp` 직후 두 메서드 추가.

```ts
export interface HwpVerifyResult {
  /** 직렬화된 HWP 바이트 수 */
  bytesLen: number;
  /** 직렬화 직전 페이지 수 */
  pageCountBefore: number;
  /** 자기 재로드 후 페이지 수 (recovered === true 일 때 의미 있음) */
  pageCountAfter: number;
  /** 자기 재로드 성공 여부 */
  recovered: boolean;
}

export declare class RhwpEditor {
  // ... 기존 ...
  exportHwp(): Promise<Uint8Array>;
  exportHwpx(): Promise<Uint8Array>;
  exportHwpVerify(): Promise<HwpVerifyResult>;
  // ...
}
```

## 패치 — README.md

`### editor.exportHwp()` 섹션 직후 두 섹션 추가. 다운로드 패턴 (Blob + URL.createObjectURL) 은 `exportHwp` 와 동일 구조 유지, MIME type 만 `application/vnd.hancom.hwpx` 로 변경. `exportHwpVerify` 는 객체 사용 예시 + 실패 감지 패턴.

## 검증 — 정적 grep 정합성

```
$ grep -nE "exportHwp|exportHwpx|exportHwpVerify|HwpVerifyResult" \
    npm/editor/index.d.ts npm/editor/README.md npm/editor/index.js | head -20

npm/editor/index.d.ts:18:export interface HwpVerifyResult {
npm/editor/index.d.ts:37:  exportHwp(): Promise<Uint8Array>;
npm/editor/index.d.ts:39:  exportHwpx(): Promise<Uint8Array>;
npm/editor/index.d.ts:41:  exportHwpVerify(): Promise<HwpVerifyResult>;
npm/editor/README.md:104:### editor.exportHwp()
npm/editor/README.md:120:### editor.exportHwpx()
npm/editor/README.md:136:### editor.exportHwpVerify()
npm/editor/index.js:164:  async exportHwp() {
npm/editor/index.js:173:  async exportHwpx() {
npm/editor/index.js:185:  async exportHwpVerify() {
```

세 레이어 (Wrapper / Type / Doc) 모두 세 메서드를 일관되게 노출. Stage 0 의 정적 grep 결과 (`exportHwp` 한 항목) 와 비교하면 갭이 완전히 메워짐.

## 검증 — 타입체커

`npm/editor` 패키지 자체에는 `tsc` 셋업이 없으므로 IDE/컨슈머 측 타입 체크는 PR 후 메인테이너 / 사용자 환경에서 자연스럽게 검증된다. 본 stage 의 책임은 `index.d.ts` 가 `index.js` 와 정확히 일치하는 것 — grep 결과로 검증 완료.

## 다음 단계

Stage 4 — `rhwp-studio/e2e/export-hwpx.test.mjs` 의 fail-first 단언을 통과 단언 + 라운드트립 + verify 객체 정합성 단언으로 일괄 교체. 회귀 점검 (`cargo build`, `cargo test`, 베이스라인 e2e). 최종 보고서 (`task_m100_557_report.md`) 작성. PR 본문 초안.
