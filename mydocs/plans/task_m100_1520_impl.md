# 구현계획서: 브라우저 확장 GitHub 비파일 HWP URL 후보 차단

- **타스크**: [#1520](https://github.com/edwardkim/rhwp/issues/1520)
- **브랜치**: `local/task1520-upstream`
- **작성일**: 2026-06-25
- **선행**: `mydocs/plans/task_m100_1520.md` (수행계획서, 승인됨)

## 0. 작업지시자 결정 사항

수행계획서 승인 지시에 따라 다음 방향으로 진행한다.

| 질문 | 결정 |
|---|---|
| 정적 classifier 도입 | 도입. 기존 `resolveDocumentUrl()`은 보존하고 후보 판정용 분류 API를 추가 |
| GitHub 비파일 URL 차단 범위 | `edit`, `commits`, `blame`, `tree` 우선 차단 |
| #1521 hover lifecycle | 본 타스크 범위 밖으로 유지 |

## 1. 핵심 설계

### 1.1 두 기준을 분리한다

기존 `resolveDocumentUrl(url)`은 "viewer가 실제로 fetch할 URL"을 반환한다. 매칭 provider가 없으면
원본 URL을 반환해야 하므로, 이 함수만으로 content-script가 배지 생성 여부를 결정할 수 없다.

이번 작업에서는 별도의 후보 분류 함수를 추가한다.

```javascript
classifyDocumentUrl(url)
```

예상 반환값:

```javascript
{
  status: 'openable' | 'not-document' | 'unknown',
  resolvedUrl?: string,
  reason?: string
}
```

상태 의미:

| 상태 | 의미 | content-script 처리 |
|---|---|---|
| `openable` | 정적으로 HWP/HWPX 후보가 맞음 | 배지/hover 허용 |
| `not-document` | 정적으로 문서가 아닌 provider 페이지 | 배지/hover 차단 |
| `unknown` | provider 규칙은 없지만 일반 URL 후보 | 기존 확장자 정책 유지 |

### 1.2 GitHub provider 정책

허용:

- `https://github.com/{owner}/{repo}/blob/{ref}/{path}.hwp[x]`
- `https://raw.githubusercontent.com/{owner}/{repo}/{ref}/{path}.hwp[x]`

차단:

- `https://github.com/{owner}/{repo}/edit/{ref}/{path}.hwp[x]`
- `https://github.com/{owner}/{repo}/commits/{ref}/{path}.hwp[x]`
- `https://github.com/{owner}/{repo}/blame/{ref}/{path}.hwp[x]`
- `https://github.com/{owner}/{repo}/tree/{ref}/...`

주의:

- slash 포함 ref는 기존 #432와 동일하게 완전 지원하지 않는다.
- `resolveDocumentUrl()`의 기존 동작은 깨지지 않아야 한다.
- provider 미매칭 일반 `.hwp/.hwpx` 직접 링크는 계속 허용한다.

### 1.3 content-script 적용 방식

Chrome/Firefox/Safari content-script는 현재 manifest의 일반 스크립트로 로드되며 ESM import를 직접
사용하지 않는다. 따라서 구현은 다음 원칙으로 진행한다.

1. `rhwp-shared/sw/document-url-resolver.js`에 테스트 가능한 classifier를 추가한다.
2. Chrome/Firefox/Safari content-script에는 같은 정책의 경량 후보 판정 helper를 적용한다.
3. 중복을 최소화하되, content-script 모듈화/번들링 전환은 이번 타스크 범위에 포함하지 않는다.

추후 별도 리팩터링에서 content-script 공통 helper를 UMD 또는 번들 기반으로 공유할 수 있다.

### 1.4 `data-hwp="true"` 정책

명시적 통합 링크는 기존 정책을 유지한다.

- `data-hwp="true"`가 있으면 확장자 없는 공공기관 다운로드 URL도 후보로 허용한다.
- 단, GitHub의 명시적 비파일 URL에 `data-hwp="true"`가 붙는 예외는 이번 타스크에서 별도로 다루지 않는다.
  일반 웹페이지가 의도적으로 명시한 링크는 기존 통합 계약을 존중한다.

## 2. 파일 변경 계획

| 파일 | 변경 |
|---|---|
| `rhwp-shared/sw/document-url-resolver.js` | `classifyDocumentUrl()` 추가, GitHub provider 분류 helper 추가 |
| `rhwp-shared/sw/document-url-resolver.test.js` | GitHub `edit`/`commits`/`blame`/`tree` 차단, `blob`/raw/일반 URL 허용 테스트 추가 |
| `rhwp-chrome/content-script.js` | `isHwpLink()`에서 GitHub 비파일 URL 제외, hover/prefetch 후보 판정에 동일 기준 적용 |
| `rhwp-firefox/content-script.js` | Chrome과 동일 |
| `rhwp-safari/src/content-script.js` | Safari 후보 판정에 동일 기준 적용 |
| `rhwp-chrome/dist/`, `rhwp-firefox/dist/`, `rhwp-safari/dist/` | 빌드 산출물은 빌드 결과에 따라 갱신 여부 결정 |
| `mydocs/working/task_m100_1520_stage*.md` | 단계별 완료 보고서 |
| `mydocs/report/task_m100_1520_report.md` | 최종 보고서 |
| `mydocs/orders/20260625.md` | 진행 상태 갱신 |

## 3. 단계 분할

### Stage 1 — classifier 순수 함수와 테스트

**변경 파일**:

- `rhwp-shared/sw/document-url-resolver.js`
- `rhwp-shared/sw/document-url-resolver.test.js`
- `mydocs/working/task_m100_1520_stage1.md`

**구현**:

- `classifyDocumentUrl(url)` 추가
- GitHub `blob`은 `openable` + raw resolved URL 반환
- `raw.githubusercontent.com` 직접 HWP/HWPX는 `openable`
- GitHub `edit`, `commits`, `blame`, `tree`는 `not-document`
- 일반 직접 파일 URL은 `unknown` 또는 `openable` 중 구현계획 내에서 최종 선택
  - content-script 호환성 관점에서는 "차단하지 않음"이 핵심이다.

**테스트 케이스**:

- `github.com/.../blob/.../sample.hwp` → `openable`
- `github.com/.../blob/.../sample.hwpx` → `openable`
- `raw.githubusercontent.com/.../sample.hwp` → `openable`
- `github.com/.../edit/.../sample.hwp` → `not-document`
- `github.com/.../commits/.../sample.hwp` → `not-document`
- `github.com/.../blame/.../sample.hwp` → `not-document`
- `github.com/.../tree/...` → `not-document`
- `github.com/.../blob/.../README.md?file=sample.hwp` → 기존처럼 변환하지 않음
- `https://example.com/files/sample.hwp` → 차단하지 않음

**검증**:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

**완료 기준**:

- resolver 기존 12개 테스트 보존
- 신규 classifier 테스트 통과

### Stage 2 — Chrome/Firefox content-script 후보 판정 적용

**변경 파일**:

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`
- `mydocs/working/task_m100_1520_stage2.md`

**구현**:

- `isHwpLink(anchor)`에서 GitHub 비파일 URL을 제외한다.
- `processLinks()` 배지 생성 경로가 같은 판정을 사용한다.
- `prefetchThumbnails()`가 같은 판정을 사용해 GitHub HTML 페이지 prefetch를 만들지 않는다.
- `data-hwp="true"` 명시 링크 정책은 기존대로 보존한다.

**검증**:

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --test rhwp-shared/sw/document-url-resolver.test.js
```

**완료 기준**:

- Chrome/Firefox content-script 문법 체크 통과
- GitHub 비파일 URL은 후보에서 제외되는 로직 확인

### Stage 3 — Safari content-script 반영 및 확장 빌드

**변경 파일**:

- `rhwp-safari/src/content-script.js`
- `mydocs/working/task_m100_1520_stage3.md`

**구현**:

- Safari의 `isHwpLink(anchor)`에도 동일한 GitHub 비파일 제외 정책을 적용한다.
- Safari의 `autoOpen`, `showBadges`, `hoverPreview` 흐름이 Chrome/Firefox와 같은 의미가 되도록 맞춘다.

**검증**:

```bash
node --check rhwp-safari/src/content-script.js
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

Safari 전체 `build.sh`는 Xcode 환경 의존성이 크므로, 이번 단계에서는 문법 체크와 소스 반영을 우선한다.
필요 시 작업지시자 환경에서 별도 Safari 빌드/수동 검증을 진행한다.

**완료 기준**:

- Chrome/Firefox 확장 빌드 통과
- Safari content-script 문법 체크 통과

### Stage 4 — 수동 재현 확인과 최종 보고

**변경 파일**:

- `mydocs/working/task_m100_1520_stage4.md`
- `mydocs/report/task_m100_1520_report.md`
- `mydocs/orders/20260625.md`

**검증 후보**:

자동:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
node rhwp-chrome/sw/fetch-security.test.mjs
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-safari/src/content-script.js
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

수동:

- GitHub `blob/main/samples/2010-01-06.hwp` 배지/hover 표시
- GitHub `edit/main/samples/2010-01-06.hwp` 배지/hover 미표시
- GitHub `commits/main/samples/2010-01-06.hwp` 배지/hover 미표시
- raw URL 또는 일반 직접 `.hwp` 링크 동작 유지

**완료 기준**:

- 자동 검증 통과
- 수동 검증 가능 범위 기록
- 최종 보고서 작성
- 오늘할일 상태 갱신

## 4. 회귀 방지 체크리스트

| 항목 | 기대 |
|---|---|
| GitHub `blob` HWP/HWPX | 배지/hover 유지, raw 변환 유지 |
| GitHub `edit` HWP/HWPX | 배지/hover 없음 |
| GitHub `commits` HWP/HWPX | 배지/hover 없음 |
| GitHub `blame` HWP/HWPX | 배지/hover 없음 |
| `raw.githubusercontent.com` HWP/HWPX | 배지/hover 유지 |
| 일반 `https://example.com/file.hwp` | 배지/hover 유지 |
| query에만 `.hwp` 존재 | 자동 감지 없음 |
| `data-hwp="true"` 명시 링크 | 기존 통합 동작 유지 |
| 내부망/private URL fetch | #1307 정책 유지 |

## 5. 비목표

- #1521 hover card 지연 표시 타이머 취소 누락 수정
- content-script 전체 모듈화 또는 번들링 전환
- GitHub slash 포함 ref 완전 파싱
- GitLab/Bitbucket provider 구현
- 확장자 없는 다운로드 URL의 일반 자동 감지

## 6. 승인 요청

본 구현계획서 승인 후 Stage 1 구현을 시작한다.
