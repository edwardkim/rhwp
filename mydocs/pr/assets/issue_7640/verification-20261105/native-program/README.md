# #7640 Native 글꼴 프로그램 보정 단계

시스템 조회 face의 글리프 번호와 실제 그리기의 프로그램이 달라 한글이 다른 문자로 표시됐다.
선택한 face의 바이트로 같은 프로그램을 다시 열고 조회·그리기가 함께 소비하도록 보정했다.
개인 글꼴 파일·등록, 문서 좌표·줄 폭·쪽 예산은 변경하지 않았다.

- `skia_native_glyph_replay::issue_7640_system_program`의 새 검사 본문은 보정 전 1 PASS/1 FAIL,
  보정 후 2 PASS였다. 처음 독립 파일에서 실행한 본문과 최종 통합 본문의 공백 제외 해시가 같다.
- 기존 Native CI 명령의 글리프 검사 전체 12 PASS/0 FAIL. CI workflow는 변경하지 않았다.
- 현재 Mac의 추가 글꼴을 차단한 프로세스에서도 같은 12 PASS, OS 정적 프로그램 대조 4건이다.
  OFL 명시 공급 스타일 검사는 개인 글꼴 없이 항상 실행한다.
- Native 전 21쪽·누락 0·PDF와 쪽수 일치. 여전히 8·11·12·18·19·20·21쪽이 90% 미만이며
  최소 33.70632%다. 8쪽은 88.68859%에서 88.84330%로 바뀌었다. 바르게 표시되는 한글과
  남은 수식/그림 원점 차이를 구분한다. 미달을 완료로 판정하지 않는다.
- `before/after-native-p008-{raw,review,overlay}.png`는 같은 입력·PDF의 실제 출력이다.
  한컴돋움의 잘못된 문자 표시가 없어졌음을 직접 확인했다. glyph 프로그램 보정은 미주 배치의
  전쪽 완료나 fresh WASM 새 결과를 뜻하지 않는다.

입력·PDF·CLI·소스 해시, 검사 본문 보존과 페이지별 gate는 `validation.json`을 따른다.
전체 TSV는 canonical Visual Sweep의 기존 PNG 쌍 비교로 다시 산출했다.
정본 로그는 ignored `output/open-issues-oldest-20261105/post-rebase-ece9/issue7640-*`에 보존한다.
