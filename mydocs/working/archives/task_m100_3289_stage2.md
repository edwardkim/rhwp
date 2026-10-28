# Task #3289 Stage 2 — 구현계획서

방식 A + sudo (a) 확정. 선행: LXC 70코어 확장 + NOPASSWD 확대(작업지시자).

## 선행: NOPASSWD 확대 목록 (작업지시자 — LXC 작업 시 함께)

svc.sh 는 내부에서 `systemctl`(daemon-reload/enable/start/stop/status) + `/etc/systemd/
system/<svc>` 쓰기를 한다. `sudo ./svc.sh install|start` 로 실행하므로, 최소 권한은:

```
# /etc/sudoers.d/app-runner
app ALL=(ALL) NOPASSWD: /usr/bin/systemctl, /home/app/actions-runner-*/svc.sh
```

(기존 apt NOPASSWD 는 유지. systemctl 전역 허용이 넓다면, svc.sh 래퍼만 허용하는
방식으로 좁힐 수 있으나 svc.sh 가 여러 디렉터리에 복제되므로 와일드카드 필요.)

## 구현 절차 (제가 SSH 자동화)

### 1. 깨끗한 러너 바이너리 확보 (6.1GB 복제 대신)
현 `~/actions-runner` 는 `_work` 빌드 캐시 포함 6.1GB. 복제하면 낭비 + 오염.
→ GitHub 러너 릴리스 tar 를 1회 받아 `~/runner-pkg/` 에 전개, 이걸 20개로 배포.
   (현 러너 버전과 동일 버전 tar. 버전은 `~/actions-runner/bin/Runner.Listener
   --version` 으로 확인.)

### 2. 20 인스턴스 배치
- 현 러너(`runner-lxc`)는 **그대로 두거나 runner-lxc-01 로 재편**. 재편 시 기존
  서비스 stop/uninstall 후 재등록(라벨·이름 정합). → **신규 20개를 01~20 으로 새로
  만들고 기존 단일 러너는 제거**하는 편이 깔끔(이름 충돌 회피).
- `~/actions-runner-01` … `~/actions-runner-20`, 각 디렉터리에 pkg 전개.

### 3. 등록 (per-instance token)
```bash
for i in $(seq -w 1 20); do
  TOKEN=$(gh api -X POST repos/edwardkim/rhwp/actions/runners/registration-token -q .token)  # 인스턴스별 재발급
  cd ~/actions-runner-$i
  ./config.sh --url https://github.com/edwardkim/rhwp --token "$TOKEN" \
    --name runner-lxc-$i --labels self-hosted,Linux,X64 --unattended --replace
  # CARGO_BUILD_JOBS 주입
  echo "CARGO_BUILD_JOBS=4" >> .env
  # node PATH (기존 러너의 .path 와 동일하게)
  cp ~/actions-runner/.path .path 2>/dev/null || true
  sudo ./svc.sh install app && sudo ./svc.sh start
done
```
(토큰은 발급 API 를 SSH 세션이 아니라 로컬에서 호출해 넘기거나, 러너에 gh 설치.
 → 로컬에서 토큰 20개 발급해 배열로 전달하는 방식이 안전.)

### 4. CARGO_BUILD_JOBS
- 초기값 **3** 으로 보수적 시작(20×3=60 < 70코어). 관측 후 4 로 상향 판단.
  (오버서브스크립션 리스크 최소화 — 계획서 리스크 항목 반영.)

### 5. 기존 단일 러너 처리
- `runner-lxc`(현재 #3286 실행 중) — **진행 중 job 완료 후** stop → svc uninstall →
  config remove(GitHub 등록 해제) → 20개 신규로 대체. (실행 중 제거 금지.)

## 디스크 관리

- 러너 pkg ~200MB × 20 + 각 `_work` 빌드 캐시. 초기엔 여유(185GB)이나 20 러너가
  각자 target/ 을 쌓으면 누적. → `~/.cargo` 는 공유(레지스트리 1벌), `_work` 만
  인스턴스별. 주기적 `_work` 정리 또는 `rust-cache` 로 관리. 관측 대상.

## 검증

1. 20 러너 online: `gh api repos/edwardkim/rhwp/actions/runners --jq '.total_count'` = 20.
2. 한 PR CI 재트리거 → 8-shard + 독립 job 이 **여러 러너에 동시 배정**(busy 다수) 실측.
3. 전체 CI 시간 호스티드(~13분) 이하 회복 측정.
4. load average(70코어), 러너별 RSS, 디스크 관측. 코어 경쟁 시 CARGO_BUILD_JOBS 하향.

## PR

- 이 작업은 **러너 인프라 구성(SSH)** 이라 저장소 코드 변경이 없다. #3286(전환 워크플로)
  이 이미 self-hosted 를 참조하므로, 20 러너 구성 후 #3286 재검증 → green 이면 merge.
- working/report 문서만 저장소에 남긴다(인프라 작업 기록). `Closes #3289`.
