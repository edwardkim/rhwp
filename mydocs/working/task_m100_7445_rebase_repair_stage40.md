# Stage 40: 글꼴 규칙의 기록과 공개 경로 검사 복구

- 시작 HEAD: `7df8352e8535954903c2793445f04d91e2bd8ef9`.
- 원인: 원 보정 `87e94094cb7eb61578a61920c7c51aa70d36fa39`는 독립 한컴 2020/2024 Print 10조건에 따라 문서 대체 필드 규칙과 change set을 추가했다. Rust 통합 검사의 명시적 추가 기록 목록과 공개 trace 분기가 이를 빠뜨렸다. 생성기 검사는 이미 활성 846개·이름 규칙 179개를 검증하고 있었다.
- 독립 대조: 봉인 v1 830개 + 이전 기록 15개 + 해당 기록 1개 = 846개. 이름 투영은 171 + 7 + 1 = 179개. 기존 의미 전체 비교·정렬된 ID 비교·봉인 v1·음성 대조군은 유지한다.
- 보정: 누락된 change set을 명시적으로 포함하고 문서 대체 필드에 원본 이름을 보존한 공개 API trace를 검증한다. 동일 이름을 주 글꼴로 쓰면 이 규칙을 적용하지 않는 것도 확인한다. 런타임·registry·독립 기대 메트릭은 변경하지 않았다.
- 실제 검증: 수정 전 전체/147건 재검사에서 해당 2건 FAIL → 수정 후 Native 관련 7건 PASS/0 FAIL(0.326초). 규칙 생성기 16건 PASS/0 FAIL. source 및 결과는 [증적](../pr/assets/issue7445/completed-20261009/stage40-font-rule-projection-20261022/)에 보존했다.
- 실행 로그: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v6/nextest.log`, `font-rule-count-audit/projection-generator.log`.
- 이 결과는 최종 전체 회귀·fresh WASM·전쪽 시각 검증의 완료를 뜻하지 않는다. 다른 미해결 실패와 같은 환경 0.1%p 하락은 계속 보정한다.
