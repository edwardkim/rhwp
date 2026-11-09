# PR #8662 표지판 관리대장 2022 Print 조건

제품 `048f44da99`에서 같은 원본 HWP와 기존 한컴 2022 Print 전1쪽을 96dpi·compat2022로 비교했다.
[원본](../../../../../../samples/issue8639/sign_board_register_card.hwp)과
[독립 PDF](../../../../../../pdf/issue8639/sign_board_register_card-print-2022.pdf)는 바꾸지 않았다.

기존 정식 회귀 `issue_8639_hwp5_no_lineseg_tac_row_growth.rs`의 주석은 Haansoft Batang을
명시한다. 실제 2022 PDF에도 그 프로그램만 내장돼 있다. 2020 PDF는 서울한강·서울남산
프로그램을 사용하므로 두 Print를 서로 다른 글꼴 조건으로 비교한다. 초기 대조에서 이
2022 조건이 누락된 35.52%는 제품 회귀로 판정하지 않았다. 임계값이나 비교 영역을 바꿔
통과시킨 것이 아니며, 실제 Print와 같은 세션 글꼴 선택을 명시해 전쪽을 다시 비교했다.

- 조건: [font-environment.json](font-environment.json). 전체 시스템 글꼴을 공급하며 이 문서의
  두 서울 계열만 독립 Print의 Haansoft Batang 대체로 선택한다.
- Native 전1쪽 raw **100.00%**: [TSV](native.tsv), [review](native-review-001.png),
  [overlay](native-overlay-001.png).
- fresh WASM 전1쪽 raw **100.00%**: [TSV](wasm.tsv), [review](wasm-review-001.png),
  [overlay](wasm-overlay-001.png).
- 직접 판독: 머리 칸의 「의무설치기한」 3행 보존, 칸 내부 포함, 표 겹침 없음, 다음 관리카드와
  설치 표의 순서·경계를 대조했다. raw 100은 픽셀 바이트 동일의 주장으로 사용하지 않는다.
- 실제 실행 manifest, 입력·PDF·PNG·producer 신원: [provenance.json](provenance.json).

원 PR의 공백 줄 끝·변경 추적 0x08 주장, 필수 회귀와 보호 PR #8389/#8397 비교 및 통합
최종 검증은 별도 완료 조건이다. 이 한 사례로 원 PR 전체의 최종 승인을 대신하지 않는다.
