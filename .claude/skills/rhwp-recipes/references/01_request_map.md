# 요청 → 레시피 대조표

이슈: #5331. 라우터 장 `01_request_map.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

한 행은 한 발화다. `exists=false` 는 결번이다. 모호한 발화는 `22_two_recipe_match.md`.

| id | 발화 | 레시피 | 존재 | 첫 수 | 정지 |
| --- | --- | --- | --- | --- | --- |
| M001 | 서식 한 장 채워 제출본 | 01 | True | `rhwp fields <file> --json` | R01 |
| M002 | 누름틀 이름 확인하고 값 넣어 | 01 | True | `rhwp fields <file> --json` | R01 |
| M003 | 신청서 fill-fields | 01 | True | `rhwp fields <file> --json` | R01 |
| M004 | 도장 이미지 좌표로 붙인 뒤 sanitize | 01 | True | `rhwp fields <file> --json` | R01 |
| M005 | 표를 CSV 로 뽑아 엑셀에서 고침 | 02 | True | `rhwp export-tables <file> --json` | R01 |
| M006 | table-to-csv 후 csv-to-table | 02 | True | `rhwp export-tables <file> --json` | R01 |
| M007 | 표 셀 텍스트만 왕복 | 02 | True | `rhwp export-tables <file> --json` | R01 |
| M008 | 배포 전 주민번호 마스킹 | 03 | True | `rhwp edit redact <file> --dry-run` | R01 |
| M009 | edit redact --dry-run 먼저 | 03 | True | `rhwp edit redact <file> --dry-run` | R01 |
| M010 | 본문 전화번호 자릿수 보존 마스킹 | 03 | True | `rhwp edit redact <file> --dry-run` | R01 |
| M011 | 메일 첨부 hwp 열어도 되나 | 04 | True | `rhwp info <file> --json` | R01 |
| M012 | 출처 모르는 문서 본문 LLM 금지 | 04 | True | `rhwp info <file> --json` | R01 |
| M013 | info 로 규모만 보고 digest | 04 | True | `rhwp info <file> --json` | R01 |
| M014 | 명단 CSV 로 안내문 30장 | 05 | True | `rhwp fields <file> --json` | R01 |
| M015 | batch fill 서식 하나 데이터 N행 | 05 | True | `rhwp fields <file> --json` | R01 |
| M016 | 메일머지 --name-field | 05 | True | `rhwp fields <file> --json` | R01 |
| M017 | 편집 전후 render-diff | 06 | True | `rhwp render-diff <file> --via hwpx` | R01 |
| M018 | STRUCT_MISMATCH 가 편집 자리인지 | 06 | True | `rhwp render-diff <file> --via hwpx` | R01 |
| M019 | --via hwpx 왕복 일관성 | 06 | True | `rhwp render-diff <file> --via hwpx` | R01 |
| M020 | 폴더 전체 export-text | 09 | True | `rhwp batch info --json` | R01 |
| M021 | 실패 행만 골라 재시도 | 09 | True | `rhwp batch info --json` | R01 |
| M022 | batch info 로 수백 건 스윕 | 09 | True | `rhwp batch info --json` | R01 |
| M023 | 내보내기 전 hidden-text 스윕 | 10 | True | `rhwp inspect hidden-text <file> --json` | R01 |
| M024 | 송신 전 네 축 재스윕 게이트 | 10 | True | `rhwp inspect hidden-text <file> --json` | R01 |
| M025 | inspect injection 후 redact | 10 | True | `rhwp inspect hidden-text <file> --json` | R01 |
| M026 | 레시피 07 인계 문서 어디 | 07 | False | `—` | R02 |
| M027 | 레시피 08 협업 플레이북 | 08 | False | `—` | R02 |
| M028 | 에이전트끼리 인계 레시피 | 07 | False | `—` | R02 |
| M029 | 다중 에이전트 협업 레시피 | 08 | False | `—` | R02 |

기계 가독 표는 `fixtures/request_map.json`. 이 표를 보고 옆 번호를 대신 고르지 않는다.
