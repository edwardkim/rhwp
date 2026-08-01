---
kind: pr-review-implementation-plan
status: active
pr: 3690
issue: 3604
last_verified: 2026-08-01
---

# PR #3690 통합 계획

## 대상 commit

| 순서 | 원 commit | 역할 |
| --- | --- | --- |
| 1 | `2c1b2624d` | MCP 보호 문서 입력 |
| 2 | `517e015c2` | HWP5 보호 저장 공통화 |
| 3 | `2965350e2` | 세 형식 공통 암호 모듈 |
| 4 | `78e6440fc` | CLI/WASM/Studio 보호 저장 |
| 5 | `03fa0c64b` | Studio save-as 암호 설정 통합 |
| 6 | `0aea12bef` | 암호 HWPX 드롭 안정화 |
| 7 | `5e7ea9ddc` | Studio lockfile 동기화 |

## 단계

1. 완료: 최신 `upstream/devel` 위 rebase. 충돌 없음.
2. 완료: Rust 암호 계약 7개와 Studio 전체 719개 테스트, typecheck, production build를 확인.
3. 완료: remote의 `devel` merge commit을 보존하고, 중복 구현 patch를 건너뛴 archive review 기록을
   `task_m100_3604`에 fast-forward push했다. 최신 head 기준 CI를 관찰한다.
4. 대기: native Finder drag/drop 수동 확인. 암호 입력 dialog와 문서 canvas 표시, renderer 비종료,
   Ctrl+S save-as 전환을 확인한다.
5. 대기: 최신 head의 required CI 통과와 작업지시자 승인 후 merge 여부를 판정한다.

## 롤백 경계

- 문제가 암호 형식별 알고리즘에 있으면 `password_crypto.rs`와 해당 contract test만 되돌린다.
- Studio 드롭 안정성 문제는 File System Access handle 보존을 복원하지 않고 별도 안전한 저장 경로를
  설계한다. native IPC 재도입은 renderer 종료 재발 위험이 있다.
- merge 전에는 force-with-lease 이외의 remote branch 강제 변경을 하지 않는다.
