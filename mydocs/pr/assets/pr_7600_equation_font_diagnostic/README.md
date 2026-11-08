# PR #7600 수식 논리 프레임 진단 입력

한컴 수식의 저장 폭·높이·기준선을 비우고 저장 줄을 제거한 HCR 대조 입력입니다.
기존 진단 문서의 본문 구조를 복제하여 수식 역할과 BaseUnit을 바꾸었습니다.
원본 시험 문서와 독립적으로 재생성한 대조군이며 원본 입력의 통과를 대체하지 않습니다.
원문 HWPX, MCP 재생성 HWP와 Print PDF의 바이트를 그대로 보존합니다.

모든 생성·Print는 MCP engine 2020, Hancom 11.0.0.9136입니다.
PDF는 `pdf_print_method: 0`, `hancom2020_pdf_driver_one_up` 인쇄 출력입니다.
원시 JSON·TSV·실행 로그·generator는 ignored `output/pr-review/pr7600/`에 있습니다.
이 자료는 검토 증거이며 실패 진단 입력을 새 회귀 golden으로 고정하지 않습니다.

| 입력 | HWP 재생성 job | PDF Print job | 검증 범위 |
| --- | --- | --- | --- |
| `radical-font-frame-roles-hcr` | `9e55a726-899d-4a7f-90e1-1e6a277946cb` | `cb0715b4-70c0-486c-9258-0d8b1e96e889` | 실패 분석용 84개/3쪽; 초기 Native 2·3쪽 미달. 회귀 golden 또는 전체 통과 증거가 아님. |
| `radical-font-frame-boundaries-hcr` | `c8147217-aa0b-4210-a6b8-842161f31d1d` | `fe27403b-00b8-4623-bec6-06557c2cfb4a` | 12개/1쪽; a441 및 bfca의 Native/fresh WASM 95.65151%. |
| `geometric-font-frame-hcr` | `6189603e-e466-4ea1-a1db-e06a3d5c2551` | `16297e61-0d94-430a-9e11-6a20c8de9c1e` | 8개/1쪽; bfca의 Native/fresh WASM 98.95100%. 기하 기호 높이 오류 가설을 기각함. |

## 원 바이트 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `radical-font-frame-roles-hcr.hwpx` | `85cee9bebed15806151681efc64919588eaa9fbf6e598cab26e8118aebb83d85` |
| `radical-font-frame-roles-hcr-2020.hwp` | `1386c56fc4da997fc8c384e69dd924290793b874f09c84c5814d0bf9d796d065` |
| `pdf/pr7600-equation-font-frame-diagnostic/radical-font-frame-roles-hcr-print-2020.pdf` | `c67d874425dfebe0224f378881225f04073cb5bbe968c309d765eb2d2a300a32` |
| `radical-font-frame-boundaries-hcr.hwpx` | `8de3ba3ff761ef529c575a6bc224f0d0990b3da4db13478375a7b9bfef2163b5` |
| `radical-font-frame-boundaries-hcr-2020.hwp` | `450d220f55da8b2b54e9d981dbf90b91158e877ef9532020f0c6539750adcce5` |
| `pdf/pr7600-equation-font-frame-diagnostic/radical-font-frame-boundaries-hcr-print-2020.pdf` | `9cfb14c7a14cccff528f6f2786e771a4a18b26256055624190394b3a812bd209` |
| `geometric-font-frame-hcr.hwpx` | `1e89bdcc596319c6fd03dc395c2019b1468f6e2fd377e51b9b4da584131a3f8f` |
| `geometric-font-frame-hcr-2020.hwp` | `1b1a2d1722a64768b20a20ecaaf8ccc6a098e4dd3502276ad4861b13bd6c9b2a` |
| `pdf/pr7600-equation-font-frame-diagnostic/geometric-font-frame-hcr-print-2020.pdf` | `514a6528a5edccfb8d821ad09aa4a531f9ed2c00c1a89480f6e0d6af3cb4fc8d` |
