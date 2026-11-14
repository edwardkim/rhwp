# 최종 구현의 Rust·lint 검증

제품 소스 `43adf3f7fe412e71797456887e2be44b8ef8d14f`의 필수 15단계가 모두 종료 코드 0이다.
[검증 기록](../../verification-record.md#필수-회귀와-lint)에 실제 명령·환경·통과/제외 수와
원시 로그 SHA-256·각 단계의 수행 시간을 남겼다. 원시 로그는 output에서만 관리한다.

- 기본 최소 글꼴 Rust: 13,562건 통과, 실패 0건, 설정상 제외 45건.
- 원 PR Native 집중: 3건 통과.
- Native lib: 본체 3,853건·하위 182건 통과, ignored 12건.
- Native 그림 2건·직접 PDF 36건 통과.
- suite·포맷·테스트 계층, Native 집중/default/WASM Clippy, workspace build·all-target Clippy 통과.

공유 `target/pr-review`, 빌드 잡 8개·테스트 스레드 12개를 사용했다.
기본·Native 회귀는 시스템·사용자 글꼴을 격리한 저장소 공개 5개 자원 환경에서 수행했다.
시각 검증은 기존과 같은 일반 시스템 글꼴·명시적 공급 조건으로 별도 수행했다.
이 검사는 Native 전쪽 보호 검증·편람 gate나 원격 CI·병합을 대신하지 않는다.
