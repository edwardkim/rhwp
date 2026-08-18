# 지도를 그만 읽고 스킬로 점프할 때

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

지도에서 절과 정본을 골랐으면 그 작업을 수행하는 스킬로 넘긴다.
이 스킬 안에서 채움·표·스윕·배치·세션을 재구현하지 않는다.

| ID | 언제 | 스킬 | 정지 |
| --- | --- | --- | --- |
| J01 | 누름틀·서식 채움·메일머지 | rhwp-form-fill | R08 |
| J02 | 표 CSV 왕복·칸 기록 | rhwp-table-exchange | R08 |
| J03 | inspect·redact·sanitize·송신 점검 | rhwp-security-sweep | R08 |
| J04 | 폴더 일괄·batch 축 | rhwp-bulk-pipeline | R08 |
| J05 | render-diff·ir-diff 레이아웃 | rhwp-visual-regression | R08 |
| J06 | mcp-serve 부착·세션 도구 | rhwp-mcp-session | R08 |
| J07 | CLI 분석·내보내기·디버그 | rhwp-cli | R08 |
| J08 | 처음 보는 문서 좁혀 읽기 | rhwp-doc-triage | R08 |
| J09 | run 계획·dry-run·verify | rhwp-safe-edit | R08 |
| J10 | untrusted* 출처 표지 | rhwp-provenance | R08 |
| J11 | replay·audit·lineage 영수증 | rhwp-work-receipt | R08 |
| J12 | 첫 설치·doctor·첫 5분 | rhwp-onboarding | R08 |
| J13 | 이슈·PR·기여 절차 | rhwp-contributor | R08 |
| J14 | PDF/이미지 → HWPX 시험지 | rhwp-exam-ingest | R08 |
| J15 | 전 명령 장 항해·실측 봉투 표본 | rhwp-codex | R09 |
| J16 | CLI/MCP 조각 추가·3층 계약 | rhwp-agent-surface | R09 |

점프 후 이 스킬로 돌아와 지도를 이어서 통독하지 않는다.
새 질문이 생기면 다시 3문 진입으로 한 절만 고른다.
