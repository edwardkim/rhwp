# 23. 발화 → goal 행렬

고객 문장은 **기록**일 뿐 라우팅 키가 아니다. 아래 행렬은
`request.json.goal` 이 어떻게 떨어지는지, 그리고 그 필드가 비었을 때
루프가 무엇을 하는지를 보여 준다.

| id | 발화(데이터) | goal 필드 | 루프 라우트 | 정지 |
| --- | --- | --- | --- | --- |
| I001 | 시청 공문: PDF로 바꿔줘 | `export-pdf` | export-pdf |  |
| I002 | 시청 공문: 이 문서를 PDF로 | `export-pdf` | export-pdf |  |
| I003 | 시청 공문: 인쇄용 PDF | `export-pdf` | export-pdf |  |
| I004 | 시청 공문: 본문만 뽑아줘 | `export-text` | export-text |  |
| I005 | 시청 공문: 텍스트로 추출 | `export-text` | export-text |  |
| I006 | 시청 공문: 표만 CSV로 | `extract-tables` | extract-tables |  |
| I007 | 시청 공문: 표를 엑셀로 | `extract-tables` | extract-tables |  |
| I008 | 시청 공문: 서식 채워줘 | `fill` | fill | C08 |
| I009 | 시청 공문: 명단으로 채워 | `fill` | fill |  |
| I010 | 시청 공문: HWPX로 바꿔 | `export-hwpx` | export-hwpx |  |
| I011 | 시청 공문: 편집 가능한 HWP로 | `convert-hwp` | convert-hwp |  |
| I012 | 시청 공문: 변환해줘 hwp | `convert-hwp` | convert-hwp |  |
| I013 | 시청 공문: 진단만 | `diagnose` | diagnose |  |
| I014 | 시청 공문: 뭐가 문제야 | `∅` | diagnose |  |
| I015 | 시청 공문: 열어봐 | `∅` | diagnose |  |
| I016 | 시청 공문: 요약해줘 | `summarize` | needs-agent | C06 |
| I017 | 시청 공문: 영문으로 번역 | `translate` | needs-agent | C06 |
| I018 | 시청 공문: 도장 찍어줘 | `stamp` | needs-agent | C06 |
| I019 | 시청 공문: 비교해줘 한컴이랑 | `fidelity-compare` | needs-agent | C06 |
| I020 | 시청 공문: 전략 보고서 써줘 | `strategy` | needs-agent | C06 |
| I021 | 시청 공문: 버그인지 찾아줘 | `hunt-bug` | needs-agent | C06 |
| I022 | 시청 공문: 메일 보내줘 | `send-mail` | needs-agent | C06 |
| I023 | 시청 공문: 암호 풀어줘 | `crack` | needs-agent | C06 |
| I024 | 시청 공문: 페이지 번호 고쳐 | `fix-page-num` | needs-agent | C06 |

나머지 136행은 `fixtures/intent_matrix.json` (전수 160).
표 밖 goal 은 모두 `needs-agent` 다. 발화가 "PDF로 바꿔줘" 여도
goal 필드가 비어 있으면 diagnose 다.
