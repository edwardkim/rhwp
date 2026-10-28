# PR #7123 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7123
- 작성자: dependabot[bot]; base: devel; 별도 관련 issue 없음.
- 검토 head: `a00cebd2ce6a6e649879a56178666c4995242720`.
- 현재 devel 기준: `06e82fc253a0b330d446aad29fee95de6e2342fb`.
- 규모: 1 commit, 2파일, +87/-87. 조회 시점 `MERGEABLE / BLOCKED`.
- base route: maintainer_general.
- modifiers: intake_and_review, local_validation, rework_and_exceptions, multi_pr_update_branch.
- loaded documents: pr_review_workflow.md, pr_review/README.md, 위 기본·보조 문서 및 codex/docs_and_git_workflow.md.
- reviewer edwardkim 추가 확인, 기존 jangster77 유지. 검토 branch `review/pr7123`.

## 검토 대상과 변경

#7124 병합 뒤 Dependabot이 head를 `68844f81…`에서 `a00cebd2…`로 갱신했다.
이전 head의 CI는 최종 증거로 사용하지 않고 새 head에서 로컬 검증을 실행했다.
Studio의 @types/chrome 0.2.9를 유지하면서 Vite 8.2.2→8.3.0을 적용한다.
원 contributor branch에 메인테이너 변경을 push하거나 update/rebase하지 않았다.

- 변경 경로: `rhwp-studio/package.json`, `rhwp-studio/package-lock.json`만.
- lockfile 변경 패키지: Vite, Rolldown 1.2.5→1.2.8 및 15개 platform binding,
  @oxc-project/types 0.146.0→0.149.0, PostCSS 8.5.26→8.5.28,
  picomatch 4.0.5→4.0.7, nanoid 3.3.18→3.3.19. 모두 dev 의존성이다.
- [Vite 공식 v8.3.0 릴리즈](https://github.com/vitejs/vite/releases/tag/v8.3.0)는 preload 처리,
  경로 판정·CRLF 코드 프레임, proxy matcher 변경을 포함한다. PR 본문의 create-vite 및 구버전
  commit 목록은 정확한 Vite 차이의 근거로 사용하지 않았다.
- Vite/Rolldown Node 요구는 `^20.19.0 || >=22.12.0`. 검증 Node v24.15.0은 충족한다.
  vite-plugin-pwa 1.3.0의 Vite peer 범위 `^8.0.0`도 충족하며 `npm ls`에서 유효하게 dedupe됐다.
- @vitejs/devtools peer 요구는 ^0.7.1로 바뀌나 Studio에는 이 선택 패키지가 없다.
- Vite config의 WASM alias, PWA, 동적 hwpctrl plugin 경로를 확인했다. Rust/조판 코드 변경은 없다.
  다만 빌드 도구는 최종 JS/CSS 산출물에 영향을 줄 수 있어 production 빌드와 브라우저를 직접 확인했다.

## 실제 검증

새 head에서 Node v24.15.0, npm 11.12.1을 사용했다.

| 검사 | 결과 |
| --- | --- |
| 최신 base와 `git merge-tree --write-tree` | 충돌 없음; tree `5e74b807ed0df2cd312e46c9735f3fff9b4de6d8` |
| `git diff --check upstream/devel...HEAD` | 통과 |
| Studio `npm ci --ignore-scripts --no-audit --no-fund` | 성공; source/lockfile 변경 없음 |
| `./node_modules/.bin/tsc --noEmit` | 통과 |
| `npm test` | 1,694건 중 1,692 PASS / 2 skipped / 실패 0 |
| `npm run build` | Vite 8.3.0 production 빌드 성공; PWA sw.js 및 workbox 생성 |
| `npm run e2e:document-title` | 실제 headless Chrome 문서 열기·실패 시 보존·HWPX 저장·새 문서 성공 |
| production preview + headless Chrome | HTTP 200, 새 문서 제목, canvas 3개, pageerror 0 |
| `npm ls vite vite-plugin-pwa rolldown --depth=1` | 의존성 충돌 없음 |

production 확인은 Vite `preview({preview:{host:'127.0.0.1',port:0}})`와 기존
`e2e/helpers.mjs`의 headless Chrome을 사용했다. 테마 선택을 초기 설정하고 실제 dist를 열어
새 문서 제목을 기다린 뒤 canvas와 pageerror를 확인했다. 개발 서버는 7701을 사용했고 두 검증 서버와
브라우저 모두 종료했다. 기존 사용자의 7700 서버는 변경하지 않았다.

WASM은 기본 작업트리의 기존 pkg를 복사해 사용했다. SHA-256은
`61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`이며 이번 PR의 새 Rust 빌드가 아니다.
Rust 변경이 없어 Cargo/WASM 재빌드는 생략했다. 브라우저 검증은 기동·문서 작업 smoke이며,
전체 편집 기능·오프라인 PWA 업그레이드·subsecond proxy·macOS/Windows native binding 실행까지 검증한 것은 아니다.
빌드에는 CanvasKit의 fs/path browser externalization 및 500kB chunk 경고가 있었다. 빌드는 성공했고
실제 production 기동 오류는 없었다. 경고를 제거하려고 설정/소스를 바꾸지 않았다.
설치의 deprecation 경고와 보안 audit 미실행은 이번 버전 갱신의 결함 판정과 구분한다.

## 공통 조판 원칙과 입력 보존

- 조판 규칙·측정/배치·샘플 특례·baseline 수정: 비해당. 빌드 의존성만 변경했다.
- 사용자 화면 영향은 번들 생성 경로의 위험으로 위 빌드·브라우저 smoke에서 검증했다.
  한컴 기준 시각 일치나 모든 문서의 무회귀를 주장하지 않는다.
- 검증 입력 커밋 확인: 기존 `samples/para-001.hwp`를 E2E가 사용한다. 검토 head가 선행 #7124를
  포함하며 해당 파일 변경이 없고 실제 SHA-256은 `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`다.
  추가 PDF/fixture는 없다. HWPX는 테스트 내부 메모리 산출물이다.
- 제품 상수·정책·호출 경로 직접 수정: 비해당. 로컬 제품 코드 보정은 하지 않았다.

## 최종 판정

**승인** — 새 head의 원격 CI 완료를 확인하여 이전의 대기 보류를 해제했다.

- [CI 34815295468](https://github.com/edwardkim/rhwp/actions/runs/34815295468): success, required Build & Test pass.
- [Render Diff 34815295227](https://github.com/edwardkim/rhwp/actions/runs/34815295227): success.
- CodeQL 34815295585, Adapter 34815295478, Proptest 34815295532: success.
- 검토 head `a00cebd2…`가 동일하고 `MERGEABLE / CLEAN`임을 재확인했다.
- 작업지시자가 #7123 단독 병합과 후속 처리를 승인했다. 실제 merge 결과는 후속 기록에서 구분한다.

## 병합 후 기록

- merge SHA: `e55a40e4cb7486581a5e7bd297a5ab2885b151ff` (2026-09-14 16:08:55 KST).
- maintainer 직접 반영 경로로 archive review·오늘할일만 devel에 보존한다.
- 관련 issue 없음. Dependabot 원격 branch는 이번 작업 소유가 아니므로 임의 삭제하지 않는다.
- 남은 Dependabot 6건은 별도 일괄 검토·승인 대상으로 처리한다.
