# Task #833 Stage 2 (회귀 검증) 보고서

**브랜치**: `local/task833`
**선행**: Stage 1 (GREEN) 완료
**목표**: 본 task 변경 (TS 만) 의 회귀 영향 검증

## 자동 검증

### cargo test (release)

```
$ cargo test --release
... (모든 test 슈트 실행)
passed=1351 failed=0
```

> 1351 통과 (Task #825/#826 PR #832 머지 전 base 기준). 본 task 는 TS 만 변경이므로 Rust test 회귀 무영향 — 모두 통과 정상.

### cargo clippy

```
$ cargo clippy --release -- -D warnings
Finished `release` profile [optimized] target(s) in 7.78s
```

clean.

### TypeScript

```
$ cd rhwp-studio && npx tsc --noEmit
(clean)
```

clean.

## 회귀 영향 분석

본 task 변경은 모두 TypeScript 만:
- `rhwp-studio/src/command/file-system-access.ts` — `forceSaveAs?: boolean` optional 추가 (기본값 false → 기존 동작 유지)
- `rhwp-studio/src/command/commands/file.ts` — `isUserCancelError` helper + `file:save` catch 정정 (기존 AbortError swallow 유지 + NotAllowedError 추가) + `file:save-as` 신규 command
- `rhwp-studio/src/command/shortcut-map.ts` — Ctrl+Shift+S 신규 binding (기존 binding 영향 없음)
- `rhwp-studio/index.html` — 메뉴 항목 1개 추가

→ Rust 컴파일 / WASM 빌드 무영향. 기존 `file:save` 동작은 catch 의 NotAllowedError 추가 swallow 만 변경 (기존 AbortError 처리 회귀 부재).

## WASM 재빌드

**불필요** — TS 만 변경. 기존 pkg 그대로 사용.

## 다음 단계

Stage 3 (시각 검증) — Vite hot-reload 즉시 적용. 작업지시자 시각 판정 요청:
- (A) Save As: 파일 메뉴 + showSaveFilePicker + Ctrl+Shift+S 단축키 + currentFileHandle 갱신
- (B) Cancel fallback 정정: Ctrl+S → "변경사항 저장" 프롬프트 → "취소" → download 미발현
