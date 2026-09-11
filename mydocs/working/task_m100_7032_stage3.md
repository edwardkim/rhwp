# #7032 Stage 3 — 전체 회귀·lint·WASM 검증

- 선행: [Stage 2](task_m100_7032_stage2.md) R1 시각 판정 및 R2 focused 완료
- 승인: 메인테이너의 “다음 절차 진행을 승인합니다.”
- 상태: **검증 진행 중. 아직 전체 통과를 선언하지 않는다.**
- 소스 기준: `532b74fc5` (Stage 2 구현), Stage 3 시작 문서 commit으로 review HEAD를 고정한다.
- 검증 worktree: `/home/edward/mygithub/rhwp-review-7032`
- 고정 Cargo target: `/home/edward/mygithub/rhwp-shared-review-target`
- 로그: `output/7032/stage3/`

## 실행 순서

1. review worktree 동기화 및 integration suite prepare, fmt, native/WASM/workspace Clippy.
2. `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 전체 회귀.
3. 새 HWPX fixture 보안 검사 및 여섯 코퍼스 래칫의 해당 여부 확인.
4. Native Skia lib + 그림 placeholder + 직접 PDF 출력 검증.
5. 표준 Docker `wasm` 빌드 후 동일 산출물로 Studio 검증 준비.

같은 Cargo target을 사용하는 실행은 순차로 한다. CPU 16개, RAM 31GiB(시작 시 가용 28GiB),
디스크 가용 366GiB를 확인했으며 nextest 기본 동시성을 사용한다. Docker 서버 응답도 확인했다.
테스트 실패·미실행·기존 ignored는 별도로 기록하고, 새 baseline이나 기대값을 임의 갱신하지 않는다.
원격 push·PR 생성·병합은 이 단계에 포함하지 않는다.
