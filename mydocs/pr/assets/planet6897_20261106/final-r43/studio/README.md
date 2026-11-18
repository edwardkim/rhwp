# 최종 R43 Studio·공개 API·브라우저 검증

실제 제품 source `0261687265f3a10a64e51809d006b2305a36d950`의 최적화 package를 공급했다. WASM SHA-256은 `2de28f9ba1f2e2154af053025c18041948858cbbe42fb8b95b2e1998a7ccfe5c`, glue SHA-256은 `0099b8a6e00f750e1dbddb3a89bf42874f24effcf8bf4dc5eb328d69e691606b`다. Root pkg와 Studio public의 실제 바이트가 같은지 검사했다.

[실행 원장](validation.tsv)의 12단계가 모두 종료 코드 0이다. Studio Node 전체 2,762 passed / 0 failed / 0 skipped, TypeScript/production build, 실제 Canvas CFF/TTF·font 공급 브라우저, 공개 frontend, CanvasKit 수식 소비자, 실제 그림 편집 API·브라우저 E2E, fontName 편집·슬롯 상한·저장 참조, 원본 marker 계약을 실제 검사했다. CI 결과로 대신하지 않았다.

PR #8552의 본문·표 셀·중첩 셀·머리말·꼬리말 5개 위치 × HWPX 내보내기 및 HWP 저장·재열기 2경로 × 언어별 fontRef 7개는 [실제 run 참조 70행](fontname-saved-references.tsv)으로 확인했다. 원자성·오류 경우를 포함한 공개 API와 슬롯 상한은 실행 원장에 연결했다. API 합성 계약에 관련 없는 문서의 PNG/raw를 붙이지 않았다.

브라우저 검증은 사용자의 7700 서버를 유지하고 검증 전용 임시 서버/포트에서 실행했다. 실제 package SHA·출력 소비자와 명령을 구분했다. 이 원장은 전쪽 Native/fresh WASM TSV와 직접 판독을 대신하지 않는다.
