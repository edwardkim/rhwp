# #8946 단계 8 최종 검증 증적

[작업 기록](../../../../working/task_m100_8946_stage8.md)의 소스 SHA와 범위에 해당한다. 2026-11-12~13 KST에 실행했으며 디렉터리 날짜는 시작일이다.

- HWPX·HWP3 각각 763쪽의 Native/fresh WASM/Host TSV와 대표 PNG, 기본 9조건 및 실제 편집·첫 화면 결과를 보존한다.
- 보호 대상 18개 입력의 양쪽 3,542쪽과 편람 383쪽 필수 게이트를 통과했다.
- 최신 devel #9017의 13개 입력·52쪽 및 실제 Studio 반복 저장을 대조했다.
- 최초 기준 WASM의 잘못된 캐시 재사용은 무효화했다. 소스별 export·실제 수신 SHA를 확인한 재실행 결과만 최종 판정에 사용한다.
- [제출 판정](submission-readiness.json), [로컬 검사](local-verification-results.json), [생산자 해시](producer.json), [원격 사전 확인](remote-preflight.json)을 구분한다. 원격 CI는 PR 생성 후 판정한다.

사용자 글꼴 파일·비공개 실행 로그·개인 경로는 포함하지 않는다. 독립 Print 원본과 프로그램 공급 출처는 [단계 6](../../../../working/task_m100_8946_stage6.md)에 연결한다. 점수와 확대 판독을 구분하며, raw 100%는 RGB 동일을 뜻하지 않는다.
