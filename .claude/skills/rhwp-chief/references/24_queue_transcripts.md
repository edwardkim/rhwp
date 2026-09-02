# 24. 큐 기록

`fixtures/queues/<id>/` 는 한 요청 폴더의 스냅샷이다.
request.json · result.json · response.md · ticket.json 네 파일이 한 세트.

| id | 제목 | goal | status | stop |
| --- | --- | --- | --- | --- |
| Q001 | 공문을 PDF로 (1차) | export-pdf | done |  |
| Q002 | 공문을 PDF로 (2차) | export-pdf | done |  |
| Q003 | 공문을 PDF로 (야간) | export-pdf | done |  |
| Q004 | 고시 인쇄본 (1차) | export-pdf | done |  |
| Q005 | 고시 인쇄본 (2차) | export-pdf | done |  |
| Q006 | 고시 인쇄본 (야간) | export-pdf | done |  |
| Q007 | 회의록 본문 (1차) | export-text | done |  |
| Q008 | 회의록 본문 (2차) | export-text | done |  |
| Q009 | 회의록 본문 (야간) | export-text | done |  |
| Q010 | 안내문 추출 (1차) | export-text | done |  |
| Q011 | 안내문 추출 (2차) | export-text | done |  |
| Q012 | 안내문 추출 (야간) | export-text | done |  |
| Q013 | HWP5를 HWPX로 (1차) | export-hwpx | done |  |
| Q014 | HWP5를 HWPX로 (2차) | export-hwpx | done |  |
| Q015 | HWP5를 HWPX로 (야간) | export-hwpx | done |  |
| Q016 | HWPX를 HWP로 (1차) | convert-hwp | done |  |

전수 44건은 `fixtures/queue_catalog.json` 과 `fixtures/queues/`.
대본은 `fixtures/transcripts/` 에 있다. `--once` 종료 코드 0 은
needs-agent 를 포함해 **시도 완료**이지 전부 done 이 아니다.
