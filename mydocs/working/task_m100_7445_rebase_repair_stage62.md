# #7445 리베이스 보정 62단계: 공동 앵커 표의 흐름과 뒤 간격 소유

## 범위와 원 해결 계약

기존 67개 입력 중 `samples/issue7687/empty-host.hwpx`와 독립 한컴 2020 Print
`pdf/issue7687/empty-host-2020.pdf`의 H1~H6, 전 6쪽만 보정했다. 대상 HWP를 확대하지 않았다.
원 해결 `7bf6a6613992e42cd8bfb41a76edd7894e46d585`(#7687·#7680,
원 기여 `839af8c92427d834d7042ec540c5e0fbab01473c`)는 오프셋 0 표는 흐름을 진행시키고,
양수 오프셋 표는 흐름을 밀지 않는 배제 띠로 정의한다. 빈 호스트 줄·뒤 문단은 표 사이를
채우며 띠에 걸린 줄만 표 아래로 이동하고 앞 간격을 흡수한다.

후속 `4a7e8f57aef57cd8d14e0441a5e10f1f7184ea97`의 형제 표 공통 상자가 두 표 모두
`NextLine`으로 전달했다. typeset은 양수 표를 배제 띠로 놓아도 `layout.rs`의 문단 종료
합집합이 그 표 아래까지 흐름을 밀어 본문이 약 178.67px 내려갔다.

## 공통 결과와 소비 경로 보정

`ParagraphFloatPlacement::from_empty_sibling_box`가 기존
`is_textless_coanchored_float_host`의 공동 앵커 소유를 그대로 전달한다. 첫 표는 `NextLine`,
양수 오프셋 표는 `Exclusion`이다. 둘째 표의 위치나 크기를 clamp하지 않는다.

공동 앵커의 실제 호스트 줄은 별도로 진행한다. 표 점유 끝에 문단 뒤 간격·호스트 줄 꼬리를
함께 넣으면 H6의 본문이 2px 더 내려간다. 개체 배제 띠는 표 본체와 원본 아래 바깥여백만
소유하며 문단 뒤 간격은 기존 호스트 줄 종료에서 한 번 소비한다. DPI를 두 공통 호출 경로에서
전달해 원본 HU를 변환하며 96DPI 상수를 구현에 넣지 않았다. 비공동 앵커 경로는 기존 상자를
유지한다.

`typeset/controls/flow_table.rs`는 이 공동 앵커 형상을 lane 적층 경로에서 제외하고
`place_table_with_text`로 보낸다. 일반 block의 수용·기록 결과와 paint의 문단 종료·배제 띠가
같은 `ParagraphFloatPlacement`를 소비한다. lane 호출도 DPI를 명시적으로 전달한다.

기존 67개 입력의 본문·표 셀·주석·머리말·꼬리말·마스터쪽·도형 텍스트·캡션을 읽은 경계
감사는 67/67 파싱 성공, 오류 0, 해당 조건 6곳이며 모두 이 대조군의 H1~H6다.
이는 변경 조건의 경계 감사이며 전체 67개 시각 통과로 보고하지 않는다.

## 검증

- 수정 전 HEAD43 후보 Native/fresh WASM 전 6쪽 최저 85.87541%, 1·6쪽 90% 미만.
- 양수 표 소유만 고친 v1은 양쪽 전쪽 raw100%였지만 기존 H6 구조 검사에 2px 차이가 남았다.
  해당 검사를 유지하고 구현을 추가 보정했다.
- v2 집중 25건: 11 PASS / 14 FAIL, 5.294초. 기존 #7687 H1~H6 관계 검사 PASS.
  통과 수를 최신 전체 nextest 통과로 인용하지 않는다.
- 최종 Native/fresh WASM 각각 전 6쪽 최저 100%, 미달 0, 6/6쪽 일치. 양쪽의 1·6쪽 review·overlay를 직접 판독했다.
  base/변경 전/변경 후 두 backend 모두 전쪽 TextRun 내용·소유·순서는 같다.
  Native 실제 출력 트리의 H6 본문 윗변은542.5→(v1)469.8→467.8px이며 base467.8px와 같다.
  두 표의 아랫변·셀 본문·앞 제목은 보존된다. 원본 아래 바깥여백283HU만 점유한다.
- root pkg·Studio public·frozen JS/WASM hash 일치. Mac locked web/pkg/no-opt 빌드 227.276초, exit 0.
  실제 Studio 초기화 응답은 미검증이며 최종 소비 경로 검증에서 확인한다.
- 같은 backend의 base 대비 0.1%p 이상 하락은 전 6쪽 모두 0이다. 표 선 두께와 글자 래스터 색 차이는
  남지만 위치·내용 차이로 보이지 않으며100%를 RGB일치로 보고하지 않는다.

고정 base605 WASM의 신규 자동 font-environment 조회 API 미지원으로 첫 대조 호출이 실패했다.
후보의 실제 자동 환경이 null임을 확인하고 canonical 명시적 null 경로로 base의 전 6쪽을
새 경로에 산출했다. 실패 실행은 비교 증거에서 제외하며 글꼴 조건을 바꿔 미달을 숨기지 않았다.
고정 base Native/fresh WASM은 각각 전 6쪽 100%다.

최신 전체 회귀·필수 lint/build/Skia/실제 소비 경로와 남은 14건은 계속 검증·보정한다.
PR·CI·merge 후속 처리는 이 필수 범위 완료 후 진행한다.

정식 전쪽 TSV·대표 PNG: `mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/coanchored-float-flow`.
최종 source/build/입력/PDF/hash·내용 대조와 로그: ignored `stage62-coanchored-float-flow/v2/`.
