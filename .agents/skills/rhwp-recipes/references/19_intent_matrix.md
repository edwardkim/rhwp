# 발화 → 레시피 행렬

이슈: #5331. 라우터 장 `19_intent_matrix.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

행 104개. 전체는 `fixtures/intent_matrix.json`. 표본 24행:

| id | 발화 | 레시피 | 명령 | 정지 |
| --- | --- | --- | --- | --- |
| I001 | 이 신청서 채워 | 01 | `rhwp fields <file> --json` | R01 |
| I002 | 누름틀에 홍길동 넣어 | 01 | `rhwp fields <file> --json` | R01 |
| I003 | 서식 제출본 만들어 | 01 | `rhwp fields <file> --json` | R01 |
| I004 | fields 보고 fill-fields | 01 | `rhwp fields <file> --json` | R01 |
| I005 | 한 장만 값 넣고 sanitize | 01 | `rhwp fields <file> --json` | R01 |
| I006 | 도장 찍은 제출 파일 | 01 | `rhwp fields <file> --json` | R01 |
| I007 | 양식 빈칸 채워줘 | 01 | `rhwp fields <file> --json` | R01 |
| I008 | 관공서 서식 제출 준비 | 01 | `rhwp fields <file> --json` | R01 |
| I009 | myMsg01 에 값 | 01 | `rhwp fields <file> --json` | R01 |
| I010 | fill-fields --verify | 01 | `rhwp fields <file> --json` | R01 |
| I011 | 표 CSV 추출 | 02 | `rhwp export-tables <file> --json` | R01 |
| I012 | 엑셀에서 고친 표 되돌리기 | 02 | `rhwp export-tables <file> --json` | R01 |
| I013 | export-tables 먼저 | 02 | `rhwp export-tables <file> --json` | R01 |
| I014 | table-to-csv --table 0 | 02 | `rhwp export-tables <file> --json` | R01 |
| I015 | csv-to-table --verify | 02 | `rhwp export-tables <file> --json` | R01 |
| I016 | 표 왕복 셀 텍스트만 | 02 | `rhwp export-tables <file> --json` | R01 |
| I017 | 병합 있는지 보고 CSV | 02 | `rhwp export-tables <file> --json` | R01 |
| I018 | 스프레드시트 왕복 | 02 | `rhwp export-tables <file> --json` | R01 |
| I019 | 3열 4행 표 채우기 | 02 | `rhwp export-tables <file> --json` | R01 |
| I020 | BOM 붙여 엑셀용 CSV | 02 | `rhwp export-tables <file> --json` | R01 |
| I021 | 주민번호 마스킹 | 03 | `rhwp edit redact <file> --dry-run` | R01 |
| I022 | 카드번호 가려 | 03 | `rhwp edit redact <file> --dry-run` | R01 |
| I023 | 배포 전 redact | 03 | `rhwp edit redact <file> --dry-run` | R01 |
| I024 | edit redact --dry-run | 03 | `rhwp edit redact <file> --dry-run` | R01 |

나머지 발화·결번·충돌·예외 행은 JSON 이 정본이다.
