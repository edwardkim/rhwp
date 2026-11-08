# Stage 42: 수식 셀의 측정과 배치가 같은 최소 높이를 소비

- 시작 HEAD: `800f93fe61`.
- 원 보정 `1d28a62b073406076fcf6d4d947f26a6ce50e17a`와 Stage20 독립 Hancom Print 증적을 먼저 대조했다. 공통 `stored_text_line_cell_minimum`은 원본 텍스트/인라인 수식 줄과 활성 패딩을 수용한다. 측정은 이 생산자를 쓰지만 리베이스 통합에서 배치 wrapper가 과거의 controls 전부 거부 가드로 돌아갔다. 실제 과학3쪽 글줄은 셀 하단보다1.88px 넘었고,2쪽 분수 괘선도 독립 위치를 벗어났다.
- 구현: 배치 wrapper를 기존 공통 생산자 호출로 복구했다. 새 문서 예외나 offset·높이 상수는 없다. `HeightMeasurer::stored_cell_text_minimum`과 layout의 행 최소가 같은 반환값을 소비한다. 비활성 셀 여백, 독립 그림/표, 편집·합성 입력의 다른 계약을 확장하지 않는다.
- 실제 Native: 과학의 기존 실패2건을 포함한5건과 upstream #7972 비활성 여백, #7872 두 번째 중첩 TAC 행, #8067 실효 패딩 정상 대조군 총9 PASS/0 FAIL(0.146초). 기대값·검사 수를 바꾸지 않았다.
- 전쪽 비교: 동일 원본 `samples/exam_science.hwp`와 독립 PDF `pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf`(SHA `1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14`), canonical Native PNG/PDFium152.0.7947.0/smooth RGB/96DPI/raw2px. fixed base·수정 전·후 각각4/4쪽, 합12쪽 TSV를 새로 산출했다. 후보 최저96.14438%, 미달0, base 대비0.1%p 이상 하락0.2쪽98.45087→98.65793%,3쪽96.94163→97.81025%,4쪽95.90341→96.14438%,1쪽98.62957% 유지다.
- 직접 판독: 후보2·3쪽의 review/overlay를 열어 분수 셀의 줄/괘선, 양 단의 문항·본문·쪽 소속을 확인했다. 수식 주변 세부 baseline 차이는 남아 있어 완전 일치를 주장하지 않는다. 새 px 검사나 기대값 갱신은 없다.
- [증적](../pr/assets/issue7445/completed-20261009/stage42-inline-cell-minimum-20261022/): 전쪽 TSV·source/producer·비교표·대표 PNG. 로그는 ignored `rebase-7752-20261022/repair-focused-v8/nextest.log`, `inline-cell-native-v8.log`.
- 현재 결과는 집중 Native 검증이다. 최신 source의 fresh WASM 전쪽·전체 회귀·실제 소비 경로와 다른 미해결 실패는 계속 검증·보정해야 한다. 이전 Stage20의 fresh 결과를 새 head 결과로 재인용하지 않는다.
