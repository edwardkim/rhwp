# Task M100 #2313 Stage 4 완료 보고 — fresh frontend 소비자 gate

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- 검증 대상 commit: `8c5a52e2f4163415bf6406f1c2443aa0d65b54bd`
- Stage 3 commit: `8c5a52e2`
- 완료일: 2026-07-17
- 상태: Stage 4 local 완료, Stage 5 승인 대기

## 1. Stage 4 판정

동일 commit에서 fresh WASM을 생성한 뒤 Studio, Chrome, Firefox, VS Code와 npm editor의 current 소비자
gate를 모두 통과했다. Studio text-flow, iframe embed transport와 CanvasKit font coverage도 독립 headless
Chrome에서 통과했다. 따라서 legacy `/web` 제거 뒤에도 current frontend의 load/edit/export/embed와 canonical
font 소비 계약은 유지된다.

Safari는 구현 계획에서 정한 static inheritance/security gate를 통과했다. Safari source, manifest와 Chrome
dist 계약에 변경이 없으므로 Xcode project 생성, unsigned/signed build와 설치 검증은 실행하지 않았다.

검증 과정에서 product source, package lockfile와 tracked build artifact는 수정하지 않았다.

## 2. 기준 commit과 실행 환경

| 항목 | 결과 |
|------|------|
| 검증 대상 `HEAD` | `8c5a52e2f4163415bf6406f1c2443aa0d65b54bd` |
| `upstream/devel` | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| merge-base | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| branch 상태 | upstream 대비 ahead 4, behind 0 |
| upstream drift | 0 |
| Node.js / npm | v24.15.0 / 11.12.1 |
| Docker Compose | standalone 5.1.3 |
| browser smoke | Google Chrome 150.0.7871.115, headless |

## 3. fresh WASM 생성

로컬 Docker CLI에는 Compose plugin이 없어 계획서의 `docker compose --env-file ...` 형식을 사용할 수 없었다.
standalone `docker-compose`는 동작했지만 Colima 공유 경로가 `/private/tmp`로 제한되어 main worktree를 bind
mount한 컨테이너의 `/app`이 비어 있었다. 이는 source/build 오류가 아니라 host mount 설정 차이다.

stale `pkg/`를 사용하지 않고 다음 방식으로 동일 commit의 fresh output을 만들었다.

1. `/private/tmp/rhwp-task2313-wasm`에 `8c5a52e2` detached worktree 생성
2. `.env.docker`를 임시 worktree에 복사
3. `docker-compose --env-file .env.docker run --rm wasm` 실행
4. release build와 `wasm-opt` 완료, 소요 3분 8초
5. 생성된 `pkg/`를 main worktree에 exact sync하고 `diff -rq`로 동일성 확인
6. 임시 worktree, compose network·volume와 task 전용 Docker image 삭제

| fresh artifact | SHA-256 |
|----------------|---------|
| `pkg/rhwp_bg.wasm` | `1d3e6d3788020f9655e2e2147b0d58969f1a8c3a4e38e77e1dd59e1fe750c93d` |
| `pkg/rhwp.js` | `cc657a26658654ac0d4027324b79385ae1d9ae1c95a2adc99fe049a9343595f8` |
| `pkg/rhwp.d.ts` | `9e2a77deb9ee8b474a715821a5975ca64cc92eb81c424c4514a799b416b5852f` |
| `pkg/rhwp_bg.wasm.d.ts` | `69b29f652a16e10a291e4ae1869f8381c939d15f2503fd5be2c5a8451c0baa3c` |

`pkg/`는 repository 정책에 따른 ignored build output이며 commit 대상이 아니다.

## 4. clean package input

각 package에서 lockfile 기준 `npm ci`를 실행했다. `npm audit fix`, dependency upgrade와 lockfile 변경은 수행하지
않았다.

| package | 설치 결과 |
|---------|-----------|
| `rhwp-studio` | PASS, 380 packages |
| `rhwp-chrome` | PASS, 18 packages |
| `rhwp-firefox` | PASS, 18 packages |
| `rhwp-vscode` | PASS, 153 packages |

Studio install의 `source-map`, `glob` deprecation warning은 설치 실패나 이번 변경에서 발생한 dependency
delta가 아니다.

## 5. CI parity와 package gate

| 검증 | 결과 |
|------|------|
| WASM binding + editor embed static contract | PASS, 3/3 |
| `@rhwp/editor` package test | PASS, 15/15 |
| shared/Chrome/Firefox service worker tests | PASS, 88/88 |
| Studio unit/contract tests | PASS, 298/298 |
| Studio TypeScript + Vite build | PASS |
| Chrome extension build | PASS, fonts 36개 |
| Firefox extension build | PASS, fonts 36개 |
| extension dist contract | PASS, 3/3 |
| VS Code compile | PASS, approved fonts 11개 |
| full font asset contract | PASS, 4/4 |

Studio build의 Node `fs`/`path` browser externalization과 chunk size 경고는 기존 Vite build warning이며 build
결과는 성공이다. extension dist contract에는 Safari의 stricter web-accessible-resources assertion도 포함된다.

## 6. Studio browser smoke

최초 기본 host 모드는 repository 기본값 `http://172.21.192.1:19222`의 기존 Chrome CDP에 연결하려 했다.
sandbox 실행은 `EPERM`, 권한 확장 실행은 해당 endpoint의 `EADDRINUSE`로 연결되지 않았다. 앱 assertion 전
실행 환경 충돌이므로 source를 수정하지 않고 헬퍼가 공식 지원하는 `--mode=headless`와 macOS
`CHROME_PATH`를 사용했다.

로컬 Vite server를 `127.0.0.1:7700`에서 실행하고 다음 smoke를 통과했다.

| smoke | 결과 | 핵심 확인 |
|-------|------|-----------|
| text flow | PASS, 5 assertions | 새 문서, 입력, 문단 분리, 2-page overflow, 문단 병합 |
| iframe embed transport | PASS, 10/10 | transferable load/export, HWPX ZIP, diagnostics, forged peer 차단, destroy, legacy path |
| CanvasKit font coverage | PASS | Noto Sans KR symbol coverage |

브라우저 smoke에서 rendering delta나 legacy `/web` 의존은 발견되지 않았다. baseline/render-diff 전체는 renderer
output 변경이 없는 이번 task의 필수 gate가 아니므로 구현 계획대로 실행하지 않았다.

## 7. Safari static gate

| 검증 | 결과 |
|------|------|
| `bash -n rhwp-safari/build.sh` | PASS |
| background/content/options `node --check` | PASS, 3/3 |
| Chrome dist 상속 | PASS, `CHROME_DIST`를 `dist`로 복제 |
| font WAR | PASS, manifest `fonts/*` 유지 |
| stricter WAR security | PASS, extension dist contract 포함 |

Safari는 current repository 흐름상 Chrome extension dist를 변환하는 후순위 consumer다. 이번 task는 Safari
source나 Xcode project를 변경하지 않으므로 signed build를 대체할 별도 산출물을 만들지 않고, 제거가 Chrome
dist와 manifest 상속을 깨뜨리지 않았다는 static evidence만 필수로 적용했다.

## 8. Stage 4 gate 결과

| 관문 | 결과 |
|------|------|
| fresh WASM build | PASS |
| CI parity package gate | PASS |
| Studio text-flow/embed/font browser smoke | PASS |
| Chrome/Firefox/VS Code font artifact contract | PASS |
| Safari static inheritance/security contract | PASS |
| legacy `/web` runtime dependency 발견 | 0 |
| source/lockfile/tracked build artifact 변경 | 0 |
| temporary task resource | 정리 완료 |
| upstream drift | 0 |

## 9. 다음 단계 경계

작업지시자 승인 후 Stage 5에서만 다음을 수행한다.

1. 최종 commit 상태의 post-removal metrics 재생성
2. same-base pre/post, legacy group과 #2124 official baseline 비교 분리
3. current Phase B hotspot 재평가 입력 정리
4. Stage 5 보고서, 최종 보고서와 orders 완료 상태 작성
5. PR 본문·리뷰 요청과 #2313/#2022 GitHub 업데이트 초안 작성

Stage 5 승인 전에는 metrics 결산, 최종 완료 판정, push, PR 생성, GitHub 게시와 issue close를 수행하지 않는다.
