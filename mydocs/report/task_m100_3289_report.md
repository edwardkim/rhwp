# Task #3289 — self-hosted 러너 20 인스턴스화 + 병렬성 회복 (최종 보고서)

`Closes #3289`. #3284(단일 러너 전환)의 후속으로, 러너 1개 직렬화 제약을 20 인스턴스로
풀고 CI 벽시계를 호스티드 이하로 회복했다.

## 구성 결과

- `runner-lxc-01`~`20` (app@192.168.2.13, LXC 70코어 확장분) 등록·systemd 서비스화, 전부 online.
- 기존 단일 `runner-lxc`는 GitHub 등록 해제·서비스 제거 완료 — 플릿은 01~20 만 남는다.
- 러너 인스턴스별 디렉터리(`~/actions-runner-NN`)는 분리, **홈(`/home/app`)은 공유**:
  `~/.cargo`·`~/.rustup`·nvm node 는 호스트에 1벌만 존재한다.

## 시간 실측 (PR #3286 브랜치, 클린 전주 기준)

| 단계 | 호스티드(devel 실측) | 1차 (22m38s) | 최종 (9m55s) |
|---|---|---|---|
| Lint | 1m34s | 3m21s | 2m25s |
| Build test archive | 7m47s | 10m56s | **4m38s** |
| 8-shard 단계 | 2m38s | 7m41s (다운로드 ~6m) | **1m58s** (캐시 8/8 hit) |
| **전체** | 12m22s~13m51s | 22m38s | **9m55s** |

- 8-shard 는 8개 러너에 1:1 동시 배정(동일 초 시작) — #3284 의 직렬화 제약 해소 실측.
- run 30144809772: CI·Render Diff·CodeQL 전부 success.

## 결함 4종 — 전부 "공유 홈 + 러너별 경로" 구조에서 나온 것

1. **`.path` 2줄 → node 미발견**: 러너는 `.path` 를 PATH 문자열 하나로 읽는다. 2줄이면
   첫 콜론 토큰이 `"...bin\n/usr/local/sbin"` 으로 깨져 **첫 줄에만 있는 항목만** 죽는다
   (시스템 경로는 뒤 토큰으로 생존 → git 은 되고 node 만 안 되는 증상). 한 줄(콜론 결합)로
   수정. **`.path` 는 반드시 한 줄**.
2. **cargo-nextest 동시 설치 레이스**: 8-shard 가 동시에 taiki-e/install-action 으로 공유
   `~/.cargo/bin` 에 설치하다 mv 충돌. → 설치 스텝을 `runner.environment == 'github-hosted'`
   로 게이트, self-hosted 는 호스트 상주 설치본 사용.
3. **rustup 재설치 레이스**: `.path` 에 `~/.cargo/bin` 이 없어 rust-toolchain 액션의
   `command -v rustup` 이 매번 실패 → **매 잡마다 rustup 인스톨러 재실행**. 단일 러너에선
   무해했으나 동시 8잡이 공유 rustup 바이너리를 교체하다 127. → `.path` 에
   `/home/app/.cargo/bin` 추가로 인스톨러 경로 자체 제거.
4. **`env!("CARGO_BIN_EXE_rhwp")` 빌더 경로 고착**: 컴파일타임 절대경로가 아카이브 빌더
   러너의 `_work` 를 가리켜, 다른 러너의 shard 에서 NotFound. 호스티드가 green 이었던 건
   전 VM 경로 동일이라는 우연. 1차 run 의 green 도 빌더 러너 `_work` 가 같은 호스트에
   남아 있던 시한부 운(다음 checkout 의 `git clean -ffdx` 가 지우면 레이스). →
   **런타임 `CARGO_BIN_EXE_rhwp`(nextest 가 shard 로컬 추출 경로로 재매핑) 우선 + 컴파일타임
   폴백** 헬퍼 `rhwp_bin()` 으로 12파일 25곳 교체. 로컬 프로브로 런타임 주입 검증,
   release-test 4003/4003 통과.

## 성능 최적화 2종

- **아카이브 호스트 캐시 공유**: 1GB 아카이브를 8-shard 가 한 호스트 회선으로 동시
  다운로드하며 ~6분 대기하던 것을, build 잡이 `$HOME/artifact-cache/$RUN_ID/` 에 원자적
  (tmp+mv) 복사 → shard 는 hardlink 우선·artifact 다운로드 폴백. 24h 지난 캐시는 build
  잡이 정리. 업로드 아티팩트는 폴백·provenance 로 유지.
- **critical path 만 `CARGO_BUILD_JOBS` 상향**: 러너 `.env` 는 3(20러너 동시 폭주 대비
  보수값) 유지, lint·native-skia·build-test-archive 잡만 워크플로 `env:` 로 12 오버라이드.
  70코어에 3잡×3코어=9코어만 쓰던 낭비 해소 (archive 빌드 9m11s→3m27s).

## 멀티러너 거버넌스 (운영 규칙)

- **공유 홈 원칙**: 잡 단위 툴 설치 액션 금지(레이스). 툴은 호스트에 1회 설치가 정본.
  현재 상주: rustup/cargo(1.93.1), cargo-nextest 0.9.140, node v24.18.0(nvm).
- **공유 홈을 변형하는 캐시 액션 금지**: Swatinem/rust-cache 는 save 전 정리로 공유
  `~/.cargo/bin` 의 상주 툴(cargo-nextest)을 삭제하고 registry 를 prune 한다 — devel push
  첫 save 에서 실측(run 30145509799, "Cleaning cargo/bin"). rust-cache 는
  `runner.environment == 'github-hosted'` 게이트 필수. 캐시 저장 쿼터도 read-only 상태라
  self-hosted 에선 이득이 없다.
- **RAM 자동 감지 도구는 캡 필수**: CodeQL 은 가용 RAM 대부분을 힙으로 잡아, 분석 2개
  동시 실행 시 JVM 각각 40~55GB → 호스트 100GB 포화·스톨 실측(2026-07-25, swap 512M).
  호스티드에서 무해했던 건 16GB VM 이라 힙이 작게 잡혔기 때문. codeql-action init 에
  `ram: 12288`/`threads: 16` 캡. 같은 유형(가용 자원 전체를 잡는 도구) 도입 시 동일 점검.
  (이후 #3290 으로 CodeQL·Render Diff 가 호스티드 복귀 — self-hosted 재배치 시 캡 필수.)

## 동시 쓰기 레이스 전수 점검 (2026-07-25, 작업지시자 지시)

self-hosted 워크플로(ci.yml·full-renderer-sweep.yml)의 공유 상태 쓰기 경로 전수 점검 결과.

**정정한 잡 단위 쓰기 (전부 hosted 게이트 또는 무해화):**

| 쓰기 경로 | 결함 | 정정 |
|---|---|---|
| dtolnay rust-toolchain 7곳 | `rustup default` 가 매 잡 `~/.rustup/settings.toml` 재작성 → 동시 잡에 파일 파손 실측 | hosted 게이트, self-hosted 는 rust-toolchain.toml override |
| taiki-e nextest 2곳 | 공유 `~/.cargo/bin` 동시 mv 교체 실측 | hosted 게이트 + 호스트 상주 설치 |
| Swatinem/rust-cache 3곳 | save 전 정리가 공유 bin·registry 파괴 실측 | hosted 게이트 |
| install-wasm-pack composite | 고정 /tmp 전개 충돌 + 공유 bin 무조건 mv | 핀 버전 존재 시 skip + mktemp + 동일 디렉터리 rename(원자 교체) |
| actions/cache 3곳 (frontend·release wasm·sweep) | restore 의 tar 전개가 공유 `~/.cargo/registry` 를 락 없이 덮어씀 | hosted 게이트 |
| apt-get 2곳 (native-skia·sweep 폰트) | 동시 잡 dpkg 락 경합 | hosted 게이트, 패키지는 호스트 선설치(sweep 폰트 포함, 2026-07-25) |

**점검 후 안전 판정 (설계상 동시성 안전):**

- cargo 자체의 `~/.cargo/registry`·`git` 접근 — cargo 내장 flock 으로 직렬화
- npm 의 `~/.npm` — cacache 가 동시 접근 안전 설계
- `artifact-cache` 호스트 캐시 — tmp+mv 원자 쓰기, prune 은 24h 경과 dir 만 + 오류 무시
- setup-node 툴 캐시 — 러너별 `_work/_tool` (공유 아님)
- `GITHUB_OUTPUT`/`GITHUB_PATH`/`GITHUB_ENV` — 잡별 임시 파일
- rustup 툴체인 자동 설치 — 평상시 no-op, 버전 범프 시 호스트 선설치 절차로 회피(위 규칙)
- **Rust 툴체인 버전 범프 절차**: CI 의 toolchain 값을 올리기 전에 호스트에서
  `rustup toolchain install <ver>` 선행 — 동시 잡의 자동 설치 레이스 방지.
- **러너 설정 파일**: `.path` 는 한 줄 콜론 결합(nvm bin + 시스템 + `/home/app/.cargo/bin`),
  `.env` 는 `LANG` + `CARGO_BUILD_JOBS=3`. 변경 후 `systemctl restart
  actions.runner.edwardkim-rhwp.runner-lxc-NN.service` 필요(20개 일괄).
- **CLI 테스트 규약**: 새 CLI 통합 테스트는 `env!("CARGO_BIN_EXE_rhwp")` 직접 사용 금지,
  각 파일의 `rhwp_bin()` 패턴(런타임 env 우선)을 따른다.
- **무해 경고**: 취소된 잡이 `_work` 를 어중간하게 남기면 다음 checkout 이
  "Unable to clean → repository will be recreated" 로 자가 복구한다(재클론 ~30s). 조치 불요.
- **네트워크**: 20 러너가 회선 1개 공유 — 대용량 아티팩트를 다수 잡이 동시 소비하는 설계는
  호스트 캐시 공유를 먼저 검토한다.

## 후속 후보

- **CodeQL·Render Diff 호스티드 재배치**(CI만 self-hosted 유지) — 호스트 동시 부하 분산
  목적. 두 워크플로 모두 `runner.environment` 분기로 이식성 확보 상태라 `runs-on` 만
  되돌리면 된다. 작업지시자 결정: 후속 이슈로 분리(2026-07-25).
- self-hosted 우선·hosted 폴백(러너 플릿 전면 장애 대응) — #3284 보고서에서 이월.
- `_work`·`artifact-cache` 디스크 누적 관측(20 인스턴스 × target/). 현재 여유 185GB.
