---
kind: report
status: active
canonical: mydocs/report/task_m100_3407/README.md
last_verified: 2026-08-01
---

# #3407 처리 기록 — info/batch 봉투 best-effort `title` (1-pass 대장화)

## 문제

대량 아카이브 대장화에서 문서 제목을 얻으려면 `info` 외에 `export-text --json` 을
한 번 더 호출해 첫 의미 줄을 소비자가 직접 파싱해야 했다(문서당 2-pass). 표지가
이미지인 문서는 `pages[0].text == ""` 라 제목이 비고, fallback 규칙을 소비자마다
재발명했다. 실측 271건 대장(이슈 #3407 본문)에서 불투명 파일명(예
`143E433F503322BD33.hwp` = "상공신문")을 사람이 열어봐야 식별 가능했다.

## 해법 (이슈 제안 1안+3안)

- `info --json` 봉투에 `title` 추가 — 렌더된 페이지 텍스트
  (`extract_page_text_native`, `export-text --json` 과 같은 원천)의 **첫 의미 줄**
  (trim 후 비어있지 않은 첫 줄). 표지가 이미지·빈 쪽이면 앞 3쪽
  (`TITLE_SCAN_PAGES`, digest 발췌와 같은 "앞 3쪽" 어휘)까지 내려가고, 그래도
  없으면 `null`(필드 생략 금지). 값은 best-effort — 추출 실패가 메타 조회를 막지
  않는다.
- `batch info --json` 은 같은 `info_json_value` 를 쓰므로 자동 전파 — 1-pass 대장화.
- `capabilities` 매니페스트·MCP `hwp_info` resultFields 광고 동기 갱신.
- 스키마는 필드 추가만이므로 `schemaVersion: "1.0"` 유지 (#3237 계약).

## 검증 (red→green)

- red: 수정 전 바이너리에서 `info_title_contract` **5건 전부 실패** 실측
  (title 필드 부재) — 테스트가 결함을 재현함을 확인.
- green: 수정 후 `info_title_contract` 5건 통과 — 표지 첫 줄
  ("2022년 국립국어원 업무계획")·이미지 표지 fallback("행정업무운영 편람",
  393쪽 실문서)·의미 줄 없음 null·`export-text` 첫 의미 줄과 동형(1-pass ==
  종전 2-pass)·batch NDJSON 전파.
- 무회귀: `cli_json_contract` 통과(기존 info 봉투 필드 불변), rustfmt clean.
- 렌더 산출물 변화 없음(JSON 메타 전용) — 시각 전/후 비교 N/A.

## 실측 (1-pass 대장)

`catalog_sample.txt` — 수정 후 바이너리로 표본 4건을 `batch info --json` 1-pass
대장화한 실출력. 불투명 파일명 옆에 title 이 바로 붙는다.
