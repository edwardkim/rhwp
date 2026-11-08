# 단계 393: 현재 WASM과 Studio JS 바인딩 정합

## 분석

현재 Rust는 `getDocumentFontSelectionPlan`,
`buildFontEnvironmentFromAvailablePrograms`, `setHangul2024Compat`를 공개하지만,
추적 중인 Studio JS에는 세 메서드가 없었습니다. Studio의 글꼴 준비 경로는
앞의 두 메서드가 없으면 준비를 중단하므로 Rust만 갱신해서는 반영되지 않습니다.

## 수정

단계 392의 Mac no-opt fresh WASM wrapper가 생성한 JS를 public 경로에 반영했습니다.
수동 바인딩이나 새 조판 규칙을 추가하지 않았습니다. 생성된 WASM과 pkg는 커밋하지 않습니다.

## 확인 결과

- Node 문법 검사 통과.
- 같은 fresh WASM을 실제 초기화하고 sample4 원본을 로드했습니다.
- 문서 글꼴 후보 27개를 반환하고, 공급 프로그램이 없는 경우 환경 `null`을 반환했습니다.
- 한컴 2024 호환 설정을 켰다가 끈 뒤 문서 정보가 보존되었습니다.
- pkg/public JS 및 WASM SHA-256이 각각 일치했습니다.
- 실행 증적: `mydocs/pr/assets/issue7445/completed-20261009/studio-bindings-validation.json`.

이는 바인딩의 실제 호출 확인이며 브라우저 표시나 전체 시각 검증 통과를 대신하지 않습니다.
새 회귀 테스트는 추가하지 않았습니다. 전체 회귀와 문서 시각 게이트는 계속 검토합니다.
