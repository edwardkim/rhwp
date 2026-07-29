---
kind: guide
status: active
canonical: mydocs/manual/dev_environment_guide.md
last_verified: 2026-07-17
---

# 개발 환경 가이드

이 문서는 macOS, Linux, Windows에서 rhwp를 로컬 빌드·테스트하고 rhwp-studio를 실행하는 공통 절차를
설명한다. 개인 PC 이름, 사설 서버, 개인 경로는 프로젝트 계약이 아니다.

## 준비 도구

- Rust stable toolchain과 Cargo
- `wasm-pack`
- Node.js와 npm
- Git
- 선택 도구: `actionlint`, Poppler(`pdfinfo`, `pdftoppm`), `rsvg-convert`

설치 여부는 다음처럼 확인한다.

```bash
rustc --version
cargo --version
wasm-pack --version
node --version
npm --version
```

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
cargo test --profile release-test --tests
```

`release-test`는 통합 테스트 시간을 줄이기 위한 프로필이며 실제 release 산출물은 계속
`cargo build --release`로 만든다.

## WASM 빌드

Rust 또는 WASM 경계가 바뀌면 저장소 루트에서 `pkg/`를 갱신한다.

```bash
wasm-pack build --target web --out-dir pkg
```

TypeScript와 CSS는 Vite가 다시 읽지만 Rust 변경은 위 빌드가 끝나야 브라우저에 반영된다.

## rhwp-studio

```bash
cd rhwp-studio
npm ci
npx vite --host 0.0.0.0 --port 7700
```

해당 포트가 이미 사용 중이면 기존 서버를 확인하거나 다른 포트를 지정한다. 브라우저 검증 절차는
[시각 검증 문서 지도](verification/README.md)와 각 E2E 가이드를 따른다.

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
