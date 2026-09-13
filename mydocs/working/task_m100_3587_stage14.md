# #3587 Stage 14 — C 종료 통합 검증

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」, 2026-09-13.
- 시작 기준: `011248b01`, `task_m100_3587`. Stage 13 기록은 이미 커밋되어 있다.
- 범위: 전체 nextest, Native Skia 3종, Rust lint, Docker WASM 실제 실행 및 C 채우기 1/10/100회 계측.
- 상태: 실행 준비. 성공을 미리 선언하지 않는다. D/Gym·push·PR·댓글은 이번 범위가 아니다.

기존 C1/C2 실물 성공 판정과 B 계측은 재사용하되 현재 C 코드의 전체 검증을 대체하지 않는다.
검증 source를 고정하고 고정 review target의 Cargo 명령은 순차 실행한다.

준비 중 신규 CLI 테스트의 실행 파일 선택을 정정했다. nextest archive가 전달하는 런타임
`CARGO_BIN_EXE_rhwp`를 먼저 읽고 컴파일 시점 경로는 fallback으로만 사용한다.
제품 동작과 기대값은 바꾸지 않는다. 이 테스트 보정도 새 검증 SHA에 포함한다.
