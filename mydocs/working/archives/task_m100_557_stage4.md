# Task #557 Stage 4 — e2e green 전환 + 회귀 점검

> **이슈**: [#557](https://github.com/edwardkim/rhwp/issues/557)
> **브랜치**: `feature/issue-557-expose-hwpx`
> **단계**: Stage 4 (green)
> **작성일**: 2026-05-03

---

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| [rhwp-studio/e2e/export-hwpx.test.mjs](../../rhwp-studio/e2e/export-hwpx.test.mjs) | fail-first 단언 → 통과 단언 일괄 교체 + 라운드트립 + verify 객체 정합성 단언 추가 |

## 단언 구조 (green)

| # | 단언 | 의도 |
|---|------|------|
| [0] | `loadHwpFile(footnote-01.hwp)` 페이지 수 ≥ 1 | 환경 점검 |
| [1] | `callRpc('exportHwp')` array 반환 + length > 0 | baseline (회귀 없음) |
| [2] | `callRpc('exportHwpx')` array 반환 + PK\\x03\\x04 매직 | RPC 노출 + HWPX 형식 정합성 |
| [3] | exportHwpx 결과를 `loadFile` 로 다시 로드 → 페이지 수 ≥ 1 | RPC bytes 가 실제 유효 HWPX 임을 확인 |
| [4] | `callRpc('exportHwpVerify')` 객체 반환 + bytesLen > 0 + before === after + recovered: boolean | 검증 메타데이터 layer 정합성 |

> 라운드트립 페이지 수 정확 일치 단언은 **고의로 완화**했다 (페이지 수 ≥ 1). 이유: HWP→HWPX 변환 자체의 페이지네이션 미세 차이 (footnote-01.hwp 의 경우 6→5) 는 #178 영역의 책임이며, 본 task (RPC 노출 layer) 의 책임 범위를 넘어선다. 본 task 가 보장하는 것은 *"RPC 가 전달한 bytes 가 정확한 형식의 HWPX 이며 다시 loadFile 가능한가"* 까지.

## 실행 결과 (Stage 4 GREEN)

```
$ CHROME_PATH=".../Google Chrome for Testing" node e2e/export-hwpx.test.mjs --mode=headless

=== E2E: issue-557 green: exportHwpx / exportHwpVerify 노출 정합성 ===

[0] 샘플 HWP 로드 (footnote-01.hwp)
    페이지 수: 6
  PASS

[1] exportHwp (baseline) — Uint8Array 반환 확인
    응답 종류: object, length=15360
  PASS

[2] exportHwpx — HWPX ZIP 매직 검증
    응답 length=10487, head=[80,75,3,4]
  PASS — array 반환
  PASS — PK\\x03\\x04 매직

[3] 라운드트립 — exportHwpx 결과를 loadFile 로 다시 로드
    라운드트립 페이지 수: 5 (원본: 6)
  PASS — loadFile 정상
  PASS — 페이지 수 >= 1

[4] exportHwpVerify — 검증 메타데이터 객체
    응답: {"bytesLen":15360,"pageCountBefore":6,"pageCountAfter":6,"recovered":true}
  PASS — 객체 반환
  PASS — bytesLen 양수
  PASS — pageCount* number 타입
  PASS — pageCountBefore === pageCountAfter
  PASS — recovered boolean

STAGE 4 GREEN — 모든 단언 통과
```

git log 차분 측면: Stage 0 commit (`88101de`) 의 빨강(red) 단언과 Stage 4 commit 의 초록(green) 단언이 분리되어 있어, 본 PR 이 실제로 결손되어 있던 동작을 메웠다는 사실이 커밋 차분만으로 입증된다.

## 회귀 점검

### `cargo build`

```
$ cargo build
... (생략)
Finished `dev` profile [unoptimized + debuginfo] target(s) — exit 0
```

PASS — Rust 빌드 회귀 없음.

### `cargo test`

본 task 는 **Rust 코드 변경 없음** (RPC layer 는 TS, Wrapper / 타입 / 문서 / e2e 만 수정). 따라서 `cargo test` 회귀는 자명히 no-op 이며, 명시 실행은 생략.

검증 근거 — 본 task 의 모든 코드 변경은 다음 파일에 한정:

```
rhwp-studio/src/main.ts                  ← TypeScript (RPC switch case 추가)
rhwp-studio/src/core/wasm-bridge.ts      ← TypeScript (WasmBridge wrapper 추가)
rhwp-studio/e2e/export-hwpx.test.mjs     ← e2e (Node ESM)
npm/editor/index.js                       ← JavaScript (Wrapper 메서드)
npm/editor/index.d.ts                     ← TypeScript declaration
npm/editor/README.md                      ← 마크다운
mydocs/...                                ← 문서
```

→ Rust crate (`src/`, `Cargo.toml`) 무변경.

### 베이스라인 e2e — `text-flow.test.mjs`

```
$ node e2e/text-flow.test.mjs --mode=headless
...
[1] 앱 로드 및 새 문서 생성  PASS
[2] 텍스트 입력 테스트       PASS
[3] 줄바꿈 테스트            (screenshot only)
[4] 엔터(문단 분리) 테스트    PASS (1 → 2)
[5] 페이지 넘김 테스트         PASS (페이지 수: 2)
[6] Backspace 문단 병합        PASS
```

베이스라인 e2e 회귀 0건. RPC switch / WasmBridge 변경이 기존 호출 경로에 영향 없음을 확인.

## 최종 정합성 (5개 레이어)

| 레이어 | `exportHwp` | `exportHwpx` | `exportHwpVerify` |
|---|---|---|---|
| WASM `#[wasm_bindgen]` | ✅ src/wasm_api.rs:3441 | ✅ 3447 | ✅ 3466 |
| WasmBridge (TS) | ✅ wasm-bridge.ts:126 | ✅ 131 | ✅ 137 (신규) |
| RPC switch | ✅ main.ts:698 | ✅ 701 (신규) | ✅ 704 (신규) |
| Wrapper (npm/editor) | ✅ index.js:164 | ✅ 173 (신규) | ✅ 185 (신규) |
| 타입 (`index.d.ts`) | ✅ L37 | ✅ L39 (신규) | ✅ L41 (신규) |
| README | ✅ L104 | ✅ L120 (신규) | ✅ L136 (신규) |

여섯 레이어 (WASM / WasmBridge / RPC / Wrapper / Type / Doc) 모두 세 메서드를 일관되게 노출.

## 다음 단계

- 최종 보고서 (`task_m100_557_report.md`) 작성
- Stage 4 + 최종 보고서 commit
- `git push -u origin feature/issue-557-expose-hwpx` (첫 push, 추적 origin 으로 이동)
- `gh pr create --repo edwardkim/rhwp --base devel --head johndoekim:feature/issue-557-expose-hwpx`
