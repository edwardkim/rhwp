# 이전 sample16 회귀의 2020 Print 대조

`tests/issue_1116.rs`의 default 2022 호환 경로와 2022 저장본을 분리해 대조한다.
두 입력 모두 내용 변경 없이 MCP의 `engine: 2020`, build `11.0.0.9136`,
PrintMethod `0`, `hancom2020_pdf_driver_one_up`으로 전쪽 출력했다.
입력·PDF SHA-256, 실제 쪽수·저장 제품·job은 [출력 근거](print-provenance.json)에 있다.

- `samples/hwp3-sample16-hwp5.hwp`의 2020 출력은 **교차 버전 진단용**이다.
  2024 저장본의 공식 기준은 기존 `input-17-hwp3-sample16-hwp5-2024.pdf`이며
  이 파일로 바꾸지 않는다. 전64쪽 2024 Native/fresh WASM 통과와 구분한다.
- `samples/hwp3-sample16-hwp5-2022.hwp`의 출력은 해당 저장본의 2020 기준이다.
  Native/fresh WASM 전쪽 측정과 직접 판독을 마치기 전에는 시각 통과로 판정하지 않는다.

두 Print 모두 확보만 완료했다. 과거의 절대 px·dump 문자열 검사와 실제 한컴의
줄 구성·문단 소유를 대조한 뒤 필요한 검사만 새 이름의 관계 검사로 교체한다.
