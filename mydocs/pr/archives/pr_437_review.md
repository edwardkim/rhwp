# PR #437 검토 — 브라우저 확장 문서 URL 해석 및 응답 검증 (#432)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#437](https://github.com/edwardkim/rhwp/pull/437) |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) — 신뢰 컨트리뷰터 |
| 이슈 | [#432](https://github.com/edwardkim/rhwp/issues/432) (Refs, 작성자 본인 이슈) |
| base / head | `devel` ← `postmelee:contrib/fix-document-url-resolver` |
| 변경 규모 | +1,250 / -26, 17 files (6 commits) |
| mergeable | `MERGEABLE` (CLEAN) |
| 검토 일자 | 2026-04-29 |
| 처리 결정 | **옵션 C — 작성자 후속 정정 요청** (작업지시자 직접 결정) |

## 본질

### 문제

GitHub wiki / blob 링크가 `.hwp` / `.hwpx` 처럼 보여도 실제로는 HTML 미리보기 페이지를 반환 → viewer 가 그대로 WASM 파서에 넘겨 CFB 오류 표시.

예: `samples/복학원서.hwp` 같은 GitHub wiki 카드 링크가 GitHub blob HTML 페이지를 반환 → viewer 에서 "파일 손상" 메시지.

### 해결 — 두 계층 일반화

1. **URL 해석 계층** (`rhwp-shared/sw/document-url-resolver.js`):
   - provider 별 파일 URL 변환 (예: `github.com/.../blob/...hwp` → `raw.githubusercontent.com/...`)
   - provider adapter 구조 — 향후 공공기관/교육청 게시판 등 확장 여지

2. **응답 검증 계층** (`rhwp-studio/src/main.ts::loadFromUrlParam`):
   - WASM 파서 호출 전 CFB / ZIP 시그니처 확인
   - HTML / 오류 페이지는 파서 호출 전에 차단
   - 원인 중심 메시지 표시 ("실제 HWP/HWPX 파일이 아닙니다")

## 변경 파일

### 신규 추가

| 파일 | 역할 |
|------|------|
| `rhwp-shared/sw/document-url-resolver.js` | URL 해석 본체 (79 라인) |
| `rhwp-shared/sw/document-url-resolver.test.js` | 12 단위 테스트 (89 라인) |

### 기존 파일 수정

| 파일 | 변경 |
|------|------|
| `rhwp-chrome/sw/document-url-resolver.js` | shared 재내보내기 |
| `rhwp-chrome/sw/thumbnail-extractor.js` | resolver 적용 (+3/-1) |
| `rhwp-chrome/sw/viewer-launcher.js` | resolver 적용 (+14/-12) |
| `rhwp-firefox/sw/*` | 동일 |
| `rhwp-studio/src/main.ts` | 응답 바이트 검증 추가 (+47) |

### 문서 (하이퍼-워터폴 절차 정확 준수 ✅)

- `mydocs/plans/task_m100_432.md` (수행계획서, 151 라인)
- `mydocs/plans/task_m100_432_impl.md` (구현계획서, 249 라인)
- `mydocs/working/task_m100_432_stage1~4.md` (단계별 보고서)
- `mydocs/report/task_m100_432_report.md` (최종 보고서)
- `mydocs/orders/20260429.md` (오늘할일 갱신)

## 처리 방향 — 옵션 C (작업지시자 결정)

본 PR 은 코드 본질 정상 + 검증 게이트 모두 통과했지만, 작은 정황 발견됨 — **작성자 명시한 자동 테스트 실행 명령이 실제로는 작동 안 함**. 옵션 C 로 작성자 자체 정정 + 절차 학습 기회.

### 발견된 정황

PR 본문 명시:
> 실행: `node --test rhwp-shared/sw/document-url-resolver.test.js`

실행 시 에러:
```
(node:XXXX) Warning: To load an ES module, set "type": "module" in the package.json or use the .mjs extension.
SyntaxError: Cannot use import statement outside a module
```

**원인**: `rhwp-shared/` 에 `package.json` 부재 + `.test.js` 확장자 → Node 가 CJS 로 해석 → ESM `import` 충돌.

**런타임 영향**: 없음 — 확장 (Chrome / Firefox SW) 환경에서는 ESM 정상 동작.

**자동 테스트 영향**: 후속 컨트리뷰터가 회귀 검증 누락 위험.

### 두 가지 정정 선택지 안내

**선택지 1** (권장): `rhwp-shared/package.json` 신규 추가 + `"type": "module"` 명시
- 변경 범위 최소
- PR 본문 명시 실행 명령이 그대로 동작
- 향후 `rhwp-shared/` ESM 추가 시 일괄 적용

**선택지 2**: 테스트 파일 `.test.mjs` 확장자로 rename + import 경로 갱신
- package.json 추가 불필요
- 단점: import 경로 갱신 범위 크고 확장 빌드 영향 점검 필요

→ **선택지 1 권장**.

## dry-run cherry-pick 결과

`local/pr437` 브랜치 (`local/devel` 분기) 에서 5 commits cherry-pick (merge commit 1개 제외) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `edc9a76` (cherry-picked) | @postmelee | Task #432: Add document URL resolver |
| `46a47a6` (cherry-picked) | @postmelee | Task #432: Resolve document URLs before viewer fetch |
| `8f9a8e3` (← `94bb46b`) | @postmelee | Task #432: Validate remote document bytes before parsing |
| `d2e2d1e` (← `4577ba6`) | @postmelee | Task #432: Add final verification report |
| `8a61bdc` (← `dd2fb2c`) | @postmelee | Task #432: Record manual verification results |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (회귀 0건) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| TypeScript 검증 (`tsc --noEmit`) | ✅ 통과 |
| **document-url-resolver 단위 테스트** | ✅ **12 tests 모두 통과** (`.mjs` 확장자로 실행 시 — `.test.js` 로는 실행 불가) |
| **Rust core 영향 0 (byte 비교)** | ✅ devel ↔ PR #437 byte 단위 동일 (4,103,796 bytes) |

## 본 PR 의 좋은 점

- **이슈 #432 의 본질 정확 식별** — "특정 GitHub wiki 링크 rule-based" 가 아닌 **provider adapter 구조** 로 일반화. 향후 공공기관/교육청 게시판 등 점진 대응 가능
- **두 계층 분리** (URL 해석 + 응답 검증) 가 깔끔. resolver 가 모르는 사이트도 응답 검증 계층이 HTML/오류 페이지 차단
- **하이퍼-워터폴 절차 정확 준수** (계획서 + 4 stage + 보고서)
- **Rust core 영향 0** — 영향 범위 한정 명확
- **단위 테스트 12건 추가** (확장 자동화 가능)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + TypeScript |
| 자기 라운드트립 ≠ 한컴 호환 | (해당 없음 — 본 PR 은 Rust core 영향 0, 한컴 호환 게이트 무관) |
| 보고서는 타스크 브랜치에서 커밋 | (옵션 C 정황 — 작성자 정정 commit 추가 후 메인테이너 보고서 commit) |
| 작은 단위 PATCH 회전 | ✅ 본 PR 은 단순 영역 (확장 + studio), Rust core 무관 |

## 다음 단계

1. ✅ [작성자 후속 정정 요청 댓글](https://github.com/edwardkim/rhwp/pull/437#issuecomment-4340728393) 작성
2. PR #437 OPEN 유지 — 작성자 후속 commit 대기
3. 작성자 정정 후 재검토 → cherry-pick 머지 진행
4. `local/pr437` 브랜치 보존 (작성자 보강 commit 흡수 시 재사용)

## 참고

- PR: [#437](https://github.com/edwardkim/rhwp/pull/437)
- 이슈: [#432](https://github.com/edwardkim/rhwp/issues/432)
- 같은 작성자 머지 PR: [#168](https://github.com/edwardkim/rhwp/pull/168), [#169](https://github.com/edwardkim/rhwp/pull/169), [#339](https://github.com/edwardkim/rhwp/pull/339)
- 본 작성자 origin 이슈 (다른 컨트리뷰터에게 양도): [#364](https://github.com/edwardkim/rhwp/issues/364) (PR #419 로 흡수)
