# Task #557 Stage 0 — fail-first e2e 로 RPC 노출 갭 자동화 캡처

> **이슈**: [#557](https://github.com/edwardkim/edwardkim/rhwp/issues/557) (#407 후속)
> **브랜치**: `feature/issue-557-expose-hwpx`
> **단계**: Stage 0 (red)
> **작성일**: 2026-05-03

---

## 목적

본 task 가 메꾸려는 결손 (`exportHwpx` / `exportHwpVerify` 의 RPC + Wrapper 노출) 이 실제로 동작하지 않음을 자동화 단언으로 캡처한다. 구현 단계로 넘어가기 전 **빨강(red)** 상태가 정확히 잡혀야 한다.

## 정적 증거 (사전 확보)

```
$ grep -nE "case '(exportHwp|exportHwpx|exportHwpVerify)'" rhwp-studio/src/main.ts
698:      case 'exportHwp':       ← exportHwpx, exportHwpVerify 없음

$ grep -n "exportHwp\b\|exportHwpx\|exportHwpVerify" npm/editor/index.js npm/editor/index.d.ts npm/editor/README.md
npm/editor/index.js:164:  async exportHwp() {
npm/editor/index.js:165:    const result = await this._request('exportHwp');
npm/editor/index.d.ts:26:  exportHwp(): Promise<Uint8Array>;
npm/editor/README.md:104:### editor.exportHwp()

$ grep -nE "#\[wasm_bindgen\(js_name = (exportHwp|exportHwpx|exportHwpVerify)" src/wasm_api.rs
3441:    #[wasm_bindgen(js_name = exportHwp)]
3447:    #[wasm_bindgen(js_name = exportHwpx)]
3466:    #[wasm_bindgen(js_name = exportHwpVerify)]
```

요점: WASM 측은 세 함수 모두 노출, RPC switch / Wrapper / 타입 / 문서는 `exportHwp` 한 항목만 존재.

## 동적 증거 (사용자 수동 검증, dev server `:7700` Chrome 콘솔)

사용자가 dev server 가동 후 동일 페이지 콘솔에서 `postMessage` RPC 직접 호출하여 확인한 응답:

```
exportHwp        → {type:'rhwp-response', error:"문서가 로드되지 않았습니다"}    ← method 인식 (baseline)
exportHwpx       → {type:'rhwp-response', error:"Unknown method: exportHwpx"}     ← default 분기
exportHwpVerify  → {type:'rhwp-response', error:"Unknown method: exportHwpVerify"}← default 분기
```

## 자동화 e2e 캡처

### 추가 파일

- [rhwp-studio/e2e/export-hwpx.test.mjs](../../rhwp-studio/e2e/export-hwpx.test.mjs) — fail-first

### 단언 구조

1. **기준선 (baseline)**: `callRpc('exportHwp')` 응답에 `Unknown method` 가 **없어야** 한다 — RPC switch 가 method 를 인식한다는 증명.
2. **갭 단언 1**: `callRpc('exportHwpx')` 응답에 `Unknown method: exportHwpx` 가 **있어야** 한다 — 현재 미구현 캡처.
3. **갭 단언 2**: `callRpc('exportHwpVerify')` 동일.

### 실행 결과 (Stage 0 red 캡처)

```
$ CHROME_PATH="$HOME/.cache/puppeteer/chrome/mac_arm-127.0.6533.119/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" \
  node e2e/export-hwpx.test.mjs --mode=headless

=== E2E: issue-557 fail-first: exportHwpx / exportHwpVerify RPC 노출 갭 ===

  [browser] headless Chrome 실행

[1] exportHwp (기준선) — method 인식 여부
    응답: {"type":"rhwp-response","id":120721462,"error":"문서가 로드되지 않았습니다"}
  PASS: exportHwp 는 RPC 가 인식해야 함 (current error: 문서가 로드되지 않았습니다)

[2] exportHwpx — RPC default 단언 (현재 미구현 기대)
    응답: {"type":"rhwp-response","id":430322126,"error":"Unknown method: exportHwpx"}
  PASS: exportHwpx 가 'Unknown method' 응답이어야 함

[3] exportHwpVerify — RPC default 단언 (현재 미구현 기대)
    응답: {"type":"rhwp-response","id":674023134,"error":"Unknown method: exportHwpVerify"}
  PASS: exportHwpVerify 가 'Unknown method' 응답이어야 함

STAGE 0 RED — RPC 노출 갭 자동화 캡처 완료
```

세 단언 모두 PASS — 빨강 상태가 정확히 잡혔다. Stage 1 에서 RPC switch 에 case 두 개를 추가하면 단언 [2], [3] 이 깨지고 (의도된 부분 실패), Stage 4 에서 통과 단언으로 일괄 교체된다.

## Wrapper 단계 갭 — e2e 미포함 사유

본 e2e 는 `rhwp-studio` 페이지를 직접 띄우므로 `window.editor` (npm/editor `RhwpEditor` 인스턴스) 는 존재하지 않는다. Wrapper 메서드 부재 (`editor.exportHwpx is not a function`) 검증은 본 e2e 의 자동화 범위에서 제외하고 다음 두 가지로 갈음한다:

- **정적 grep**: 위 `npm/editor/index.js`, `index.d.ts` 결과 — Wrapper 메서드 부재 명백
- **타입 체크**: `index.d.ts` 가 두 메서드를 export 하지 않으므로 컨슈머 측 `tsc` 가 자동으로 잡음

## 다음 단계

Stage 1 — RPC switch 에 `case 'exportHwpx'` / `case 'exportHwpVerify'` 추가. 본 stage0 단언이 일부 깨지면 (의도된 부분 실패) Stage 4 에서 통과 단언으로 일괄 교체.

## 산출물

- `rhwp-studio/e2e/export-hwpx.test.mjs` (신규)
- `mydocs/plans/task_m100_557.md` (수행계획서, 이미 작성)
- `mydocs/plans/task_m100_557_impl.md` (구현계획서, 이미 작성)
- 본 보고서 `mydocs/working/task_m100_557_stage0.md`
