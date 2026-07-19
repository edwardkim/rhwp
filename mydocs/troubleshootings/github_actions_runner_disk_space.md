---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# GitHub Actions runner 디스크 부족 — `No space left on device` (Task #1109)

| 항목 | 내용 |
|------|------|
| 발견일 | 2026-05-24 |
| 이슈 | [#1109](https://github.com/edwardkim/rhwp/issues/1109) |
| 실패 run | https://github.com/edwardkim/rhwp/actions/runs/26359599960 |
| 영향 영역 | `.github/workflows/ci.yml` 의 `build-and-test` job |
| 해결 영역 | 메인테이너 레포 설정 변경 + workflow 정정 (다층 안전) |

## 증상

GitHub Actions CI 의 `build-and-test` job 의 `Run tests` step (`cargo test --verbose`) 에서:

```
##[error]No space left on device : '/home/runner/actions-runner/cached/2.334.0/_diag/pages/65f0451d-...log'
```

본 실패는 step 의 빌드 진행 중 cargo 작업이 아니라 runner 의 diag log 작성 영역에서 발생.

## 본질 분석

### 1. 실패 step 의 영역

`cargo test --verbose` 는 다음을 모두 컴파일:
- lib (`src/`) — base crate
- 모든 binary target (`src/bin/*`)
- 모든 integration test (`tests/*.rs`)
- **모든 `examples/*.rs`** — 개별 binary 컴파일

본 프로젝트의 `examples/` = 28 개 (진단/inspection/reproduce 도구 누적).
각 example 의 binary 가 `target/release/examples/` 에 별도 생성.

### 2. 누적 영역

| 영역 | 누적 크기 (~) |
|------|--------------|
| `actions/cache@v4` 의 `target/` cache | 4-8GB |
| `~/.cargo/registry` + `~/.cargo/git` | 1-2GB |
| 본 빌드 산출물 (target/debug + target/release) | 2-4GB |
| apt-get install (libfontconfig + libfreetype) | <100MB |
| runner 자체 toolchain + diag log | 매우 변동적 |

### 3. ubuntu-latest 디스크 한도

GitHub-hosted runner `ubuntu-latest`:
- 총 디스크 ~14GB
- 사전 설치 toolchain (rust 외):
  - `/usr/share/dotnet` ~2GB
  - `/usr/local/lib/android` ~14GB
  - `/opt/ghc` ~5GB
  - `/opt/hostedtoolcache/CodeQL` ~3GB
- 위 모두 미사용 (Rust 전용 프로젝트)

### 4. WASM 빌드 무관

`wasm-build` job 은 `workflow_dispatch || tag` 만 실행 — 본 PR 빌드는 영역 아님.
작업지시자 가설 ("WASM 이미지 누적") 은 본 실패 원인 아님.

## 해결 — 다층 안전망

### 해결 (1) — 메인테이너 레포 설정 (2026-05-24)

GitHub 레포지터리 Settings → Actions 영역:
- Actions cache 한도: **10GB → 15GB**
- Cache 지속 일: **7일 → 3일**

본 조치 후 장애 CI 재시작 → 통과.

### 해결 (2) — workflow Free disk space step (Task #1109)

`.github/workflows/ci.yml` 의 `build-and-test` + `wasm-build` 두 job 의 checkout 직후
미사용 사전 설치 toolchain 제거:

```yaml
- name: Free disk space (remove unused pre-installed toolchains)
  run: |
    sudo apt-get clean
    sudo rm -rf /usr/share/dotnet
    sudo rm -rf /usr/local/lib/android
    sudo rm -rf /opt/ghc
    sudo rm -rf /opt/hostedtoolcache/CodeQL
    sudo docker image prune --all --force || true
    df -h
```

기대 효과: ~24GB toolchain 회수.

위치: checkout 직후, Install Rust toolchain 직전:
- runner 의 cached/ 영역 보호 (실패 발생 영역)
- 후속 step 들이 여유 공간에서 동작
- `df -h` 출력으로 CI log 에 회수 효과 가시화

## 핵심 학습

### 1. CI 디스크 부족의 다중 영역

본 실패가 한 step 의 빌드 산출물 크기 단독이 아니라:
- runner 의 사전 설치 toolchain
- actions/cache 누적
- 본 빌드 산출물
- 런타임 diag log

→ **단일 해결책 부족**. 메인테이너 cache 한도 + workflow toolchain 정리 둘 다.

### 2. examples 누적의 영향

본 프로젝트의 `cargo test` 가 28 examples 컴파일 — 진단/reproduce 도구 누적이 빌드 시간 + 디스크 영향.

**선제 검토**: 새 example 추가 시 일회성/지속성 판단 + 분리 고려. 일회성은 별도 폴더 (`examples/oneshot/`) 또는 task 완료 후 제거.

### 3. WASM job 의 영역 분리 확인

작업지시자 가설 ("WASM 이미지 누적") 은 본 실패 원인 아님 — `wasm-build` job 은
`workflow_dispatch || tag` 만 실행되며 일반 PR 영역과 분리. CI 실패 본질 식별 시
workflow trigger 조건 확인 필수.

### 4. 다층 안전망

본 task 의 해결 패턴:
- 메인테이너 외부 조치 (레포 설정) — 즉시 해결
- workflow 정정 — 재발 대비 안전망

둘 다 적용 — `feedback_self_verification_not_hancom` 류 정합 (단일 해결 의존 회피).

## 관련 commits

Task #1109 (2026-05-24).

## 관련 메모리 룰

- `feedback_diagnosis_layer_attribution` — 실패 step + 누적 영역 본질 정확 식별
- `feedback_check_open_prs_first` — open PR 확인 (CI 인프라 영역과 별개 확인)
- `feedback_search_troubleshootings_first` — 본 문서 등록 후 재발 시 사전 검색 자료
