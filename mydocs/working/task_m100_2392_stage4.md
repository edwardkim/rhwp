# Task M100 #2392 Stage 4 완료 보고 - 회귀, metrics 결산과 PR 준비

- 이슈: #2392
- 상위 추적: #2022
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- source 기준: `upstream/devel@af5902b659be9a4d86ad458d79c63353dba88167`
- 측정 source commit: `deb87ad98a5e76b86f486333d22bc518091265f6`
- 작성일: 2026-07-19
- 원격 상태: push, PR 생성, GitHub comment와 issue 편집 미수행

## 1. 완료 요약

Stage 4의 필수 local gate와 정량·계약 감사를 완료했다.

- Studio 전체 unit 390/390과 production build를 통과했다.
- 실제 Vite server와 Chrome headless에서 undo contracts 6개 시나리오를 통과했다.
- clean-source same-base post metrics와 #2124 official 비교 snapshot을 생성했다.
- `handleOk`의 CC/LOC 감소가 전체 Total CC와 threshold 합의 순감소로 이어졌음을 확인했다.
- product/test allowlist, mutation 5회, internal export와 package/WASM/dependency 무변동을 확인했다.
- 최종 보고서와 PR 본문·리뷰 요청·추적 댓글 초안을 작성했다.

## 2. upstream과 열린 PR 재확인

Stage 4 시작 시 `git fetch upstream` 뒤 `upstream/devel`은 기준 commit `af5902b6`에서 변하지 않았다.

| 항목 | 상태 | #2392 판단 |
|------|------|------------|
| PR #2370 | OPEN, BEHIND | `rhwp-studio/src/command/commands/insert.ts`만 변경, 대상 파일 직접 중첩 0 |
| PR #2394 | OPEN, BEHIND | core/renderer 변경, 대상 dialog/model 직접 중첩 0 |

#2394가 먼저 merge되면 global metrics 모집단은 달라질 수 있다. 이 경우 PR 통합 전에 최신 base를 다시
확인하되, #2392의 인과 판정은 `af5902b6` same-base pre/post를 보존한다.

## 3. full local gate

| Gate | 결과 | 근거 |
|------|------|------|
| `npm --prefix rhwp-studio test` | PASS | 390/390 |
| `npm --prefix rhwp-studio run build` | PASS | TypeScript와 Vite production build |
| `e2e:undo -- --mode=headless` | PASS | Chrome headless, 6개 시나리오 |
| E2E command failure 수집 | PASS | 그림 속성 실키 Ctrl+Z에서 0건 |
| `git diff --check` | PASS | Stage 4 문서 포함 whitespace 오류 0 |

E2E는 `rhwp-studio`를 Vite 실행 디렉터리로 고정하고 `127.0.0.1:7700` 준비를 확인한 뒤 실행했다.
저장소 루트에서 Vite binary를 직접 실행한 첫 재시도는 Studio config의 `@wasm` alias를 읽지 못했으며,
제품 회귀가 아닌 실행 디렉터리 오류로 판정하고 올바른 경로에서 재실행했다. 최종 서버와 Chrome process는
명령 종료 시 정리했다.

통과한 browser 시나리오는 모두 바꾸기 undo, 그림 속성 undo, 그림 속성 실제 Ctrl+Z, 수식 속성 undo,
표/셀 속성 undo, `Through` 배치 보존이다. HTML evidence는 ignored
`output/e2e/undo-contracts-report.html`에 생성됐고 SHA-256은
`b03239afd8702c1840e44f92ef0d3e0da534adb9ce02756b169cd758c8286952`다.

build의 CanvasKit `fs`/`path` browser externalize와 500 kB chunk 경고는 기존 비차단 경고다. 이번 diff는
DOM, CSS, renderer와 bundling 설정을 변경하지 않는다.

Vite log에는 E2E 문서 초기화 중 `(0,0,0)` cursor rect를 아직 찾지 못했다는 `CursorState.updateRect` 경고가
기록됐다. 경고 코드는 현재 `upstream/devel`의 `cursor.ts`에 있고 #2392 diff와 겹치지 않으며, 각 시나리오의
apply/undo assertion과 process exit는 모두 통과했다. 이번 리팩터링의 회귀나 별도 완료 차단으로 분류하지
않는다.

## 4. final metrics

### 4.1 same-base 직접 변화

| 지표 | pre | post | delta |
|------|----:|-----:|------:|
| included files | 215 | 216 | +1 |
| reported functions | 2,386 | 2,407 | +21 |
| Total CC | 12,369 | 12,093 | -276 |
| Top 20 합 | 2,660 | 2,359 | -301 |
| CC>25 개수 / 합 | 70 / 4,297 | 69 / 3,949 | -1 / -348 |
| CC>100 개수 | 7 | 6 | -1 |
| Max CC | 453 | 453 | 0 |

maintainer의 #1904 결산과 #2130 산식 교훈에 따라 Max나 threshold 개수만 보지 않고 Total CC, Top 20 합,
CC>25 합·개수, CC>100 개수와 stable function diff를 함께 판정했다. stable `changed` 함수는 대상
`handleOk` 하나뿐이며 `348 -> 2`다. 대상 밖 stable changed function은 0이다.

### 4.2 대상 경계

| 항목 | pre dialog | post dialog | post model | post 합계 |
|------|-----------:|------------:|-----------:|----------:|
| physical LOC | 2,825 | 2,612 | 555 | 3,167 |
| reported functions | 35 | 36 | 20 | 56 |
| Total CC | 647 | 291 | 80 | 371 |
| CC>25 개수 / 합 | 2 / 560 | 1 / 212 | 0 / 0 | 1 / 212 |
| Max CC | 348 | 212 | 13 | 212 |
| `handleOk` CC / LOC | 348 / 381 | 2 / 11 | - | 2 / 11 |

물리 LOC와 함수 수 증가는 typed form/target model과 fixture 가능한 작은 helper가 추가된 결과다. 신규 helper의
Max CC는 13이고 CC>25 신규 함수는 0이므로 고복잡도를 다른 함수로 옮긴 결과가 아니다.

### 4.3 #2124 official snapshot과의 구분

현재 post를 `mydocs/metrics/frontend/2026-07-11/metrics.json`과 비교하면 Total CC +288, Top 20 합 -222,
CC>25 합 +17이다. 이 변화에는 #2124 이후 merge된 다른 frontend 변경과 legacy `/web` 제거가 함께 포함되므로
#2392 성과로 귀속하지 않는다. official snapshot은 변경하지 않았고 추세 참고값으로만 제시한다.

### 4.4 재현 정보

| artifact | SHA-256 |
|----------|---------|
| same-base pre JSON | `02ab67076683a091b1c77f1c9c9889867af42f100dc7fc6ef6092485a59f5a93` |
| same-base post JSON | `fc49c696a00ac69e289828640e838a5dcbfe95372d492abdf1d57e97e3d95d2a` |
| post-vs-official JSON | `71b1f55d3309a1ae58eb8c585a83ef2c539516ea45b9ee8d43a0cb9a5f82c3a3` |
| official JSON | `2f84ea2aa60421e628e73be79cb1e5b69bf1e35506f966bf5cd7de7fd7aee765` |
| metrics script | `5d100c90f47671240f463b0a48fe61d34eb8aedbf8c22bbe333f31241f11d087` |
| metrics lock | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |

post snapshot은 source commit `deb87ad9`에서 `measured source clean: true`로 생성했다. 뒤의 Stage 4 commit은
보고서만 추가하므로 측정 source를 변경하지 않는다. JSON과 summary는 ignored output에 두고 공식 artifact로
승격하지 않는다.

## 5. allowlist와 contract audit

| 감사 항목 | 결과 |
|-----------|------|
| product diff | dialog와 internal apply model 2개만 변경 |
| test diff | model fixture와 undo source guard 2개만 변경 |
| DOM builder/CSS/`populateFromProps` | 기능 diff 0 |
| core/Rust/WASM/generated binding | diff 0 |
| package manifest/lock/runtime dependency | diff 0 |
| runtime UI framework | 추가 0 |
| model runtime import | 0, core type의 `import type`만 존재 |
| model 소비자 | dialog와 test만 존재, package/barrel re-export 0 |
| `any` | model 0, 기존 dialog의 범위 외 7개 증가 없음 |
| mutation setter | dialog 5회 baseline 유지 |
| setter/인자 순서 | 5/5 source guard PASS |
| target priority | image header/footer가 cell path보다 우선, fixture PASS |
| undo/fallback | snapshot `objectProps`, cursor return, fallback event, empty patch guard PASS |

## 6. SOLID review 단위

프론트 전체 점수나 `54/100` 같은 예비 합산값은 사용하지 않는다. #2392 diff와 apply pipeline만 다음
review 질문으로 평가한다.

| 원칙 | 반영 결과 |
|------|-----------|
| SRP | form capture, pure patch policy, target resolution, WASM mutation, undo orchestration을 분리 |
| DIP | 계산 model이 DOM/WASM/EventBus/CommandServices concrete runtime에 의존하지 않음 |
| OCP | object type field 규칙과 target 종류를 제한된 model/union에서 검토 가능 |
| LSP | 대체 타입 계약이 없어 점수화하지 않음 |
| ISP | target/form type이 적용에 필요한 값만 전달하고 거대 runtime context를 받지 않음 |

공식 SOLID 점수는 maintainer/collaborator가 동일 평가 단위와 근거를 리뷰한 경우에만 기록한다.

## 7. 잔여 위험과 후속 경계

- `populateFromProps`는 CC 212로 남지만 이번 이슈의 명시적 범위 밖이다. 자동 후속 구현으로 연결하지 않고
  #2022에서 위험·characterization coverage와 함께 다시 비교한다.
- Stage 1에서 확인한 omitted/stale control 의미는 current behavior로 보존했다. UX 정책 변경이 필요하면
  리팩터링과 섞지 않고 별도 이슈로 분리한다.
- source guard는 setter surface와 인자 순서를 고정하는 보조 수단이다. 실제 apply/undo는 browser E2E가
  보완하지만 모든 object type의 실제 문서 조합을 exhaustive하게 검증하지는 않는다.
- PR 생성 전 또는 review 중 `devel`이 대상 파일을 변경하면 통합 방식과 재검증 범위를 다시 제시한다.

## 8. Stage 4 관문

- [x] Studio 전체 unit 390/390
- [x] Studio production build
- [x] Chrome headless undo contracts 6개 시나리오
- [x] same-base clean-source post metrics
- [x] #2124 official 누적치와 #2392 직접 delta 분리
- [x] 대상 밖 stable changed function 0
- [x] allowlist·public/WASM/dependency audit
- [x] 최종 보고서와 GitHub 게시 초안
- [x] 최종 문서 `git diff --check`
- [ ] 작업지시자의 push·draft PR·GitHub comment 승인

승인 전에는 push, PR 생성, GitHub comment와 issue body 편집을 수행하지 않는다.
