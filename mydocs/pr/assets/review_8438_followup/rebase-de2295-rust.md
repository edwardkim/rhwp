# 기존 통합 원 PR 재검토 — 리베이스 후 최소 글꼴 Rust 회귀

- base: `de2295d430b588f0741021f66fae086eb4ff4009`.
- 검사 대상 제품 코드 기준: `0fdbbf87cc2c999499ad86c26fa676d8434daffd`. 이후 리뷰 문서 변경에 제품 코드·테스트·Cargo 의존성·검증 스크립트 차이가 없음을 Git diff로 확인했다.
- Ubuntu에서 bubblewrap으로 시스템·사용자 글꼴 경로와 `RHWP_FONT_PATH`를 숨기고 저장소의 `ttfs/opensource` 글꼴 5개만 Fontconfig에 노출했다. 빌드 작업은 8개, 테스트 스레드는 12개이며 `target/pr-review`를 사용했다.
- 전체 선택 목록은 12,369건이고 기존 ignored 45건은 제외됐다. 첫 실행은 12,062건 PASS까지 기록한 뒤 프로세스가 종료돼 최종 summary가 없다. 다음 실행은 첫 로그의 완료 항목과 `cargo nextest list`의 바이너리 ID·테스트 이름을 대조해 빠진 307건만 선택했고 307 PASS로 정상 종료했다.
- 두 로그의 완료 항목 합집합을 실제 선택 목록과 대조해 12,369건 모두 PASS, 누락·실패·목록 외 결과가 각각 0건임을 확인했다. 두 실행을 한 번의 연속 실행으로 주장하지 않는다.
- 첫 로그 SHA-256 `17481346db582a49eabfecbc7fe667e018f6832bcebaf749a71726fc223ebd62`, 나머지 검사 로그 SHA-256 `950062b38eb842229bf1996994a8efc3fa5c0261209cb2965d1f4e4b9cd38b5b`, 검사 목록 JSON SHA-256 `28172be1b78c9411c1d642b167d1eba46d4bd0d9be22bcf02313072454d44a6f`.
- 같은 최소 글꼴 격리에서 `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib -- --test-threads=12`도 실행했다. 워크스페이스 라이브러리 합계 4,040 PASS, 12 ignored, 실패 0, 종료 코드 0이다. 로그 SHA-256은 `69ee926a6d924d04259f871b6682e0cbb0281f8e9b323002ef53f9ddbe5f3bb9`다.
- 기존 실패 이력이 있는 JPEG 채우기 검사도 같은 격리에서 Native Skia로 실행해 HWP·HWPX 2건이 모두 통과했다. 첫 시도는 생성된 묶음 번호를 잘못 지정해 0건 선택으로 종료 코드 4였고, 실제 배정된 `regression_suite_017`로 다시 실행했다. 첫 로그 SHA-256은 `8df57010990024fca4c299efee3356cf3107618c90a48f85aa46c3f9b0a16d99`, 재실행 로그는 `68dabb2a053e0c10ed8e0458582730e3558bf221e4c55cb2a3262a5b4ae705a8`이다.

이는 위에 명시한 제품 코드의 Rust 회귀 결과다. 이후 제품 코드가 바뀌면 영향받은 검사를 다시 실행한다. 개별 PR의 독립 Print 대조, 시각 미달 보정, Native/fresh WASM 전쪽 TSV와 리베이스된 기능 보존 검증을 대신하지 않는다.
