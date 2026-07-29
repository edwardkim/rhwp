---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-multirunner-fleet
description: "self-hosted 러너 실험 종료(2026-07-25) — 전 워크플로 GitHub 호스티드, 함정 카탈로그는 report 존치"
metadata: 
  node_type: memory
  type: project
---

self-hosted 러너 실험(#3284 전환 → #3289 20 인스턴스화, CI 9m55s 실측 달성)은 **2026-07-25 작업지시자 결정으로 종료**됐다. 결정 배경: 공유 홈(/home/app) 동시 쓰기 레이스 결함 연쇄(6계열 — nextest·rustup 설치, rust-cache 파괴, rustup settings.toml 파손, wasm-pack, actions/cache, apt 락), CPU 공유로 시간 상한 테스트 flake, CodeQL 무캡 JVM의 호스트 메모리 포화, GitHub Pro 전환(호스티드 동시 job 20→40).

**현재 상태**: 검증 워크플로 전부 GitHub 호스티드(#3284 이전으로 완전 복원, PR #3297). 러너 20개 등록 해제·서비스 제거·디렉터리 삭제 완료(LXC 163GB 회수). LXC(192.168.2.13, 70코어·160GiB)는 러너 없이 존치.

**유지된 개선**: timeout-minutes(전 잡), CLI 테스트 `rhwp_bin()` 규약(런타임 CARGO_BIN_EXE 우선 — 신규 CLI 테스트 필수 패턴), install-wasm-pack mktemp·원자 교체.

**Why:** 실험은 기술적으로 성공(9m55s < 호스티드 ~13m)했으나 공유 홈 구조의 운영 복잡도가 계속 새 결함을 만들었다.

**How to apply:** 재실험 논의가 나오면 `mydocs/report/task_m100_3289_report.md`(함정 카탈로그·거버넌스·전수 점검표)를 먼저 정독. 새 CLI 통합 테스트는 `env!("CARGO_BIN_EXE_rhwp")` 직접 사용 금지 → `rhwp_bin()` 패턴. 관련 [[project-branch-policy]]
