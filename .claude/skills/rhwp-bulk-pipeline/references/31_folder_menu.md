# 부록 — 폴더 유형별 축 선택

실무 폴더 이름과 첫 축. 새 명령이 아니라 기존 9축의 선택 표다.

| 폴더 | 첫 축 | 목적 | 다음 |
| --- | --- | --- | --- |
| 민원접수 | `info` | 당일 접수 HWP 쪽수 합 | 선별 후 본작업 축 |
| 고시원문 | `export-text` | 고시 본문 코퍼스 | jq 실패 분리 |
| 법령편람 | `export-structure` | 조문 모드로 조항 인덱스 | 필요 문서만 단건 |
| 통계월보 | `export-tables` | 병합 헤더 있는 월보 표 | table-exchange 또는 집계 |
| 신청서열 | `fields` | 누름틀 있는 서식만 | form-fill / batch fill |
| 감사지적 | `search` | 질의 '횡령' | 히트 문서 export-text |
| 예산서 | `extract-data` | 금액 kind=amount | normalized null 육안 |
| HWPX입고 | `convert` | 편집용 HWP5 | 이름 예약 확인 |
| 채용지원 | `fill` | 합격자 명단 메일머지 | form-fill 게이트 |
| 회의록 | `export-text` | 검색 전 본문 | jq 실패 분리 |
| 보도자료 | `info` | 형식 오인(HWP3) 선별 | 선별 후 본작업 축 |
| 계약서철 | `search` | 질의 '위약금' | 히트 문서 export-text |
| 인사발령 | `extract-data` | 날짜 kind=date | normalized null 육안 |
| 재고표 | `export-tables` | 수량 표 | table-exchange 또는 집계 |
| 교육교재 | `export-structure` | outline | 필요 문서만 단건 |
| 민원서식함 | `fields` | fieldCount>0 만 fill 후보 | form-fill / batch fill |
| 기록이관 | `convert` | 장기 보존 HWP5 | 이름 예약 확인 |
| 급여명세서 | `extract-data` | 금액+수량 | normalized null 육안 |
| 이사회의사록 | `search` | 질의 '의결' | 히트 문서 export-text |
| 연구보고서 | `export-text` | 10쪽 이상만 | jq 실패 분리 |
| 공고문 | `info` | pageCount==1 전단 | 선별 후 본작업 축 |
| 첨부서식 | `fill` | dry-run 먼저 | form-fill 게이트 |
| 외부수신 | `info` | 암호 실패 분리 | 선별 후 본작업 축 |
| 스캔결합 | `export-text` | 손상 파일 격리 | jq 실패 분리 |
| 다국어자료 | `search` | 대소문자 구분 두 번 | 히트 문서 export-text |
| 양식개정 | `fields` | 이름 변경 탐지 | form-fill / batch fill |
| 월간보고 | `extract-data` | limit 20 | normalized null 육안 |
| 도면첨부 | `info` | 거대 pageCount 단건 하향 | 선별 후 본작업 축 |
| 메일머지대량 | `fill` | 12행 표본 | form-fill 게이트 |
| 충돌변환 | `convert` | stem 충돌 분할 | 이름 예약 확인 |
| 검증변환 | `convert` | --verify --verify-pages | 이름 예약 확인 |
| 조문검색 | `search` | 질의 '제3조' — 수량이 아님 | 히트 문서 export-text |
| 표없는한글 | `export-tables` | tableCount 0 성공 | table-exchange 또는 집계 |
| 필드없는한글 | `fields` | fieldCount 0 인계 | form-fill / batch fill |
| 숫자만있는표 | `extract-data` | 단위 없는 숫자는 항목 아님 | normalized null 육안 |
| 부분날짜 | `extract-data` | normalized 2026-01 | normalized null 육안 |
| BOM목록 | `info` | 첫 줄 FEFF 제거 | 선별 후 본작업 축 |
| 상대경로 | `info` | cwd 확인 | 선별 후 본작업 축 |
| UNC경로 | `export-text` | 네트워크 os error | jq 실패 분리 |
| 잠금파일 | `convert` | sharing violation 재시도 | 이름 예약 확인 |

이 표는 여정 `fixtures/journeys.json` 과 같은 정지 규칙을 쓴다.
폴더 이름이 달라도 축을 발명하지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `31_folder_menu.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
