# Task #3284 — CI 검증 워크플로 self-hosted 러너 전환 (최종 보고서)

`Closes #3284`.

## 배경

GitHub 호스티드 큐 적체로 CI 병목. 사내 self-hosted 러너 `runner-lxc`(192.168.2.13,
Proxmox LXC) 구성·검증 후 검증 워크플로를 전환했다.

## 러너 사양·검증 (SSH 실측)

- Xeon E5-2697 v4 **56 논리코어** / **100 GiB RAM** / 185 GB 여유 / Ubuntu 24.10 / podman 5.0.3
- 설치: build-essential·pkg-config·libssl-dev·unzip + Native Skia(libfontconfig1-dev·
  libfreetype-dev·fonts-dejavu-core). app 계정 apt NOPASSWD sudo.
- 실증: rustup 자체 설치(rustc 1.93.1 정합), rhwp clean 빌드 **1m28s**(로컬 WSL2 2m12s보다
  빠름), 최대 RSS 2.77 GB, 외부망(static.rust-lang.org·github.com 200) 도달.
- 라벨: `self-hosted, Linux, X64, podman`.

## 변경 (검증 워크플로 4개, 13 job)

`runs-on: ubuntu-latest` → `runs-on: [self-hosted, Linux, X64]` + `timeout-minutes` 명시.

| 파일 | job(timeout분) |
|---|---|
| `ci.yml` | preflight(10)·lint(20)·frontend-package-gates(20)·build-test-archive(30)·native-skia-tests(30)·test-shard(30)·build-and-test(45)·wasm-build(30) |
| `codeql.yml` | preflight(10)·analyze(45) |
| `render-diff.yml` | preflight(10)·canvas-visual-diff(45) |
| `full-renderer-sweep.yml` | full-renderer-sweep(60) |

`podman` 라벨은 요구하지 않는다(호스트 직접 실행). timeout 은 로컬 실측 대비 2~3배 여유로,
러너 hang 을 실패로 드러내는 상한이지 성능 목표가 아니다.

## 제외 (무수정)

- `deploy-pages.yml`·`npm-publish.yml`·`close-issues-on-devel-push.yml`.
- `release-binary.yml` — matrix.runner(ubuntu-latest)가 macos-14/windows-latest 와 같은
  매트릭스라 건드리면 크로스플랫폼 릴리스가 깨진다. 릴리스 게시 job 도 호스티드 유지.

## 방침·제약 (작업지시자 결정)

- **단일 러너**(멀티 러너는 후속) / **폴백 미구현**(needs 그래프 무변경).
- 인지된 제약: 러너 1 인스턴스 = job 직렬. test-shard 8 은 순차 실행되어 호스티드 병렬보다
  벽시계는 느릴 수 있다. timeout 으로 무한 대기(24h 큐잉)는 방지하나 폴백은 없어 러너 장애
  시 CI 전면 중단 — 발생 시 대응. 멀티 러너·폴백은 실측 후 별도 이슈 후보.

## 검증

- actionlint 1.7.12: 4파일 오류·경고 **0**.
- YAML 파싱(python yaml.safe_load) 4파일 정합, runs-on 배열·timeout-minutes 값 확인.
- diff 정확성: 대상 4파일만 변경, release-binary/deploy/npm/close 무변경.
- 실 검증: 이 전환 PR 의 CI 가 self-hosted 러너에서 실제로 도는 것으로 확인(PR 브랜치
  워크플로가 즉시 적용됨).

## 후속 후보

- 멀티 러너(단일 LXC 다중 인스턴스 + `CARGO_BUILD_JOBS` 제한) — 8-shard 병렬성 회복.
- self-hosted 우선·hosted 폴백(preflight API 감지) — 단일점 장애 대응.
