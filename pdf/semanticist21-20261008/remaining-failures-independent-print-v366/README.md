# 남은 회귀 원본의 독립 한컴2020 Print

현재 원본을 바꾸지 않고 MCP client0.9.0의 실제 한컴2020 engine에서 출력했다.
두 job의 succeeded와 다운로드 exit0을 확인했다. 입력·PDF SHA, job, 한컴 version,
PrintMethod 및 출력 경로는 [출처 원장](provenance.json)에 고정한다.
이 자료는 기존 실패의 원인 대조이며 새 정상 회귀 fixture 또는 baseline이 아니다.

- [issue157 Print2쪽](issue157-64b06fc23ffb-2020.pdf): 입력은
  `samples/hwpx/hancom-hwp/issue_157.hwp`다. 현재 Native3쪽으로 날짜와 마지막 안내가
  추가 쪽에 넘어가므로 실제 쪽수 회귀를 계속 보정한다.
- [issue5833 Print1쪽](float5833-a0e58d445f81-2020.pdf): 입력은
  `samples/issue5833/cell_multi_para_float_pics.hwp`다. 기존 고정px 그림 좌표와 독립
  출력·셀 소유를 대조한다. 빈/축소 그림의 위치를 본문 점수만으로 검증했다고 쓰지 않는다.

별도로 `samples/hwpx/k-water-rfp.hwpx`의 같은 start는 서버의
`HANCOM_MACRO_MANUAL_PROCESSING_REQUIRED`로 거절되었으며 job이 생성되지 않았다.
스크립트를 실행하거나 원본에서 제거하지 않았다. 기존 대응 Print의 출처를 별도로 확인한다.

실제 start/status/download 명령·종료·PDF 관측은 ignored
`output/pr-review/semanticist21-20261008/remaining-failures-independent-print-v366/`에 있다.
private env-file 값, endpoint와 인증정보는 포함하지 않는다.
