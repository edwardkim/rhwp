# v0.8.2 핫픽스 릴리즈 최종 보고서

Issue: #3445
브랜치: `task/3445-release-v0.8.2`
태그: `v0.8.2` = `9b16aa9e2`
릴리즈 범위: `v0.8.1..v0.8.2` 16커밋

## 1. 목적과 결과

v0.8.1 배포 직후 발견된 **브라우저 확장 인쇄 전면 실패**를 복구하는 핫픽스다. v0.8.0 부터
Chrome·Edge·Firefox 에서 Ctrl+P 가 "파일을 찾을 수 없음" 으로 실패해 왔다.

| 대상 | 방식 | 결과 |
|---|---|---|
| GitHub Pages | 자동(main push) | success, 사이트 HTTP 200 |
| npm `@rhwp/core` · `@rhwp/editor` | 자동(Release) | **0.8.2** |
| VS Code Marketplace | 자동(Release) | **0.8.2** |
| Open VSX | 자동(Release) | **0.8.2** |
| Chrome / Edge / Firefox | 수동 업로드 | zip 준비 완료(메인테이너 작업) |

`Publish All Packages` 4개 job 전부 success. VS Code Marketplace 는 v0.8.1 때와 마찬가지로
인덱싱 지연이 있어 폴링 4회차(약 3분)에 0.8.2 로 확인됐다. Open VSX 와 npm 2종은 1회차에
반영됐다.

**확장 실기 인쇄: 작업지시자 Chrome 확장 동작 테스트 통과.** 이번 핫픽스의 본체가 실기로
검증됐다.

## 2. 릴리즈 범위

| 항목 | 내용 |
|---|---|
| 확장 인쇄 복구 | #3433 — `print.html` 이 확장 빌드에 복사되지 않았다. `build.mjs` 복사 추가 + 필수 산출물 게이트 (PR #3446) |
| 렌더 정정 | #3396 — TAC 인라인 표 x-원점에 outMargin 좌/우 배선 (PR #3410) |

버그 수정만 담기며 신규 기능이 없어 PATCH 로 했다.

### 근인과 구조적 처방

`print-surface.ts` 가 `new URL('print.html', baseUrl)` 로 확장 루트 기준 파일을 열지만
`build.mjs` 가 그 파일을 복사하지 않았다. 확장은 `vite publicDir:false` 라 `public/` 자산을
개별 복사해야 하는데 `theme-init.js`·`favicon.ico`·아이콘 SVG 만 대상이었다.

더 근본적인 문제는 **`copy()` 가 원본 부재 시 경고만 내고 넘어가 누락이 빌드 성공으로
위장된 것**이다. 그래서 v0.8.0·v0.8.1 두 번의 릴리즈를 그대로 통과했다. 빌드 말미에 런타임
필수 6종의 존재를 확인하고 없으면 `exit 1` 로 실패시키는 게이트를 추가했고, 원본을 임시
제거해 실제로 잡히는지(`MISSING: print.html` → exit 1) 실증했다.

기존 print 테스트 15개는 URL 계산·명령 계약만 검증해 이 결함을 잡지 못했다. 로직 테스트와
산출물 검증은 다른 층위라는 점이 이번 사건의 교훈이다.

## 3. 계획 대비 차이

v0.8.1 과 달리 **계획대로 진행됐다.** 기준선 이동도, 중간 보완도 없었다.

| 항목 | v0.8.1 | v0.8.2 |
|---|---|---|
| 기준선 이동 | 3회 | **0회** (`732147a30` 한 번에 확정) |
| CHANGELOG 보완 PR | 필요(#3424) | 불필요 |
| 릴리즈 이슈 자동 close | 발생(재개방) | **미발생** |
| 버전 갱신 파일 수 | 계획 10 → 실제 9 | 계획 9 = 실제 9 |

## 4. v0.8.1 교훈의 적용 결과

계획서 5절에 명시한 다섯 가지가 실제로 효과를 냈다.

| 교훈 | 적용 결과 |
|---|---|
| 기준선은 고정 대상이 아니다 | 각 단계 전 fetch. 이번엔 변동이 없어 재실행 불필요 |
| CHANGELOG 작성 후 유입 확인 | main 통합 직전 `v0.8.1..devel` 실질 변경 재점검 — 누락 0 |
| main 통합은 merge commit | 사전 main 독자 변경 0 확인, 사후 트리 동일성·ancestry 검증 |
| publish step success ≠ 배포 완료 | 레지스트리 실측으로 확인 |
| 릴리즈 이슈에 `Closes` 금지 | PR 본문에 `관련: #3445` 만 사용 → **이슈 OPEN 유지** |

특히 **merge commit 정책의 복리 효과**가 확인됐다. v0.8.1 을 merge commit 으로 통합한 결과
이번 merge-base 가 `ace187d52`(v0.8.1 CHANGELOG 보완)로 이어졌고, main 독자 변경이 0파일이라
충돌 없이 통합됐다. v0.8.0 에서 20파일 충돌을 겪은 것과 대조된다.

## 5. 검증

| 항목 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` | **4160 passed / 0 failed** |
| `cargo clippy --all-targets -- -D warnings` | 경고 0 |
| `cargo fmt --check` / `git diff --check` | 통과 |
| Docker WASM 빌드 | 성공 (4m31s) |
| studio `npm test` | 641 pass / 0 fail |
| `web-ext lint`(Firefox) | **errors 0**, warnings 7 |
| PR #3451 CI(devel) | SUCCESS 21 / SKIPPED 1 |
| PR #3453 CI(main) | SUCCESS 40 / SKIPPED 2 |

### main 통합 무결성

- 사전: merge-base `ace187d52` 이후 **main 독자 변경 0파일**
- merge: **merge commit**(`--admin`), 부모 2개(`1dbf024fd`, `0f1701b52`) 확인
- 사후: main 트리 == devel 트리(`d20c62122` 동일), devel 이 main 의 조상 — **ancestry 보존**

`--admin` 은 main 브랜치 보호의 `required_reviews: 1` 우회용이며 CI 는 우회하지 않았다.

## 6. 확장 배포 산출물

빌드 기준: main = `v0.8.2` 태그 트리, `npm ci` 선행.

| 파일 | 크기 | 검증 |
|---|---|---|
| `rhwp-chrome/rhwp-chrome-0.8.2.zip` | 30.0MB | manifest v0.8.2, **print.html 1436 bytes** |
| `rhwp-chrome/rhwp-edge-0.8.2.zip` | 30.0MB | chrome 과 md5 동일(`3fe799f0…`) |
| `rhwp-firefox/rhwp-firefox-0.8.2.zip` | 30.0MB | manifest v0.8.2, **print.html 포함**, lint errors 0 |
| `rhwp-firefox/rhwp-source-0.8.2-amo.zip` | 26.9MB | 200MB 제한 충족, `public/print.html` 원본 포함 |

**0.8.1 zip 에서 `print.html` 이 0건이던 것이 이번엔 양쪽 모두 포함된다.** source zip 에도
원본이 들어가 AMO 재빌드 가능성이 확보됐다.

위생 검증: dist 에 `.env`·token·pem 없음. source zip 금지 경로 7종 미포함, 필수 5종 포함.

제출 문서 4종은 `mydocs/feedback/` 에 있다 — `chrome-0.8.2_{kor,eng}.md`,
`edge-0.8.2_reviewer_notes.md`, `firefox-0.8.2_amo_notes.md`.

## 7. 미해결

| 이슈 | 내용 |
|---|---|
| #3450 | studio E2E `print-pdf-issue3126` PDF 안내 모달 실패. **인쇄 surface 자체는 정상**. 테스트·소스가 v0.8.1 태그 이후 무변경이라 이번 범위 밖에서 비롯했다. 근인 미진단 |
| #3412 | studio E2E `issue-2214` 페이지 로컬 리페인트 계약 실패. v0.8.1 에서 이어지며 회귀 여부 미확정 |

두 건 모두 CHANGELOG·릴리즈 노트의 "알려진 문제" 에 미확정 사실과 함께 공표했다.

### 스토어 수동 업로드

v0.8.1 심사는 작업지시자가 취소했으므로 0.8.2 는 **신규 제출**이다. 인쇄가 동작하지 않는
0.8.1 이 사용자에게 배포되는 일은 없다. AMO 는 확장 zip 과 source zip 을 함께 올린다.

## 8. 운영 기록

| 단계 | 산출물 |
|---|---|
| 수행계획서 | `mydocs/plans/task_m100_3445.md` |
| 1단계 보고서 | `mydocs/working/task_m100_3445_stage1.md` |
| 2단계 보고서 | `mydocs/working/task_m100_3445_stage2.md` |
| 최종 보고서 | 이 문서 |

관련 PR: [#3446](https://github.com/edwardkim/rhwp/pull/3446)(`732147a30`),
[#3451](https://github.com/edwardkim/rhwp/pull/3451)(`0f1701b52`),
[#3453](https://github.com/edwardkim/rhwp/pull/3453)(`9b16aa9e2`).
