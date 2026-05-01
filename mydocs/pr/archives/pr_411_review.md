# PR #411 검토 — `editor.exportHwp()` API 추가

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#411](https://github.com/edwardkim/rhwp/pull/411) |
| 작성자 | [@ggoban](https://github.com/ggoban) — 신규 컨트리뷰터 (첫 PR) |
| base / head | `devel` ← `ggoban:feature/editor-export-hwp` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 통계 | +30 / -0, 4 files |
| **CI** | **statusCheckRollup 비어있음** — 첫 PR 정황 |
| 이슈 | (본문 명시 없음) — 이슈 #377 (Export API 가이드, close 됨) 의 후속으로 추정 |

## 작성자 정황

@ggoban — **신규 컨트리뷰터 (첫 PR)**. 머지 이력 0건.

## 변경 내용

### 본질 — `@rhwp/editor` (iframe wrapper) 에 `exportHwp()` 공개 API 추가

- `@rhwp/core` 의 WASM 은 `exportHwp()` 직접 노출 (Vec\<u8\> → Uint8Array)
- 그러나 iframe 기반 `@rhwp/editor` 에서는 message channel 경유 필요 — 본 PR 이 그 경로를 명시적 API 로 노출

### 코드 변경 (4 files / +30 / -0)

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/main.ts` | message handler 에 `case 'exportHwp'` 추가 — `Array.from(wasm.exportHwp())` 응답 |
| `npm/editor/index.js` | `RhwpEditor.exportHwp()` 메서드 추가 (`_request('exportHwp')` → Uint8Array 변환) |
| `npm/editor/index.d.ts` | TypeScript declaration 추가 |
| `npm/editor/README.md` | HWP bytes 다운로드 예시 추가 |

### 메시지 흐름

```
부모 페이지              iframe (rhwp-studio)
  │                       │
  │  postMessage:         │
  │  rhwp-request          │
  │  method='exportHwp'    │
  │ ────────────────────►  │
  │                        │
  │                        │ wasm.exportHwp() → Uint8Array
  │                        │ Array.from(...) → number[] (postMessage 가능)
  │                        │
  │  postMessage:          │
  │  rhwp-response         │
  │  result=number[]       │
  │ ◄────────────────────  │
  │                        │
  │ new Uint8Array(result) │
  │ → return Promise<Uint8Array>
```

### `Array.from(Uint8Array)` 정황

작성자가 일반 array (`number[]`) 로 변환 후 postMessage 전달 — postMessage 의 structured clone 호환성. 받는 측에서 `result instanceof Uint8Array ? result : new Uint8Array(result || [])` 로 안전한 방어 코드.

**성능 정황**: 큰 파일 (예: hwpspec.hwp 4 MB) 에서 `Array.from` 변환은 메모리 사용량 증가. 그러나:
- postMessage 의 structured clone 은 Uint8Array 도 직접 지원 (transferable)
- 다른 case (`getPageSvg` 등) 와 같은 일관 패턴이라 본 PR 만 변경하는 건 부적절

→ **별도 task 후보** (postMessage 를 transferable Uint8Array 로 통일 — 별개 PR / 별개 task)

## 검증

### 본 검토에서 dry-run merge 결과

devel 위에 자동 머지 성공. 머지 후 검증:

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1037 passed (동일 — Rust core 영향 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 통과 |

→ 모든 검증 통과. Rust 영향 없음, TS 컴파일도 정상.

## 평가

### 강점

1. **변경 범위 매우 작음** (+30 / -0, 4 files) — deletion 0
2. **WASM core (`exportHwp`) 이미 존재** — 본 PR 은 iframe 경로 노출만 추가
3. **API surface 좁음** — 단일 메서드 + TS 타입 + README 예시
4. **기존 패턴과 일관** — `loadFile`, `pageCount`, `getPageSvg` 등의 message handler 와 같은 흐름
5. **TypeScript declaration 포함** — TS 사용자에게도 정상 노출
6. **README 예시 명확** — Blob 생성 + 다운로드 링크 패턴
7. **Rust core 변경 없음** — 회귀 위험 없음
8. **dry-run merge** — 자동 성공
9. **README + TS 문서화** — 사용자 친화

### 약점 / 점검 필요

#### 1. CI 실행 안 됨

`statusCheckRollup` 비어있음 (PR #397, #400, #405 와 같은 정황). 첫 PR 정황으로 추정.

#### 2. 이슈 연결 명시 없음

PR 본문 "관련 이슈" 섹션 비어있음. 이슈 #377 (Export API 가이드, close 됨) 의 후속으로 추정되지만 명시 없음. 작성자가 이슈 작성 절차를 알지 못한 정황 가능성.

#### 3. devel BEHIND

PR #395, #396, #401, #405 머지 전 base. 다행히 자동 머지 성공.

#### 4. 작성자 빌드 검증 정황

PR 본문 명시:
> 참고: `rhwp-studio`의 `npm run build`는 현재 로컬 checkout에서 생성된 WASM alias 파일 `@wasm/rhwp.js`가 없어 실행이 중단되었습니다. 변경 범위는 JS/TS wrapper 및 message handler이며, Rust core는 수정하지 않았습니다.

→ 작성자가 WASM 빌드 못해서 rhwp-studio 빌드 검증 못함. 다만 변경 자체가 rhwp-studio 빌드를 손상시키는 영역 아님 (TS 컴파일은 본 검토에서 통과 확인).

#### 5. message handler 의 `params` 처리

```javascript
case 'exportHwp':
    reply(Array.from(wasm.exportHwp()));
    break;
```

`exportHwp` 는 인자 없는 호출이라 `params` 처리 불필요 — 정확.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR 의 영향 파일 (4 files) 은 다른 PR 과 무관:
- `rhwp-studio/src/main.ts` — Task #394 (투명선) 시점 변경 후 본 PR 과 다른 영역 ✅
- `npm/editor/{index.js,index.d.ts,README.md}` — 다른 PR 미변경 ✅

dry-run merge 자동 성공 확인.

## 처리 방향 후보

| 옵션 | 내용 |
|------|------|
| **A** | cherry-pick 머지 (작성자 attribution 보존) |
| B | 작성자에게 rebase 요청 후 재제출 (PR #397 패턴) |
| C | 거절 / close (추천 안 함) |

## 권장 — 옵션 A (cherry-pick 머지)

이유:
1. **변경 범위 작음** — 4 files / +30 / -0
2. **Rust core 변경 없음** → 회귀 위험 없음
3. **기존 패턴 일관** — message handler / TS wrapper
4. **dry-run merge 자동 성공** + 1037 passed
5. **이슈 #377 (Export API 가이드) 의 자연스러운 후속** — 가이드만으로는 부족했던 iframe wrapper API 직접 노출
6. **신규 컨트리뷰터의 첫 기여** — 절차 부담 큰 재제출 요청보다 메인테이너가 직접 처리

### 시각 판정 정황

본 PR 은 **API 노출만 — 시각적 변경 없음** → 시각 판정 불필요.

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 신규 컨트리뷰터 (첫 PR), 변경 단순 ⚠️
- [x] 코드 품질 — 합리적, 기존 패턴 일관 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1037 passed (동일) ✅
- [x] cargo clippy — warning 0 ✅
- [x] tsc --noEmit (rhwp-studio) — 통과 ✅
- [x] Rust core 변경 없음 — 회귀 위험 없음 ✅
- [ ] CI 실행 — 비어있음 (본 검토에서 cargo test + tsc 로 보완) ⚠️

## 다음 단계 — 작업지시자 결정

A / B / C 중 결정 부탁드립니다.

권장: **A** — cherry-pick 머지 (작성자 attribution 보존).

## 참고

- PR: [#411](https://github.com/edwardkim/rhwp/pull/411)
- 관련 이슈 (작업지시자 추정): [#377](https://github.com/edwardkim/rhwp/issues/377) (Export API 가이드, close 됨)
- 관련 위키: [Export API 사용 가이드](https://github.com/edwardkim/rhwp/wiki/Export-API-사용-가이드)
