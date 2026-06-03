# Task #557 Stage 1 — RPC switch 에 exportHwpx / exportHwpVerify case 추가

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557)
> **브랜치**: `feature/issue-557-expose-hwpx`
> **단계**: Stage 1
> **작성일**: 2026-05-03

---

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| [rhwp-studio/src/main.ts](../../rhwp-studio/src/main.ts#L698-L706) | RPC `switch` 에 `case 'exportHwpx'` / `case 'exportHwpVerify'` 추가 |
| [rhwp-studio/src/core/wasm-bridge.ts](../../rhwp-studio/src/core/wasm-bridge.ts#L131-L142) | `WasmBridge` 타입에 `exportHwpVerify(): string` wrapper 메서드 추가 |

## 패치 (main.ts)

```ts
case 'exportHwp':
  reply(Array.from(wasm.exportHwp()));
  break;
case 'exportHwpx':                                   // 신규
  reply(Array.from(wasm.exportHwpx()));
  break;
case 'exportHwpVerify':                              // 신규
  // WASM 은 JSON 문자열을 반환 — Wrapper 사용자가 매번 파싱하지 않도록 RPC 단계에서 객체화
  reply(JSON.parse(wasm.exportHwpVerify()));
  break;
case 'ready':
```

## 패치 (wasm-bridge.ts) — 부수 변경

타입 진단 `'WasmBridge' 형식에 'exportHwpVerify' 속성이 없습니다` 를 해결하기 위해 `WasmBridge` 클래스에 wrapper 메서드 추가. `exportHwp` / `exportHwpx` 의 패턴을 그대로 모방.

```ts
/** HWP 직렬화 + 자기 재로드 검증 메타데이터를 JSON 문자열로 반환 (#178). */
exportHwpVerify(): string {
  if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
  return this.doc.exportHwpVerify();
}
```

WASM 측 `exportHwpx` 는 이미 `WasmBridge.exportHwpx()` 로 wrap 되어 있어 추가 작업 없음.

## 검증

### e2e 재실행 — 의도된 부분 실패

```
$ node e2e/export-hwpx.test.mjs --mode=headless

[1] exportHwp (기준선) — method 인식 여부
    응답: {"error":"문서가 로드되지 않았습니다"}
  PASS

[2] exportHwpx — RPC default 단언 (현재 미구현 기대)
    응답: {"error":"문서가 로드되지 않았습니다"}              ← 'Unknown method' 가 아님
  FAIL: exportHwpx 가 'Unknown method' 응답이어야 함

[3] exportHwpVerify — RPC default 단언 (현재 미구현 기대)
    응답: {"error":"문서가 로드되지 않았습니다"}              ← 'Unknown method' 가 아님
  FAIL: exportHwpVerify 가 'Unknown method' 응답이어야 함
```

→ Stage 0 의 단언 [2], [3] 이 깨졌다. 이는 plan 에 명시한 **의도된 부분 실패**. RPC switch 가 두 메서드를 인식하면서 wasm 호출까지 진행 → 문서 미로드 에러 (`exportHwp` 와 동일 baseline 으로 합류). RPC 노출 갭이 메워진 결정적 증거.

Stage 4 에서 통과 단언 (`Uint8Array` / 객체 정합성 / 라운드트립) 으로 일괄 교체.

### 타입 에러

본 패치 후 `tsc` (IDE 진단) 에서 본 task 관련 타입 에러 없음. unused 변수 hint (`CellPathEntry`, `ruler`, `styleBar`) 는 사전 존재하던 항목으로 본 task 무관.

## 다음 단계

Stage 2 — `npm/editor/index.js` 의 `RhwpEditor` 클래스에 `exportHwpx()` / `exportHwpVerify()` 메서드 추가.
