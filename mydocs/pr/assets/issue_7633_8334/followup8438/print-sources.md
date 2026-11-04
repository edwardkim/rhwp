# #8336 재저장 문서의 한컴 2024 Print 기준

2026-11-02에 현재 검토 엔진으로 원본을 HWPX로 재저장한 뒤 해당 HWPX를
한컴 2024 MCP의 Print 방식으로 출력했다. 아래 값은 입력 및 출력 파일의 SHA-256이다.
쪽수는 PDF의 전체 쪽수이며, 이 기록만으로 시각 검증 통과를 의미하지 않는다.

| 원본 HWP | 재저장 HWPX | Print PDF | 쪽수 | MCP 작업 ID |
| --- | --- | --- | ---: | --- |
| `samples/hwpspec.hwp` (`64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d`) | `hwpspec-saved.hwpx` (`a57f69af3faacda68af7c424b0b645e00a8adec55854c10b92b02defd807c48a`) | `pdf/issue7633/batch-8336-hwpspec-2024.pdf` (`206ff1ecc19e4af3812ec88cc73be93afbdbab0a247b97cf837c83b62e2dc8cd`) | 178 | `f3b963f9-48d7-4e20-866d-26f30ddb9b75` |
| `samples/hwp-3.0-HWPML.hwp` (`1121cdabe5929d7aa68704d0516488852bcdc78b4f4b368ad054e8c121d04023`) | `hwpml-saved.hwpx` (`563bc982e80721dcca69f252813ca152d2adfbd4271c3dfe91bf3979d3f35258`) | `pdf/issue7633/batch-8336-hwpml-2024.pdf` (`1a993cb0b2838c2d859fb74911a095c6eb5a993f2f57e09ee4ac98d115b1a9d3`) | 122 | `5b1d7a0c-7a70-4b6a-9da9-6bda69dd786f` |
| `samples/issue6924/148751598-briefing.hwp` (`03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610`) | `brief-saved.hwpx` (`3d83c9b4cf537abea1bff50622702fd18ad256cb0aa674a27f7155a839253864`) | `pdf/issue7633/batch-8336-brief-2024.pdf` (`48995edea46890df320620214de26bd55cb2c1e63db3e36365441fbc7c6239ec`) | 6 | `2fe843f2-d603-4ac0-98aa-5dd83b477b0c` |
