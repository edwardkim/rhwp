---
kind: guide
status: active
canonical: mydocs/manual/dev_environment_guide.md
last_verified: 2026-08-11
---

# 개발 환경 가이드

이 문서는 macOS, Linux, Windows에서 rhwp를 로컬 빌드·테스트하고 rhwp-studio를 실행하는 공통 절차를
설명한다. 개인 PC 이름, 사설 서버, 개인 경로는 프로젝트 계약이 아니다.

## 준비 도구

- Rust stable toolchain과 Cargo
- `wasm-pack`
- Node.js와 npm
- Python 3.12(PDF/SVG 기준 비교를 수행할 때)
- Git
- 선택 도구: `actionlint`, Poppler(`pdfinfo`, `pdftoppm`), `rsvg-convert`

설치 여부는 다음처럼 확인한다.

```bash
rustc --version
cargo --version
wasm-pack --version
node --version
npm --version
python3.12 --version
```

## Python 로컬 가상환경

PDF raster·픽셀 대조처럼 추가 Python 패키지가 필요한 저장소 도구는 시스템 Python에
직접 설치하지 않고 저장소 루트의 Python 3.12 `venv/`만 사용한다. macOS·Linux의 시스템
Python은 PEP 668에 따라 직접 설치를 거부할 수 있고, 전역 설치는 다른 작업의 의존성을
바꿀 수 있다. `venv/`는 `.gitignore`의 `/venv/` 규칙으로 Git 대상에서 제외한다.

저장소 루트에서 최초 1회 다음처럼 만든다.

```bash
python3.12 -m venv venv
venv/bin/python --version
```

POSIX 셸에서는 activate 여부와 무관하게 `venv/bin/python`을 명시해 실행한다. Windows에서는
`venv\\Scripts\\python.exe`를 사용한다. 설치 실패를 피하려고 시스템 Python에
`--break-system-packages`를 사용하거나 `venv/` 내용을 Git에 추가하지 않는다. 필요한 패키지와
실행 명령은 각 도구 문서가 정의하며, PDF/SVG 기준 비교는
[`tools/fidelity_compare/README.md`](../../tools/fidelity_compare/README.md)를 따른다.

## 저장소와 브랜치

로컬 검증 기준은 최신 `upstream/devel`이다. 일반 변경은 작업 브랜치에서 검증한 뒤 `devel` 대상 PR로
통합하며 `upstream/devel`에 직접 push하지 않는다.

fork를 clone하면 원격은 fork를 가리키는 `origin` 하나뿐이다. 원본 저장소를 가리키는 `upstream`을
최초 1회 등록한다.

```bash
git remote add upstream https://github.com/edwardkim/rhwp.git
git remote -v
```

이후 작업 브랜치는 최신 `upstream/devel`에서 만든다.

```bash
git fetch upstream
git switch -c <work-branch> upstream/devel
```

원격 이름이나 쓰기 가능한 원격은 clone 방식과 권한에 따라 다를 수 있다. PR 준비·merge의 역할별 절차는
[PR 리뷰·통합 워크플로우](pr_review_workflow.md)를 따른다.

## 네이티브 빌드와 테스트

```bash
cargo build
cargo test
cargo build --release
cargo fmt --check
```

PR 전 전체 회귀 범위는 변경 위험도와 [PR 리뷰·통합 워크플로우](pr_review_workflow.md)에 따라 결정한다.
macOS에서 통합 테스트 바이너리별 release LTO 링크가 오래 걸릴 때는 다음 프로필을 사용한다.

```bash
cargo test --release --lib
cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
```

`release-test`는 통합 테스트 시간을 줄이기 위한 프로필이며 실제 release 산출물은 계속
`cargo build --release`로 만든다.
`--test-threads 12`는 CPU·메모리가 충분한 host의 기본값이며, 작은 host에서는 논리 CPU 이하로 낮춘다.

## WASM 빌드

Rust 또는 WASM 경계가 바뀌면 저장소 루트에서 `pkg/`를 갱신한다.

```bash
wasm-pack build --target web --out-dir pkg
```

TypeScript와 CSS는 Vite가 다시 읽지만 Rust 변경은 위 빌드가 끝나야 브라우저에 반영된다.

## 웹한글컨트롤 호환 층

`@rhwp/hwpctrl` 개발은 일반 WASM 빌드 외에 패키지 공개 경로 검사와 OS별 시나리오 gate를 쓴다.
macOS·Linux도 WASM 자체 시나리오 검증을 실행할 수 있으며, Hancom 2022 COM Oracle의 새 수집은
Windows 전용이다. 소비자 연결 방법, 지원 범위, fixture 대조와 live Oracle의 검증 경계는
[웹한글컨트롤 호환 개발 가이드](webhwpctrl_compat_development.md)를 따른다.

## rhwp-studio

```bash
cd rhwp-studio
npm ci
npx vite --host 0.0.0.0 --port 7700
```

해당 포트가 이미 사용 중이면 기존 서버를 확인하거나 다른 포트를 지정한다. 브라우저 검증 절차는
[시각 검증 문서 지도](verification/README.md)와 각 E2E 가이드를 따른다.

## Subsecond 핫패치 (개발 전용, unix 호스트 전용)

Rust 를 고쳐도 WASM 재빌드 없이 실행 중인 브라우저에 반영하는 개발 전용 경로다. 기본 빌드에는
들어가지 않는다 — 루트 `Cargo.toml` 의 `subsecond-dev` feature 뒤에 있고, 그 feature 로 빌드해야
`applySubsecondDevtoolsMessage` 같은 export 가 생긴다.

```bash
cd rhwp-studio
npm run subsecond:install   # dioxus-cli 0.7.10 을 target/dioxus-cli 에 고정 설치
npm run subsecond:serve     # 별도 터미널: dx serve --hot-patch (127.0.0.1:7711)
npm run dev:subsecond       # RHWP_SUBSECOND=1 vite
```

### 지원 플랫폼 — unix 호스트에서만 동작한다

`tools/rhwp-subsecond/build.rs` 는 `dx` 가 찾는 `deps/librhwp-dioxus.rlib` 별칭을 **심링크**로
만든다. 이 별칭의 대상(`librhwp.rlib`)은 이 빌드 스크립트가 끝난 뒤에야 생기므로 복사로 대체할 수
없고, 심링크 생성은 `#[cfg(unix)]` 뒤에 있다. 따라서 **Windows 호스트에서는 핫패치가 동작하지
않는다.** WSL 안에서 빌드하면 unix 경로이므로 동작한다.

이때 겉으로 드러나는 증상은 "아무 일도 일어나지 않음"이다 — `subsecond:install` 성공,
`dx serve` 정상 출력, Vite 기동, `/_dioxus` 소켓 연결, 데브서버의 `HotReload` 수신까지 모두 정상이고
화면만 바뀌지 않는다. 그래서 unix 가 아닌 호스트에서 wasm32 대상으로 빌드하면 빌드 스크립트가
`cargo:warning` 한 줄을 남긴다.

### 실패를 어디서 읽는가

핫패치는 층이 여럿이라 "안 바뀐다"는 증상만으로는 어느 층이 멈췄는지 알 수 없다. 층별 신호는 다음과
같다.

| 층 | 실패 신호 | 보는 곳 |
|---|---|---|
| 별칭 심링크 (`build.rs`) | `cargo:warning=rhwp-subsecond: …` | `subsecond:serve` 터미널 |
| `dx` 패치 링크 | dx 오류 출력 | `subsecond:serve` 터미널 |
| 메시지 판정 (`src/subsecond_dev.rs`) | `[subsecond] …` 진단 | 브라우저 콘솔 |
| wasm 패치 적용 (subsecond 크레이트 내부) | Rust panic + 전역 오류 경고 | 브라우저 콘솔 |

마지막 층에는 구조적인 한계가 있다. wasm32 에서 `subsecond::apply_patch` 는 patch wasm 의
fetch/compile/instantiate future 를 띄우고 **즉시** `Ok(())` 를 돌려주므로(subsecond 0.7.10
`src/lib.rs:551`, `:690`), 적용 성공 여부는 `applySubsecondDevtoolsMessage` 의 반환값이 될 수 없다.
그래서 그 반환값은 `patch-dispatched`("넘겼다")까지만 말하고 "적용됐다"고 말하지 않는다. future
안의 실패는 전부 `.unwrap()`/`panic!`(`lib.rs:578-582`)이라 다음 두 곳으로만 나온다.

- `console_error_panic_hook`(기본 feature)이 남기는 `console.error` 의 Rust panic 메시지
- panic 이 wasm trap 이 되어 microtask 경계를 넘은 전역 `error`/`unhandledrejection` 이벤트 —
  `rhwp-studio/src/core/subsecond-runtime.ts` 가 이를 듣고 "패치를 넘긴 뒤 오류가 도달했다"로 보고한다

브라우저 콘솔의 `[subsecond]` 진단은 개발 빌드에서만 나온다(`import.meta.env.DEV`).

## OS별 참고

### macOS

- Apple Silicon과 Intel 환경에서 Homebrew 경로가 다를 수 있으므로 실행 파일 경로를 하드코딩하지 않는다.
- GUI 앱은 shell의 PATH를 상속하지 않을 수 있다. 확장이나 MCP 설정은 `which <command>` 결과를 확인한다.

### Linux

- 네이티브 라이브러리나 폰트 패키지가 필요한 테스트는 배포 대상 Linux와 같은 패키지를 설치한다.
- CI와 교차 검증은 임시 복제본보다 지정된 실제 작업 디렉터리를 사용한다.

### Windows

- PowerShell, `cmd`, SSH 기본 셸의 quoting과 PATH가 다르므로 CLI 변경은 필요한 셸에서 각각 확인한다.
- WSL 경로와 Windows 경로를 같은 명령에서 혼용하지 않는다.

## 로컬 전용 파일

다음 항목은 생성물 또는 비밀 정보이므로 Git에 커밋하지 않는다.

- `target/`, `pkg/`, `node_modules/`
- `.env*`의 토큰과 서버 주소
- 개인 SSH 키
- 라이선스상 재배포할 수 없는 로컬 폰트

공개 문서 예시는 `/Users/me/...`, `/home/me/...`, `C:\Users\me\...` 같은 일반 경로를 사용한다.
