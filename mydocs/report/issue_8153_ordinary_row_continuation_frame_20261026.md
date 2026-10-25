# #8153: 일반 행 이어받기의 물리 공간 소비

[선행 분석과 계획](../plans/issue_8153_ordinary_row_continuation_frame_20261026.md)에 따라
원본·저장 컷·독립 Print와 실제 생산/소비 경로를 대조했다. 원본·PDF·기준값을 바꾸지 않는다.

## 독립 기대값과 실제 수정 전 실패

정식 테스트 원문의 SHA-256은
`7f3cd291bebc5b4e6214c0a278d567d98a925a31db60adf2f125c14bab4af319`다.
고정 `b8ca19e50d90d9b3117d01e7b10a3dd14fb5e473` Native 라이브러리에 연결해
빌드 exit 0, 두 형식 각각의 독립 세로 종료선 검사에서 FAIL을 확인했다. 두 건 모두
기대 644.264±2px에 실제 637.524px이며, 실행은 4.94초·exit 101이다. 앞 조각과 다음
조각의 내용 컷·같은 행/셀 소유·원본 선언 높이·세 열 경계 검사를 통과한 뒤 실패했다.

두 형식의 실제 Native 312쪽 PNG는 앞선 `89b929b8` 출력과 바이트까지 같다. 새로운
표준 최적화 fresh WASM 집중 98조건도 양쪽 최저 99.25584%, 미달·누락 0이며 해당
312쪽은 양쪽 99.99044%·미대응 9픽셀이다. Native와 fresh WASM 모두 해당 쪽 98%
이상인 조건에서 같은 독립 Print의 ±2px 종료선을 보호한다. 반복 제목·바깥 세로선·
뒤 조각을 함께 판독한 결과를 점수와 별도로 유지한다.

## 공통 구현

이미 존재하는 `stored_ordinary_row_end_frame`의 물리 상자 선택을 이어받기에도 쓴다.
스캔이 refit과 실제 컷을 확정한 뒤, 같은 시작 행이며 블록 컷이 아닌 경우에만 기존
`start_row_height_override`를 전달한다. 새로 만난 행이 앞 행의 잔여를 받지 않는다.
원본 일반 셀·유효한 저장 컷·편집/재조판 제외와 내용 요구 검사는 기존 생산자를 쓴다.

첫 조각의 독립 선언과 종료 프레임 검증은 유지한다. 이어받은 경우는 유효한 물리 잔여가
있을 때만 기존 예산/내용 하한과 같은 선택을 한다. 원본 전체 높이로 되돌아가 이미 받은
공간을 다시 예약하지 않으며 세 쪽 이상에 걸친 원본 행도 같은 잔여를 소비한다.
표현만 늘리는 paint 보정·쪽/문서 분기·새 px 상수는 없다.

선택한 높이는 기존 `consumed/end_row_height_override/split_end_reserved`에 전달된다.
emit은 이 같은 값을 실제 PageItem·수용 원장에 기록하고 기존 잔여 생산자가 다음 상자에서
차감한다. 같은 partial-table 배치도 이 값을 소비하므로 종료선을 따로 덮어쓰지 않는다.
`scan.rs`는 수정 블록의 중복 주석·빈 줄을 정리해 1,000줄 이내로 유지했다.

## 남은 실제 검증

같은 테스트 원문을 `tests/cases/issue_8153_ordinary_row_continuation_frame.rs`로 복사했다.
기본 suite에서 실제 수집하며 Native 전용 기능을 요구하지 않는다. 수정 후 Native 빌드와
해당 두 건, 기존 95건, 영향 쪽·앞뒤 종료와 전쪽 Native/fresh WASM 비교는 이어서
실행한다. 이 기록 시점에는 수정 후 PASS나 전쪽 100%·최종 테스트·PR 준비 완료가 아니다.

자료는 ignored 작업 루트 `output/pr-review/issue8153-handbook-raw100-20261022/`의
`handbook-ordinary-row-continuation-candidate/`와 `remaining-raw100-diagnostic/p312-*`에
보존한다. 원 해결 이슈·PR의 본문과 전체 댓글은 읽기 전용 snapshot으로 남겼다.
사용자 지시대로 upstream/devel 동기화와 원격 게시·push·PR 생성은 하지 않는다.
