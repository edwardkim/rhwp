# PR #411 처리 보고서 — `editor.exportHwp()` API 추가

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#411](https://github.com/edwardkim/rhwp/pull/411) |
| 작성자 | [@ggoban](https://github.com/ggoban) — 신규 컨트리뷰터 (첫 PR) |
| 처리 결정 | **옵션 A (cherry-pick 머지)** |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 1: cherry-pick

`local/pr411` 브랜치 (`local/devel` 분기) 에서 PR head 까지 1 commit cherry-pick — 작성자 attribution 보존:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `eebfc25` (cherry-pick) | @ggoban | Add editor exportHwp API |

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1037 passed (동일 — Rust core 영향 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 통과 |

## 변경 요약

### 본질

`@rhwp/editor` (iframe wrapper) 에 `exportHwp()` 공개 API 추가. WASM core (`@rhwp/core`) 의 `exportHwp()` 는 이미 노출됐지만 iframe 기반 wrapper 에서 message channel 경유 명시 API 가 없었음.

### 변경 파일 (+30 / -0)

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/main.ts` | message handler 에 `case 'exportHwp'` 추가 (3 줄) |
| `npm/editor/index.js` | `RhwpEditor.exportHwp()` 메서드 추가 (9 줄) |
| `npm/editor/index.d.ts` | TypeScript declaration 추가 (2 줄) |
| `npm/editor/README.md` | HWP bytes 다운로드 예시 추가 (16 줄) |

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
  │                        │ Array.from(...) → number[]
  │                        │
  │  postMessage:          │
  │  rhwp-response         │
  │  result=number[]       │
  │ ◄────────────────────  │
  │                        │
  │ new Uint8Array(result) │
  │ → return Promise<Uint8Array>
```

기존 message handler (`loadFile`, `pageCount`, `getPageSvg`) 와 같은 패턴.

## 시각 판정 정황

본 PR 은 **API 노출만 — 시각적 변경 없음** → 시각 판정 불필요. Rust core 변경 없으므로 회귀 위험도 없음.

PR #405 와 같은 패턴 (알고리즘 영향 없는 surface 노출).

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1037 + tsc + clippy 0 |
| PR 댓글 톤 — 과도한 표현 자제 | ✅ |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr411` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | (본 PR 은 closes 명시 없음) |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr411` → `local/devel` → `devel` 머지 + push
3. PR #411 close + 작성자 댓글 (이슈 #377 위키 가이드와 함께 사용 안내)

## 참고

- 검토 문서: `mydocs/pr/pr_411_review.md`
- PR: [#411](https://github.com/edwardkim/rhwp/pull/411)
- 관련 이슈: [#377](https://github.com/edwardkim/rhwp/issues/377) (Export API 가이드, close 됨) — 본 PR 의 자연스러운 후속
- 위키: [Export API 사용 가이드](https://github.com/edwardkim/rhwp/wiki/Export-API-사용-가이드)
