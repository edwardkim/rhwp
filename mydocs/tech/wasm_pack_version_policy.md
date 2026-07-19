---
kind: decision
status: active
canonical: mydocs/tech/wasm_pack_version_policy.md
last_verified: 2026-07-19
---

# wasm-pack 버전 고정 정책 (#2233)

## 배경

GitHub Actions 의 `curl | sh` installer(`init.sh`)는 스크립트에 **하드코딩된 버전**
(관측 시점 0.13.1)을 설치하며 pin 을 지원하지 않는다. WASM 전용 Docker 는
`cargo install wasm-pack@0.15.0` 으로 0.15.0 을 쓴다. 두 경로의 버전이 달라
runner/이미지 시점에 따라 toolchain 이 갈릴 수 있었다(PR #2216 review 지적).

## 정책 — 단일 버전 0.15.0

Actions·Docker 모두 **wasm-pack 0.15.0** 으로 고정한다.

| 경로 | 설치 방식 | 버전 지정 위치 |
|------|----------|---------------|
| GitHub Actions | prebuilt 릴리스 바이너리(빠름, 컴파일 없음) | `.github/actions/install-wasm-pack/action.yml` 의 `WASM_PACK_VERSION` |
| WASM Docker | `cargo install wasm-pack@<ver>` | `Dockerfile`의 `cargo install wasm-pack@<ver>` 명령 |

Actions 는 composite action `./.github/actions/install-wasm-pack` 하나를 6개
워크플로우(ci·deploy-pages·full-renderer-sweep·npm-publish·render-diff)가 참조하므로
**버전 변경 지점은 composite action 1곳 + Dockerfile 1곳** 뿐이다.

## 버전 갱신 절차

1. `.github/actions/install-wasm-pack/action.yml` 의 `WASM_PACK_VERSION` 갱신.
2. `Dockerfile` 의 `cargo install wasm-pack@<ver>` 를 같은 버전으로 갱신.
3. 로그로 확인: 각 워크플로우 "Install wasm-pack" 스텝 끝 `wasm-pack --version`,
   Docker 는 빌드 로그. 두 경로가 같은 버전을 찍는지 확인.
4. dev/release WASM 빌드 + `wasm-opt` + frontend consumer gate + Render Diff gate 통과 확인.

## 왜 Actions 는 prebuilt 바이너리인가

`init.sh` 는 pin 불가, `cargo install` 은 컴파일 비용(수 분)이 있다. 릴리스 prebuilt
바이너리(`wasm-pack-v<ver>-x86_64-unknown-linux-musl.tar.gz`)를 직접 받으면 pin +
저비용을 동시에 만족한다. Docker 는 기존 `cargo install` 유지(이미지 빌드는 캐시됨).
