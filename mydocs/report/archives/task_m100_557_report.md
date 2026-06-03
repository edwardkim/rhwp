# Task #557 최종 보고서 — npm/editor RPC + Wrapper에 exportHwpx / exportHwpVerify 노출 (#407 후속)

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557)
> **브랜치**: `feature/issue-557-expose-hwpx` (Fork: `johndoekim/rhwp`, 베이스: `upstream/devel`)
> **마일스톤**: v1.0.0
> **라벨**: enhancement
> **PR 대상**: `edwardkim/rhwp:devel`
> **작성일**: 2026-05-04
> **상태**: 구현/검증 완료 — push + PR 생성 단계

---

## 작업 요약

#407 (`exportHwp`) 이 closed/merged 된 사례와 동일한 *"WASM 측에는 노출됐지만 RPC + Wrapper 에 누락"* 패턴이 두 함수에서 추가로 발견됐다. 본 task 는 동일 패턴을 그대로 적용해 두 함수를 노출하고 fail-first → green 자동화로 결손을 입증하며 메꿨다.

| 함수 | WASM 시그니처 | 노출 형태 |
|------|--------------|----------|
| `exportHwpx` | `() -> Result<Vec<u8>, JsValue>` | `Promise<Uint8Array>` (HWPX bytes) |
| `exportHwpVerify` | `() -> Result<String, JsValue>` (JSON 문자열) | `Promise<HwpVerifyResult>` (RPC 단계에서 `JSON.parse`) |

## 변경 파일 (6 file)

| 파일 | 추가 라인 | 역할 |
|------|----------|------|
| [rhwp-studio/src/main.ts](../../rhwp-studio/src/main.ts#L701-L707) | +7 | RPC switch case 2개 추가 |
| [rhwp-studio/src/core/wasm-bridge.ts](../../rhwp-studio/src/core/wasm-bridge.ts#L137-L141) | +5 | `WasmBridge.exportHwpVerify()` wrapper |
| [npm/editor/index.js](../../npm/editor/index.js#L169-L188) | +20 | `RhwpEditor.exportHwpx()` / `exportHwpVerify()` |
| [npm/editor/index.d.ts](../../npm/editor/index.d.ts#L18-L41) | +14 | `HwpVerifyResult` interface + 두 메서드 시그니처 |
| [npm/editor/README.md](../../npm/editor/README.md#L120-L153) | +33 | API 섹션 + 다운로드 예제 + 검증 패턴 |
| [rhwp-studio/e2e/export-hwpx.test.mjs](../../rhwp-studio/e2e/export-hwpx.test.mjs) | +89 (신규) | fail-first → green 자동화 |

WASM 코어(`src/wasm_api.rs`) 무변경 — 이미 노출된 두 함수를 노출 layer 에만 바인딩.

## 단계별 진행 (5 stage)

| Stage | 결과 | Commit |
|-------|------|--------|
| 0 — fail-first e2e | RED 단언 3건 PASS (RPC default 캡처) | `88101de` |
| 1 — RPC switch + WasmBridge | Stage 0 단언 [2][3] 깨짐 (의도된 부분 실패) | `91958fb` |
| 2 — Wrapper 메서드 + JSDoc | 정적 grep 정합 (3 메서드 노출) | `3149053` |
| 3 — 타입 정의 + README | 6개 레이어 정합 완성 | `73dcdf9` |
| 4 — e2e green 전환 + 회귀 점검 | 모든 단언 PASS, baseline 회귀 0건 | (본 commit) |

git history 측면: Stage 0 의 RED 커밋과 Stage 4 의 GREEN 커밋이 분리되어 있어, 본 PR 이 실제로 결손되어 있던 동작을 메웠다는 사실이 커밋 차분만으로 입증된다.

## 검증 결과

### 자동화 e2e (Stage 4 GREEN)

```
[0] 샘플 HWP 로드 (footnote-01.hwp)         페이지 수: 6  PASS
[1] exportHwp baseline                      length=15360   PASS (회귀 없음)
[2] exportHwpx HWPX 매직                    head=[80,75,3,4] PASS
[3] 라운드트립 (loadFile 정상 + ≥1 page)    페이지 수=5   PASS
[4] exportHwpVerify 객체 정합성             {bytesLen:15360, pageCountBefore:6, pageCountAfter:6, recovered:true} PASS

STAGE 4 GREEN — 모든 단언 통과
```

> 라운드트립 페이지 수 정확 일치는 HWP→HWPX 변환 자체 (#178 영역) 책임이라 본 task 단언에서는 의도적으로 완화. 본 task 는 *"RPC bytes 가 실제 유효 HWPX 인가"* 까지만 보장.

### 회귀 점검

| 검사 | 결과 |
|------|------|
| `cargo build` | PASS (exit 0) |
| `cargo test` | 자명히 no-op — Rust 코드 무변경 |
| `e2e/text-flow.test.mjs` | PASS (페이지 수, 문단 분리, 페이지 넘김, Backspace 병합) |
| 타입 진단 (`tsc` IDE) | 본 task 관련 에러 0 |

### 정합성 매트릭스 (6 레이어)

| 레이어 | `exportHwp` | `exportHwpx` | `exportHwpVerify` |
|---|---|---|---|
| WASM `#[wasm_bindgen]` | ✅ src/wasm_api.rs:3441 | ✅ 3447 | ✅ 3466 |
| WasmBridge (TS) | ✅ wasm-bridge.ts:126 | ✅ 131 | ✅ **137 (신규)** |
| RPC switch | ✅ main.ts:698 | ✅ **701 (신규)** | ✅ **704 (신규)** |
| Wrapper (npm/editor) | ✅ index.js:164 | ✅ **173 (신규)** | ✅ **185 (신규)** |
| 타입 (`index.d.ts`) | ✅ L37 | ✅ **L39 (신규)** | ✅ **L41 (신규)** |
| README | ✅ L104 | ✅ **L120 (신규)** | ✅ **L136 (신규)** |

## 의도적 선택 / 유의사항

- **`exportHwpVerify` 반환 형태**: WASM 은 JSON 문자열을 반환하지만 RPC 단계에서 `JSON.parse` 후 객체로 보낸다. Wrapper 사용자가 매번 파싱하지 않도록 한 노출 layer 어댑팅. (`exportHwp` / `exportHwpx` 의 `Array.from(...)` 변환과 같은 결.)
- **버전**: `npm/editor/package.json` 의 `version: 0.7.9` 갱신은 본 task 범위 외 (릴리즈 시점 일괄 처리).
- **WASM 재빌드 불필요**: 본 작업은 노출 layer 한정. 기존 docker WASM 빌드 산출물이 두 함수를 그대로 노출.
- **라운드트립 정확 일치 단언 제외**: footnote 등이 포함된 HWP 의 HWP→HWPX 변환 시 페이지네이션 미세 차이 가능. 별도 이슈 (#178 영역) 로 추적.

## 후속 제안 (별도 이슈)

본 PR 의 패턴은 다음 묶음에 동일하게 적용 가능 (각각 별도 이슈/PR 권장).

1. `getDocumentInfo` + `getSourceFormat` + `getValidationWarnings` (단순 getter 3종)
2. `searchText` + `replaceOne` / `replaceAll` 계열 (검색/치환)
3. `saveSnapshot` / `restoreSnapshot` / `discardSnapshot` (undo / snapshot)
4. `renderPageHtml` (SVG 대안)

각 묶음이 본 task 와 동일하게 *"WASM 노출 + WasmBridge wrap + RPC switch + Wrapper + Type + Doc + e2e"* 6 레이어 작업이며 변경량은 묶음당 50~100 줄 수준.

## PR 본문 초안

다음 본문으로 `gh pr create` 진행 예정 (사용자 승인 후).

```
## Summary
- WASM 측에 이미 노출된 `exportHwpx` / `exportHwpVerify` 두 함수를 npm/editor RPC + Wrapper + 타입 + README 에 #407 패턴 그대로 노출.
- fail-first e2e → green 단계 분리 커밋으로 결손 → 메움 입증.

## 변경
- 6개 파일 수정/신규, +168 lines (e2e 포함). WASM 코어 무변경.
- 6 레이어 정합 (WASM / WasmBridge / RPC / Wrapper / Type / Doc).

## 검증
- cargo build PASS, baseline e2e (text-flow) PASS, 본 task e2e 모든 단언 PASS.
- 라운드트립 정확 일치는 #178 영역 책임이라 단언 완화.

## 영역 검토
- #272 (v2.0.0) 에서 메인테이너가 명시한 "WASM HwpDocument 의 high-level method 일부 영역 노출" 카테고리 (v1.0.0 영역) 에 정확히 부합.
- 312 Action / HwpCtrl 노출 / WASM 새 함수 추가 어느 것에도 손대지 않음.

closes #557
```

## 결론

`exportHwpx` / `exportHwpVerify` 의 RPC + Wrapper 노출 갭이 #407 과 동일한 패턴으로 메워졌다. 모든 단언 통과, 회귀 0건, 6 레이어 정합 완성. push + PR 생성을 진행한다.
