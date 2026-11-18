# 닫힌 표 본체 뒤의 중복 빈 간격 보정

부모 `e2604d5a14`의 HPV 원본 `samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp`와 독립 `pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf`를 그대로 사용했다. 표15 원본의 전체 행 선언 57,622HU는 첫 조각 37,580HU와 다음 쪽 저장 원점 20,042HU의 합이다. 실제 종료 조각은 행24~35의 본체 19,739HU로 닫혔지만, 같은 잔여를 다음 문단 위치로 다시 사용해 303HU(4.04px)를 빈 띠로 중복 점유했다.

정수 행 합의 공통 인증을 첫 조각 fit과 종료 간격 Query에서 공유했다. 원본 HWP5의 비편집 일반 셀만 적용하고 캡션·바깥 여백·후속 개체·문단 앞 간격·반복 머리 행의 독립 공간은 유지한다. 같은 Query를 typeset의 전진/commit과 실제 paint가 소비한다. 문서명·쪽 번호·기대 좌표로 제품을 분기하지 않는다. HWPX 지역 좌표 및 일반 전체 표 높이 fit은 바꾸지 않는다.

## 실제 검증

- 83쪽 Native 71.61440% → **92.23484%**. fresh WASM **92.25131%**. PDFium·96 DPI·compat2022·색상 경계 보정 전2px raw이며 글꼴 최소 환경과 같은 공개 웹폰트 공급을 사용했다.
- Native/fresh WASM 각각 전235쪽을 끝까지 측정했다. 전체 문서는 아직 미달이며 세부 최저·미달 목록은 [정본 측정/producer](validation.json)에 분리했다. 83쪽 개선으로 다른 미달을 상쇄하지 않는다.
- `native-skia` 최소 글꼴 정상 대조군 41 PASS/0 FAIL: #8149 rowfit/reservefit/headerrow 및 원본 사양서의 제목행·캡션·후속 간격, 짧은 행 이월, R7/R9 소속 보호 포함.
- 새 종료 관계 검사는 같은 compiled harness·원본에서 R9 frozen CLI로 실제 1 FAIL(행 끝376.5, 다음 줄380.6) → R10 frozen CLI로1 PASS다. 정규 nextest 모듈은2 PASS다. 검사는 마지막 합계 행의 실제 하단과 다음 빈 문단의 시작을 비교하며 절대 px 좌표를 고정하지 않는다. CLI 소수 직렬화 오차만 정수 격자에서 비교한다.
- 첫 조각 상한을 넓힌 중간 후보는 정상5건을 실패시켜 철회했다. 정상 기대값은 그대로 유지했다. 실패 로그도 ignored output에 보존하고 최종 성공으로 합치지 않는다.

## 남은 범위

83쪽 아래 표17의 첫 부분 행 누락, 95쪽을 포함한 기존/후보 미달은 계속 보정한다. 최신 전체 nextest·보호 전쪽·실제 소비 경로를 완료했다는 자료가 아니며 PR 준비는 미완료다. `.log` 및 임시 진단 파일은 Git에 넣지 않았다.

[Native review](native-p083-review.png) · [overlay](native-p083-overlay.png)

[fresh WASM review](wasm-p083-review.png) · [overlay](wasm-p083-overlay.png)
