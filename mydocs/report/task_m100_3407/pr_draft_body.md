## 문제 (실측 기반)

대량 아카이브 대장화에서 문서 **제목**을 얻으려면 `info` 로는 안 되고, 문서마다 `export-text --json` 을 한 번 더 호출해 첫 의미 줄을 소비자가 직접 파싱해야 합니다(2-pass). 표지가 이미지인 문서는 `pages[0].text == ""` 라 제목이 비고, fallback 규칙을 소비자마다 재발명합니다. #3407 실측(271건 대장)에서 불투명 파일명(`143E433F503322BD33.hwp` = 실제 "상공신문")은 사람이 열어봐야 식별됐습니다.

## 구현 (이슈 제안 1안+3안)

- `info --json` 봉투에 `title` — 렌더된 페이지 텍스트(`export-text` 와 같은 원천)의 **첫 의미 줄**(trim 후 비어있지 않은 첫 줄). 표지가 이미지·빈 쪽이면 앞 3쪽(`TITLE_SCAN_PAGES`, digest 발췌와 같은 "앞 3쪽" 어휘)까지 내려가고, 그래도 없으면 `null`(생략 금지). best-effort — 쪽 추출 실패가 메타 조회를 막지 않습니다.
- `batch info --json` 은 공유 함수(`info_json_value`, #3237)로 **자동 전파** — 1-pass 대장화가 됩니다.
- `capabilities` 매니페스트·MCP `hwp_info` resultFields 광고 동기 갱신. 필드 추가만이므로 `schemaVersion: "1.0"` 유지.

## 검증 (red→green 실행)

- **red**: 수정 전 바이너리에서 신규 `info_title_contract` **5건 전부 실패** 실측(title 부재) — 테스트가 결함을 재현.
- **green**: 수정 후 5건 통과 —
  - 표지 첫 의미 줄이 title ("2022년 국립국어원 업무계획")
  - 이미지 표지 fallback ("행정업무운영 편람", 393쪽 실문서 — 이슈 3안)
  - 앞쪽에 의미 줄 없으면 `null` + 필드 존재 보장
  - **동형 계약**: 1-pass(title) == 종전 2-pass(`export-text` 첫 의미 줄)
  - `batch info` NDJSON 레코드 전파(성공·null 혼합 2건)
- 무회귀: `cli_json_contract` 통과(기존 info 봉투 필드 불변), rustfmt clean.
- 렌더 산출물 변화 없음(JSON 메타 전용) — 시각 전/후 비교 N/A.
- 처리 문서: `mydocs/report/task_m100_3407/` (1-pass 대장 실출력 `catalog_sample.txt` 동봉).

closes #3407

🤖 Generated with [Claude Code](https://claude.com/claude-code)
