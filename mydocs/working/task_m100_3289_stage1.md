# Task #3289 Stage 1 — self-hosted 러너 20 인스턴스화 (수행계획서)

## 배경

#3284(#3286) 단일 러너 전환이 직렬 처리로 호스티드보다 느려짐(실측 ~13분 vs 24분+).
GitHub Free 동시 상한 20 job 에 맞춰 **20 러너**로 병렬성 회복. LXC 단일 노드에 러너
인스턴스 20개 등록, 공통 라벨 유지(워크플로 runs-on 무변경).

## 서비스 방식 확정 (작업지시자 결정)

**방식 A — systemd system 서비스**(현 러너 1개와 동일 방식). svc.sh install/start 로
20개 서비스 등록. 부팅 자동 시작, 일관성.

## 선행 조건 (작업지시자 직접 — 제가 SSH 로 불가)

1. **LXC 코어 56 → 70 확장** (Proxmox). 호스트 72 중 시스템 2 남김. 진행 예정.
2. **svc.sh install 용 sudo 확대**. 방식 A 는 각 러너의 systemd system 서비스
   등록에 sudo(systemctl, cp to /etc/systemd/system)가 필요한데, 현재 app 계정
   NOPASSWD 는 apt 한정이다. 두 안 중 택1:
   - (a) app 계정에 svc.sh install 이 쓰는 명령(systemctl, /etc/systemd/system 쓰기)
     NOPASSWD 확대 → 제가 20개 등록·서비스화 자동화.
   - (b) 러너 등록(config.sh)은 sudo 불필요하니 제가 20개 등록까지 하고,
     **svc install/start(sudo 부분)만 작업지시자가 스크립트 실행**.
   → 계획서 승인 시 (a)/(b) 확정.

## 구성 절차 (선행 완료 후)

1. **디렉터리 배치**: `~/actions-runner-01` ~ `~/actions-runner-20` (또는 현 러너를
   01 로 재편). 각 디렉터리에 러너 바이너리 복사(또는 tar 재전개).
2. **등록**: 각 인스턴스 `./config.sh --url https://github.com/edwardkim/rhwp
   --token <REG_TOKEN> --name runner-lxc-NN --labels self-hosted,Linux,X64
   --unattended --replace`. 등록 토큰은 `gh api -X POST
   .../actions/runners/registration-token` 로 발급(만료 짧음 — 인스턴스별 재발급).
3. **CARGO_BUILD_JOBS 주입**: 각 러너 `.env` 에 `CARGO_BUILD_JOBS=4` (코어 경쟁 억제,
   20 × 4 = 80 요청 vs 70코어 — 약간 오버서브스크립션이나 job 이 늘 다 병렬은 아님).
4. **서비스화**: 각 인스턴스 `sudo ./svc.sh install && sudo ./svc.sh start`
   (방식 A). 서비스명은 인스턴스별 고유(runner-lxc-NN).
5. **공유 자원**: 툴체인(rustup·node·chromium libs·apt)은 LXC 공유라 재설치 불필요.
   단, 20 러너가 같은 `~/.cargo`/`~/.rustup` 을 공유하면 빌드 캐시 경합 가능 →
   러너별 `CARGO_TARGET_DIR`/작업 디렉터리 분리는 actions-runner 가 `_work` 를
   인스턴스별로 갖도록 이미 격리됨(확인 필요).

## 리스크

- **오버서브스크립션**: 20러너 × CARGO_BUILD_JOBS=4 = 80 > 70코어. 모든 러너가 동시에
  빌드 피크를 치면 경쟁. 그러나 실제로는 job 이 빌드/테스트/대기 단계가 섞여 평균
  부하는 낮다. `CARGO_BUILD_JOBS=3`(60) 으로 보수적 시작도 가능 → 관측 후 조정.
- **디스크**: 20 러너 각 `_work` + target 캐시. 185GB 여유이나 20 × 수 GB 누적 감시.
- **~/.cargo 공유 레지스트리 락**: 여러 cargo 가 동시에 레지스트리 갱신 시 락 대기.
  보통 문제없으나 관측 대상.

## 검증

- 20 러너 online 확인(`gh api .../actions/runners`).
- 한 PR CI 의 8-shard + 독립 job 이 **동시 실행**(여러 러너 busy) 되는지 실측.
- 전체 CI 시간이 호스티드(~13분) 이하로 회복되는지 측정.
- load average(70코어 기준), 러너별 RSS, 디스크 관측.

## 다음 단계

승인 + 선행 2조건(LXC 70, sudo 방식 a/b) 확정 후 Stage 2(구현계획서) — 등록 자동화
스크립트, CARGO_BUILD_JOBS 값 확정, 서비스 템플릿.
