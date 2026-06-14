# 개발 환경 가이드: 사무실 PC vs 홈 PC

## 1. 환경 비교

| 항목 | 사무실 PC | 홈 PC |
|------|-----------|-------|
| **OS** | Linux (Ubuntu native) | Windows 11 + WSL2 |
| **WSL 호스트명** | - | yarehang |
| **사용자** | app | app |
| **Rust** | 1.93.0 | 1.93.1 |
| **Node.js** | nvm (v24+) | nvm v24.11.0 |
| **Docker** | 네이티브 Docker | Docker Desktop (Windows) |
| **wasm-pack** | 1.93.0 기준 빌드 | 0.14.0 |
| **Git 기본 브랜치** | `devel` / `local/taskXXX` | `home` |
| **GitLab** | gitlab.opxhome.com | gitlab.opxhome.com (동일) |
| **원격 서버 접근** | 192.168.2.154, 192.168.2.19 | 동일 (같은 네트워크) |

---

## 2. 네트워크 서버 구성

두 PC 모두 아래 서버에 접근 가능하다.

| 서버 | IP | 호스트명 | 용도 |
|------|-----|---------|------|
| 원격 도커 서버 | 192.168.2.154 | d7910 | Rust 빌드 보조, Docker |
| GPU 서버 | 192.168.2.19 | ollama | AI/ML (RTX 3090 × 2) |

### SSH 접속

```bash
# 원격 도커 서버
ssh -i ~/.ssh/gpu_key app@192.168.2.154

# GPU 서버
ssh -i ~/.ssh/gpu_key app@192.168.2.19
```

SSH 키 위치: `~/.ssh/gpu_key` (양쪽 PC 동일)

---

## 3. Git 브랜치 전략

### 사무실 PC

```
main ← devel ← local/taskXXX
```

- 타스크 작업: `devel`에서 `local/taskXXX` 브랜치 생성
- 타스크 완료: `local/taskXXX` → `devel` merge
- `main` merge: 작업지시자 요청 시만 수행

### 홈 PC

- **`home` 브랜치**를 기준으로 운영
- 사무실에서 작업한 코드를 이어서 작업할 경우:

```bash
# GitLab에서 최신 코드 가져오기
git fetch origin
git merge origin/devel   # 또는 origin/main
```

- 홈에서 완료한 작업을 사무실에 반영할 경우:

```bash
# home 브랜치 push 후 GitLab에서 devel로 merge
git push origin home
```

---

## 4. 빌드 명령

사무실/홈 모두 **동일한 명령**을 사용한다.

### 네이티브 빌드/테스트 (로컬 cargo)

```bash
cargo build          # 빌드
cargo test           # 일반 개발 테스트
cargo build --release
```

### macOS 로컬 빌드/테스트 검증

macOS(aarch64-apple-darwin)에서는 `tests/*.rs` 각 파일이 별도 통합 테스트
바이너리로 링크된다. `[profile.release]` 는 산출물 크기 최적화를 위해
`lto = true` / `codegen-units = 1` 을 사용하므로, `cargo test --release --tests`
는 통합 테스트 바이너리별 LTO 링크 때문에 매우 오래 걸릴 수 있다.

macOS 로컬 push/머지 전 검증은 release 산출물 프로필을 건드리지 않고, 통합 테스트만
별도 `release-test` 프로필로 실행한다.

```bash
cargo build --release                    # 산출물 release 빌드
cargo test --release --lib               # release 프로필 unit 회귀
cargo test --profile release-test --tests # 통합 테스트 회귀(macOS 권장)
cargo fmt --check
```

2026-06-09 macOS 실측 기준:

| 명령 | 용도 | real |
|------|------|------|
| `cargo test --release --tests --no-run` | release LTO 통합 테스트 빌드/링크 | 886.75s |
| `cargo test --profile release-test --tests --no-run` | LTO 없는 별도 테스트 프로필 cold build | 149.49s |
| `cargo test --profile release-test --tests -q` | 캐시 후 통합 테스트 실행 포함 | 65.02s |

`release-test` 는 로컬 검증 속도를 위한 프로필이다. 릴리즈 산출물 자체는
계속 `[profile.release]` 의 LTO 설정으로 빌드한다.

### Linux(WSL2) 실측 — release-test 프로필 권장

2026-06-12 WSL2(홈 PC) 실측 기준, Linux에서도 push/머지 전 통합 테스트 검증은
`release-test` 프로필이 빠르다.

| 명령 | 용도 | real |
|------|------|------|
| `cargo test --tests` (dev, warm) | 종전 push 전 검증 방식 | 262s |
| `cargo test --profile release-test --tests -q` (cold) | target/release-test 삭제 후 최초 빌드 포함 | 131s |
| `cargo test --profile release-test --tests -q` (warm) | 캐시 후 반복 실행 | 108s |

warm 기준 약 2.4배 빠르며, cold(최초 1회)조차 dev warm의 절반이다. 이후 push/머지 전
검증은 Linux에서도 `cargo test --profile release-test --tests` + `cargo fmt --check`
조합을 사용한다.

### WASM 빌드 (Docker)

```bash
docker compose --env-file .env.docker run --rm wasm
# 출력: pkg/rhwp_bg.wasm, pkg/rhwp.js
```

### rhwp-studio 개발 서버

```bash
cd rhwp-studio
npx vite
# http://localhost:7700
```

---

## 5. 홈 PC 주의사항

### Docker Desktop 특이점

- WSL2 환경에서 Docker Desktop을 사용하므로, Docker 명령은 Windows Docker Desktop이 실행 중이어야 동작한다.
- Docker Desktop이 꺼져 있으면 `docker: Cannot connect to the Docker daemon` 오류 발생 → Windows 트레이에서 Docker Desktop 실행 후 재시도.

### PATH 설정

홈 PC의 `~/.bashrc` 말미에 다음이 등록되어 있다:

```bash
export PATH=/home/app/vips/bin:$PATH
export LD_LIBRARY_PATH=/home/app/vips/lib/x86_64-linux-gnu
export GEMINI_API_KEY="..."
export GOOGLE_API_KEY="..."
export NVM_DIR="$HOME/.nvm"
. "$HOME/.cargo/env"
```

새 터미널을 열면 자동 적용된다.

### 필수 파일 (gitignore 대상)

아래 파일들은 git에 포함되지 않으므로 클론 후 별도로 준비해야 한다.

| 파일/폴더 | 설명 | 준비 방법 |
|-----------|------|-----------|
| `saved/blank2010.hwp` | 새 문서 생성용 템플릿 | 별도 복사 |
| `pkg/` | WASM 빌드 결과물 | `docker compose ... run --rm wasm` |
| `rhwp-studio/node_modules/` | npm 패키지 | `npm install` (또는 `npx vite`가 자동 처리) |
| `~/.ssh/gpu_key` | 원격 서버 SSH 키 | `.env` 파일의 키를 복사 후 `chmod 600` |

---

## 6. 사무실 → 홈 작업 전환 체크리스트

```
□ GitLab에 사무실 작업 push 완료
□ 홈 PC에서 git fetch && git merge
□ saved/blank2010.hwp 존재 확인
□ WASM 빌드 실행 (소스 변경 시)
□ cargo test 통과 확인
□ Docker Desktop 실행 중 확인
```

## 7. 홈 → 사무실 작업 전환 체크리스트

```
□ 홈 PC에서 작업 커밋 & push (origin/home)
□ 사무실 PC에서 git fetch
□ home 브랜치 내용을 devel에 반영
```

---

*최초 작성: 2026-02-28*
