---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: reference-docker-wsl-recovery
description: "Docker Desktop WSL stale mount 반복 장애의 증상 판별과 복구 절차 (로컬 환경 국한, GitHub 이슈 등록 금지)"
metadata: 
  node_type: memory
  type: reference
---

**증상 2형** (rhwp WASM 빌드 시 반복 발생, 2026-07-17 하루 3회):
1. `docker` 실행 시 "The command 'docker' could not be found in this WSL 2 distro" + `wsl -l -v`에서 docker-desktop 배포판 Stopped — Docker Desktop 자체가 내려간 상태
2. `/usr/bin/docker: Input/output error` — **stale mount** (프록시 바이너리 마운트가 죽음). exit code 0으로 성공처럼 보일 수 있어 출력 내용 확인 필수

**복구 절차** (WSL 안에서 실행 가능):
```bash
/mnt/c/Windows/System32/taskkill.exe /F /IM "Docker Desktop.exe"
/mnt/c/Windows/System32/taskkill.exe /F /IM "com.docker.backend.exe"
wsl.exe --terminate docker-desktop
cd /mnt/c && /mnt/c/Windows/System32/cmd.exe /c start "" "C:\Program Files\Docker\Docker\Docker Desktop.exe"
# 이후 docker version 폴링 (보통 재시작 후 ~10초 내 복구, 최초 기동은 수 분)
```

**금지**: `wsl --shutdown` — 현재 WSL 세션(작업 중인 Ubuntu)까지 죽인다 (세션 자살).

**근본 원인 (2026-07-17 해결)**: Docker Desktop **Resource Saver** (`UseResourceSaver=True`, `AutoPauseTimeoutSeconds=300`) — 유휴 5분 후 VM 일시정지가 WSL2 mirrored 환경에서 재개 실패 → stale mount / 배포판 Stopped. `settings-store.json`(경로: `/mnt/c/Users/edward/AppData/Roaming/Docker/settings-store.json`)에서 `UseResourceSaver=False`로 영구 정정 완료 (백업: `settings-store.json.bak-20260717`). 이후 재발 시 이 설정이 되돌아갔는지부터 확인.

**Why**: 작업지시자가 이 장애는 로컬 환경 국한이라 GitHub 이슈 등록이 불필요하다고 결정 (2026-07-17). 저장소 troubleshootings 문서에도 넣지 않는다.

**How to apply**: WASM 빌드 실패 시 즉시 `docker version` 출력 내용으로 2형 중 어느 쪽인지 판별 → 위 절차 → 폴링 → 빌드 재시도. 관련: [[user-role-identity]] (Windows + WSL2 환경).
