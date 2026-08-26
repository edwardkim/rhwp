# 26. 검증 게이트

"파일이 생겼다"는 done 이 아니다. goal 마다 기계 검사가 있다.

| goal | 게이트 id | 통과 조건 | 실패 시 |
| --- | --- | --- | --- |
| diagnose | ticket | ticket.json + route | 트리아지 실패 경로 |
| export-text | json-envelope | stdout JSON 파싱 | failed |
| export-pdf | pdf-magic | 파일 + `%PDF-` | C17 failed |
| export-hwpx | self-verify | `--verify` 0 + 파일 | C18 failed |
| convert-hwp | self-verify | `--verify` 0 + 파일 | C19 failed |
| extract-tables | csv-count | 표 수만큼 CSV (0도 OK) | C20 failed |
| fill | fill-envelope | 3종 빈 배열 + 파일 | C09 unlink + failed |

픽스처: `fixtures/verification_gates.json`.

## 공통 규율

성공처럼 보이는 미완성 산출물 금지. 게이트 실패 후 2부에 경로를 적지 않는다.
에이전트가 게이트를 생략하고 "열어 보니 된 것 같다"고 회신하는 것도 금지.

## 새 행의 최소 게이트

핸들러 PR 은 위 표에 `gate` 이름을 적고, 실패 시 산출을 남기지 않는 코드를
같이 넣는다. "일단 실행하고 사람이 보자"는 행은 거절한다.
