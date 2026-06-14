# PR #1399 검토 — createEmpty 기본 A4 구역/빈 문단 생성

- PR: https://github.com/edwardkim/rhwp/pull/1399
- 제목: `feat: createEmpty가 기본 A4 구역 1개 + 빈 문단을 포함해 즉시 편집 가능 (#1386)`
- 작성일: 2026-06-13
- 작성자: `oksure`
- 관련 이슈: #1386
- base: `devel`
- head: `oksure:contrib/fix-1386-create-empty-section` (`80c072cb62696878330f8537b6f1a4ff0c2a8383`)
- 검토 브랜치: `review/pr-1399`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 공개 WASM API `HwpDocument.createEmpty()`가 구역 0개 문서를 만들던 문제를 고쳐,
기본 A4 구역 1개와 빈 문단 1개를 포함한 즉시 편집 가능한 문서를 생성한다. 내부
`DocumentCore::new_empty()` 계약은 유지하고 공개 API 경계에서만 기본 문서 골격을 보강해,
기존 내부 테스트 스캐폴딩 영향 범위를 줄인 점이 적절하다.

로컬 macOS 권장 검증과 추가 WASM 빌드 검증이 모두 통과했다. 머지 전 GitHub Actions는
문서 커밋 push 이후 새 head 기준으로 다시 확인한다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeStateStatus | `BLOCKED` — 문서 커밋 전 `Build & Test` 진행 중 |
| 변경량 | 3 files, +72 / -1 |
| 작성자 | `oksure` |
| closing issues | PR 본문에 `Closes #1386` 명시 |

커밋:

- `04d49c2e` — `feat: createEmpty가 기본 A4 구역 1개 + 빈 문단을 포함해 즉시 편집 가능하도록 수정 (#1386)`
- `a88189fb` — `refactor: A4 기본 용지를 PageDef::a4_default()로 공용화 (#1386 리뷰 반영)`
- `110636a7` — `Merge branch 'devel' into contrib/fix-1386-create-empty-section`
- `80c072cb` — `Merge branch 'devel' into contrib/fix-1386-create-empty-section`

문서 반영 커밋:

- `0f5b9359` — `docs: PR #1400 처리 문서 정리`
- PR #1399 검토 문서와 2026-06-13 오늘할일 갱신 커밋을 PR head에 추가 예정

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | 문서 커밋 전 head 기준 진행 중 |
| Canvas visual diff | 문서 커밋 전 head 기준 pass |
| CodeQL | 문서 커밋 전 head 기준 pass |
| Analyze rust | 문서 커밋 전 head 기준 pass |
| Analyze javascript-typescript | 문서 커밋 전 head 기준 pass |
| Analyze python | 문서 커밋 전 head 기준 pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 코드 변경

`src/model/page.rs`:

- `PageDef::a4_default()`를 추가해 한컴 새 문서 기본 용지값을 모델 레벨에서 재사용한다.
- A4 세로 용지, 좌우 30mm, 위 20mm, 아래 15mm, 머리말/꼬리말 15mm 값을 사용한다.

`src/wasm_api.rs`:

- `HwpDocument::create_empty()`가 빈 `DocumentCore`에 기본 구역과 빈 문단을 구성한다.
- `set_document()`를 통해 스타일, composed cache, pagination 초기화를 기존 경로에 맡긴다.

`src/wasm_api/tests.rs`:

- `test_create_empty_document_is_editable` 회귀 테스트를 추가한다.
- section count, text insert/range query, HWP/HWPX export, 재파싱 후 텍스트 보존을 확인한다.

### 3.2 기존 계약과의 정합

- `DocumentCore::new_empty()`는 그대로 유지되어 내부 테스트가 직접 section을 push하던 기존 사용법을 깨지 않는다.
- 공개 API `createEmpty()`만 사용자 기대에 맞는 편집 가능한 기본 문서를 반환한다.
- 기본 A4 값은 렌더러 테스트 헬퍼와 같은 값으로 단일 생성자에 모았다.

## 4. 로컬 검증

검토 브랜치: `review/pr-1399`

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `cargo build --release` | 통과, 3m 47s |
| `cargo test --release --lib` | 통과, 1752 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과, 1m 41s |

`wasm-pack` 산출물 `pkg/`는 ignored 검증 산출물이므로 커밋 대상에 포함하지 않는다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| 내부 빈 코어 계약 변경 | 낮음 | `DocumentCore::new_empty()` 미변경 |
| 기본 문서 값의 한컴 정합 | 낮음 | 기존 렌더러 A4 테스트 헬퍼와 동일 값 |
| createEmpty 후 pagination/cache 초기화 누락 | 낮음 | `set_document()` 경유 및 회귀 테스트 추가 |

## 6. 권고

로컬 검증 기준으로는 merge 가능하다.

머지 전 마지막 확인:

- PR #1399 head에 이 검토 문서와 오늘할일 갱신 커밋을 push
- PR diff에 `mydocs/pr/archives/pr_1399_review.md`, `mydocs/orders/20260613.md`, PR #1400 문서 정리 커밋이 포함됐는지 확인
- 문서 커밋 push 후 GitHub Actions 전체 통과 확인
- 머지 후 #1386 auto-close 여부 확인
