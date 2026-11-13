# #9018 검토 증적 — 리베이스 전 기록

원 PR head `be67439b02f42779b50361ace7b8a47dd0d1439a`를
base `6217d3cbedc9aba707812e5d4e5ea2a1ed53845a` 위에 체리픽했다.
이 묶음의 실제 producer는 [producer.json](producer.json)에 명시했다.
현재 소스 보정·리베이스 이후의 재실행 결과로 표기하지 않는다.

- 원본·편집본 각각 2쪽의 Native/fresh WASM 전쪽 TSV와 같은 쪽 raw/PDF/review/overlay는 `cases/`에 있다.
- 일반 글꼴 환경의 Visual Sweep과 최소 글꼴 환경의 Rust 회귀를 구분했다.
- [편람 Native 383쪽](handbook-current/silhouette.tsv)은 raw 최저 99.96%(130쪽), 공식 보호 표시값 하락 0쪽이다.
- [기존 보호 결과의 재사용 근거](protected-reuse-audit.json)는 원 producer·입력·PDF·글꼴·읽기 전용 출력 경로의 동일성을 기록한다. 현 head의 전쪽 재캡처로 주장하지 않는다.
- Windows의 Latin 글꼴 우선 선택에서 한글 주석 두부문자를 재현하고, Unicode cmap 확인 뒤 한글 글꼴을 선택하는 도구 보정을 확인했다. 문서 렌더링 점수는 바꾸지 않았다.
- 별도 누름틀 끝 표식 반례는 실제 WASM 저장의 원시 줄 시작 이동으로 확인했다. 추가 Rust 검사는 아직 미완료이며, 원 PR의 12개 검사가 통과한 기록과 구분한다.

최종 판정·개별 PR 리뷰는 리베이스와 필요한 보정 검증 후 정리한다.
