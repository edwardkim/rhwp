# #6950 Stage 3 — 영향 검증·PR 준비

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 시작: 2026-09-10 메인테이너 진행 승인.
- 선행: [Stage 2](task_m100_6950_stage2.md) §9의 시각 판정 통과·작은 용지 실험 범위 제외.
- Stage 2 확정 커밋: `390d81e74d77a1541ab838b204263a07a2d0a972`.
- 작업 브랜치: `task_m100_6950`.
- 상태: 사전 점검 완료. 최신 base 병합의 문서 충돌 처리 방침과 Docker 기동 대기.
  전체 검증·PR 준비 완료가 아니다. 원격 변경 없음.

## 1. 최신 base와 병합 사전 검사

`git fetch upstream devel`로 원격 기준을
`13c92feb67d2bf0ae62349f41c5f5cd83845a4a5`에서
`0d36da4096fab2fef0e0a654e466fa449330d7a6`으로 갱신했다.
당시 `HEAD...upstream/devel`은 작업 측 24 / 원격 측 44커밋 차이였다
(이후 Stage 2 문서 확정 커밋 1개 추가). 로컬 `devel`은 변경하지 않았다.

`git merge-tree --write-tree HEAD upstream/devel`은 작업트리를 바꾸지 않는 사전 검사다.
Stage 2 문서 확정 전 HEAD에서 다음 결과를 확인했다.

- 소스 `src/document_core/queries/rendering.rs`,
  `src/renderer/layout/paragraph_layout.rs`는 자동 병합. 의미상 회귀 없음의 증명은 아니다.
- 유일한 충돌: `mydocs/orders/20260910.md`의 add/add.
  로컬은 #6950 기록, 원격은 PR #6962 기록이다.
- 제안: 하나의 날짜 제목 아래 양쪽 이슈별 기록을 모두 보존한다. 원격의 과거 체크 상태를
  추정으로 변경하지 않는다. 충돌 해결 방침 승인 후 최신 base를 작업 브랜치에 병합하고 검증한다.
- `.github/workflows/ci.yml`은 현재 작업 HEAD와 원격 devel 사이에 차이가 없다.

실제 merge는 시작하지 않았고 review worktree도 이전 검증 상태로 유지한다.
`local_validation.md` §4.2에 따라 충돌 해결 방침은 메인테이너에게 확인한다.

## 2. 실행 환경

- Linux/WSL: 16 logical CPUs, 메모리 31GiB 중 available 약29GiB, 디스크 가용373GiB.
- `cargo-nextest 0.9.137` 확인. 실행 중 Cargo/Rust 작업은 점검 시 없었다.
- 기존 review worktree: `/home/edward/mygithub/rhwp-6950-review` (tracked clean).
- 고정 공유 target: `/home/edward/mygithub/rhwp-shared-review-target`. 새 target이나 worktree는
  만들지 않았고 기존 캐시·Studio 산출물은 삭제하지 않았다.
- `docker info`는 WSL 통합 CLI를 사용할 수 없다고 반환했다. `/var/run/docker.sock`과
  Docker Desktop의 WSL CLI 경로가 없다.
- 호스트 `docker.exe info`도 `dockerDesktopLinuxEngine` named pipe가 없어서 실패했다.
  PATH 문제만이 아니라 Linux engine 접속도 현재 불가능하다. Docker Desktop 기동 및
  해당 Ubuntu WSL 통합 확인이 필요하다. 대체 native WASM 빌드는 실행하지 않았다.

## 3. 선행 조건 충족 후 검증 순서

1. 승인된 문서 병합 방침으로 최신 devel 반영 → code head 고정 → 기존 review worktree 정합.
2. review 전용 파생 suite 준비 → fmt → native Clippy → WASM Clippy → workspace build →
   all-targets Clippy → manifest 및 source unit-tier 검사. Cargo는 공유 target에서 순차 실행한다.
3. 해당 code head의 focused 검사와 release-test 전체 nextest, Native Skia 3종.
   현재 자원 기준 Cargo build 2 jobs, nextest 8 threads로 시작하고 실행 시간을 기록한다.
4. 새 fixture `samples/hwpx/20260909-para-table.hwpx`에 대해 필수 코퍼스 래칫·명시적
   보안 검사 입력을 확인한다. PDF 쪽수 원장 등록 여부도 확인한다. 실패를 숨기기 위한
   baseline 갱신은 하지 않는다.
5. Docker 표준 WASM 빌드 및 필요한 실제 WASM 확인. 원본 시각 판정 범위를 유지하며
   작은 용지의 추가 구현·대체 실험은 하지 않는다.
6. 최종 보고서·PR 본문 초안과 정확한 결과/미검증 범위 제시. push·PR 생성은 별도 승인.

현재까지 새로 실행한 것은 Git/환경 사전 점검과 Stage 2 문서 확정이다.
최신 base와 합쳐질 소스가 달라지므로 이전 code head에서 긴 전체 검증을 먼저 반복하지 않는다.
