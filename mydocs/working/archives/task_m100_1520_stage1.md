# Stage 1 완료보고서: URL classifier 순수 함수와 테스트

- **타스크**: [#1520](https://github.com/edwardkim/rhwp/issues/1520)
- **브랜치**: `local/task1520-upstream`
- **작성일**: 2026-06-25
- **단계**: Stage 1 — classifier 순수 함수와 테스트

## 1. 범위

이번 단계에서는 content-script 동작을 아직 변경하지 않고, 공통 URL resolver에 정적 후보 분류 API와
회귀 테스트를 추가했다.

변경 파일:

| 파일 | 변경 |
|---|---|
| `rhwp-shared/sw/document-url-resolver.js` | `classifyDocumentUrl()`, `classifyGithubDocumentUrl()` 추가 |
| `rhwp-shared/sw/document-url-resolver.test.js` | GitHub 비파일 URL 차단 및 정상 URL 허용 테스트 추가 |

## 2. 구현 내용

### 2.1 `classifyDocumentUrl(url)`

content-script 후보 판정에 사용할 수 있도록 URL을 다음 상태로 분류한다.

| 상태 | 의미 |
|---|---|
| `openable` | 정적으로 HWP/HWPX 문서 후보로 판단 가능 |
| `not-document` | 정적으로 문서가 아닌 provider 페이지로 판단 가능 |
| `unknown` | 문서 후보로 확정할 수 없음 |

기존 `resolveDocumentUrl(url)` 동작은 변경하지 않았다. viewer/thumbnail fetch 대상 정규화는 기존
흐름을 유지하고, 후보 표시 여부 판단만 별도 classifier로 분리했다.

### 2.2 GitHub provider 분류

정상 후보:

- `github.com/{owner}/{repo}/blob/{ref}/{path}.hwp[x]`
- `raw.githubusercontent.com/{owner}/{repo}/{ref}/{path}.hwp[x]`
- 일반 직접 파일 URL `https://example.com/files/sample.hwp`

비파일 페이지:

- `github.com/{owner}/{repo}/edit/{ref}/{path}.hwp[x]`
- `github.com/{owner}/{repo}/commits/{ref}/{path}.hwp[x]`
- `github.com/{owner}/{repo}/blame/{ref}/{path}.hwp[x]`
- `github.com/{owner}/{repo}/tree/{ref}/...`
- `github.com/{owner}/{repo}/blob/{ref}/README.md?file=sample.hwp`

## 3. 검증

실행:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

결과:

```text
tests 23
pass 23
fail 0
```

기존 12개 resolver 테스트와 신규 11개 classifier 테스트가 모두 통과했다.

## 4. 다음 단계

Stage 2에서 Chrome/Firefox content-script의 `isHwpLink()`와 thumbnail prefetch 후보 판정에
이번 classifier와 같은 정책을 적용한다.

주의:

- `data-hwp="true"` 명시 링크 정책은 유지한다.
- content-script는 현재 일반 스크립트로 로드되므로 Stage 2에서는 import 구조 변경 없이 경량 helper를 적용한다.
- #1521 hover lifecycle 버그는 본 타스크에서 수정하지 않는다.
