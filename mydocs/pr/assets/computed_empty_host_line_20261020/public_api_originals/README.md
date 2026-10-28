# PR #7559의 기존 공개 API 원 재현본

원 PR의 `generated_table_padding_origin_survives_hwpx_save`와
`generated_table_offset_origin_survives_hwpx_save`가 사용하는 공개 API 순서를 그대로
실행해 이전에 보존한 네 입력이다. 빈 문서를 만들고 2×2 표를 생성한 뒤 셀 안 여백 또는
표 offset을 지정했다. 외부 문서를 읽거나 sample16·편람을 변형한 입력이 아니다.

이번에는 기존 입력·독립 한컴 Print PDF의 바이트만 정식 경로로 복사했다. 새 HWP/HWPX·PDF를
생성하지 않았고, Git index·실제 바이트를 대조해 같은 추적 파일이 없음을 확인했다.
[출처·원 생성 소스 SHA·입력/PDF SHA·실제 Print job](provenance.json)을 보존한다.
PDF는 MCP engine 2020의 PrintMethod 0·1-up이며 실제 제품 빌드와 전처리는 파일별로 기록했다.

| 기존 입력 | 독립 Print PDF | 현재 Native255 / 최적화 fresh WASM073 |
| --- | --- | --- |
| [offset HWP](inputs/offset-adapter-hwp.hwp) | [PDF](../../../../../pdf/pr7559_review/offset-adapter-hwp-2020.pdf) | 전 1쪽 raw 100% / 100% |
| [offset HWPX](inputs/offset-hwpx.hwpx) | [PDF](../../../../../pdf/pr7559_review/offset-hwpx-2020.pdf) | 전 1쪽 raw 100% / 100% |
| [padding HWP](inputs/padding-adapter-hwp.hwp) | [PDF](../../../../../pdf/pr7559_review/padding-adapter-hwp-2020.pdf) | 전 1쪽 raw 100% / 100% |
| [padding HWPX](inputs/padding-hwpx.hwpx) | [PDF](../../../../../pdf/pr7559_review/padding-hwpx-2020.pdf) | 전 1쪽 raw 100% / 100% |

실제 runtime은 `bbf3aed18dd82bbf344072b77b5ac590e7a71f3a`, 코드·검사 후보는
`61528013511c87c732524f91c6f224286a055337`다. 위 원 테스트 소스는 원 PR과 같은
`e5ef237b0abe215694b4c3ec0e6492d576ef18474d58aa11d8c42ae8e8f2bb2d`이며
기대값을 바꾸지 않았다. 두 검사는 전체 nextest에서 통과했다. 표의 저장 전후 원점 차이
0.2px는 같은 원점의 불변 검사이며 독립 PDF의 절대 배치 좌표 핀으로 설명하지 않는다.

후보 캡처 8개와 고정 base 대조를 완료했다. 전쪽 쪽수가 같고 0.1%p 초과 하락은 없다.
대표 review·standalone overlay의 표 원점·행 높이를 직접 대조했다. 두 형식과 backend의
실제 RGB가 같은 padding 조건 및 두 형식의 RGB가 같은 offset 조건은 그 동일성도 확인했다.
[전쪽 TSV·현재 PNG·해시](../../source_pr_qualified_20261026/README.md)를 보존한다.
이 증거를 전체 PR의 원격 CI·merge 완료로 설명하지 않는다.
