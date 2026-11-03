# 제목 차례 HWPX 재저장본의 한컴 2020 Print

- 원본: `samples/issue4599/1790387_policy_progress.hwpx`, SHA-256 `947e51913c561611397c4f7d88feaad7112da4adc658b21cbfd9471e00192411`.
- 제품 코드 `d536455b4e`로 재저장한 [실제 HWPX](policy-resave.hwpx), SHA-256 `14ef498440ef15c3fe0a667a072c2141b1761eb5f0e0e970f6da44495be466c9`.
- 별도 HWP 2020 MCP의 작업 `cad5d8f8-30a9-408a-9e71-0af475ff286b`로 같은 HWPX를 인쇄했다. 완료 결과는 `pdf_export_mode=print`, `print_method=0`, validation `ok`였다.
- [Print PDF](../../../../../pdf/issue8305/batch-8341-policy-resave-2020.pdf)는 64쪽, SHA-256 `5a273f437cd94a868228b9210405bb2d1503fe5101edb05953767a7a57406494`다. PDF producer는 cairo이며, MCP의 Print action과는 구분해 기록한다.
- 이 자료는 일괄 검토의 입력과 기준이며 파일 생성만으로 시각 검증 통과를 판정하지 않는다.
# 추가 대조: 원본과 재저장본의 2020 Print 및 2024 Print

2026-11-02에 같은 원본 HWPX를 한컴 2020 MCP로 Print한
`pdf/issue8305/batch-8341-policy-source-2020.pdf`의 SHA-256은
`b1a68adab1c84ec3abc610879e8374f026c3a796bc934e185e85eb5374fce05d`이며
작업 ID는 `99119c4b-c39f-4c32-ae3a-9453e834bbe0`이다. 29쪽 차트는
재저장본의 2020 Print와 마찬가지로 확대되어 잘린다.

같은 재저장본을 한컴 2024 MCP로 Print한
`pdf/issue8305/batch-8341-policy-resave-2024.pdf`의 SHA-256은
`43e2768472d834033d55ecb181b432dc62bb2b20e08ebe569db129738c4ea94f`이며
작업 ID는 `3fc832d4-c194-41f9-8036-2befae9b077a`이다. 같은 쪽 차트의 전체가
표시된다. 두 출력은 모두 64쪽이다. 이는 두 한컴 출력의 차이에 대한 관찰이며,
2020 Print 기준을 2024 Print로 대체해 같은 조건의 통과로 판정하지 않는다.
