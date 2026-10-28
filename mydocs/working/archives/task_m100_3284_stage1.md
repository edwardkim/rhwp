# Task #3284 Stage 1 — CI 검증 워크플로 self-hosted 전환 (수행계획서)

## 배경

GitHub 호스티드 큐 적체로 CI 병목. self-hosted 러너 `runner-lxc`(192.168.2.13,
56코어/100GB, 라벨 `self-hosted, Linux, X64, podman`) 구성·검증 완료(clean 빌드 1m28s).
"오늘은 CI 쪽 모두를 외부 러너로 전환" 지시에 따라, **검증 워크플로 전체**를 옮긴다.

## 대상 확정 (push/pull_request 검증 워크플로)

| 워크플로 | job 수 | runs-on 위치(행) |
|---|---|---|
| `ci.yml` | 8 | 56, 413, 512, 588, 621, 693, 791, 908 |
| `codeql.yml` | 2 | 48, 250 |
| `render-diff.yml` | 2 | 64, 302 |
| `full-renderer-sweep.yml` | 1 | 21 |

→ 총 **13 job**, 전부 `runs-on: ubuntu-latest` → `runs-on: [self-hosted, Linux, X64]`.

## 제외 (배포/퍼블리시 — 호스티드 유지)

- `deploy-pages.yml`(2) — GitHub Pages 배포
- `npm-publish.yml`(4) — 레지스트리 퍼블리시
- `release-binary.yml` — Linux 1 + **macos-14×2 + windows-latest×1 매트릭스**(크로스 플랫폼,
  self-hosted 불가)
- `close-issues-on-devel-push.yml`(1) — 경량 API job(전환 이득 없음)

제외 근거: 배포·릴리스는 시크릿·플랫폼 매트릭스가 걸려 있어 단일 Linux self-hosted로
못 옮기거나 옮기면 안 된다. 검증(CI)만 전환해도 병목의 대부분을 해소한다.

## 전환 방식

1. 13개 `runs-on: ubuntu-latest` → `runs-on: [self-hosted, Linux, X64]` 치환.
   - 라벨 배열 매칭 — 셋 모두 만족하는 러너에 배정. `podman` 라벨은 요구하지 않는다
     (컨테이너 실행이 아니라 호스트 직접 실행이므로).
2. **안정성 가드**: 러너가 단일 대수라 다운 시 job 이 무한 대기한다. 각 job 에
   `timeout-minutes` 를 명시(현재 전무) — 빌드/테스트 job 은 30~45, 경량 job 은 15.
   러너 장애 시 무한 대기 대신 타임아웃 실패로 드러나게 한다.
3. 도구 설치 스텝 무수정 — dtolnay/rust-toolchain(rustup 자체 설치)·setup-node·
   `sudo apt-get`(NOPASSWD 준비됨) 모두 러너에서 동작 실증됨.

## 리스크 및 대응

- **단일 러너 장애 → 전 CI 정지**: 검증 워크플로 전부를 한 러너에 몰면 그 러너가 죽을 때
  PR 검증이 전면 중단된다. timeout-minutes 로 무한 대기는 막지만, 근본 대응은 러너 증설
  또는 "self-hosted 우선, 실패 시 호스티드 폴백" 구성이다. → **초기엔 timeout 가드만 두고,
  1주 관측 후 폴백/증설을 별도 이슈로 판단**(오늘 범위는 전환까지).
- **동시성**: 13 job 이 단일 러너에 몰리면 직렬화된다. 러너의 job 동시 실행 수
  (runner 설정 `--runnergroup`/병렬)를 확인해야 하나, 56코어라 여러 job 병렬 수용 가능.
  → 관측 대상, 오늘은 전환 후 실측.
- **캐시**: Swatinem/rust-cache 는 self-hosted 에서 로컬 디스크에 남아 오히려 빠르다
  (185G 여유). 부작용 없음.

## 검증 (전환 후)

- 워크플로 YAML lint(actionlint) + 파싱.
- 전환 커밋을 PR 로 올려 **실제로 self-hosted 러너에서 도는지** + 시간 단축 확인.
- 8-shard 병렬이 OOM 없이 도는지 실측(100GB 여유라 낙관).

## 다음 단계

승인 시 Stage 2(구현계획서) — 파일별 정확한 치환 목록 + timeout 값 확정 후 구현.
