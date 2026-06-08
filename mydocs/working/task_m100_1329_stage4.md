# Task M100-1329 Stage 4 검증 보고서 — 글머리표 빈 줄 caret 위치

## 범위

- 대상 이슈: #1329 `rhwp-studio: 글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨`
- 브랜치: `issue-1329-bullet-caret`
- 검증 대상:
  - Rust 포맷
  - Issue 1329 회귀 테스트
  - 인접 cursor rect 회귀 테스트
  - 라이브러리 전체 테스트
  - rhwp-studio 빌드 가능 여부

## 통과한 검증

### Rust 포맷

```bash
cargo fmt --all -- --check
```

결과: 통과

### Issue 1329 회귀 테스트

```bash
cargo test --test issue_1329_bullet_caret
```

결과:

```text
running 3 tests
test issue_1329_plain_empty_paragraph_caret_keeps_original_start ... ok
test issue_1329_number_enter_empty_line_caret_stays_after_marker ... ok
test issue_1329_bullet_enter_empty_line_caret_stays_after_marker ... ok

test result: ok. 3 passed; 0 failed
```

### 인접 cursor rect 회귀 테스트

```bash
cargo test --test issue_1308_forced_break_hanging_indent
```

결과:

```text
running 8 tests
test result: ok. 8 passed; 0 failed
```

### 라이브러리 전체 테스트

```bash
cargo test --lib
```

결과:

```text
test result: ok. 1613 passed; 0 failed; 6 ignored
```

## rhwp-studio 빌드 및 로컬 서버 검증

처음 실행:

```bash
npm run build
```

결과:

```text
sh: tsc: command not found
```

원인: `rhwp-studio/node_modules`가 없는 상태였다.

의존성 설치:

```bash
npm ci
```

결과:

```text
added 379 packages
found 0 vulnerabilities
```

의존성 설치 후 재실행:

```bash
npm run build
```

결과:

```text
src/core/wasm-bridge.ts(1,44): error TS2307: Cannot find module '@wasm/rhwp.js'
src/hwpctl/index.ts(377,57): error TS2307: Cannot find module '@wasm/rhwp.js'
```

원인: `rhwp-studio`의 Vite alias는 `@wasm`을 저장소 루트 `pkg/`로 매핑한다. 현재 `pkg/rhwp.js`가 없다.

WASM 산출물 생성을 위해 프로젝트 문서의 Docker 경로를 먼저 시도했다.

```bash
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
unknown flag: --env-file
```

현재 환경은 `docker compose` 플러그인이 아니라 `docker-compose` 명령을 제공했다.

```bash
docker-compose --env-file .env.docker run --rm wasm
```

결과:

```text
Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?
```

Docker daemon이 실행 중이 아니어서 Docker 경로는 사용할 수 없었다. 이후 로컬에 설치된 `wasm-pack`과 `wasm32-unknown-unknown` target을 확인하고 로컬 WASM 빌드로 전환했다.

```bash
wasm-pack build --target web
```

첫 실행은 `wasm-bindgen` 보조 바이너리 설치 권한 문제로 실패했으나, 외부 권한 실행으로 재시도해 성공했다.

```text
Your wasm pkg is ready to publish at /Users/melee/Documents/projects/forks/rhwp/pkg.
```

WASM 산출물 생성 후 `rhwp-studio` 빌드를 다시 실행했다.

```bash
npm run build
```

결과:

```text
✓ built in 531ms
```

로컬 dev server 실행:

```bash
npm run dev -- --host 127.0.0.1 --port 7700
```

결과:

```text
VITE v8.0.16 ready
Local: http://127.0.0.1:7700/
```

서버 응답 확인:

```bash
curl -I http://127.0.0.1:7700/
```

결과:

```text
HTTP/1.1 200 OK
```

작업지시자가 `http://127.0.0.1:7700/`에서 직접 시각 검증할 수 있도록 dev server를 실행해 둔 상태다.

## 검증 결론

Rust 레벨 검증은 통과했다.

- 새 회귀 테스트는 글머리표/번호 문단에서 Enter 직후 빈 list 줄 caret x가 실제 입력 후 첫 글자 시작 x와 일치함을 확인한다.
- 일반 빈 문단 cursor x 회귀도 함께 확인했다.
- 인접 cursor rect 회귀 테스트와 `cargo test --lib`도 통과했다.

rhwp-studio 빌드와 로컬 서버 응답 확인까지 완료했다. 브라우저에서 글머리표 Enter 직후 caret 위치를 직접 눈으로 확인하는 단계는 작업지시자가 실행 중인 로컬 서버에서 수행한다.

## 다음 단계

Stage 5 최종 보고서를 작성한다. 최종 보고서 승인 후 커밋, push, PR 작성 단계로 진행한다.
