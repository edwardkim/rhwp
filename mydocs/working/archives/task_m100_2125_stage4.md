# Task M100 #2125 Stage 4 실행 보고 - fresh build와 소비자 gate

- 이슈: #2125
- 브랜치: `task2125-assets-fonts-canonical`
- source 기준: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 검증 대상 commit: `540db0169dc5ca9418c67c9709e0de0b1e478fb3`
- 완료일: 2026-07-13
- 상태: Stage 4 PASS, Stage 5 승인 대기

## 1. 판정 요약

fresh Docker WASM으로 Studio, Chrome, Firefox, VS Code와 frontend contract를 다시 빌드·검증했다.
canonical 36개 WOFF2는 모든 전체-copy 소비자와 byte/hash가 일치했고, VS Code 11개 subset도 일치했다.
Studio browser와 legacy `/web/fonts` URL에서도 대표 font가 실제로 응답했다.

Safari는 Chrome dist 상속, 36개 font parity, manifest WAR와 unsigned Xcode compile이 통과했다. 다만 로컬
keychain에 team `FSFG3AQ4KZ`의 `Mac Development` 인증서가 없어 기본 signed `xcodebuild`는 실패했다.
작업지시자 승인에 따라 signed build는 설치·배포용 release gate로 분류하고, unsigned compile과 최종
`.appex` resource parity를 #2125의 Safari PR gate로 확정했다.

## 2. 실행 환경과 fresh WASM

| 항목 | 값 |
|------|----|
| Node.js / npm | `v24.15.0` / `11.12.1` |
| Docker client / server | `29.4.0` / `29.2.1` |
| Docker context | `colima` |
| Compose | standalone `docker-compose` `5.1.3` |
| Xcode | `26.6` (`17F113`) |
| source 상태 | metrics 측정 시 clean |

현재 Colima 설정은 `/private/tmp`만 writable mount로 허용해 저장소 경로를 compose container의 `/app`에
직접 노출하지 못했다. 동일 commit `540db016`의 임시 detached worktree를 `/private/tmp`에 만들고 repository
`.env.docker`와 compose 정의를 그대로 사용했다. build 전 `pkg/`가 없음을 확인했으므로 stale output을
재사용하지 않았다.

```bash
docker-compose --env-file .env.docker run --rm wasm
```

release compile과 `wasm-opt`가 2분 19초에 완료됐다. 생성물을 주 worktree에 복사한 뒤 hash가 동일함을
확인했다.

| 생성물 | bytes / SHA-256 |
|--------|-----------------|
| `pkg/rhwp_bg.wasm` | 6,662,474 / `41d675bebe3c981903ef7c0ab67b0e38393c379a215f12693901d57e73f2cb92` |
| `pkg/rhwp.js` | `6d86f0d590ebd94b36a3705f349ffda40a84f4503c808200e3c512fc6af84e5f` |
| `pkg/rhwp.d.ts` | `f971dab0908cad13394245918bb37d6b1165120d55fde968b8b5c6415d2c2971` |

`scripts/frontend-metrics`, Studio, Chrome, Firefox와 VS Code에서 각각 `npm ci`를 실행했다. lockfile과
tracked source 변경은 발생하지 않았다.

## 3. package와 contract gate

| gate | 결과 |
|------|------|
| WASM binding + editor embed | 3 PASS |
| `@rhwp/editor` package | 15 PASS, runtime/peer dependency 0 유지 |
| shared/Chrome/Firefox service worker | 88 PASS |
| Studio unit | 230 PASS |
| Studio production build | PASS |
| CanvasKit font coverage | PASS |
| Chrome build | PASS, WOFF2 36개 |
| Firefox build | PASS, WOFF2 36개 |
| extension dist contract | 3 PASS |
| VS Code compile | PASS, 승인된 WOFF2 11개 |
| font asset contract | 4 PASS |

font와 extension contract를 함께 재실행한 최종 확인도 7/7 PASS였다. Studio build의 Node `fs`/`path`
browser externalization과 500 kB 초과 chunk 경고는 발생했지만 build 실패나 font 계약 변화는 없었다.

## 4. runtime smoke와 artifact parity

### 4.1 Studio와 legacy URL

headless Chrome text-flow E2E에서 새 문서, 입력·줄바꿈, 문단 분할, 2페이지 pagination과 merge를 확인했다.
Studio local server에서는 다음 대표 font가 HTTP 200과 WOFF2 signature로 응답했다.

| URL | bytes | 결과 |
|-----|------:|------|
| `fonts/NotoSansKR-Regular.woff2` | 562,220 | HTTP 200, `document.fonts.check` PASS |
| `fonts/LatinModernMath-Regular.woff2` | 391,760 | HTTP 200, `document.fonts.check` PASS |
| `fonts/SourceHanSerifK-OldHangul-subset.woff2` | 239,628 | HTTP 200, WOFF2 PASS |

legacy `/web/fonts/NotoSansKR-Regular.woff2`도 HTTP 200, 562,220 bytes로 응답했고 canonical hash와
일치했다. 따라서 source ownership 이전 뒤에도 기존 runtime `fonts/...` URL은 보존된다.

### 4.2 배포 artifact

| 소비자 | 기대 | 결과 |
|--------|------|------|
| Studio `dist/fonts` | canonical 36개 exact parity | PASS |
| Chrome `dist/fonts` | canonical 36개 exact parity | PASS |
| Firefox `dist/fonts` | canonical 36개 exact parity | PASS |
| Safari `dist/fonts` | canonical 36개 exact parity | PASS |
| VS Code `dist/media/fonts` | 승인된 11개 exact subset/hash | PASS |

canonical inventory는 36개, 22,651,296 bytes이며 `NotoSansKR-Regular.woff2` hash는
`d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a`다. Studio와 legacy link target은
각각 `../../assets/fonts`, `../assets/fonts`로 측정됐다.

## 5. Safari gate

`bash rhwp-safari/build.sh`의 Chrome rebuild, Safari dist copy, Safari JS 문법 검사와 source 적용은
통과했다. Safari manifest WAR도 기존의 `wasm/*`, `fonts/*`, `icons/*`만 유지했다.

signed Xcode 단계는 다음 환경 사유로 실패했다.

```text
No signing certificate "Mac Development" found for team FSFG3AQ4KZ
```

동일 project/scheme을 `CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO`으로 별도 빌드한 결과
`** BUILD SUCCEEDED **`였고, 생성된 app extension bundle의 WOFF2 36개도 canonical source와 hash mismatch
0이었다. Xcode project가 `rhwp-safari/dist/fonts`를 실제 extension Resources에 포함하는 단계까지 검증했으므로
source compile과 resource packaging에 관한 #2125 Safari gate를 충족한다.

signed build는 Apple developer identity, team certificate와 설치·배포 가능성을 검증한다. font source path와
Xcode resource 포함 여부를 검증하는 이번 PR의 정확성 관문은 아니므로 비차단 release gate로 남긴다. 실제
Safari 설치·수동 동작 확인과 App Store/TestFlight 검증도 release 단계에서 수행한다.

추가로 `rhwp-safari/build.sh`는 `set -e`만 사용한 상태에서 `xcodebuild | tail -3`을 실행한다. 현재 shell은
파이프 앞의 `xcodebuild` 실패를 script exit code에 반영하지 않아 `BUILD FAILED` 뒤에도 `빌드 완료`를
출력했다. `set -o pipefail` 또는 `PIPESTATUS` 처리가 필요한 별도 build reliability 결함이며 #2125의 font
path 변경에는 섞지 않는다. Stage 5에서 후속 이슈 초안을 제시한다.

## 6. frontend metrics

maintainer의 최신 복잡도 교훈에 맞춰 Total CC, Top 20 합, CC>25 합·개수, CC>100 합·개수, max와 stable
function diff를 함께 확인했다. 측정 결과에는 clean/dirty 상태, Node/OS, metrics script와 lockfile hash도
남았다.

### 6.1 #2124 공식 snapshot 대비 누적 변화

| 지표 | #2124 `3077f96d` | 현재 | delta |
|------|----------------:|-----:|------:|
| reported functions | 2,282 | 2,373 | +91 |
| Total CC | 11,805 | 12,159 | +354 |
| Top 20 합 | 2,581 | 2,581 | 0 |
| CC>25 개수 | 62 | 64 | +2 |
| CC>25 합 | 3,932 | 3,994 | +62 |
| CC>100 개수 | 6 | 6 | 0 |
| Max CC | 453 | 453 | 0 |

이 수치는 #2124 이후 `devel`에 merge된 CanvasKit/local font/embed 등 전체 누적 변화다. #2125가 product
함수를 변경한 결과로 귀속하지 않는다.

### 6.2 동일 upstream 기준 #2125 직접 변화

`upstream/devel` `e750e02f`를 별도 clean worktree에서 같은 metrics dependency로 측정하고 현재 branch와
비교했다.

| 지표 | upstream | #2125 | delta |
|------|---------:|------:|------:|
| reported functions | 2,373 | 2,373 | 0 |
| Total CC | 12,159 | 12,159 | 0 |
| Top 20 합 | 2,581 | 2,581 | 0 |
| CC>25 개수 | 64 | 64 | 0 |
| CC>25 합 | 3,994 | 3,994 | 0 |
| CC>100 개수 | 6 | 6 | 0 |
| Max CC | 453 | 453 | 0 |
| stable function diff | - | - | 0건 |

따라서 path-only migration의 product complexity 0-delta 기대를 충족한다. #2124 공식 snapshot
`metrics.json`과 `summary.md` hash도 각각 `2f84ea2a...765`, `f5a2d34f...20d6`으로 변경되지 않았다.

## 7. Stage 4 관문 상태

| 관문 | 상태 |
|------|------|
| fresh WASM과 frontend package gate | PASS |
| canonical/font distribution/runtime 계약 | PASS |
| Chrome/Firefox/VS Code | PASS |
| Safari asset inheritance와 unsigned compile | PASS |
| Safari `.appex` 36개 font resource parity | PASS |
| Safari signed Xcode build | 로컬 인증서 부재로 미통과, 비차단 release gate |
| #2125 직접 complexity delta | PASS, 전 항목 0 |

로컬 migration과 PR-level Safari 검증은 완료됐고 font source 또는 artifact 결함은 발견되지 않았다.
작업지시자가 unsigned compile·최종 `.appex` parity를 #2125의 동등 gate로 승인했으므로 Stage 4는 PASS다.
Stage 5는 별도 승인 전 시작하지 않는다.
