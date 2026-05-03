# Task #557: 구현계획서 — npm/editor RPC + Wrapper 에 exportHwpx / exportHwpVerify 노출

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557) (#407 후속)
> **브랜치**: `feature/issue-557-expose-hwpx` (베이스: `upstream/devel`)
> **수행계획서**: [task_m100_557.md](task_m100_557.md)
> **작성일**: 2026-05-03

---

## 개요

WASM 측에 이미 노출된 `exportHwpx` / `exportHwpVerify` 두 함수를 npm/editor RPC + Wrapper + 타입 + 문서 + e2e 테스트 5개 레이어에 동일 패턴으로 추가한다. 코드 변경량은 매우 작고 (각 stage 당 1~2개 파일, 5~30줄), 패턴은 #407 의 `exportHwp` 추가를 그대로 모방한다.

## 사전 환경 확인 (선행 검증)

| 항목 | 상태 |
|------|------|
| working tree 깨끗 (lockfile 잡음 처리됨) | ✅ |
| `pkg/` WASM 산출물에 세 함수 노출 | ✅ `exportHwp` / `exportHwpx` / `exportHwpVerify` 모두 `pkg/rhwp.d.ts` 에 존재 |
| vite dev server `:7700` 가동 | ✅ |
| 사용자 수동 검증 (Stage 0 fail 재현) | 🟡 사용자 콘솔에서 `Unknown method: exportHwpx` 응답 확인 진행 중 |

## Stage 0 — 실패 e2e 작성 (red)

### 변경 파일

- `rhwp-studio/e2e/export-hwpx.test.mjs` (신규)
- `mydocs/working/task_m100_557_stage0.md` (단계 보고서)

### 코드 (요지)

`rhwp-studio/e2e/text-flow.test.mjs` + `helpers.mjs` 의 puppeteer-core 패턴을 모방. 핵심 단언만:

```js
// fail-first
import puppeteer from 'puppeteer-core';
import { connectOrLaunch, loadSampleHwp } from './helpers.mjs';

const { browser, page } = await connectOrLaunch();
await loadSampleHwp(page, 'samples/2010-01-06.hwp');

// 1) Wrapper 메서드 부재
const r1 = await page.evaluate(async () => {
  try { await window.editor.exportHwpx(); return { ok: true }; }
  catch (e) { return { ok: false, name: e.name, message: e.message }; }
});
console.assert(r1.ok === false && /not a function/i.test(r1.message),
  `expected TypeError exportHwpx is not a function, got ${JSON.stringify(r1)}`);

// 2) RPC 단계 default
const r2 = await page.evaluate(() => new Promise((resolve) => {
  const id = Math.random();
  const h = (e) => { if (e.data?.id === id) { window.removeEventListener('message', h); resolve(e.data); } };
  window.addEventListener('message', h);
  window.postMessage({ type: 'rhwp-request', id, method: 'exportHwpx', params: {} }, '*');
}));
console.assert(/Unknown method: exportHwpx/.test(r2.error),
  `expected RPC default Unknown method, got ${JSON.stringify(r2)}`);

// 3) exportHwpVerify 도 동일
// (생략 — 동일 패턴)

console.log('STAGE 0 RED — 두 메서드 모두 노출 갭 확인');
await browser.close();
```

> 베이스라인 헬퍼(`connectOrLaunch`, `loadSampleHwp`)가 helpers.mjs 에 있는지 Stage 0 작업 시 확인하고 없으면 새로 추가하지 않고 `text-flow.test.mjs` 의 init 패턴을 인라인 복사한다 (단순 fail-first 테스트라 의존성 최소화).

### 검증

```
node rhwp-studio/e2e/export-hwpx.test.mjs
# → STAGE 0 RED — 두 메서드 모두 노출 갭 확인
```

종료 코드 0 (단언 실패 없음 = "예상한 fail 이 정확히 일어남" 을 의미). 사용자가 이미 콘솔에서 확인한 동일 fail 을 자동화 형태로 캡처하는 것이 본 단계의 목적.

### 단계 보고서

`mydocs/working/task_m100_557_stage0.md` 에 다음을 기록:
- 정적 증거 (grep 표)
- 사용자가 콘솔에서 받은 응답 캡처
- 자동화 e2e 출력 로그
- "구현 단계로 진행 가능" 결론

### 커밋

```
git add rhwp-studio/e2e/export-hwpx.test.mjs \
        mydocs/plans/task_m100_557.md \
        mydocs/plans/task_m100_557_impl.md \
        mydocs/working/task_m100_557_stage0.md
git commit -m "Task #557: stage 0 - fail-first e2e 로 노출 갭 캡처"
```

→ 사용자 승인 후 첫 push (`git push -u origin feature/issue-557-expose-hwpx`)

---

## Stage 1 — RPC 핸들러 추가

### 변경 파일

- `rhwp-studio/src/main.ts` (RPC switch 에 case 2개 추가)
- `mydocs/working/task_m100_557_stage1.md`

### 패치 (정확한 위치)

[rhwp-studio/src/main.ts:698-700](../../rhwp-studio/src/main.ts) `case 'exportHwp'` 직후, `case 'ready'` 직전:

```ts
case 'exportHwpx':
  reply(Array.from(wasm.exportHwpx()));
  break;
case 'exportHwpVerify':
  // WASM 은 JSON 문자열을 반환 — Wrapper 사용자가 매번 파싱하지 않도록 RPC 단계에서 객체화
  reply(JSON.parse(wasm.exportHwpVerify()));
  break;
```

### 검증

1. vite dev server 자동 HMR (이미 가동 중) → reload 후 콘솔에서:
   ```js
   await callRpc('exportHwpx');       // {result: [...]} (문서 미로드면 다른 에러, 단 'Unknown method' 아님)
   await callRpc('exportHwpVerify');  // {result: {bytesLen, ...}} 또는 다른 에러
   ```
2. `cargo build` (네이티브 회귀 없음 확인)
3. `node rhwp-studio/e2e/export-hwpx.test.mjs` — Stage 0 단언 중 RPC default 부분이 깨짐 (의도된 부분 실패). Wrapper 단언은 여전히 fail. 본 stage 후 e2e 는 일시적으로 mixed 상태 — Stage 4 에서 일괄 green 화.

### 커밋

```
git add rhwp-studio/src/main.ts mydocs/working/task_m100_557_stage1.md
git commit -m "Task #557: stage 1 - RPC switch 에 exportHwpx/exportHwpVerify case 추가"
git push
```

---

## Stage 2 — Wrapper 메서드 + JSDoc

### 변경 파일

- `npm/editor/index.js`
- `mydocs/working/task_m100_557_stage2.md`

### 패치

[npm/editor/index.js:167](../../npm/editor/index.js#L167) `async exportHwp()` 직후, `get element()` 직전:

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
 * @returns {Promise<{bytesLen: number, pageCountBefore: number, pageCountAfter: number, recovered: boolean}>}
 */
async exportHwpVerify() {
  return this._request('exportHwpVerify');
}
```

### 검증

콘솔에서 Wrapper 객체로 직접 호출 (npm/editor 가 사용되는 부모 페이지가 별도로 없으면 임시 데모 HTML 또는 e2e 로 검증).

가장 빠른 검증: e2e 의 `r1` 단언이 바뀐다 → Stage 0 fail-first 에서 `not a function` 단언이 깨짐 (의도된 부분 실패).

### 커밋

```
git add npm/editor/index.js mydocs/working/task_m100_557_stage2.md
git commit -m "Task #557: stage 2 - npm/editor Wrapper 에 exportHwpx/exportHwpVerify 메서드 추가"
git push
```

---

## Stage 3 — 타입 정의 + README

### 변경 파일

- `npm/editor/index.d.ts`
- `npm/editor/README.md`
- `mydocs/working/task_m100_557_stage3.md`

### 패치 — index.d.ts

[npm/editor/index.d.ts:14-16](../../npm/editor/index.d.ts#L14-L16) `LoadResult` 인터페이스 직후 새 인터페이스 추가:

```ts
export interface HwpVerifyResult {
  bytesLen: number;
  pageCountBefore: number;
  pageCountAfter: number;
  recovered: boolean;
}
```

[npm/editor/index.d.ts:26](../../npm/editor/index.d.ts#L26) `exportHwp(): Promise<Uint8Array>;` 직후 두 줄 추가:

```ts
  /** 현재 문서를 HWPX 바이너리로 내보냅니다 */
  exportHwpx(): Promise<Uint8Array>;
  /** HWP 직렬화 + 자기 재로드 검증 결과 (#178) */
  exportHwpVerify(): Promise<HwpVerifyResult>;
```

### 패치 — README.md

[npm/editor/README.md:104](../../npm/editor/README.md#L104) `### editor.exportHwp()` 섹션 직후 두 섹션 추가:

```markdown
### editor.exportHwpx()

현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다.

```javascript
const bytes = await editor.exportHwpx();   // Uint8Array
const blob = new Blob([bytes], { type: 'application/vnd.hancom.hwpx' });
const url = URL.createObjectURL(blob);
// 다운로드 링크 등에 활용
```

### editor.exportHwpVerify()

HWP 직렬화 + 자기 재로드 검증 메타데이터를 반환합니다 (#178). 검증 메타만 반환하며, 실제 bytes 가 필요하면 `exportHwp()` 를 별도 호출.

```javascript
const verify = await editor.exportHwpVerify();
// { bytesLen, pageCountBefore, pageCountAfter, recovered }
```
```

API 표가 README 상단에 별도로 있으면 함께 갱신 (Stage 작업 시 README 전체를 한 번 살피고 결정).

### 검증

- `tsc --noEmit` 또는 IDE 타입체커 (npm/editor 자체에 tsc 셋업이 없으면 컨슈머 환경에서 확인 필요 — 단, 본 PR 에서는 `index.d.ts` 정합성만 보장)
- README 미리보기 마크다운 렌더링 확인

### 커밋

```
git add npm/editor/index.d.ts npm/editor/README.md mydocs/working/task_m100_557_stage3.md
git commit -m "Task #557: stage 3 - 타입 정의 + README API 표 갱신"
git push
```

---

## Stage 4 — e2e green 전환

### 변경 파일

- `rhwp-studio/e2e/export-hwpx.test.mjs` (Stage 0 의 fail 단언을 통과 단언으로 교체 + 라운드트립 검증 추가)
- `mydocs/working/task_m100_557_stage4.md`
- `mydocs/report/task_m100_557_report.md` (최종 보고서)

### 패치 — e2e

```js
import puppeteer from 'puppeteer-core';
import { connectOrLaunch, loadSampleHwp } from './helpers.mjs';

const { browser, page } = await connectOrLaunch();
await loadSampleHwp(page, 'samples/2010-01-06.hwp');

const pageCountBefore = await page.evaluate(() => window.editor.pageCount());

// 1) exportHwpx → Uint8Array (PK 매직)
const hwpxBytes = await page.evaluate(async () => {
  const arr = await window.editor.exportHwpx();
  return Array.from(arr.slice(0, 4));
});
console.assert(hwpxBytes[0] === 0x50 && hwpxBytes[1] === 0x4B && hwpxBytes[2] === 0x03 && hwpxBytes[3] === 0x04,
  `HWPX must start with PK\\x03\\x04, got ${hwpxBytes}`);

// 2) 라운드트립 — exportHwpx → loadFile → 페이지 수 일치
const pageCountAfter = await page.evaluate(async () => {
  const bytes = await window.editor.exportHwpx();
  const result = await window.editor.loadFile(bytes.buffer, 'roundtrip.hwpx');
  return result.pageCount;
});
console.assert(pageCountAfter === pageCountBefore,
  `roundtrip page count mismatch: before=${pageCountBefore}, after=${pageCountAfter}`);

// 3) exportHwpVerify → 객체 정합성
const verify = await page.evaluate(() => window.editor.exportHwpVerify());
console.assert(typeof verify.bytesLen === 'number' && verify.bytesLen > 0
  && verify.pageCountBefore === verify.pageCountAfter && verify.recovered === true,
  `verify object invalid: ${JSON.stringify(verify)}`);

console.log('STAGE 4 GREEN — 모두 통과');
await browser.close();
```

### 검증 (전체 회귀 점검)

```
cargo build
cargo test
cd rhwp-studio
node e2e/export-hwpx.test.mjs                    # 본 task
node e2e/text-flow.test.mjs                       # 베이스라인 회귀 점검 (선택, 한두 개 핵심)
node e2e/edit-pipeline.test.mjs                   # (선택)
```

### 최종 보고서

`mydocs/report/task_m100_557_report.md` 에 다음 기록:
- 변경 파일 목록 + 라인 차분 요약
- Stage 0 → Stage 4 의 e2e 출력 변화 (red → green 캡처)
- 회귀 검사 결과
- PR 본문 초안 (제목/본문/closes #557)

### 커밋

```
git add rhwp-studio/e2e/export-hwpx.test.mjs \
        mydocs/working/task_m100_557_stage4.md \
        mydocs/report/task_m100_557_report.md
git commit -m "Task #557: stage 4 - e2e green 전환 + 라운드트립 + 최종 보고서"
git push
```

---

## PR 생성 (Stage 4 완료 + 최종 검증 통과 후)

```
gh pr create --repo edwardkim/rhwp \
  --base devel --head johndoekim:feature/issue-557-expose-hwpx \
  --title "npm/editor RPC + Wrapper에 exportHwpx / exportHwpVerify 노출 (#557, followup #407)" \
  --body-file mydocs/report/task_m100_557_pr_body.md
```

PR 본문 끝줄에 `closes #557` 명시 (이슈 자동 close).

---

## 회귀 / 안전 장치

- WASM 코어는 수정하지 않음 → wasm 회귀 0%
- 본 변경은 RPC switch 에 2 case 추가, Wrapper 에 2 메서드 추가, 타입/문서 부수 갱신 → 기존 호출 경로 변경 없음
- 베이스라인 e2e (`text-flow.test.mjs` 등) 회귀 0건 확인 필수
- npm/editor 패키지 버전은 본 PR 에서 갱신하지 않음 (릴리즈 시점 일괄 처리)

## 단계별 승인 포인트

| 시점 | 승인 받을 항목 |
|------|----------------|
| 본 구현계획서 작성 직후 | 본 문서 자체 (다음 단계 = Stage 0 진행 여부) |
| 각 Stage 완료 후 | 단계 보고서 + git push 여부 |
| Stage 4 + 최종 보고서 작성 후 | PR 생성 (`gh pr create`) |
