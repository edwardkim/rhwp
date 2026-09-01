# Task M100 #6584 Stage R4 — 로컬 release candidate 검증 결과

- 작업 브랜치: `task_m100_6584`
- 검증 기준 HEAD: `43dcabe3d4cfcf81476a627e189351f34cc4da4e`
- release base: `upstream/devel` `063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- 대상 버전: `0.8.6`
- 실행일: 2026-09-02 KST
- 현재 판정: Stage R4 종료 게이트 PASS

## 1. 요약

Rust release build, full nextest, Native Skia, Docker WASM, npm package, Studio, Chrome·Firefox·
VS Code 확장 빌드와 호스트 Chrome CDP E2E를 실행했다. 제품 회귀 실패는 0건이다.

배포 archive 검사에서 보호 폰트·실제 token·개인키·불필요한 빌드 경로 유입은 0건이었다.
다만 `THIRD_PARTY_LICENSES.md`의 Studio·VS Code 의존성 버전이 lock과 다른 결함을 발견했다.
작업 트리에서 `@noble/hashes` 2.4.0과 `canvaskit-wasm` 0.42.0으로 정정했고 font asset·
라이선스 계약 6건을 재검증했다.

정정·보고, credential-shaped fixture 제거와 브라우저 확장 README 현행화를 commit
`adb40d62a6a2f91e2ac37c7f66286a3c6748321c`로 고정한 뒤 AMO source
zip을 정본 `git archive ... HEAD`로 재생성했다. archive comment의 commit SHA, 정정된
라이선스 버전, integrity, allowlist·denylist, symlink 내부 타겟과 비밀정보 재검사가 통과했다.

## 2. 사전 계약

| 검증 | 결과 |
|---|---|
| release workflow Python 계약 | 43 PASS |
| 릴리스 문서 13개 Markdown 링크 | PASS |
| `cargo metadata --locked --no-deps` | PASS |
| 파생 integration suite | source 1,110개, static attr 4,777개, target 48/48, 최소 6,559 case |
| 파생 manifest `--check` | PASS |

## 3. Rust release candidate

### 3.1 필수 lint 묶음

| 단계 | 결과 | 소요 |
|---|---:|---:|
| `cargo fmt --all` + `--check` | PASS | - |
| native Clippy `-D warnings` | PASS | 57.72s |
| WASM lib Clippy `-D warnings` | PASS | 52.09s |
| workspace build | PASS | 1m 39s |
| workspace all-target Clippy `-D warnings` | PASS | 1m 26s |

### 3.2 release build·test

| 단계 | 결과 |
|---|---|
| `cargo build --locked --release --target-dir target/pr-review` | PASS, 12m 28s |
| release lib | 4,071 PASS, 13 ignored, 0 fail |
| full nextest | 8,925 PASS, 46 skipped, 4 slow, 0 fail, 346.383s |
| Native Skia lib | root 3,946 PASS/13 ignored + contracts 15 + OOXML 165 + crypto 2, 0 fail |
| Native Skia placeholder | 2/2 PASS |
| Native Skia direct PDF | 4/4 PASS |
| doctest | 8 PASS, 3 ignored |

현재 nextest 0.9.137은 프로젝트 권장 0.9.140보다 낮아 `report-skipped` 키 경고를 냈지만,
테스트 실행·결과에는 영향이 없었다. 릴리스 호스트 도구 버전 경고로 기록한다.

## 4. WASM·npm·Studio

| 단계 | 결과 |
|---|---|
| Docker 정본 WASM build | PASS, 6m 33s |
| `pkg` 버전·필수 파일 | 0.8.6, PASS |
| `rhwp_bg.wasm` | 9,936,365 bytes, SHA-256 `d9f5000578b82349289ee85c3f680c7050dfc9b6af1605707560c21ee988f9b4` |
| editor tests | 32/32 PASS |
| frontend binding/embed | 3/3 PASS |
| editor declaration compile | PASS |
| `@rhwp/editor` dry pack | 0.8.6, 6 files, 23,580 bytes |
| `@rhwp/core` dry pack | 0.8.6, 6 files, packed 3,787,564 bytes |
| Studio TypeScript | PASS |
| Studio tests | 1,362 PASS, 1 skip, 0 fail |
| Studio production build | PASS |

Studio build의 Vite file-system/path, chunk size 경고는 기존 빌드 경고이며 실패로 분류하지
않았다. Studio `dist` 안의 WASM은 Docker 산출물과 동일한 크기와 최신 생성 시각을
가진다.

## 5. 확장·CDP E2E

### 5.1 빌드·package gate

| 단계 | 결과 |
|---|---|
| Chrome build | PASS |
| Chrome packaged smoke | page budget 4/4, viewer/options/print/worker/content PASS |
| Chrome download E2E | XLSX 2 case viewer 0, HWP 2 case viewer 1 |
| Firefox build | PASS |
| shared/Chrome/Firefox service-worker test | 131/131 PASS |
| Chrome options test | 4/4 PASS |
| VS Code compile | PASS |
| VSIX | 0.8.6, 37 files, 18,558,124 bytes |
| VS Code production dependency audit | vulnerability 0 |

VS Code의 `npm run package`는 로컬 `vsce` executable이 없어 실패했다. CI·배포 정본인
`npx vsce`로 전환하는 과정에서 기존 `node_modules` 설치가 lock보다 이전 버전임도 확인했다.
`npm ci`로 lock 정합을 회복한 후 `npx --yes vsce package`가 통과했다. `npm ci`의 개발
의존성 트리 high 1건과 별개로, 배포 production dependency audit은 0건이다. 자동 `npm audit fix`는
수행하지 않았다.

### 5.2 호스트 Chrome CDP

- CDP: Chrome 151.0.7922.174, protocol 1.3
- Studio 7700: HTTP 200
- 서빙 WASM SHA-256: Docker package와 일치
- 추적 sample manifest: 126/126
- 현행·수동 기능 15개: PASS
- responsive 정본 headless E2E: 1,082 PASS, 0 fail
- edit pipeline: 49 PASS, 0 fail; image insert 1건은 기존 내부 skip
- TAC 현행 검증: 18/18 PASS

첫 실행에서 `화면 스킨 선택` modal이 기능 테스트를 차단했다. 테스트 Chrome에서
기본 스킨을 1회 선택하고 `skinChosen=true` 상태를 고정한 뒤 재실행했다.

호스 responsive 스크립트는 Windows Chrome의 최소 window width 500 제약 때문에 375 등의
요청을 재현하지 못해 104건이 실패했다. 제품 회귀와 분리하기 위해 현행 CI 정본
`npm run e2e:responsive`를 실행했고 1,082/1,082건이 통과했다.

과거 수동 `tac-inline-table` 스크립트는 삭제된 Studio API `getParaText`를 호출해
실행 불가했다. 이 사실은 `mydocs/working/task_m100_1470_stage2.md`에 이미 기록된 기존
테스트 노후화이다. 현행 대체 검증 `tac-verify`가 18/18 통과했으므로 정책 skip으로
분류하되, 실패한 스크립트를 통과했다고 기록하지 않는다.

## 6. archive·보안·라이선스

### 6.1 사전 산출물

| 파일 | 크기 | SHA-256 | 상태 |
|---|---:|---|---|
| `rhwp-chrome-0.8.6.zip` | 33,661,620 | `8d87cebd464674b841107b13ad7ac7f75bdd06d30a1d819af19bcc40aa4fe42f` | PASS |
| `rhwp-edge-0.8.6.zip` | 33,661,620 | `8d87cebd464674b841107b13ad7ac7f75bdd06d30a1d819af19bcc40aa4fe42f` | PASS |
| `rhwp-firefox-0.8.6.zip` | 33,657,414 | `172277ee70fb1ea172947e0ff2b4990f8cd35dd6eb2b6ff316cbe142b92ca087` | PASS |
| `rhwp-vscode-0.8.6.vsix` | 18,558,124 | `6a2d22f5b5814ca1c4d45a886178b7bed8882c4749923b607ba01a6002755fa7` | PASS |
| `rhwp-source-0.8.6-amo.zip` | 33,862,701 | `289fac0204d0e2efd59a652a5db9666f58df8ddc70b0ec9cdf449a812fb2e9f3` | PASS |

Chrome·Edge·Firefox·VSIX와 재생성한 source zip은 모두 archive integrity 검사를 통과했다.
source zip은 2,436 entry, 33.9 MB로 AMO 200 MB 제한 이하이다. 5개 symlink는 Firefox shared
service-worker 4개와 Studio canonical font link 1개이며 모두 archive 내부 타겟을 가진다.

### 6.2 allowlist·denylist

- source zip의 `node_modules/`, `target/`, `dist/`, `output/`, `pdf-large/`, `samples/`: 0건
- token·AWS key·private-key header·실값이 든 배포 token assignment 형식: 0건
- `font_decision_trace_contract.test.mjs`의 민감값 거부 테스트는 가짜 GitHub token을
  런타임에서 조합하도록 바꿔 source scanner에 credential 형식 문자열을 남기지 않았다.
  개인 식별 호스트 경로도 `/home/tester/...`로 익명화했다. 민감값 거부 계약과
  W1 digest drift exact-set test를 포함한 focused suite 12/12가 통과했다.
- `.pem`, `.key`, `.p12`, `.pfx`: 0건
- 환경 관련 파일: 템플릿 `.env.docker.example`, `legacy-peer-deps=true`만 든
  `rhwp-studio/.npmrc`
- bundled WOFF2: canonical open font 36개
- TTF: `ttfs/opensource/NotoSansKR-Regular.ttf` 1개, 비공개·개인 폰트 0개
- 라이선스 파일: root `LICENSE`, `THIRD_PARTY_LICENSES.md`, Source Han Serif OFL 포함
- font distribution contract: 6/6 PASS

### 6.3 발견·정정

lock 정본은 Studio·VS Code 모두 `@noble/hashes` 2.4.0, `canvaskit-wasm` 0.42.0이지만
`THIRD_PARTY_LICENSES.md`에 2.3.0, 0.41.1이 남아 있었다. 두 package 구간 4개 cell을 정정했고
package-lock 대사, font license contract 6건, 문서 링크 검사를 통과했다.

commit `adb40d62a6a2f91e2ac37c7f66286a3c6748321c`에서 `mydocs/manual/publish_guide.md`의
exact `git archive` 명령으로 source zip을 재생성했다. archive comment와 commit SHA가 일치하고,
새 크기·SHA-256·integrity·allowlist·denylist·entry별 secret scan이 통과했다.
AMO source allowlist에 포함되는 Firefox README의 v0.8.6·`.xlsx`·privacy 설명도 archive 내부에서
재확인했다.

## 7. 디스크·작업 트리

- `target/pr-review`: 78 GB
- 파일시스템 여유: 453 GB
- 생성 archive·VSIX: ignore 대상
- 작업 트리 소스 변경: 제3자 라이선스 정정과 본 Stage R4 기록

`target/pr-review`는 이번 release candidate 장시간 검증 cache이며 현재 여유 공간은 충분하다.
R5 exact-head 재검증 전에 지우면 수십 분 수준의 재빌드 비용이 발생하므로 메인테이너의
별도 정리 승인 전까지 유지한다.

## 8. 판정과 다음 게이트

1. Rust·WASM·Studio·확장·CDP 회귀: **PASS**
2. 정책 skip: 2건, 사유·현행 대체 검증 기록 완료
3. 비밀·개인 자산·저작권 폰트 유입: **0건**
4. 제3자 라이선스 기록 결함: 1건 발견, 작업 트리 정정·focused 재검증 완료
5. AMO source zip: 정정 commit 기준 정본 재생성·재검증 **PASS**

따라서 Stage R4 종료 게이트는 **PASS**다. 다음은 본 재검증 결과를 기록으로 고정한 뒤,
최신 `upstream/devel` drift를 다시 확인하고 R5 release-prep PR·5플랫폼 dry-run으로 이행하는 절차다.
