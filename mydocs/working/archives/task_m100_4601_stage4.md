# Task M100 #4601 Stage 4 — v0.8.3 로컬 릴리즈 게이트

- 이슈: [#4601](https://github.com/edwardkim/rhwp/issues/4601)
- 기준: `upstream/devel@572786d02`
- 검증일: 2026-08-11~12 KST
- 상태: 로컬 릴리즈 게이트 완료

## 1. 네이티브 코어

릴리즈 프로필의 실제 CLI와 기본·Skia 표면을 검증했다.

| 게이트 | 결과 |
| --- | --- |
| `cargo build --release --bin rhwp` | 통과, `rhwp v0.8.3` |
| `cargo test --release --lib` | 3,498 passed / 13 ignored |
| release-test 전체 nextest | 5,767 passed / 36 skipped / 0 failed / 6 slow |
| Native Skia lib | 58 passed |
| Native Skia picture integration | 2 passed |
| Native Skia direct PDF | 4 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| doctest | 4 passed / 2 ignored |

nextest 최장 항목은 overflow cell 테스트의 239.703초였다. 설치된 nextest 0.9.137보다
0.9.140을 권고한다는 경고가 있었지만 테스트 결과에는 영향을 주지 않았다.

Skia 최초 실행은 샌드박스 DNS가 rust-skia GitHub 다운로드를 막아 실패했다. 호스트
네트워크에서 같은 명령을 한 번 재실행해 세 묶음을 모두 통과시켰으며, 이 최초 결과는
코드 실패로 세지 않는다.

## 2. WASM·Studio·편집기

프로젝트의 공식 Docker 절차로 WASM을 새로 만들고 Studio가 직접 참조하는 `pkg/`에
배치했다.

| 항목 | 결과 |
| --- | --- |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, 5분 48초 |
| `pkg/rhwp_bg.wasm` | 8,038,719 bytes, SHA-256 `0731be6a6717c0a71e170b34e4ede571df06055163ccd036c5f75671e544dff1` |
| `pkg/rhwp.js` | 404,678 bytes, SHA-256 `d2d2a33e1d75e122a8772fbdb372dcd547f3de4e5a16bf2e663bfb591f17656f` |
| Studio TypeScript | 통과 |
| Studio production build | 통과 |
| Studio 전체 테스트 | 846 passed / 1 skipped / 0 failed |
| `npm/editor` | 8 passed |
| frontend WASM/editor 계약 | 2 passed |
| `@rhwp/editor` dry-run pack | `0.8.3`, 5 files |

Studio의 Vite alias는 복사본이 아니라 `../pkg/rhwp.js`를 직접 가리키므로 dev 서버에도
새 WASM이 적용된다. production build의 500kB 초과 chunk 경고는 기존 크기 경고이며
빌드는 통과했다.

TypeScript 7은 명시적 파일과 프로젝트 config를 함께 읽지 않으므로 editor 공개 d.ts
검증에는 `--ignoreConfig`가 필요했다. 또한 npm 11은 root에서 `--prefix ... pack`을
실행할 때 root `package.json`을 찾는 동작을 보였다. 실제 통과 명령에 맞춰
`local_validation.md`를 고쳤다.

## 3. 호스트 CDP E2E

- Chrome `150.0.7871.129`, CDP protocol 1.3 연결 확인
- 실행 중인 Studio dev 서버 `http://localhost:7700/` 응답 확인
- `edit-pipeline.test.mjs --mode=host`: 49 passed / 0 failed
- 이미지 삽입 1건은 기존 스크립트의 명시적 SKIP(`arg.charCodeAt is not a function`)
- E2E manifest: tracked 94개 / manifest 94행, 이상 없음

HTML 결과는 `output/e2e/edit-pipeline-report.html`에 만들었다. E2E가 갱신한 스크린샷은
기존 추적 파일과 바이트가 같아 작업 diff에는 남지 않았다.

## 4. Python 공개 명령 표면 보완

전체 Python gate가 처음에는 capability parity 2건에서 실패했다. 최신 devel이 추가한
JSON 명령 11개가 공식 Python 바인딩에 노출되지 않았기 때문이다.

- 누락 top-level: `keygen`, `verify-signature`, `harness`, `anchor`, `gate`, `bundle`,
  `disclose`, `settle`, `audit-report`, `recall-scope`, `conformance`
- Node 공식 바인딩과 CLI parser를 대조해 Python 함수 20개와 선택 인자를 같은 계약으로 구현
- `lineage`의 `keyring`·`anchor_log` 선택 축도 함께 노출
- 판정 명령은 `raise_on_verdict`를 끝까지 전달
- capability·통합 coverage 고정 집합과 argv 단위 테스트를 갱신

최종 Python 결과는 다음과 같다.

| 게이트 | 결과 |
| --- | --- |
| pytest 전체 | 304 passed / 2 skipped |
| mypy 1.10.1 | 오류 0 |
| ruff 0.11.13 | 오류 0 |
| capability parity | 공개 agent-value 명령 누락 0 |

LlamaIndex·LangChain adapter가 `commands.DEFAULT_TIMEOUT`이라는 비명시 export에
의존하던 기존 mypy 오류 2건은 내부 실행 계층에서 상수를 직접 import하도록 고쳤다.

## 5. Python·Node 릴리즈 버전과 패키지

`tools/set_package_version.py`가 pyproject와 package.json만 `0.8.3`으로 바꾸고 공개
런타임 상수는 `0.1.0`으로 남기는 배포 결함을 발견했다. 도구가 다음 네 표면을 한 번에
정렬하도록 고치고 회귀 테스트를 추가했다.

1. `bindings/python/pyproject.toml`
2. Python `rhwp.__version__`
3. `bindings/node/package.json`
4. Node `VERSION`

실제 정렬 상태에서 패키지를 만들고 다시 개발 표지 `0.1.0`으로 돌렸다. 이 일시 정렬은
릴리즈 workflow와 같은 동작이며 작업 diff에는 포함하지 않는다.

| 산출·게이트 | 결과 |
| --- | --- |
| Python sdist | `rhwp-0.8.3.tar.gz`, 108KiB |
| Python universal wheel | `rhwp-0.8.3-py3-none-any.whl`, 54KiB |
| Python bundled Linux wheel | `rhwp-0.8.3-py3-none-manylinux_2_39_x86_64.whl`, 7.9MiB |
| bundled wheel fresh venv | `rhwp.__version__=0.8.3`, 내장 CLI `rhwp v0.8.3`, capabilities 1.0 |
| Node build/runtime | `VERSION=0.8.3`, CJS types 14개 |
| Node 전체 integration gate | 466 passed / 1 skipped |
| Node 재검증 unit gate | 427 passed |
| `@rhwp/node` dry-run pack | `0.8.3`, 40 files |
| `@rhwp/core` dry-run pack | `0.8.3`, 7 files, 약 3.1MB |

Node·Studio가 자식 Node 드라이버를 띄우는 테스트는 샌드박스에서 stdout/stderr가 빈 채
종료되는 `spawn` 차단 서명을 보였다. 알려진 false failure이므로 한 번 진단한 뒤 호스트
실행 결과만 판정에 사용했다.

## 6. VS Code·브라우저 확장

VS Code Actions와 같은 순서로 최신 `pkg/rhwp.js`·`pkg/rhwp_bg.wasm`을 ignored
`media/`에 복사하고 패키징했다. 복사 전에는 이전 WASM 해시가 남아 있었고, 복사 후
두 파일의 해시는 `pkg/`와 일치했다.

| 대상 | 결과 |
| --- | --- |
| VS Code typecheck + production webpack | 통과 |
| VSIX | `rhwp-vscode-0.8.3.vsix`, 37 files, 16.96MB |
| Chrome/Edge build | 통과, manifest `0.8.3` |
| Firefox build | 통과, manifest `0.8.3` |
| Firefox `web-ext lint` | errors 0 / notices 0 / warnings 7 |

VS Code의 오래된 `node_modules`에는 새 직접 의존성 `@noble/hashes`가 없어 최초
typecheck가 실패했다. `npm ci`로 lockfile 상태를 재현했다. 이어서 audit에서 webpack
개발 경로의 `fast-uri 3.1.4` high 취약점 1건을 확인해 호환 패치 `3.1.5`로 lockfile만
갱신했고, 재감사는 취약점 0건이다.

Firefox 경고 7건은 production bundle의 동적 `innerHTML` 정적 탐지다. 원본은 비교·이력
UI에서 escape를 적용하거나 고정된 수식 명령 데이터를 쓰는 기존 경로이며 AMO 차단
오류는 0건이다.

검증용 확장·VSIX·Python 산출물과 해시는
`output/4601/release-candidate/`에 모았다. 이 폴더는 gitignore 대상이며 최종 GitHub
Release 자산이 아니라 로컬 후보 증빙이다.

최종 Stage 4 커밋을 원천으로 `rhwp-source-0.8.3-amo.zip`도 다시 만들었다. 압축 크기는
30MB이고 ZIP 무결성 검사를 통과했다. 재빌드에 필요한 Cargo lock·Native와 두 도구
workspace·임베드 문서·blank HWP·폰트를 포함하며, `samples/`, `node_modules/`, `target/`,
`dist/`, `output/`, `pdf-large/`는 없다. `.env.docker.example`은 비밀값이 아닌 공식
재현 설정 예제로 명시적으로 포함한다.

## 7. Docker CLI

`docker-publish.yml`과 같은 `Dockerfile.cli`로 이미지를 처음부터 만들었다. 컨테이너
내부 release compile은 7분 30초였고 최종 스모크는 다음과 같다.

| 항목 | 결과 |
| --- | --- |
| image | `sha256:5067d4e2ccad54330a98ca44d91102da4b86516a4f33c51d618fa1bd91f61e6e` |
| size | 94,481,003 bytes |
| `rhwp --version` | `rhwp v0.8.3` |
| `capabilities` | schema 1.0, tool version 0.8.3 |
| runtime user | `uid=10001(rhwp)`, image config user `rhwp` |

로컬 Docker 전송 context는 작업공간의 ignored build·output 파일까지 포함해 약 5GB였다.
GitHub Actions의 깨끗한 checkout에는 이 로컬 산출물이 없으므로 workflow source 크기와는
구분해 기록한다. 최종 image에는 builder tree가 아니라 CLI만 복사된다.

## 8. 의도적으로 남은 채널 게이트

- Safari는 `xcodebuild`와 converter가 필요한 macOS 전용이라 이 Ubuntu 환경에서는
  manifest `0.8.3` 정합만 확인한다.
- deb/rpm/MSI와 4플랫폼 CLI archive는 tag workflow의 각 OS runner에서 최종 생성한다.
- crates.io는 현재 활성 채널이 아니다. `cargo package` 실측은 15,064 files,
  1.6GiB raw / 1.3GiB compressed였고 문서화된 10MB 선행 정비 조건을 넘는다.
  `release-installers.yml`은 token과 `publish --dry-run` 이중 게이트로 게시를 건너뛴다.
- Scoop·Homebrew·Winget·AUR 해시는 실제 태그 자산의 `SHA256SUMS.txt`가 나온 뒤 후속
  PR에서 갱신한다.

## 9. 다음 단계

최종 diff·문서 정합 검사를 마친 뒤 Stage 4 변경을 커밋한다. 그 다음 최신
`upstream/devel`을 다시 조회해 기준선 드리프트가 없는지 확인하고, 작업지시자 승인
전에는 push·PR·tag·publish를 하지 않는다.
