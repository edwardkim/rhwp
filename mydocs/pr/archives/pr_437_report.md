# PR #437 처리 보고서 — 브라우저 확장 문서 URL 해석 및 응답 검증

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#437](https://github.com/edwardkim/rhwp/pull/437) |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) |
| 이슈 | [#432](https://github.com/edwardkim/rhwp/issues/432) (Refs, 작성자 본인 이슈) |
| 처리 결정 | **cherry-pick 머지** (작성자 후속 정정 commit 1건 추가 흡수) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 1차 검토 (옵션 C — 작성자 후속 정정 요청)

[1차 검토 댓글](https://github.com/edwardkim/rhwp/pull/437#issuecomment-4340728393):
- 코드 본질 정상 + 검증 게이트 모두 통과 + Rust core 영향 0
- 그러나 PR 본문 명시 자동 테스트 명령 (`node --test rhwp-shared/sw/document-url-resolver.test.js`) 이 실제로는 작동 안 함 (rhwp-shared/package.json 부재 → ESM 충돌)
- 두 선택지 안내 (선택지 1 권장: `rhwp-shared/package.json` 신규 추가)

### Stage 1: 작성자 신속 정정

작성자가 1차 검토 후 **약 17분 만에** 권장안 정확 채택:

- 신규 commit `2b06a41` "fix: declare rhwp-shared as ESM package" 추가
- `rhwp-shared/package.json` 신규 (5 라인):
  ```json
  {
    "name": "rhwp-shared",
    "private": true,
    "type": "module"
  }
  ```
- 작성자 본인 재검증 결과:
  - `node --test rhwp-shared/sw/document-url-resolver.test.js`: **12 tests passed** (ESM warning 없이)
  - `node --test rhwp-shared/sw/download-interceptor-common.test.js`: **26 tests passed** (보너스)
  - `rhwp-firefox / rhwp-chrome npm run build`: 둘 다 통과

### Stage 2: 작성자 신규 commit cherry-pick

`local/pr437` 브랜치 (기존 5 commits cherry-pick 위에) `2b06a41` 추가:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `0e58276` | @postmelee | Task #432: Add document URL resolver |
| `0570b46` | @postmelee | Task #432: Resolve document URLs before viewer fetch |
| `8f9a8e3` | @postmelee | Task #432: Validate remote document bytes before parsing |
| `d2e2d1e` | @postmelee | Task #432: Add final verification report |
| `8a61bdc` | @postmelee | Task #432: Record manual verification results |
| `ca4a61a` (← `2b06a41`) | @postmelee | **fix: declare rhwp-shared as ESM package** (Stage 1 후속) |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 3: 메인테이너 재검증

| 항목 | 결과 |
|------|------|
| **document-url-resolver 단위 테스트** | ✅ **12 tests passed** (작성자 PR 본문 명시 명령으로 정확히 실행됨) |
| **download-interceptor-common 단위 테스트** | ✅ **26 tests passed** (보너스 — 다른 ESM 테스트도 함께 정상화) |
| `cargo test --lib` | ✅ **1066 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ 0건 |
| **Rust core 영향 0 (byte 비교)** | ✅ devel ↔ PR #437 byte 단위 동일 (4,103,796 bytes) |

## 변경 요약

### 본질 — 두 계층 일반화

| 계층 | 파일 | 역할 |
|------|------|------|
| **URL 해석** | `rhwp-shared/sw/document-url-resolver.js` (79 라인) | provider adapter 구조 — `github.com/.../blob/...hwp` → `raw.githubusercontent.com/...` 변환. 향후 공공기관/교육청 게시판 등 점진 대응 가능 |
| **응답 검증** | `rhwp-studio/src/main.ts::loadFromUrlParam` (+47) | WASM 파서 호출 전 CFB / ZIP 시그니처 확인. HTML/오류 페이지 차단 |
| **확장 적용** | `rhwp-chrome/sw/{viewer-launcher, thumbnail-extractor}.js`, `rhwp-firefox/sw/*` | resolver + 검증 적용 |
| **단위 테스트** | `rhwp-shared/sw/document-url-resolver.test.js` (89 라인, 12 tests) | 자동 회귀 검증 |
| **ESM 패키지 선언** (Stage 1 후속) | `rhwp-shared/package.json` (5 라인) | 자동 테스트 명령 한 번에 실행 가능 |

### 영향 범위

- **Rust core (라이브러리 본체) 변경 없음** — byte 단위 동일 검증
- **확장 + rhwp-studio** 만 정정
- 사용자 영향: GitHub wiki/blob 의 HWP 링크 카드 정상 동작 (썸네일 + viewer 열기)

## 본 PR 의 좋은 점

1. **이슈 본질 정확 식별** — "특정 GitHub wiki 링크 rule-based" 가 아닌 **provider adapter 구조** 로 일반화
2. **두 계층 분리** (URL 해석 + 응답 검증) — resolver 가 모르는 사이트도 응답 검증 계층이 HTML/오류 페이지 차단
3. **하이퍼-워터폴 절차 정확 준수** (계획서 + 4 stage + 보고서)
4. **단위 테스트 12건 추가** — 자동화 가능
5. **Rust core 영향 0** — 영향 범위 한정 명확
6. **신속한 메인테이너 응답 + 정정** — 1차 검토 후 17분 만에 권장안 정확 채택 + 자체 재검증 보고
7. **보너스 효과**: ESM 패키지 선언으로 `download-interceptor-common.test.js` (26 tests) 도 함께 정상화

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + TypeScript + 자동 단위 테스트 38건 |
| 시각 판정 게이트 (push 전 필수) | (해당 없음 — Rust core 영향 0, 한컴 호환 게이트 무관) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr437` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| PR 댓글 톤 | ✅ 차분, 사실 중심 |
| 작은 단위 PATCH 회전 | ✅ 본 PR 은 단순 영역, 빠른 회전 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr437` → `local/devel` → `devel` 머지 + push
3. PR #437 close + 작성자 댓글 (이슈 #432 close)

## 참고

- 검토 문서: `mydocs/pr/pr_437_review.md` (1차 검토)
- PR: [#437](https://github.com/edwardkim/rhwp/pull/437)
- 이슈: [#432](https://github.com/edwardkim/rhwp/issues/432)
- 같은 작성자 머지 PR: [#168](https://github.com/edwardkim/rhwp/pull/168), [#169](https://github.com/edwardkim/rhwp/pull/169), [#339](https://github.com/edwardkim/rhwp/pull/339)
- 본 작성자 origin 이슈: [#364](https://github.com/edwardkim/rhwp/issues/364) (PR #419 로 흡수, 다른 컨트리뷰터에게 양도)
