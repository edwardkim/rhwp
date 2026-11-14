# 최종 구현의 완료된 Rust 검증

제품 소스 `43adf3f7fe412e71797456887e2be44b8ef8d14f`에서 실행한 결과다.
[검증 영수증](completed-rust-validation.json)은 4,849개 검사 대상 소스의 유지 확인,
제품 트리·실제 로그 해시·종료 코드와 수행 시간을 기록한다. Native 3종 로그는
말미의 빈 줄만 정리했으며 원본·보존본 SHA-256을 함께 기록한다.

- [전체 최소 글꼴 Rust](full-minimal-rust.txt): 13,562건 통과, 실패 0건, 설정상 제외 45건.
- [원 PR의 Native 집중 검사](original-claim-native-focused.txt): 3건 통과.
- [Native 라이브러리](final-native-lib.txt): 본체 3,853건과 하위 라이브러리 182건 통과, 명시적 제외 12건.
- [Native 그림 검사](final-native-picture-placeholder.txt): 2건 통과.
- [Native 직접 PDF 내보내기](final-native-direct-pdf.txt): 36건 통과.
- [suite 준비](suite-prepare.txt)·[manifest](suite-manifest.txt)·[단위 검사 계층](unit-tiers.txt)·[포맷](format.txt): 모두 종료 코드 0.

빌드 잡은 8개, 테스트 스레드는 12개이며 공유 `target/pr-review`를 사용했다.
회귀 검사는 시스템 글꼴과 사용자 글꼴을 격리한 공개 5개 자원 환경이다.
노출 결과는 [실제 최소 글꼴 목록](../../print-primary-stage/minimal-font-exposure.txt)을 따른다.
시각 측정은 기존과 같은 전체 시스템 글꼴·명시 환경으로 별도 실행하며 두 조건을 혼동하지 않는다.

이 파일은 완료된 검사만 기록한다. Native CLI 빌드·lint·두 경로 전쪽 보호 검증·
편람 gate와 원격 CI·PR 생성·병합의 완료를 뜻하지 않는다.
