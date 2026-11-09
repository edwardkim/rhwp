# 최종 R43 Rust 검증

- 실제 제품·검사 source: `0261687265f3a10a64e51809d006b2305a36d950`.
- 정책 비교 base: `5077b216cbc51d2046da07350de562f6f521a7c1`.
- 최소 공개 글꼴 5개 격리, 공용 `target/pr-review`, 컴파일 8 jobs·테스트 12 threads.
- 전체 회귀 12,910 passed / 0 failed / 45 skipped. 종료 코드 0, 785.738초.
- 충돌 경계 19 passed. 선택에서 제외된 검사를 전체 skip으로 합산하지 않았다.
- Native Skia 4,035 passed / 0 failed / 12 ignored. 기존 ignored는 실행 성공으로 합산하지 않았다.
- format, Native/WASM32/workspace Clippy, workspace build, suite·unit tier 정책, 문서 예제를 포함한 완료 항목만 [실행 원장](validation.tsv)에 기록했다.
- 이 원장은 새 head의 fresh WASM·Studio·시각 검증을 대신하지 않는다. 해당 증거는 별도로 연결한다.
- 로그의 SHA-256과 실제 종료 결과를 보존했다. 임시 실행 로그 자체는 Git에 추가하지 않는다.

## 신규·변경 sample의 실제 보안 검사

검토 base 대비 추가·변경한 sample 35개 모두를 최소 글꼴 환경에서 세 탐지기로 실제 검사했다. 105회 모두 clean이며, 검사 전후 입력 SHA도 35개 모두 동일하다. [문서별 실행 원장](new-sample-security-coverage.tsv)을 따른다.

같은 35개 목록을 명시적으로 공급한 정식 집중 회귀 2건도 모두 통과했다(exit 0, suite 009/010). 입력 없이 성공을 반환할 수 있는 기존 전체 회귀의 pass와 구분한다. 필터로 제외된 625건은 전체 회귀의 skip 45건과 합산하지 않는다.

처음에는 불필요한 전체 바이너리 빌드를 지정했으나 검사 실행 전에 중단하고 대상을 두 suite로 바로잡았다. 이 중단은 제품이나 회귀 실패가 아니다. [검사 시도 원장](new-sample-security-attempts.tsv)에 초기 중단과 최종 통과의 시각·범위·실제 종료 코드·로그 SHA를 구분해 보존했다. 임시 로그는 검토 종료 정리 대상이다.
