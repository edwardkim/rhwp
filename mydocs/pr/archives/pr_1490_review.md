# PR #1490 리뷰 기록 - 외부 웹폰트 사용 안 함 옵션

- PR: https://github.com/edwardkim/rhwp/pull/1490
- 작성일: 2026-06-23
- 경로: collaborator self-merge 후보
- base/head: `devel` <- `task_m100_1487`
- 문서 작성 시점 참고 head: `01eeacf7` (본 review 문서/오늘할일 커밋 전)

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | 문서 작성 시점 참고값: open, draft 아님 |
| base | `devel` |
| 관련 이슈 | `Closes #1487` |
| 규모 | 문서 작성 시점 참고값: 27 files, +681/-38 |
| merge 상태 | 문서 작성 시점 참고값: `MERGEABLE`, `BLOCKED`; 최종 merge 전 최신 상태 재확인 필요 |
| CI 상태 | 문서 작성 시점 참고값: JS/Python CodeQL 및 Canvas visual diff 통과, Build & Test와 Rust CodeQL 진행 중 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 확정 사실로 기록하지 않는다.
최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후에만 수행한다.

## 2. 변경 범위

### 2.1 확장 옵션 추가

- Chrome/Firefox/Safari options 페이지에 `외부 웹폰트 사용 안 함` 체크박스를 추가했다.
- 기본값은 off로 유지했다.
- 안내 문구는 내부망/오프라인 권장과 렌더링 차이 가능성을 명시한다.
- 확장별 설정 저장소 차이를 반영했다.
  - Chrome/Firefox: `storage.sync`
  - Safari: `storage.local`

### 2.2 viewer 폰트 로딩 경로

- `rhwp-studio/src/core/extension-settings.ts`를 추가해 확장 설정을 viewer 초기화 전에 읽는다.
- `loadWebFonts()`에 `disableExternalWebFonts` 옵션을 전달한다.
- 옵션이 켜진 경우 외부 `http(s)` 웹폰트는 CSS `@font-face` 등록과 `FontFace.load()` 대상에서 제외한다.
- 옵션이 꺼진 기본 모드에서는 기존 외부 CDN 폰트 사용 경로를 유지한다.

### 2.3 로컬 글꼴 감지 모달 문구 정리

- `웹 대체` 표현을 `대체 글꼴`로 바꿔 외부 웹폰트 옵션과의 혼선을 줄였다.
- `외부 웹폰트 사용 안 함` 옵션이 켜진 경우 모달에 해당 상태를 표시한다.
- 모달은 외부 CDN 로딩 여부가 아니라 로컬 원본 글꼴 감지 권한을 묻는 기능이므로 유지한다.

### 2.4 빈 viewer 클릭 오류 방지

- 문서가 로드되지 않은 빈 viewer에서 클릭할 때 `hitTest` 경로로 들어가지 않도록 방어했다.
- Chrome 확장 오류 목록에 불필요한 `[InputHandler] hitTest 실패`가 남지 않도록 했다.

### 2.5 CodeQL 권고 반영

- GitHub Advanced Security가 테스트 코드의 URL 부분 문자열 검사에 대해
  `Incomplete URL substring sanitization` 경고를 제기했다.
- 테스트 assertion의 `includes('https://cdn.jsdelivr.net/')` 기반 검사를 제거했다.
- CSS/FontFace source에서 `url(...)` 값을 추출해 `new URL()`로 파싱하고,
  `protocol`과 `hostname`을 비교하도록 변경했다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 온라인 렌더링 정합성 저하 | 기본값 off로 유지해 일반 온라인 환경에서는 기존 외부 웹폰트 경로를 유지한다. |
| 오프라인 모드에서 PDF 기준 시각 차이 | 안내 문구에 글꼴, 줄바꿈, 페이지 배치 차이 가능성을 명시했다. |
| 로컬 글꼴 감지 모달 혼선 | `대체 글꼴` 표현과 외부 웹폰트 비활성 상태 안내로 구분했다. |
| 확장별 설정 저장소 차이 | Chrome/Firefox/Safari 별도 저장소를 공통 helper에서 흡수했다. |
| CodeQL URL 검사 경고 재발 | URL 부분 문자열 검사 대신 `URL.hostname` 비교로 assertion을 변경했다. |

## 4. 로컬 검증

단계별 작업 중 확인된 로컬 검증은 다음과 같다.

- `cd rhwp-studio && npm test`: 통과 (최종 130 passed)
- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-chrome && npm ci`: 통과
- `cd rhwp-chrome && node build.mjs`: 통과
- 확장 JS `node --check`: 통과
- Chrome/Firefox locale JSON parse: 통과
- `git diff --check`: 통과

Stage 4 CodeQL 대응 후 추가 검증:

- `cd rhwp-studio && npm test`: 통과 (130 passed)
- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과

## 5. 판단

PR #1490은 #1487의 내부망/오프라인 피드백에 대응해 확장 옵션 기반으로 외부 웹폰트 로딩을 차단할 수 있게 한다.
기본값을 off로 둬 온라인 환경의 기존 렌더링 정합성을 유지하고, 옵션 on 상태에서는 내부망 생존성을 우선한다.

GitHub Advanced Security의 URL 부분 문자열 검사 권고는 최신 코드에서 URL 파싱 기반 비교로 반영했다.
추가 dismiss나 보안 경고 무시는 필요하지 않다.

최종 merge 조건:

1. 본 review 문서 2건과 오늘할일 문서가 PR diff에 포함된다.
2. PR head 최신 커밋 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.
4. merge 전 `mergeable` / `mergeStateStatus` / 관련 이슈 상태를 최신으로 재확인한다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용 가능하다.
