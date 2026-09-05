# 21 — 실사용 여정

여정 90개. 모두 `notGym: true`. 정지 ID 는 `stop_rules.json` 에 있다.

| ID | 제목 | 축 | 정지 | 걸음 |
| --- | --- | --- | --- | --- |
| J01 | 목록만 만들고 멈춤 | — | B17 | list |
| J02 | 빈 폴더 | — | B01 | list |
| J03 | info 선점검만 | — | B04 | list → batch info |
| J04 | info 전건 실패 | — | B02 | list → batch info |
| J05 | 암호 플래그 거부 | — | B03 | batch info --password |
| J06 | 본문 추출 전건 성공 | export-text | B17 | list → info → export-text → gate |
| J07 | 본문 추출 혼합 실패 | export-text | B05 | list → export-text → jq split → retry |
| J08 | 개요 일괄 auto | export-structure | B17 | list → export-structure --mode auto |
| J09 | 조문 일괄 clause | export-structure | B17 | list → export-structure --mode clause |
| J10 | mode 오타 | export-structure | B06 | export-structure --mode chapters |
| J11 | 표 수확 | export-tables | B07 | list → export-tables |
| J12 | 표 없는 문서 | export-tables | B07 | export-tables |
| J13 | 서식 조사 | fields | B08 | list → fields |
| J14 | 누름틀 0 | fields | B08 | fields |
| J15 | 전역 검색 | search | B17 | list → search --query 위임전결 |
| J16 | 검색어 누락 | search | B09 | search --json |
| J17 | 날짜 금액 수확 | extract-data | B10 | list → extract-data --limit 3 |
| J18 | kind=amount 만 | extract-data | B17 | extract-data --kind amount |
| J19 | limit 절단 | extract-data | B10 | extract-data --limit 3 |
| J20 | convert 성공 | convert | B17 | list → convert --out-dir out |
| J21 | convert 이름 충돌 | convert | B11 | convert --out-dir out |
| J22 | convert verify | convert | B15 | convert --verify |
| J23 | convert verify-pages | convert | B16 | convert --verify-pages |
| J24 | 메일머지 | fill | B17 | fields → batch fill --form --data --out-dir |
| J25 | fill 에 stdin 목록 | fill | B12 | printf paths | batch fill |
| J26 | fill dry-run | fill | B17 | batch fill --dry-run |
| J27 | fill 빈 CSV | fill | B12 | batch fill --data empty.csv |
| J28 | 게이트 통과 5=4+1 | — | B17 | export-text → gate |
| J29 | 게이트 실패 증발 | — | B13 | export-text | head → gate |
| J30 | stderr 요약만 읽고 행 무시 | — | B14 | export-text |
| J31 | out-dir 대시 | convert | B18 | convert --out-dir -결과 |
| J32 | 선별 후 추출 | — | B17 | info → jq pageCount>=10 → export-text |
| J33 | 암호 문서 분리 | — | B03 | info → 단건 --password |
| J34 | Windows 목록 | — | B17 | Get-ChildItem → info |
| J35 | threads 8 순서 보존 | export-text | B17 | export-text --threads 8 |
| J36 | 재시도 후 게이트 | — | B05 | export-text → retry → concat → gate |
| J37 | 검색 매치 0 | search | B17 | search --query ZZNOHIT |
| J38 | fields 후 form-fill 인계 | — | B08 | fields |
| J39 | tables 후 table-exchange 인계 | — | B07 | export-tables |
| J40 | 질문이 검색만 | search | B17 | search --query 위임전결 |
| J41 | 폴더 스윕 후 10쪽 이상만 본문 | export-text | B05 | info → jq → export-text |
| J42 | HWPX 만 convert | convert | B17 | list hwpx → convert |
| J43 | HWP3 와 HWP5 혼합 info | info | B04 | info |
| J44 | 편람 금액만 수확 | extract-data | B10 | extract-data --kind amount |
| J45 | 편람 날짜만 수확 | extract-data | B17 | extract-data --kind date |
| J46 | 편람 수량만 수확 | extract-data | B17 | extract-data --kind number |
| J47 | 검색 후 매치 문서만 본문 | search | B05 | search → jq matchCount>0 → export-text |
| J48 | 서식 있는 파일만 fill 후보 | fields | B08 | fields → jq fieldCount>0 |
| J49 | 표 있는 파일만 수확 | export-tables | B07 | export-tables → jq tableCount>0 |
| J50 | convert 후 verify 게이트 | convert | B16 | convert --verify --verify-pages |
| J51 | fill name-field 성명 | fill | B17 | batch fill --name-field 성명 |
| J52 | fill 이름 겹침 _2 | fill | B17 | batch fill |
| J53 | fill verify 행별 | fill | B15 | batch fill --verify |
| J54 | PowerShell Get-Content 파이프 | info | B04 | Get-Content 목록.txt | rhwp batch info --json |
| J55 | UTF-8 경로 한글 | info | B04 | list → info |
| J56 | 상대경로 vs 절대경로 | info | B02 | info |
| J57 | 같은 파일을 stdin 두 번 | extract-data | B10 | extract-data --limit 3 |
| J58 | MCP 로 convert 시도 | convert | B17 | hwp_batch convert |
| J59 | 단건 triage 후 폴더로 확대 | info | B04 | doc-triage → list → info |
| J60 | 보안 스윕 후 본문 | export-text | B05 | security-sweep → export-text |
| J61 | 작업 영수증으로 배치 증빙 | export-text | B17 | export-text → replay |
| J62 | jq 로 실패 경로만 수정 | export-text | B05 | export-text → jq error → rewrite list |
| J63 | os error 2 부류는 재시도 금지 | export-text | B05 | export-text |
| J64 | 암호 문서는 재시도하지 않고 분리 | info | B03 | info |
| J65 | panic 행도 봉투 | export-text | B05 | export-text |
| J66 | broken pipe 후 게이트 실패 | export-text | B13 | export-text | head -1 |
| J67 | NDJSON 을 JSON 배열로 오파싱 | — | B14 | jq without -s |
| J68 | stderr 를 결과 파일에 리다이렉트 | — | B14 | 2>&1 |
| J69 | 성공 행 pageCount 집계 | info | B04 | info → jq add |
| J70 | 검색 대소문자 구분 | search | B17 | search --query Hwp |
| J71 | structure outline 만 | export-structure | B17 | export-structure --mode outline |
| J72 | convert out-dir 필수 누락 | convert | B18 | convert --json |
| J73 | fill out-dir 필수 누락 | fill | B12 | batch fill --form --data |
| J74 | fill dry-run 에도 out-dir | fill | B17 | batch fill --dry-run --out-dir |
| J75 | threads 1 결정성 | export-text | B17 | export-text --threads 1 |
| J76 | threads 기본 CPU | export-text | B17 | export-text |
| J77 | 목록에 빈 줄 | info | B01 | list → info |
| J78 | 목록에 주석 줄 | info | B02 | list → info |
| J79 | PDF 혼입 | info | B02 | list → info |
| J80 | 디렉터리 경로 혼입 | info | B01 | list → info |
| J81 | 같은 이름 다른 폴더 convert | convert | B11 | convert |
| J82 | Report.HWP 와 report.hwp | convert | B11 | convert |
| J83 | 메일머지 12행 | fill | B17 | batch fill |
| J84 | 메일머지 notFound 행 | fill | B15 | batch fill |
| J85 | 메일머지 ambiguous 행 | fill | B15 | batch fill |
| J86 | info 후 질문 종료 | info | B17 | info |
| J87 | 검색 0건을 실패로 오독 금지 | search | B17 | search |
| J88 | extract-data 0건을 실패로 오독 금지 | extract-data | B17 | extract-data |
| J89 | fields 0건을 실패로 오독 금지 | fields | B08 | fields |
| J90 | exit 1 을 파이프 전체 실패로만 읽지 않기 | export-text | B14 | export-text |

기계 원본: `fixtures/journeys.json`.
## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `21_journeys.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
