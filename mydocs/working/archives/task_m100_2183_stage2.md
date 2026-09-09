# Task M100 #2183 Stage 2 완료 보고 — local frontend consumer gate

- 이슈: #2183
- 상위 추적: #2022
- 브랜치: `task2183-frontend-ci-gate`
- 검증 기준: `upstream/devel@48c3345526d20720e9f0a80743bbfb8dde5813d4`
- 검증 HEAD: `82a297ab53e0e67c15f2622d4a8819699fef8cb7`
- 작성일: 2026-07-11
- 수행 계획서: `mydocs/plans/task_m100_2183.md`
- 구현 계획서: `mydocs/plans/task_m100_2183_impl.md`

## 1. 최신 upstream 동기화

Stage 2 시작 시 기존 계획 기준 `3cf6d949` 이후 upstream이 44커밋 진행된 것을 확인했다. Studio 설정·unit
test와 Rust renderer 변경이 포함돼 기존 기준으로 consumer gate를 실행하지 않고 최신 `48c33455`를 fetch한
뒤, 아직 push하지 않은 #2183의 4개 커밋을 rebase했다.

충돌은 `mydocs/orders/20260711.md`에서만 발생했다. upstream의 최신 운영 기록과 M100 표를 모두 보존하고
#2183 행의 단계 상태를 합쳤다. `.github/workflows/ci.yml`에는 upstream 충돌이 없었으며 rebase 후
`actionlint`와 `git diff --check`를 다시 통과했다.

## 2. local 실행 환경

| 항목 | 값 |
|------|----|
| host | macOS arm64 |
| Node.js | `v24.15.0` |
| npm | `11.12.1` |
| host Rust | `rustc 1.93.1 (01f6ddf75 2026-02-11)` |
| Docker server | Engine `29.2.1`, linux/arm64 |
| Compose | standalone `docker-compose 5.1.3` |
| daemon | Colima, Virtualization.Framework, 4 CPU, 6 GiB, virtiofs |
| wasm-pack | `0.15.0` |

Docker CLI와 compose는 설치돼 있었지만 daemon은 정지 상태였고 Docker Desktop 애플리케이션은 없었다. 이미
설치된 Colima를 사용했으며, worktree가 `/private/tmp`에 있으므로 해당 경로를 writable mount로 지정해
재시작했다. 검증 완료 후 Colima는 다시 중지했다. local-only `.env.docker`는 main worktree의 ignored 설정을
복사했으며 Git 대상이 아니다.

CI worker는 Node.js 22와 `wasm-pack --dev`를 사용한다. 이 정확한 runner 조합은 Stage 3 GitHub Actions에서
검증하고, Stage 2는 저장소 규칙에 따라 Docker release WASM과 host Node 버전으로 consumer 동작을 검증했다.

## 3. fresh WASM

실행:

```bash
docker-compose --env-file .env.docker run --rm wasm
```

결과:

- Docker image와 `wasm-pack 0.15.0`: 준비 성공
- Rust WASM release compile과 `wasm-opt`: PASS
- 전체 소요: 2분 23초
- `pkg/rhwp_bg.wasm`: 6,651,962 bytes
- SHA-256: `f17f274b51ef968fe615ac1191cf87e46819623419609fd1f81094b851ba4148`

binding test는 fresh `pkg/rhwp.d.ts`가 최신 `src/wasm_api.rs`의 explicit export를 모두 포함함을 확인했다.
tracked `web/rhwp.*`나 Studio public output을 `pkg/` 대용으로 복사하지 않았다.

## 4. package install

네 lockfile에서 각각 `npm ci`를 실행했다.

| package | 결과 | audit 관찰 |
|---------|------|------------|
| `rhwp-studio` | PASS, 379 packages | low 1 |
| `rhwp-chrome` | PASS, 18 packages | 0 |
| `rhwp-firefox` | PASS, 18 packages | 0 |
| `rhwp-vscode` | PASS, 153 packages | high 1 |

audit 결과는 관찰만 기록했다. `npm audit fix`, package manifest 변경, lockfile 갱신은 수행하지 않았다. 네
package-lock은 설치 전후 tracked diff가 없다.

## 5. contract, test, build 결과

| gate | 결과 | 근거 |
|------|------|------|
| WASM binding + editor embed | PASS, 2 tests | fresh declaration과 무의존 iframe/message contract |
| `npm/editor test --if-present` | PASS | 현재 test script 부재로 정상 no-op; #2187 이후 자동 실행 대상 |
| shared + Chrome/Firefox SW | PASS, 88 tests | URL, download state, fetch security 포함 |
| Studio unit | PASS, 186 tests | 최신 upstream의 clipView test 포함 |
| Studio production build | PASS | TypeScript + Vite 8.1.4 + PWA output |
| Chrome production build | PASS | fresh WASM과 36 fonts copy |
| Firefox production build | PASS | fresh WASM과 36 fonts copy |
| extension dist contract | PASS, 3 tests | Chrome/Firefox CSP·WAR·font·WASM, Safari WAR |
| VS Code production compile | PASS | extension host와 webview webpack 5.105.4 |

Chrome, Firefox, VS Code에 복사된 `rhwp_bg.wasm`은 모두 fresh `pkg`와 같은 6,651,962 bytes와 SHA-256을
가진다.

## 6. 비실패 warning

다음은 기존 build warning이며 gate를 실패시키지 않았다.

- CanvasKit의 `fs`, `path` browser externalization
- Vite minified chunk 500 kB 초과
- extension Vite의 runtime SVG 경로 보존 메시지
- extension outDir이 Studio root 밖이라는 안내
- Studio install의 deprecated dependency 2건

extension dist contract가 실제 copied asset, CSP, WAR, inline script, font/WASM 결과를 별도로 검사했으므로 위
warning을 숨기거나 설정 변경으로 섞지 않았다.

## 7. clean 상태

- generated `pkg/`, package `node_modules/`, Studio/extension/VS Code `dist/`는 ignored output이다.
- package-lock tracked diff 없음.
- 제품 source와 workflow 추가 수정 없음.
- Stage 2 문서 작성 전 tracked worktree clean 확인.

## 8. 다음 단계

Stage 3은 draft PR의 clean Ubuntu runner에서 다음을 실측한다.

1. Node.js 22와 `wasm-pack build --target web --dev` 조합
2. workflow 변경 경로가 frontend worker를 실제 실행시키는지
3. 기존 Rust jobs와 frontend worker의 병렬 실행 및 `Build & Test` 집계
4. frontend WASM cache hit/miss와 job duration
5. PR restore-only cache 정책

push와 draft PR 생성 전에 branch push/PR 본문/리뷰 요청 초안을 작업지시자에게 제시한다. 승인 전에는
GitHub 상태를 변경하지 않는다.
